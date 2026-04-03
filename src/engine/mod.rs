pub mod logger;
pub mod time;

mod dialogue;
mod event_log;
mod input;
mod movement;
mod render;
mod session;
mod stage;
mod update;

use crate::{
    events::{EventBus, thresholds::ThresholdTracker},
    input::InputState,
    player::Role,
    renderer::{terminal::TerminalRenderer, RenderError, Renderer},
    stage::get_stage_def,
    state::StateManager,
    world::entity::Entity,
};

use session::Session;
use stage::build_session;
use time::Time;

const FOV_RADIUS: f32 = 8.0;

pub struct Engine {
    time:               Time,
    session:            Session,
    human_idx:          usize,
    state:              StateManager,
    events:             EventBus,
    input:              InputState,
    renderer:           TerminalRenderer,
    session_logger:     logger::SessionLogger,
    progression:        crate::stage::Progression,
    chosen_role:        Option<Role>,
    threshold_tracker:  ThresholdTracker,
}

impl Engine {
    pub fn new() -> Self {
        let stage_def = get_stage_def(0);
        let default_roles = [Role::Blind, Role::Delayed, Role::Hallucinating];
        let session = build_session(&stage_def, default_roles, None);

        Self {
            time: Time::new(),
            session,
            human_idx: 0,
            state: StateManager::new(),
            events: EventBus::new(),
            input: InputState::new(),
            renderer: TerminalRenderer::new(),
            session_logger: logger::SessionLogger::new(),
            progression: crate::stage::Progression::new(),
            chosen_role: None,
            threshold_tracker: ThresholdTracker::new(),
        }
    }

    // --- Convenience accessors ---

    fn human(&self) -> &crate::player::Player {
        &self.session.players[self.human_idx]
    }

    fn human_mut(&mut self) -> &mut crate::player::Player {
        &mut self.session.players[self.human_idx]
    }

    fn player_entities(&self) -> [Entity; 3] {
        std::array::from_fn(|i| self.session.players[i].entity)
    }

    fn npc_base_trust(&self, npc: Entity) -> f32 {
        self.session.world.get_npc_marker(npc)
            .map(|m| m.base_trust)
            .unwrap_or(0.5)
    }

    // --- Core loop ---

    pub fn run(&mut self) -> Result<(), RenderError> {
        self.renderer.init()?;
        let result = self.main_loop();
        let _ = self.renderer.shutdown();
        self.session_logger.finish(self.session.activated_puzzles.len(), 7);
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
