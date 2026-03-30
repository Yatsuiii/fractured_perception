pub mod time;

use std::collections::HashSet;

use crate::{
    events::{Event, EventBus},
    input::{InputState, Key},
    map::generate_test_map,
    perception,
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

const MAP_W: usize = 50;
const MAP_H: usize = 20;
const FOV_RADIUS: f32 = 8.0;

pub struct Engine {
    time:              Time,
    world:             World,
    players:           [Player; 3],
    active_idx:        usize,       // which player's view is shown (Tab cycles)
    state:             StateManager,
    events:            EventBus,
    input:             InputState,
    renderer:          TerminalRenderer,
    activated_puzzles: HashSet<u32>,
}

// --- World population ---

fn build_world() -> (World, [Entity; 3], [Entity; 3]) {
    let map = generate_test_map(MAP_W, MAP_H);
    let mut world = World::new(map);

    // Three player entities — each starts in a different room.
    let e0 = world.spawn(); world.add_position(e0, Position { x: 7.0,  y: 5.0 });
    let e1 = world.spawn(); world.add_position(e1, Position { x: 24.0, y: 4.0 });
    let e2 = world.spawn(); world.add_position(e2, Position { x: 40.0, y: 6.0 });

    // The Watcher NPC — cryptic, stands in the corridor between rooms.
    let watcher = world.spawn();
    world.add_position(watcher, Position { x: 31.0, y: 7.0 });
    world.add_npc_marker(watcher, NpcMarker { name: "The Watcher", base_trust: 0.6 });

    let puzzle_1 = world.spawn();
    world.add_position(puzzle_1, Position { x: 14.0, y: 4.0 });
    world.add_puzzle_tile(puzzle_1, PuzzleTile { sequence_id: 1, is_active: false });

    let puzzle_2 = world.spawn();
    world.add_position(puzzle_2, Position { x: 20.0, y: 6.0 });
    world.add_puzzle_tile(puzzle_2, PuzzleTile { sequence_id: 2, is_active: false });

    let puzzle_3 = world.spawn();
    world.add_position(puzzle_3, Position { x: 33.0, y: 6.0 });
    world.add_puzzle_tile(puzzle_3, PuzzleTile { sequence_id: 3, is_active: false });

    (world, [e0, e1, e2], [e0, e1, e2])
}

// --- Engine ---

impl Engine {
    pub fn new() -> Self {
        let (world, entities, _) = build_world();
        let (w, h) = (world.map.width, world.map.height);

        let mut players = [
            Player::new(entities[0], Role::Blind,          w, h),
            Player::new(entities[1], Role::VisualAnalyst,  w, h),
            Player::new(entities[2], Role::Hallucinating,  w, h),
        ];

        // Compute initial FOV for all three.
        for player in &mut players {
            world.compute_fov_into(player.entity, FOV_RADIUS, &mut player.fov);
        }

        Self {
            time: Time::new(),
            world,
            players,
            active_idx: 0,
            state: StateManager::new(),
            events: EventBus::new(),
            input: InputState::new(),
            renderer: TerminalRenderer::new(),
            activated_puzzles: HashSet::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), RenderError> {
        self.renderer.init()?;
        let result = self.main_loop();
        let _ = self.renderer.shutdown();
        result
    }

    fn main_loop(&mut self) -> Result<(), RenderError> {
        loop {
            self.time.tick();
            self.input.capture();

            if self.handle_input() {
                break;
            }

            self.update();

            // Build the active player's perception view and render it.
            let player_entities: Vec<Entity> =
                self.players.iter().map(|p| p.entity).collect();

            let view = perception::build_view(
                &self.players[self.active_idx],
                &player_entities,
                &self.world,
            );

            self.renderer.clear()?;
            self.renderer.draw_view(self.state.current(), &view)?;

            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        Ok(())
    }

    /// Returns true if engine should quit.
    fn handle_input(&mut self) -> bool {
        if self.input.is_pressed(&Key::Q) {
            return true;
        }

        // Tab always cycles the active player view.
        if self.input.is_pressed(&Key::Space) {
            self.active_idx = (self.active_idx + 1) % 3;
        }

        match self.state.current().clone() {
            GameState::MainMenu => {
                if self.input.is_pressed(&Key::Enter) {
                    self.state.transition(GameState::Playing);
                }
            }
            GameState::Playing => {
                if self.input.is_pressed(&Key::Escape) {
                    self.state.transition(GameState::Paused);
                } else {
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

        let player = &self.players[self.active_idx];
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
            if self.players[self.active_idx].role == Role::VisualAnalyst {
                self.players[self.active_idx].hidden_state.add_illusion(0.05);
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
                puzzle_tile.is_active = true;
                self.events.emit(Event::PuzzleActivated {
                    sequence_id: puzzle_tile.sequence_id,
                });
                self.players[self.active_idx].hidden_state.add_truth(0.05);
            }
        }

        self.world.compute_fov_into(entity, FOV_RADIUS, &mut self.players[self.active_idx].fov);

        // Walking through a tile the Hallucinating thought was a wall = balance point.
        if self.players[self.active_idx].role == Role::Hallucinating {
            let was_distorted = crate::perception::is_distorted(
                self.world.map.seed,
                nx as usize,
                ny as usize,
            );
            if was_distorted {
                self.players[self.active_idx].hidden_state.add_balance(0.02);
            }
        }

        self.players[self.active_idx].hidden_state.add_truth(0.01);
    }

    fn update(&mut self) {
        self.state.apply_pending();

        for event in self.events.drain() {
            match event {
                Event::PlayerMoved { entity: _entity, x: _x, y: _y } => {
                    // Movement events can drive future systems like footsteps, AI, or sound.
                }
                Event::PuzzleActivated { sequence_id } => {
                    if self.activated_puzzles.insert(sequence_id) {
                        if self.activated_puzzles.len() >= 3 {
                            self.state.transition(GameState::GameOver);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        let (world, entities, _) = build_world();
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
        self.active_idx = 0;
        self.activated_puzzles.clear();
    }
}
