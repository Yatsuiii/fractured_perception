/// The Perception system — the thematic core of Fractured Perception.
///
/// One authoritative World produces three completely different PlayerViews.
/// The renderer never touches World directly; it only draws what a PlayerView says.
///
/// Each role's builder lives in its own module:
///   blind.rs        → no map, only sound cues
///   analyst.rs      → full map + fabricated tiles that look identical to real ones
///   hallucinating.rs → distorted map, doubled entities, unstable reality

mod analyst;
mod blind;
mod hallucinating;

use crate::{
    map::Tile,
    player::{Player, Role},
    world::{entity::Entity, World},
};

// --- Output types (what the renderer consumes) ---

#[derive(Clone, Copy)]
pub enum CellColor {
    Hidden,      // never seen / Blind's entire world
    Memory,      // revealed but not currently visible
    Floor,       // visible real floor
    Wall,        // visible real wall
    Door,        // visible door between rooms
    Fabricated,  // Analyst: tile that might not be real (visually = Floor, subtly different)
    Distorted,   // Hallucinating: tile the mind got wrong
}

#[derive(Clone)]
pub struct PerceivedCell {
    pub glyph: char,
    pub color: CellColor,
}

pub struct PerceivedEntity {
    pub col: u16,
    pub row: u16,
    pub glyph: char,
    pub color: EntityColor,
    pub is_ghost: bool,
}

#[derive(Clone, Copy)]
pub enum EntityColor {
    Self_,      // the observer themselves
    Ally,       // other players
    Npc,        // NPC, trusted
    NpcDoubt,   // NPC, trust < 0.5
    Ghost,      // Hallucinating: ghost duplicate
    AuraTrust,  // Blind: NPC aura, high trust (green)
    AuraDoubt,  // Blind: NPC aura, low trust (red)
    AuraAlly,   // Blind: teammate aura (cyan)
    AuraPuzzle, // Blind: unsolved puzzle pulse (yellow)
}

pub struct PanelLine {
    pub text: String,
    pub color: PanelColor,
}

#[derive(Clone, Copy)]
pub enum PanelColor {
    White,
    Grey,
    DarkGrey,
    Yellow,
    Red,
    Green,
    Cyan,
}

pub struct PlayerView {
    pub role: Role,
    pub map_width: usize,
    pub map_height: usize,
    /// Row-major: index = y * map_width + x
    pub cells: Vec<PerceivedCell>,
    pub entities: Vec<PerceivedEntity>,
    pub panel_lines: Vec<PanelLine>,
}

// --- Entry point ---

pub fn build_view(
    player: &Player,
    player_entities: &[Entity],
    world: &World,
) -> PlayerView {
    match player.role {
        Role::Blind         => blind::build(player, player_entities, world),
        Role::VisualAnalyst => analyst::build(player, player_entities, world),
        Role::Hallucinating => hallucinating::build(player, player_entities, world),
    }
}

// --- Deterministic noise helpers ---

/// Fast integer hash for deterministic per-tile effects.
fn tile_hash(x: usize, y: usize, seed: u64) -> u64 {
    let mut h = seed;
    h ^= (x as u64).wrapping_mul(0x9e3779b97f4a7c15);
    h ^= (y as u64).wrapping_mul(0x517cc1b727220a95);
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^ (h >> 27)
}

/// True for ~25 % of floor tiles — these are the Analyst's fabricated tiles.
fn is_fabricated(map_seed: u64, x: usize, y: usize) -> bool {
    tile_hash(x, y, map_seed) % 100 < 25
}

/// True for ~18 % of tiles — these are distorted in the Hallucinating view.
pub fn is_distorted(map_seed: u64, x: usize, y: usize) -> bool {
    tile_hash(x, y, map_seed.wrapping_add(0xDEAD)) % 100 < 18
}

/// Ghost offset for a given entity — stable per entity, shifts between turns.
fn ghost_offset(entity: Entity) -> (i32, i32) {
    let id = entity.0 as i32;
    let dx = match id % 3 { 0 => -1, 1 => 1, _ => 0 };
    let dy = match id % 7 { n if n < 3 => -1, n if n < 6 => 1, _ => 0 };
    (dx, dy)
}

fn direction_arrow(dx: f32, dy: f32) -> &'static str {
    let angle = dy.atan2(dx).to_degrees();
    match angle as i32 {
        -22..=22   => "→",
        23..=67    => "↘",
        68..=112   => "↓",
        113..=157  => "↙",
        -67..=-23  => "↗",
        -112..=-68 => "↑",
        -157..=-113=> "↖",
        _          => "←",
    }
}

// --- Shared cell helpers ---

fn hidden_cell() -> PerceivedCell {
    PerceivedCell { glyph: ' ', color: CellColor::Hidden }
}

fn cell_from_tile(tile: Tile, visible: bool, revealed: bool) -> PerceivedCell {
    if visible {
        match tile {
            Tile::Wall  => PerceivedCell { glyph: '#', color: CellColor::Wall },
            Tile::Floor => PerceivedCell { glyph: '.', color: CellColor::Floor },
            Tile::Door  => PerceivedCell { glyph: '+', color: CellColor::Door },
        }
    } else if revealed {
        PerceivedCell { glyph: tile.glyph(), color: CellColor::Memory }
    } else {
        hidden_cell()
    }
}

// --- Shared entity helpers ---

fn build_entities_standard(
    player: &Player,
    player_entities: &[Entity],
    world: &World,
) -> Vec<PerceivedEntity> {
    let w = world.map.width;
    let h = world.map.height;
    let mut out = Vec::new();

    for &e in player_entities {
        let Some(pos) = world.get_position(e) else { continue };
        let x = pos.x as usize;
        let y = pos.y as usize;
        if x >= w || y >= h || !player.fov.is_visible(x, y) { continue; }

        let color = if e == player.entity { EntityColor::Self_ } else { EntityColor::Ally };
        out.push(PerceivedEntity {
            col: pos.x as u16, row: pos.y as u16,
            glyph: '@', color, is_ghost: false,
        });
    }

    for (_tile_entity, tile_pos, tile) in world.all_puzzle_tiles() {
        let x = tile_pos.x as usize;
        let y = tile_pos.y as usize;
        if x >= w || y >= h || !player.fov.is_visible(x, y) { continue; }

        let glyph = if tile.is_active { '✓' } else { '*' };
        out.push(PerceivedEntity {
            col: tile_pos.x as u16,
            row: tile_pos.y as u16,
            glyph,
            color: if tile.is_active { EntityColor::Npc } else { EntityColor::NpcDoubt },
            is_ghost: false,
        });
    }

    out
}

// --- Shared panel helpers ---

fn append_puzzle_progress(panel: &mut Vec<PanelLine>, world: &World) {
    let total = world.all_puzzle_tiles().count();
    let active = world.all_puzzle_tiles().filter(|(_, _, tile)| tile.is_active).count();
    panel.push(PanelLine { text: String::new(), color: PanelColor::Grey });
    panel.push(PanelLine {
        text: format!("Puzzle progress: {}/{} activated", active, total),
        color: PanelColor::White,
    });
}

fn stat_bar(label: &str, v: f32) -> String {
    let width = 8usize;
    let filled = (v * width as f32) as usize;
    format!("  {}:[{}{}] {:.2}", label, "█".repeat(filled), "░".repeat(width - filled), v)
}
