pub mod thresholds;

use crate::{player::Role, state::GameState, world::entity::Entity};

use thresholds::Threshold;

pub enum Event {
    PlayerMoved { entity: Entity, x: f32, y: f32 },
    EntityDied { entity: Entity },
    PuzzleActivated { sequence_id: u32 },
    StateChange { to: GameState },
    Ping { from_role: Role },
    TrustChanged { npc: Entity, delta: f32, reason: TrustReason },
    DialogueStarted { npc: Entity },
    DialogueEnded { npc: Entity },
    EncounterResolved { entity: Entity },
    ThresholdCrossed { threshold: Threshold },
}

#[derive(Debug, Clone, Copy)]
pub enum TrustReason {
    PuzzleSolved,
    NpcProximity,
    PingNearby,
    Dialogue,
}

pub struct EventBus {
    queue: Vec<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn emit(&mut self, event: Event) {
        self.queue.push(event);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.queue.drain(..)
    }
}
