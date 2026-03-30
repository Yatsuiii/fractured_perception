pub mod logger;
pub mod time;

use std::collections::{HashSet, VecDeque};

use crate::{
    events::{Event, EventBus, TrustReason},
    input::{InputState, Key},
    map::generate_test_map,
    perception::{self, PanelColor, PanelLine},
    player::{Player, Role},
    renderer::{terminal::TerminalRenderer, RenderError, Renderer},
    state::{GameState, StateManager},
    world::{
        components::{NpcMarker, Position, PuzzleTile},
        entity::Entity,
        World,
    },
};

use time::Time;

// --- Shared team event log ---

struct LogEntry {
    text:    String,
    color:   PanelColor,
    elapsed: f32,
}

struct EventLog {
    entries: VecDeque<LogEntry>,
}

impl EventLog {
    fn new() -> Self { Self { entries: VecDeque::new() } }

    fn clear(&mut self) { self.entries.clear(); }

    fn is_empty(&self) -> bool { self.entries.is_empty() }

    /// Keeps the most recent 8 entries; drops the oldest when full.
    fn push(&mut self, text: String, color: PanelColor, elapsed: f32) {
        if self.entries.len() >= 8 {
            self.entries.pop_front();
        }
        self.entries.push_back(LogEntry { text, color, elapsed });
    }
}

const MAP_W: usize = 80;
const MAP_H: usize = 34;
const FOV_RADIUS: f32 = 8.0;

pub struct Engine {
    time:              Time,
    world:             World,
    players:           [Player; 3],
    /// Index of the human player — set during role selection.
    human_idx:         usize,
    state:             StateManager,
    events:            EventBus,
    input:             InputState,
    renderer:          TerminalRenderer,
    activated_puzzles:     HashSet<u32>,
    /// Stores (sequence_id, time.elapsed at activation) for the timed UI flash.
    puzzle_flash:          Option<(u32, f32)>,
    /// Shared rolling log of team events — visible in every role's side panel.
    event_log:             EventLog,
    /// Writes every event to a timestamped file in logs/ for post-session review.
    session_logger:        logger::SessionLogger,
    /// Entities driven by the AI module (currently just the Watcher).
    npc_entities:          Vec<Entity>,
    /// time.elapsed value when the AI last took a step (NPCs move every 0.5 s).
    last_ai_tick:          f32,
    /// time.elapsed value when AI companions last took a step.
    last_companion_tick:   f32,
}

// --- World population ---

fn build_world() -> (World, [Entity; 3], Vec<Entity>) {
    let map = generate_test_map(MAP_W, MAP_H);
    let mut world = World::new(map);

    // Players start in the entrance hall (room 0).
    let e0 = world.spawn(); world.add_position(e0, Position { x: 5.0,  y: 4.0 });
    let e1 = world.spawn(); world.add_position(e1, Position { x: 6.0,  y: 4.0 });
    let e2 = world.spawn(); world.add_position(e2, Position { x: 5.0,  y: 5.0 });

    // --- NPCs — each inhabits a different wing ---

    // The Watcher — cryptic, roams the observation room (room 2).
    let watcher = world.spawn();
    world.add_position(watcher, Position { x: 33.0, y: 5.0 });
    world.add_npc_marker(watcher, NpcMarker { name: "The Watcher", base_trust: 0.6 });

    // The Echo — repeats fragments of truth, lurks in the echo chamber (room 10).
    let echo = world.spawn();
    world.add_position(echo, Position { x: 24.0, y: 27.0 });
    world.add_npc_marker(echo, NpcMarker { name: "The Echo", base_trust: 0.4 });

    // The Keeper — guards the central hall (room 7), slow to trust.
    let keeper = world.spawn();
    world.add_position(keeper, Position { x: 38.0, y: 15.0 });
    world.add_npc_marker(keeper, NpcMarker { name: "The Keeper", base_trust: 0.3 });

    // The Witness — silent observer near the terminus (room 12).
    let witness = world.spawn();
    world.add_position(witness, Position { x: 56.0, y: 27.0 });
    world.add_npc_marker(witness, NpcMarker { name: "The Witness", base_trust: 0.5 });

    // --- Puzzle tiles — scattered across the map ---

    let puzzles: &[(f32, f32, u32)] = &[
        (18.0,  4.0,  1),  // corridor junction (room 1)
        (30.0,  5.0,  2),  // observation room (room 2)
        (62.0,  5.0,  3),  // signal chamber (room 4)
        (7.0,  14.0,  4),  // storage vault (room 5)
        (38.0, 18.0,  5),  // central hall (room 7)
        (24.0, 30.0,  6),  // echo chamber (room 10)
        (56.0, 30.0,  7),  // terminus (room 12)
    ];

    for &(px, py, seq) in puzzles {
        let e = world.spawn();
        world.add_position(e, Position { x: px, y: py });
        world.add_puzzle_tile(e, PuzzleTile { sequence_id: seq, is_active: false });
    }

    let npc_entities = vec![watcher, echo, keeper, witness];
    (world, [e0, e1, e2], npc_entities)
}

