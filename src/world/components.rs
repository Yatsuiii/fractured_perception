#[derive(Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// Marks an entity as a Mirror NPC.
/// NPCs appear completely differently to each role — see perception module.
pub struct NpcMarker {
    pub name: &'static str,
    /// Base trust level (0.0–1.0). Per-player trust overrides stored in Player.
    pub base_trust: f32,
}

/// Marks a tile entity as part of the Echo Chamber puzzle.
pub struct PuzzleTile {
    /// Position in the canonical activation sequence.
    pub sequence_id: u32,
    pub is_active: bool,
}
