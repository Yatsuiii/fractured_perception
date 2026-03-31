pub mod logger;
pub mod time;

use std::collections::{HashSet, VecDeque};

use crate::{
    dialogue::{self, DialogueSession},
    encounter::EncounterMarker,
    events::{Event, EventBus, TrustReason},
    input::{InputState, Key},
    perception::{self, PanelColor, PanelLine},
    player::{Player, Role},
    renderer::{terminal::TerminalRenderer, RenderError, Renderer},
    stage::{Progression, StageDef, get_stage_def, get_theme, maps},
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

    fn push(&mut self, text: String, color: PanelColor, elapsed: f32) {
        if self.entries.len() >= 8 {
            self.entries.pop_front();
        }
        self.entries.push_back(LogEntry { text, color, elapsed });
    }
}

const FOV_RADIUS: f32 = 8.0;

pub struct Engine {
    time:              Time,
    world:             World,
    players:           [Player; 3],
    human_idx:         usize,
    state:             StateManager,
    events:            EventBus,
    input:             InputState,
    renderer:          TerminalRenderer,
    activated_puzzles:     HashSet<u32>,
    puzzle_flash:          Option<(u32, f32)>,
    event_log:             EventLog,
    session_logger:        logger::SessionLogger,
    npc_entities:          Vec<Entity>,
    encounter_entities:    Vec<Entity>,
    last_ai_tick:          f32,
    last_companion_tick:   f32,
    dialogue_session:      Option<DialogueSession>,
    /// Linear stage progression.
    progression:           Progression,
    /// The role chosen by the human — preserved across stage transitions.
    chosen_role:           Option<Role>,
}

// --- Stage-driven world building ---

fn build_stage(stage_def: &StageDef) -> (World, [Entity; 3], Vec<Entity>, Vec<Entity>) {
    let map = maps::generate_stage_map(stage_def.theme.stage_number() - 1);
    let mut world = World::new(map);

    // Spawn players at the stage's spawn point.
    let (sx, sy) = stage_def.spawn_position;
    let e0 = world.spawn(); world.add_position(e0, Position { x: sx,       y: sy });
    let e1 = world.spawn(); world.add_position(e1, Position { x: sx + 1.0, y: sy });
    let e2 = world.spawn(); world.add_position(e2, Position { x: sx,       y: sy + 1.0 });

    // Spawn NPCs from stage definition.
    let mut npc_entities = Vec::new();
    for npc_def in &stage_def.npcs {
        let e = world.spawn();
        world.add_position(e, Position { x: npc_def.x, y: npc_def.y });
        world.add_npc_marker(e, NpcMarker { name: npc_def.name, base_trust: npc_def.base_trust });
        npc_entities.push(e);
    }

    // Spawn encounters from stage definition.
    let mut encounter_entities = Vec::new();
    for enc_def in &stage_def.encounters {
        let e = world.spawn();
        let (ex, ey) = enc_def.position;
        world.add_position(e, Position { x: ex, y: ey });
        world.add_encounter(e, EncounterMarker::from_def(enc_def));
        encounter_entities.push(e);
    }

    // Spawn the exit gate as a puzzle tile (sequence_id 0 = gate marker).
    let gate = world.spawn();
    let (gx, gy) = stage_def.gate_position;
    world.add_position(gate, Position { x: gx, y: gy });
    world.add_puzzle_tile(gate, PuzzleTile { sequence_id: 0, is_active: false });

    (world, [e0, e1, e2], npc_entities, encounter_entities)
}

// --- Engine ---

