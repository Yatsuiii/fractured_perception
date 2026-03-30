use crate::{state::GameState, world::entity::Entity};

pub enum Event {
    PlayerMoved { entity: Entity, x: f32, y: f32 },
    EntityDied { entity: Entity },
    PuzzleActivated { sequence_id: u32 },
    StateChange { to: GameState },
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
