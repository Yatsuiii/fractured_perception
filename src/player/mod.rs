pub mod hidden_state;
pub mod role;

pub use hidden_state::HiddenState;
pub use role::Role;

use std::collections::HashMap;

use crate::{fov::Fov, world::entity::Entity};

pub struct Player {
    pub entity: Entity,
    pub role: Role,
    pub hidden_state: HiddenState,
    /// Per-NPC trust level (0.0 = no trust, 1.0 = full trust).
    pub npc_trust: HashMap<Entity, f32>,
    /// This player's own FOV — tracks both current visibility and reveal history.
    pub fov: Fov,
}

impl Player {
    pub fn new(entity: Entity, role: Role, map_width: usize, map_height: usize) -> Self {
        Self {
            entity,
            role,
            hidden_state: HiddenState::new(),
            npc_trust: HashMap::new(),
            fov: Fov::new(map_width, map_height),
        }
    }

    /// Returns trust for `npc`, falling back to the NPC's own `base_trust` on first access.
    pub fn trust_for(&self, npc: Entity, base_trust: f32) -> f32 {
        *self.npc_trust.get(&npc).unwrap_or(&base_trust)
    }

    pub fn adjust_trust(&mut self, npc: Entity, delta: f32, base_trust: f32) {
        let t = self.npc_trust.entry(npc).or_insert(base_trust);
        *t = (*t + delta).clamp(0.0, 1.0);
    }
}