// --- Engine ---

impl Engine {
    pub fn new() -> Self {
        let (world, entities, npc_entities) = build_world();
        let (w, h) = (world.map.width, world.map.height);

        // Roles are assigned after the player picks one on the selection screen.
        // Default all to Blind — reassigned in select_role().
        let mut players = [
            Player::new(entities[0], Role::Blind,          w, h),
            Player::new(entities[1], Role::VisualAnalyst,  w, h),
            Player::new(entities[2], Role::Hallucinating,  w, h),
        ];

        for player in &mut players {
            world.compute_fov_into(player.entity, FOV_RADIUS, &mut player.fov);
        }

        Self {
            time: Time::new(),
            world,
            players,
            human_idx: 0,
            state: StateManager::new(),
            events: EventBus::new(),
            input: InputState::new(),
            renderer: TerminalRenderer::new(),
            activated_puzzles: HashSet::new(),
            puzzle_flash: None,
            event_log: EventLog::new(),
            session_logger: logger::SessionLogger::new(),
            npc_entities,
            last_ai_tick: 0.0,
            last_companion_tick: 0.0,
        }
    }

    pub fn run(&mut self) -> Result<(), RenderError> {
        self.renderer.init()?;
        let result = self.main_loop();
        let _ = self.renderer.shutdown();
        self.session_logger.finish(self.activated_puzzles.len(), 7);
        result
    }

    fn main_loop(&mut self) -> Result<(), RenderError> {
        const FRAME_TARGET: std::time::Duration = std::time::Duration::from_millis(16);

        loop {
            let frame_start = std::time::Instant::now();

            self.time.tick();
            self.input.capture();

            if self.handle_input() {
                break;
            }

            self.update();

            // Build the active player's perception view.
            let player_entities: Vec<Entity> =
                self.players.iter().map(|p| p.entity).collect();

            let mut view = perception::build_view(
                &self.players[self.human_idx],
                &player_entities,
                &self.world,
            );

            // Inject a timed puzzle-activation flash into the side panel (2 s window).
            if let Some((seq_id, flash_time)) = self.puzzle_flash {
                if self.time.elapsed - flash_time < 2.0 {
                    view.panel_lines.push(PanelLine { text: String::new(), color: PanelColor::Grey });
                    view.panel_lines.push(PanelLine {
                        text: format!("  * PUZZLE #{} ACTIVATED!", seq_id),
                        color: PanelColor::Green,
                    });
                }
            }

            // Inject the shared team event log below the role-specific content.
            if !self.event_log.is_empty() {
                view.panel_lines.push(PanelLine { text: String::new(),        color: PanelColor::Grey    });
                view.panel_lines.push(PanelLine { text: "─ TEAM LOG ─".into(), color: PanelColor::DarkGrey });
                for entry in &self.event_log.entries {
                    let age = self.time.elapsed - entry.elapsed;
                    let color = if age < 4.0 { entry.color } else { PanelColor::DarkGrey };
                    view.panel_lines.push(PanelLine { text: entry.text.clone(), color });
                }
            }

            self.renderer.clear()?;
            self.renderer.draw_view(self.state.current(), &view)?;

            // Sleep only the remaining time so computation costs don't accumulate.
            let spent = frame_start.elapsed();
            if spent < FRAME_TARGET {
                std::thread::sleep(FRAME_TARGET - spent);
            }
        }
        Ok(())
    }

