/// The Perception system — the thematic core of Fractured Perception.
///
/// One authoritative World produces three completely different PlayerViews.
/// The renderer never touches World directly; it only draws what a PlayerView says.
///
/// Blind        → no map, only sound cues
/// VisualAnalyst → full map + fabricated tiles that look identical to real ones
/// Hallucinating → distorted map, doubled entities, unstable reality
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
    Self_,     // the observer themselves
    Ally,      // other players
    Npc,       // NPC, trusted
    NpcDoubt,  // NPC, trust < 0.5
    Ghost,     // Hallucinating: ghost duplicate
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
    player_entities: &[Entity], // all three player entities
    world: &World,
) -> PlayerView {
    match player.role {
        Role::Blind => build_blind(player, player_entities, world),
        Role::VisualAnalyst => build_analyst(player, player_entities, world),
        Role::Hallucinating => build_hallucinating(player, player_entities, world),
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

// --- Blank cell helpers ---

fn hidden_cell() -> PerceivedCell {
    PerceivedCell { glyph: ' ', color: CellColor::Hidden }
}

fn cell_from_tile(tile: Tile, visible: bool, revealed: bool) -> PerceivedCell {
    if visible {
        match tile {
            Tile::Wall  => PerceivedCell { glyph: '#', color: CellColor::Wall },
            Tile::Floor => PerceivedCell { glyph: '.', color: CellColor::Floor },
        }
    } else if revealed {
        PerceivedCell { glyph: tile.glyph(), color: CellColor::Memory }
    } else {
        hidden_cell()
    }
}

// --- THE BLIND ---

fn build_blind(player: &Player, player_entities: &[Entity], world: &World) -> PlayerView {
    let w = world.map.width;
    let h = world.map.height;

    // Entire map is dark — the Blind sees nothing.
    let cells = vec![PerceivedCell { glyph: '░', color: CellColor::Hidden }; w * h];

    // Only their own entity is rendered.
    let mut entities = Vec::new();
    if let Some(pos) = world.get_position(player.entity) {
        entities.push(PerceivedEntity {
            col: pos.x as u16,
            row: pos.y as u16,
            glyph: '@',
            color: EntityColor::Self_,
            is_ghost: false,
        });
    }

    // Side panel: sound cues from nearby NPCs and players.
    let mut panel = vec![
        PanelLine { text: "[ THE BLIND ]".into(),  color: PanelColor::Cyan },
        PanelLine { text: String::new(),            color: PanelColor::Grey },
        PanelLine { text: "YOU HEAR:".into(),       color: PanelColor::White },
        PanelLine { text: String::new(),            color: PanelColor::Grey },
    ];

    let mut heard_anything = false;

    if let Some(obs) = world.get_position(player.entity) {
        let (ox, oy) = (obs.x, obs.y);

        // NPCs within sound radius (larger than visual FOV).
        for (_, npc_pos, marker) in world.all_npcs() {
            let dx = npc_pos.x - ox;
            let dy = npc_pos.y - oy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 12.0 {
                let arrow = direction_arrow(dx, dy);
                let intensity = if dist < 4.0 { "close" } else if dist < 8.0 { "near" } else { "far" };
                panel.push(PanelLine {
                    text: format!("  {} {} [{}]", arrow, marker.name, intensity),
                    color: PanelColor::Yellow,
                });
                panel.push(PanelLine { text: "    \"...\"".into(), color: PanelColor::Grey });
                heard_anything = true;
            }
        }

        // Footsteps from other players.
        for &e in player_entities {
            if e == player.entity { continue; }
            if let Some(p) = world.get_position(e) {
                let dx = p.x - ox;
                let dy = p.y - oy;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 8.0 {
                    let arrow = direction_arrow(dx, dy);
                    panel.push(PanelLine {
                        text: format!("  {} Footsteps", arrow),
                        color: PanelColor::Grey,
                    });
                    heard_anything = true;
                }
            }
        }
    }

    if !heard_anything {
        panel.push(PanelLine { text: "  (silence)".into(), color: PanelColor::DarkGrey });
    }

    // T/C/I/B readout — Blind has the most self-reflection.
    let hs = &player.hidden_state;
    panel.push(PanelLine { text: String::new(), color: PanelColor::Grey });
    panel.push(PanelLine { text: "─────────────────────".into(), color: PanelColor::DarkGrey });
    panel.push(PanelLine { text: stat_bar("T", hs.bar(hs.truth)),   color: PanelColor::Green  });
    panel.push(PanelLine { text: stat_bar("C", hs.bar(hs.chaos)),   color: PanelColor::Red    });
    panel.push(PanelLine { text: stat_bar("I", hs.bar(hs.illusion)),color: PanelColor::Yellow });
    panel.push(PanelLine { text: stat_bar("B", hs.bar(hs.balance)), color: PanelColor::Cyan   });

    append_puzzle_progress(&mut panel, world);

    PlayerView { role: player.role, map_width: w, map_height: h, cells, entities, panel_lines: panel }
}

// --- THE VISUAL ANALYST ---

fn build_analyst(player: &Player, player_entities: &[Entity], world: &World) -> PlayerView {
    let w = world.map.width;
    let h = world.map.height;
    let seed = world.map.seed;

    let mut cells = Vec::with_capacity(w * h);

    for y in 0..h {
        for x in 0..w {
            let visible  = player.fov.is_visible(x, y);
            let revealed = player.fov.is_revealed(x, y);
            let true_tile = world.map.get(x, y);

            if visible {
                // Fabricated tiles look like floors but are secretly walls.
                // Visually identical — the Analyst cannot tell by looking.
                let fabricated = true_tile == Tile::Floor && is_fabricated(seed, x, y);
                let color = if fabricated { CellColor::Fabricated } else {
                    match true_tile { Tile::Floor => CellColor::Floor, Tile::Wall => CellColor::Wall }
                };
                cells.push(PerceivedCell { glyph: true_tile.glyph(), color });
            } else if revealed {
                cells.push(PerceivedCell { glyph: true_tile.glyph(), color: CellColor::Memory });
            } else {
                cells.push(hidden_cell());
            }
        }
    }

    // Entities
    let mut entities = build_entities_standard(player, player_entities, world);

    // NPCs with trust-based appearance.
    for (npc_entity, npc_pos, marker) in world.all_npcs() {
        let x = npc_pos.x as usize;
        let y = npc_pos.y as usize;
        if x < w && y < h && player.fov.is_visible(x, y) {
            let trust = player.trust_for(npc_entity, marker.base_trust);
            let (glyph, color) = if trust >= 0.5 {
                ('W', EntityColor::Npc)
            } else {
                ('?', EntityColor::NpcDoubt)
            };
            entities.push(PerceivedEntity {
                col: npc_pos.x as u16,
                row: npc_pos.y as u16,
                glyph,
                color,
                is_ghost: false,
            });

            // Low trust: entity flickers (skip every other frame via time-based toggle).
            // Simple approximation: don't render at all when trust < 0.3.
            if trust < 0.3 {
                entities.pop();
            }
        }
    }

    // Side panel — trust levels + fabrication warning.
    let mut panel = vec![
        PanelLine { text: "[ VISUAL ANALYST ]".into(), color: PanelColor::Cyan },
        PanelLine { text: String::new(),               color: PanelColor::Grey },
        PanelLine { text: "NPC TRUST:".into(),         color: PanelColor::White },
    ];

    for (npc_entity, _, marker) in world.all_npcs() {
        let t = player.trust_for(npc_entity, marker.base_trust);
        let bar_w = 8usize;
        let filled = (t * bar_w as f32) as usize;
        let bar = format!("{}{}",
            "█".repeat(filled),
            "░".repeat(bar_w - filled),
        );
        let color = if t >= 0.7 { PanelColor::Green }
                    else if t >= 0.4 { PanelColor::Yellow }
                    else { PanelColor::Red };
        panel.push(PanelLine {
            text: format!("  {:<12} {:.1}", marker.name, t),
            color,
        });
        panel.push(PanelLine { text: format!("  [{}]", bar), color });
    }

    // Count fabricated tiles currently visible.
    let fabricated_nearby = (0..h).flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            player.fov.is_visible(x, y)
                && world.map.get(x, y) == Tile::Floor
                && is_fabricated(seed, x, y)
        })
        .count();

    panel.push(PanelLine { text: String::new(), color: PanelColor::Grey });
    panel.push(PanelLine { text: "─────────────────────".into(), color: PanelColor::DarkGrey });
    if fabricated_nearby > 0 {
        panel.push(PanelLine {
            text: format!("  ⚠ {} anomal{} nearby", fabricated_nearby,
                if fabricated_nearby == 1 { "y" } else { "ies" }),
            color: PanelColor::Yellow,
        });
        panel.push(PanelLine {
            text: "  (some tiles are false)".into(),
            color: PanelColor::DarkGrey,
        });
    } else {
        panel.push(PanelLine { text: "  No anomalies visible".into(), color: PanelColor::DarkGrey });
    }

    append_puzzle_progress(&mut panel, world);

    PlayerView { role: player.role, map_width: w, map_height: h, cells, entities, panel_lines: panel }
}