impl Engine {
    pub fn new() -> Self {
        let stage_def = get_stage_def(0);
        let (world, entities, npc_entities, encounter_entities) = build_stage(&stage_def);
        let (w, h) = (world.map.width, world.map.height);

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
            encounter_entities,
            last_ai_tick: 0.0,
            last_companion_tick: 0.0,
            dialogue_session: None,
            progression: Progression::new(),
            chosen_role: None,
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

            // Render based on current state.
            match self.state.current() {
                GameState::StageTransition => {
                    let theme = get_theme(self.progression.current_stage);
                    self.renderer.clear()?;
                    self.renderer.draw_stage_transition(theme)?;
                }
                _ => {
                    let player_entities: Vec<Entity> =
                        self.players.iter().map(|p| p.entity).collect();

                    let mut view = perception::build_view(
                        &self.players[self.human_idx],
                        &player_entities,
                        &self.world,
                    );

                    // Inject puzzle flash.
                    if let Some((seq_id, flash_time)) = self.puzzle_flash {
                        if self.time.elapsed - flash_time < 2.0 {
                            view.panel_lines.push(PanelLine { text: String::new(), color: PanelColor::Grey });
                            view.panel_lines.push(PanelLine {
                                text: format!("  * PUZZLE #{} ACTIVATED!", seq_id),
                                color: PanelColor::Green,
                            });
                        }
                    }

                    // Inject stage info into panel.
                    let theme = get_theme(self.progression.current_stage);
                    view.panel_lines.push(PanelLine { text: String::new(), color: PanelColor::Grey });
                    view.panel_lines.push(PanelLine {
                        text: format!("Stage {}: {}", theme.stage_number(), theme.name()),
                        color: PanelColor::White,
                    });

                    let stage_def = get_stage_def(self.progression.current_stage);
                    let resolved = self.progression.encounters_resolved;
                    let total = stage_def.clear_threshold;
                    view.panel_lines.push(PanelLine {
                        text: format!("Encounters: {}/{}", resolved, total),
                        color: if self.progression.gate_open { PanelColor::Green } else { PanelColor::Grey },
                    });

                    if self.progression.gate_open {
                        view.panel_lines.push(PanelLine {
                            text: "  GATE OPEN — find the exit!".into(),
                            color: PanelColor::Green,
                        });
                    }

                    // Inject team event log.
                    if !self.event_log.is_empty() {
                        view.panel_lines.push(PanelLine { text: String::new(), color: PanelColor::Grey });
                        view.panel_lines.push(PanelLine { text: "─ TEAM LOG ─".into(), color: PanelColor::DarkGrey });
                        for entry in &self.event_log.entries {
                            let age = self.time.elapsed - entry.elapsed;
                            let color = if age < 4.0 { entry.color } else { PanelColor::DarkGrey };
                            view.panel_lines.push(PanelLine { text: entry.text.clone(), color });
                        }
                    }

                    self.renderer.clear()?;
                    self.renderer.draw_view(self.state.current(), &view)?;

                    if *self.state.current() == GameState::Dialogue {
                        if let Some(ref session) = self.dialogue_session {
                            self.renderer.draw_dialogue_overlay(session)?;
                        }
                    }
                }
            }

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
                        // Priority: NPC dialogue > encounter interaction > ping
                        if !self.try_start_dialogue() && !self.try_interact_encounter() {
                            let role = self.players[self.human_idx].role;
                            self.events.emit(Event::Ping { from_role: role });
                        }
                    }
                    self.handle_movement();
                }
            }
            GameState::Dialogue => {
                self.handle_dialogue_input();
            }
            GameState::StageTransition => {
                if self.input.is_pressed(&Key::Enter) {
                    self.state.transition(GameState::Playing);
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

    fn select_role(&mut self, chosen: Role) {
        let all_roles = [Role::Blind, Role::VisualAnalyst, Role::Hallucinating];
        let remaining: Vec<Role> = all_roles.iter().copied().filter(|r| *r != chosen).collect();

        self.players[0].role = chosen;
        self.players[1].role = remaining[0];
        self.players[2].role = remaining[1];
        self.human_idx = 0;
        self.chosen_role = Some(chosen);

        let text = format!("  Role selected: {}", chosen.name());
        self.session_logger.log(&text);

        // Show stage transition screen before starting.
        self.state.transition(GameState::StageTransition);
    }

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

        if !self.world.map.is_walkable(nx, ny) {
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

        // Check for puzzle tile activation.
        if let Some((_puzzle_entity, puzzle_tile)) = self.world.puzzle_tile_at_mut(nx, ny) {
            if !puzzle_tile.is_active {
                let seq_id = puzzle_tile.sequence_id;

                // seq_id 0 = exit gate. Only activate if gate is open.
                if seq_id == 0 && self.progression.gate_open {
                    puzzle_tile.is_active = true;
                    self.advance_stage();
                    return;
                } else if seq_id > 0 {
                    puzzle_tile.is_active = true;
                    self.events.emit(Event::PuzzleActivated { sequence_id: seq_id });
                    self.players[self.human_idx].hidden_state.add_truth(0.05);
                }
            }
        }

        self.world.compute_fov_into(entity, FOV_RADIUS, &mut self.players[self.human_idx].fov);

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

    /// Try to interact with a nearby encounter. Returns true if interaction happened.
    fn try_interact_encounter(&mut self) -> bool {
        let player_pos = match self.world.get_position(self.players[self.human_idx].entity) {
            Some(p) => *p,
            None => return false,
        };

        // Find closest active encounter within 2 tiles.
        let mut closest: Option<(Entity, f32)> = None;
        for &enc in &self.encounter_entities {
            let enc_pos = match self.world.get_position(enc) {
                Some(p) => *p,
                None => continue,
            };
            let enc_marker = match self.world.get_encounter(enc) {
                Some(m) => m,
                None => continue,
            };
            if !enc_marker.is_active() { continue; }

            let dx = player_pos.x - enc_pos.x;
            let dy = player_pos.y - enc_pos.y;
            let dist2 = dx * dx + dy * dy;

            if dist2 <= 4.0 {
                if closest.is_none() || dist2 < closest.unwrap().1 {
                    closest = Some((enc, dist2));
                }
            }
        }

        let (enc_entity, _) = match closest {
            Some(c) => c,
            None => return false,
        };

        // Resolve the encounter.
        let (enc_name, enc_kind, role_text) = {
            let marker = self.world.get_encounter(enc_entity).unwrap();
            let role = self.players[self.human_idx].role;
            (marker.name, marker.kind, marker.text_for_role(role))
        };

        // Log what the player's role perceives.
        self.event_log.push(
            format!("  [{}] {}", enc_kind.label(), enc_name),
            PanelColor::Yellow,
            self.time.elapsed,
        );
        self.event_log.push(
            format!("  {}", truncate_text(role_text, 24)),
            PanelColor::Grey,
            self.time.elapsed,
        );
        self.session_logger.log(&format!("  Encounter: {} ({})", enc_name, enc_kind.label()));

        // Mark as resolved.
        if let Some(marker) = self.world.get_encounter_mut(enc_entity) {
            marker.resolve();
        }

        self.events.emit(Event::EncounterResolved { entity: enc_entity });

        // Give stat rewards based on encounter type.
        let hs = &mut self.players[self.human_idx].hidden_state;
        match enc_kind {
            crate::encounter::EncounterKind::Puzzle   => { hs.add_truth(0.08); }
            crate::encounter::EncounterKind::Enemy    => { hs.add_chaos(0.05); hs.add_balance(0.03); }
            crate::encounter::EncounterKind::Obstacle => { hs.add_balance(0.05); hs.add_truth(0.03); }
        }

        // Check if gate should open.
        let stage_def = get_stage_def(self.progression.current_stage);
        let gate_just_opened = self.progression.resolve_encounter(stage_def.clear_threshold);
        if gate_just_opened {
            self.event_log.push(
                "  >>> GATE OPENED <<<".into(),
                PanelColor::Green,
                self.time.elapsed,
            );
            self.session_logger.log("  Gate opened!");
        }

        // Trust boost from resolving encounters.
        for &npc in &self.npc_entities {
            let base = self.world.get_npc_marker(npc)
                .map(|m| m.base_trust)
                .unwrap_or(0.5);
            self.players[self.human_idx].adjust_trust(npc, 0.08, base);
        }

        true
    }

    /// Advance to the next stage, or end the game if all stages are complete.
    fn advance_stage(&mut self) {
        let theme = get_theme(self.progression.current_stage);
        self.session_logger.log(&format!("  Stage cleared: {}", theme.name()));

        if self.progression.is_final_stage() {
            self.state.transition(GameState::GameOver);
            return;
        }

        self.progression.advance();
        self.load_current_stage();
        self.state.transition(GameState::StageTransition);
    }

    /// Loads the current stage into the world, preserving the player's role and hidden state.
    fn load_current_stage(&mut self) {
        let stage_def = get_stage_def(self.progression.current_stage);
        let (world, entities, npc_entities, encounter_entities) = build_stage(&stage_def);
        let (w, h) = (world.map.width, world.map.height);

        // Preserve hidden state and role across stage transitions.
        let hidden_states: Vec<_> = self.players.iter().map(|p| p.hidden_state.clone()).collect();
        let roles: Vec<_> = self.players.iter().map(|p| p.role).collect();

        let mut players = [
            Player::new(entities[0], roles[0], w, h),
            Player::new(entities[1], roles[1], w, h),
            Player::new(entities[2], roles[2], w, h),
        ];

        players[0].hidden_state = hidden_states[0].clone();
        players[1].hidden_state = hidden_states[1].clone();
        players[2].hidden_state = hidden_states[2].clone();

        for player in &mut players {
            world.compute_fov_into(player.entity, FOV_RADIUS, &mut player.fov);
        }

        self.world = world;
        self.players = players;
        self.npc_entities = npc_entities;
        self.encounter_entities = encounter_entities;
        self.activated_puzzles.clear();
        self.puzzle_flash = None;
        self.event_log.clear();
        self.last_ai_tick = 0.0;
        self.last_companion_tick = 0.0;
        self.dialogue_session = None;

        let theme = get_theme(self.progression.current_stage);
        self.session_logger.log(&format!("  Entering: {}", theme.name()));
    }

    fn update(&mut self) {
        self.state.apply_pending();

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
                Event::PlayerMoved { .. } => {}
                Event::PuzzleActivated { sequence_id } => {
                    if self.activated_puzzles.insert(sequence_id) {
                        self.puzzle_flash = Some((sequence_id, self.time.elapsed));
                        let role = self.players[self.human_idx].role;
                        let text = format!("  {} → Puzzle #{} ✓", role.name(), sequence_id);
                        self.session_logger.log(&text);
                        self.event_log.push(text, PanelColor::Green, self.time.elapsed);
                    }
                }
                Event::Ping { from_role } => {
                    let text = format!("  [{}] PING!", from_role.name());
                    self.session_logger.log(&text);
                    self.event_log.push(text, PanelColor::Cyan, self.time.elapsed);

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
                Event::TrustChanged { delta, reason, .. } => {
                    let dir = if delta > 0.0 { "+" } else { "" };
                    let text = format!("  Trust {}{:.2} ({:?})", dir, delta, reason);
                    self.session_logger.log(&text);
                    self.event_log.push(text, PanelColor::Yellow, self.time.elapsed);
                }
                Event::DialogueStarted { npc } => {
                    let name = self.world.get_npc_marker(npc)
                        .map(|m| m.name)
                        .unwrap_or("???");
                    self.session_logger.log(&format!("  >> Dialogue: {}", name));
                }
                Event::DialogueEnded { npc } => {
                    let name = self.world.get_npc_marker(npc)
                        .map(|m| m.name)
                        .unwrap_or("???");
                    self.session_logger.log(&format!("  << Dialogue end: {}", name));
                }
                Event::EncounterResolved { .. } => {}
                _ => {}
            }
        }
    }

    fn try_start_dialogue(&mut self) -> bool {
        let player = &self.players[self.human_idx];
        let player_pos = match self.world.get_position(player.entity) {
            Some(p) => *p,
            None => return false,
        };

        let mut closest: Option<(Entity, f32)> = None;
        for &npc in &self.npc_entities {
            let npc_pos = match self.world.get_position(npc) {
                Some(p) => *p,
                None => continue,
            };
            let dx = player_pos.x - npc_pos.x;
            let dy = player_pos.y - npc_pos.y;
            let dist2 = dx * dx + dy * dy;

            if dist2 <= 4.0 {
                if closest.is_none() || dist2 < closest.unwrap().1 {
                    closest = Some((npc, dist2));
                }
            }
        }

        let (npc_entity, _) = match closest {
            Some(c) => c,
            None => return false,
        };

        let npc_name = match self.world.get_npc_marker(npc_entity) {
            Some(m) => m.name,
            None => return false,
        };

        let role = self.players[self.human_idx].role;
        let base_trust = self.world.get_npc_marker(npc_entity)
            .map(|m| m.base_trust)
            .unwrap_or(0.5);
        let trust = self.players[self.human_idx].trust_for(npc_entity, base_trust);

        let lines = match dialogue::get_dialogue(npc_name, role, trust) {
            Some(l) => l,
            None => return false,
        };

        self.dialogue_session = Some(DialogueSession {
            npc_entity,
            npc_name,
            lines,
            current_line: 0,
        });

        self.events.emit(Event::DialogueStarted { npc: npc_entity });
        self.session_logger.log(&format!("  Dialogue started: {}", npc_name));
        self.state.transition(GameState::Dialogue);
        true
    }

    fn handle_dialogue_input(&mut self) {
        if self.input.is_pressed(&Key::Escape) {
            self.end_dialogue();
            return;
        }

        if self.input.is_pressed(&Key::E) || self.input.is_pressed(&Key::Enter) {
            self.apply_current_dialogue_line();

            let finished = match self.dialogue_session.as_mut() {
                Some(session) => !session.advance(),
                None => true,
            };

            if finished {
                self.end_dialogue();
            }
        }
    }

    fn apply_current_dialogue_line(&mut self) {
        let (npc_entity, trust_delta, stat_nudge) = match &self.dialogue_session {
            Some(session) => match session.current() {
                Some(line) => (session.npc_entity, line.trust_delta, line.stat_nudge),
                None => return,
            },
            None => return,
        };

        if trust_delta.abs() > f32::EPSILON {
            let base = self.world.get_npc_marker(npc_entity)
                .map(|m| m.base_trust)
                .unwrap_or(0.5);
            self.players[self.human_idx].adjust_trust(npc_entity, trust_delta, base);
            self.events.emit(Event::TrustChanged {
                npc: npc_entity,
                delta: trust_delta,
                reason: TrustReason::Dialogue,
            });
        }

        let hs = &mut self.players[self.human_idx].hidden_state;
        let (truth, chaos, illusion, balance) = stat_nudge;
        if truth > 0.0    { hs.add_truth(truth); }
        if chaos > 0.0    { hs.add_chaos(chaos); }
        if illusion > 0.0  { hs.add_illusion(illusion); }
        if balance > 0.0   { hs.add_balance(balance); }
    }

    fn end_dialogue(&mut self) {
        if let Some(session) = self.dialogue_session.take() {
            let npc_name = session.npc_name;
            self.events.emit(Event::DialogueEnded { npc: session.npc_entity });
            self.session_logger.log(&format!("  Dialogue ended: {}", npc_name));
            self.event_log.push(
                format!("  Spoke with {}", npc_name),
                PanelColor::Cyan,
                self.time.elapsed,
            );
        }
        self.state.transition(GameState::Playing);
    }

    fn reset(&mut self) {
        self.progression = Progression::new();
        let stage_def = get_stage_def(0);
        let (world, entities, npc_entities, encounter_entities) = build_stage(&stage_def);
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
        self.encounter_entities = encounter_entities;
        self.human_idx = 0;
        self.activated_puzzles.clear();
        self.puzzle_flash = None;
        self.event_log.clear();
        self.last_ai_tick = 0.0;
        self.last_companion_tick = 0.0;
        self.dialogue_session = None;
        self.chosen_role = None;
        self.session_logger.log("--- NEW GAME ---");
    }
}

/// Truncates text to fit in the side panel.
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}