    /// Returns true if engine should quit.
    fn handle_input(&mut self) -> bool {
        if self.input.is_pressed(&Key::Q) {
            return true;
        }

        match self.state.current().clone() {
            GameState::MainMenu => {
                if self.input.is_pressed(&Key::Enter) {
                    self.state.transition(GameState::RoleSelect);
                }
            }
            GameState::RoleSelect => {
                if self.input.is_pressed(&Key::One) {
                    self.select_role(Role::Blind);
                } else if self.input.is_pressed(&Key::Two) {
                    self.select_role(Role::VisualAnalyst);
                } else if self.input.is_pressed(&Key::Three) {
                    self.select_role(Role::Hallucinating);
                } else if self.input.is_pressed(&Key::Escape) {
                    self.state.transition(GameState::MainMenu);
                }
            }
            GameState::Playing => {
                if self.input.is_pressed(&Key::Escape) {
                    self.state.transition(GameState::Paused);
                } else {
                    if self.input.is_pressed(&Key::E) {
                        let role = self.players[self.human_idx].role;
                        self.events.emit(Event::Ping { from_role: role });
                    }
                    self.handle_movement();
                }
            }
            GameState::Paused => {
                if self.input.is_pressed(&Key::Escape) {
                    self.state.transition(GameState::Playing);
                }
            }
            GameState::GameOver => {
                if self.input.is_pressed(&Key::Enter) {
                    self.reset();
                    self.state.transition(GameState::MainMenu);
                }
            }
        }

        false
    }

    /// Assigns the chosen role to the human player. The other two get the remaining roles.
    fn select_role(&mut self, chosen: Role) {
        let all_roles = [Role::Blind, Role::VisualAnalyst, Role::Hallucinating];
        let remaining: Vec<Role> = all_roles.iter().copied().filter(|r| *r != chosen).collect();

        self.players[0].role = chosen;
        self.players[1].role = remaining[0];
        self.players[2].role = remaining[1];
        self.human_idx = 0;

        let text = format!("  Role selected: {}", chosen.name());
        self.session_logger.log(&text);

        self.state.transition(GameState::Playing);
    }

    /// Moves the active player one tile per key press (turn-based feel).
    fn handle_movement(&mut self) {
        let mut dx = 0_i32;
        let mut dy = 0_i32;

        if self.input.is_pressed(&Key::W) || self.input.is_pressed(&Key::Up)    { dy -= 1; }
        if self.input.is_pressed(&Key::S) || self.input.is_pressed(&Key::Down)  { dy += 1; }
        if self.input.is_pressed(&Key::A) || self.input.is_pressed(&Key::Left)  { dx -= 1; }
        if self.input.is_pressed(&Key::D) || self.input.is_pressed(&Key::Right) { dx += 1; }

        if dx == 0 && dy == 0 {
            return;
        }

        let player = &self.players[self.human_idx];
        let entity = player.entity;

        let (nx, ny) = match self.world.get_position(entity) {
            Some(pos) => (pos.x as i32 + dx, pos.y as i32 + dy),
            None => return,
        };

        // Movement is always against the *true* map — not the perceived one.
        // This means the Analyst can walk into fabricated walls (invisible barrier)
        // and the Hallucinating can walk through walls that visually look solid.
        if !self.world.map.is_walkable(nx, ny) {
            // Walking into a wall the Analyst thought was a floor = illusion point.
            if self.players[self.human_idx].role == Role::VisualAnalyst {
                self.players[self.human_idx].hidden_state.add_illusion(0.05);
            }
            return;
        }

        if let Some(pos) = self.world.get_position_mut(entity) {
            pos.x = nx as f32;
            pos.y = ny as f32;
        }

        self.events.emit(Event::PlayerMoved { entity, x: nx as f32, y: ny as f32 });

        if let Some((_puzzle_entity, puzzle_tile)) = self.world.puzzle_tile_at_mut(nx, ny) {
            if !puzzle_tile.is_active {
                let seq_id = puzzle_tile.sequence_id;
                puzzle_tile.is_active = true;
                self.events.emit(Event::PuzzleActivated { sequence_id: seq_id });
                self.players[self.human_idx].hidden_state.add_truth(0.05);
            }
        }

        self.world.compute_fov_into(entity, FOV_RADIUS, &mut self.players[self.human_idx].fov);

        // Walking through a tile the Hallucinating thought was a wall = balance point.
        if self.players[self.human_idx].role == Role::Hallucinating {
            let was_distorted = crate::perception::is_distorted(
                self.world.map.seed,
                nx as usize,
                ny as usize,
            );
            if was_distorted {
                self.players[self.human_idx].hidden_state.add_balance(0.02);
            }
        }

        self.players[self.human_idx].hidden_state.add_truth(0.01);
    }