// --- THE HALLUCINATING ---

fn build_hallucinating(player: &Player, player_entities: &[Entity], world: &World) -> PlayerView {
    let w = world.map.width;
    let h = world.map.height;
    let seed = world.map.seed;

    let mut cells = Vec::with_capacity(w * h);

    for y in 0..h {
        for x in 0..w {
            let visible  = player.fov.is_visible(x, y);
            let revealed = player.fov.is_revealed(x, y);
            let true_tile = world.map.get(x, y);

            if visible {
                let distorted = is_distorted(seed, x, y);
                if distorted {
                    // Flip floor ↔ wall visually. Navigation uses the true map.
                    let (glyph, color) = match true_tile {
                        Tile::Floor => ('#', CellColor::Distorted), // floor looks like wall
                        Tile::Wall  => ('.', CellColor::Distorted), // wall looks like floor
                    };
                    cells.push(PerceivedCell { glyph, color });
                } else {
                    cells.push(cell_from_tile(true_tile, true, true));
                }
            } else if revealed {
                cells.push(PerceivedCell { glyph: true_tile.glyph(), color: CellColor::Memory });
            } else {
                cells.push(hidden_cell());
            }
        }
    }

    // Entities — each one appears twice (real + ghost duplicate).
    let mut entities = Vec::new();

    for &e in player_entities {
        let Some(pos) = world.get_position(e) else { continue };
        let x = pos.x as usize;
        let y = pos.y as usize;
        if x >= w || y >= h || !player.fov.is_visible(x, y) { continue; }

        let (glyph, color) = if e == player.entity {
            ('@', EntityColor::Self_)
        } else {
            ('@', EntityColor::Ally)
        };

        // Real position.
        entities.push(PerceivedEntity {
            col: pos.x as u16, row: pos.y as u16,
            glyph, color, is_ghost: false,
        });

        // Ghost duplicate at offset.
        let (gox, goy) = ghost_offset(e);
        let gc = pos.x as i32 + gox;
        let gr = pos.y as i32 + goy;
        if gc >= 0 && gr >= 0 && (gc as usize) < w && (gr as usize) < h {
            entities.push(PerceivedEntity {
                col: gc as u16, row: gr as u16,
                glyph, color: EntityColor::Ghost, is_ghost: true,
            });
        }
    }

    // NPCs also appear doubled.
    for (npc_entity, npc_pos, _) in world.all_npcs() {
        let x = npc_pos.x as usize;
        let y = npc_pos.y as usize;
        if x >= w || y >= h || !player.fov.is_visible(x, y) { continue; }

        entities.push(PerceivedEntity {
            col: npc_pos.x as u16, row: npc_pos.y as u16,
            glyph: 'W', color: EntityColor::Npc, is_ghost: false,
        });

        let (gox, goy) = ghost_offset(npc_entity);
        let gc = npc_pos.x as i32 + gox;
        let gr = npc_pos.y as i32 + goy;
        if gc >= 0 && gr >= 0 && (gc as usize) < w && (gr as usize) < h {
            entities.push(PerceivedEntity {
                col: gc as u16, row: gr as u16,
                glyph: 'W', color: EntityColor::Ghost, is_ghost: true,
            });
        }
    }

    // Stability = how much of what they see is real.
    let visible_count = (0..h).flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| player.fov.is_visible(x, y))
        .count()
        .max(1);
    let distorted_count = (0..h).flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| player.fov.is_visible(x, y) && is_distorted(seed, x, y))
        .count();
    let stability = 1.0 - (distorted_count as f32 / visible_count as f32);

    let ghost_count = entities.iter().filter(|e| e.is_ghost).count();

    let stab_bar_w = 12usize;
    let filled = (stability * stab_bar_w as f32) as usize;
    let stab_color = if stability > 0.7 { PanelColor::Green }
                     else if stability > 0.4 { PanelColor::Yellow }
                     else { PanelColor::Red };

    let mut panel = vec![
        PanelLine { text: "[ HALLUCINATING ]".into(),  color: PanelColor::Cyan },
        PanelLine { text: String::new(),               color: PanelColor::Grey },
        PanelLine { text: "STABILITY:".into(),         color: PanelColor::White },
        PanelLine {
            text: format!("  [{}{}]  {:.0}%",
                "█".repeat(filled),
                "░".repeat(stab_bar_w - filled),
                stability * 100.0),
            color: stab_color,
        },
        PanelLine { text: String::new(),               color: PanelColor::Grey },
        PanelLine {
            text: format!("  Ghosts seen: {}", ghost_count),
            color: PanelColor::Red,
        },
        PanelLine { text: String::new(),               color: PanelColor::Grey },
        PanelLine { text: "─────────────────────".into(), color: PanelColor::DarkGrey },
        PanelLine { text: "  [reality is uncertain]".into(), color: PanelColor::DarkGrey },
    ];

    append_puzzle_progress(&mut panel, world);

    PlayerView { role: player.role, map_width: w, map_height: h, cells, entities, panel_lines: panel }
}

fn append_puzzle_progress(panel: &mut Vec<PanelLine>, world: &World) {
    let total = world.all_puzzle_tiles().count();
    let active = world.all_puzzle_tiles().filter(|(_, _, tile)| tile.is_active).count();
    panel.push(PanelLine { text: String::new(), color: PanelColor::Grey });
    panel.push(PanelLine {
        text: format!("Puzzle progress: {}/{} activated", active, total),
        color: PanelColor::White,
    });
}

// --- Shared helpers ---

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

fn stat_bar(label: &str, v: f32) -> String {
    let width = 8usize;
    let filled = (v * width as f32) as usize;
    format!("  {}:[{}{}] {:.2}", label, "█".repeat(filled), "░".repeat(width - filled), v)
}
