use std::collections::HashSet;

use crate::{
    dialogue::DialogueSession,
    player::Player,
    world::{entity::Entity, history::PositionHistory, World},
};

use super::event_log::EventLog;

/// Groups all state that is replaced when transitioning between stages.
/// When a new stage loads, a fresh Session is created — no manual field-by-field reset.
pub(super) struct Session {
    pub world: World,
    pub players: [Player; 3],
    pub npc_entities: Vec<Entity>,
    pub encounter_entities: Vec<Entity>,
    pub activated_puzzles: HashSet<u32>,
    pub puzzle_flash: Option<(u32, f32)>,
    pub position_history: PositionHistory,
    pub event_log: EventLog,
    pub dialogue_session: Option<DialogueSession>,
    pub last_ai_tick: f32,
    pub last_companion_tick: f32,
}

impl Session {
    pub fn new(
        world: World,
        players: [Player; 3],
        npc_entities: Vec<Entity>,
        encounter_entities: Vec<Entity>,
    ) -> Self {
        Self {
            world,
            players,
            npc_entities,
            encounter_entities,
            activated_puzzles: HashSet::new(),
            puzzle_flash: None,
            position_history: PositionHistory::new(5.0),
            event_log: EventLog::new(),
            dialogue_session: None,
            last_ai_tick: 0.0,
            last_companion_tick: 0.0,
        }
    }
}