    fn update(&mut self) {
        self.state.apply_pending();

        // Step NPC AI at 0.5 s intervals — one move per tick, no per-frame jitter.
        if self.time.elapsed - self.last_ai_tick >= 0.5 {
            self.last_ai_tick = self.time.elapsed;
            let player_entities: Vec<Entity> = self.players.iter().map(|p| p.entity).collect();
            for i in 0..self.npc_entities.len() {
                let npc = self.npc_entities[i];
                match crate::ai::decide(npc, &player_entities, &self.world) {
                    crate::ai::AiAction::Move { dx, dy } => {
                        let (nx, ny) = match self.world.get_position(npc) {
                            Some(p) => (p.x as i32 + dx, p.y as i32 + dy),
                            None    => continue,
                        };
                        if self.world.map.is_walkable(nx, ny) {
                            if let Some(pos) = self.world.get_position_mut(npc) {
                                pos.x = nx as f32;
                                pos.y = ny as f32;
                            }
                        }
                    }
                    crate::ai::AiAction::Wait => {}
                }
            }

            // Proximity trust: being near an NPC each tick nudges trust.
            for &npc in &self.npc_entities {
                let npc_pos = match self.world.get_position(npc) {
                    Some(p) => *p,
                    None => continue,
                };
                let base = self.world.get_npc_marker(npc)
                    .map(|m| m.base_trust)
                    .unwrap_or(0.5);

                let human = self.players[self.human_idx].entity;
                if let Some(p) = self.world.get_position(human) {
                    let dx = p.x - npc_pos.x;
                    let dy = p.y - npc_pos.y;
                    let dist2 = dx * dx + dy * dy;

                    // Within 3 tiles: small trust gain. Within 6: smaller gain.
                    let delta = if dist2 <= 9.0 {
                        0.02
                    } else if dist2 <= 36.0 {
                        0.005
                    } else {
                        0.0
                    };

                    if delta > 0.0 {
                        self.players[self.human_idx].adjust_trust(npc, delta, base);
                        self.events.emit(Event::TrustChanged {
                            npc,
                            delta,
                            reason: TrustReason::NpcProximity,
                        });
                    }
                }
            }
        }

        // AI companions follow the human player loosely (every 0.8 s).
        if self.time.elapsed - self.last_companion_tick >= 0.8 {
            self.last_companion_tick = self.time.elapsed;
            let human_entity = self.players[self.human_idx].entity;
            if let Some(target) = self.world.get_position(human_entity).copied() {
                for i in 0..3 {
                    if i == self.human_idx { continue; }
                    let companion = self.players[i].entity;
                    let (cx, cy) = match self.world.get_position(companion) {
                        Some(p) => (p.x as i32, p.y as i32),
                        None    => continue,
                    };
                    let dx = target.x as i32 - cx;
                    let dy = target.y as i32 - cy;
                    let dist2 = dx * dx + dy * dy;

                    // Stay 2–3 tiles away — don't crowd the human.
                    if dist2 <= 4 { continue; }

                    let (mx, my) = if dx.abs() >= dy.abs() {
                        (dx.signum(), 0)
                    } else {
                        (0, dy.signum())
                    };

                    if self.world.map.is_walkable(cx + mx, cy + my) {
                        if let Some(pos) = self.world.get_position_mut(companion) {
                            pos.x = (cx + mx) as f32;
                            pos.y = (cy + my) as f32;
                        }
                        self.world.compute_fov_into(
                            companion, FOV_RADIUS, &mut self.players[i].fov,
                        );
                    }
                }
            }
        }

        for event in self.events.drain() {
            match event {
                Event::PlayerMoved { entity: _entity, x: _x, y: _y } => {
                    // Movement events can drive future systems like footsteps, AI, or sound.
                }
                Event::PuzzleActivated { sequence_id } => {
                    if self.activated_puzzles.insert(sequence_id) {
                        self.puzzle_flash = Some((sequence_id, self.time.elapsed));
                        let role = self.players[self.human_idx].role;
                        let text = format!("  {} → Puzzle #{} \u{2713}", role.name(), sequence_id);
                        self.session_logger.log(&text);
                        self.event_log.push(text, PanelColor::Green, self.time.elapsed);

                        // Solving puzzles builds NPC trust — cooperative action.
                        for &npc in &self.npc_entities {
                            let base = self.world.get_npc_marker(npc)
                                .map(|m| m.base_trust)
                                .unwrap_or(0.5);
                            self.players[self.human_idx].adjust_trust(npc, 0.1, base);
                        }

                        if self.activated_puzzles.len() >= 7 {
                            self.state.transition(GameState::GameOver);
                        }
                    }
                }
                Event::Ping { from_role } => {
                    let text = format!("  [{}] PING!", from_role.name());
                    self.session_logger.log(&text);
                    self.event_log.push(text, PanelColor::Cyan, self.time.elapsed);

                    // Pinging near an NPC draws attention — slight trust shift.
                    let human = self.players[self.human_idx].entity;
                    if let Some(hp) = self.world.get_position(human).copied() {
                        for &npc in &self.npc_entities {
                            let npc_pos = match self.world.get_position(npc) {
                                Some(p) => p,
                                None => continue,
                            };
                            let dx = hp.x - npc_pos.x;
                            let dy = hp.y - npc_pos.y;
                            if dx * dx + dy * dy <= 25.0 {
                                let base = self.world.get_npc_marker(npc)
                                    .map(|m| m.base_trust)
                                    .unwrap_or(0.5);
                                self.players[self.human_idx].adjust_trust(npc, -0.05, base);
                            }
                        }
                    }
                }
                Event::TrustChanged { npc: _, delta, reason } => {
                    let dir = if delta > 0.0 { "+" } else { "" };
                    let text = format!("  Trust {}{:.2} ({:?})", dir, delta, reason);
                    self.session_logger.log(&text);
                    self.event_log.push(text, PanelColor::Yellow, self.time.elapsed);
                }
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        let (world, entities, npc_entities) = build_world();
        let (w, h) = (world.map.width, world.map.height);

        let mut players = [
            Player::new(entities[0], Role::Blind,          w, h),
            Player::new(entities[1], Role::VisualAnalyst,  w, h),
            Player::new(entities[2], Role::Hallucinating,  w, h),
        ];

        for player in &mut players {
            world.compute_fov_into(player.entity, FOV_RADIUS, &mut player.fov);
        }

        self.world = world;
        self.players = players;
        self.npc_entities = npc_entities;
        self.human_idx = 0;
        self.activated_puzzles.clear();
        self.puzzle_flash = None;
        self.event_log.clear();
        self.last_ai_tick = 0.0;
        self.last_companion_tick = 0.0;
        self.session_logger.log("--- NEW GAME ---");
    }
}
