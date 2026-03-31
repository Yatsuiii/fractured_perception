pub mod logger;
pub mod time;

mod dialogue;
mod input;
mod movement;
mod render;
mod stage;
mod update;

use std::collections::{HashSet, VecDeque};

use crate::{
    dialogue::DialogueSession,
    events::{EventBus, thresholds::ThresholdTracker},
    input::InputState,
    perception::PanelColor,
    player::{Player, Role},
    renderer::{terminal::TerminalRenderer, RenderError, Renderer},
    stage::{Progression, get_stage_def},
    state::StateManager,
    world::{entity::Entity, history::PositionHistory, World},
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
    /// Position history for the Delayed role's stale entity rendering.
    position_history:      PositionHistory,
    /// Linear stage progression.
    progression:           Progression,
    /// The role chosen by the human — preserved across stage transitions.
    chosen_role:           Option<Role>,
    /// Tracks which T/C/I/B thresholds have fired.
    threshold_tracker:     ThresholdTracker,
}

// --- Stage-driven world building ---

use stage::build_stage;

// --- Engine ---

impl Engine {
    pub fn new() -> Self {
        let stage_def = get_stage_def(0);
        let (world, entities, npc_entities, encounter_entities) = build_stage(&stage_def);
        let (w, h) = (world.map.width, world.map.height);

        let mut players = [
            Player::new(entities[0], Role::Blind,          w, h),
            Player::new(entities[1], Role::Delayed,  w, h),
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
            position_history: PositionHistory::new(5.0),
            progression: Progression::new(),
            chosen_role: None,
            threshold_tracker: ThresholdTracker::new(),
        }
    }

    // --- Convenience accessors ---

    fn human(&self) -> &Player {
        &self.players[self.human_idx]
    }

    fn human_mut(&mut self) -> &mut Player {
        &mut self.players[self.human_idx]
    }

    fn player_entities(&self) -> [Entity; 3] {
        std::array::from_fn(|i| self.players[i].entity)
    }

    /// Get an NPC's base trust, defaulting to 0.5 if the marker is missing.
    fn npc_base_trust(&self, npc: Entity) -> f32 {
        self.world.get_npc_marker(npc)
            .map(|m| m.base_trust)
            .unwrap_or(0.5)
    }

    // --- Core loop ---

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
            self.render()?;

            let spent = frame_start.elapsed();
            if spent < FRAME_TARGET {
                std::thread::sleep(FRAME_TARGET - spent);
            }
        }
        Ok(())
    }

}
