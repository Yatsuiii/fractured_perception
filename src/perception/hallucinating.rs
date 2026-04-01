use crate::{
    map::Tile,
    player::Player,
    world::{entity::Entity, World},
};

use super::{
    append_puzzle_progress, cell_from_tile, ghost_offset, hidden_cell, is_distorted, is_distorted_wide,
    CellColor, EntityColor, PanelColor, PanelLine, PerceivedCell, PerceivedEntity, PlayerView,
};

pub fn build(player: &Player, player_entities: &[Entity], world: &World, double_distortion: bool) -> PlayerView {
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
                let distorted = if double_distortion {
                    is_distorted_wide(seed, x, y)
                } else {
                    is_distorted(seed, x, y)
                };
                if distorted {
                    // Flip floor ↔ wall visually. Navigation uses the true map.
                    // Doors shimmer — they look like walls when distorted.
                    let (glyph, color) = match true_tile {
                        Tile::Floor => ('#', CellColor::Distorted),
                        Tile::Wall  => ('.', CellColor::Distorted),
                        Tile::Door  => ('#', CellColor::Distorted),
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

    // Encounters — real + ghost duplicate, active ones only.
    for (enc_entity, enc_pos, marker) in world.all_encounters() {
        if !marker.is_active() { continue; }
        let x = enc_pos.x as usize;
        let y = enc_pos.y as usize;
        if x >= w || y >= h || !player.fov.is_visible(x, y) { continue; }

        let glyph = marker.kind.glyph();

        // Real position.
        entities.push(PerceivedEntity {
            col: enc_pos.x as u16,
            row: enc_pos.y as u16,
            glyph,
            color: EntityColor::Npc,
            is_ghost: false,
        });

        // Ghost duplicate at offset.
        let (gox, goy) = ghost_offset(enc_entity);
        let gc = enc_pos.x as i32 + gox;
        let gr = enc_pos.y as i32 + goy;
        if gc >= 0 && gr >= 0 && (gc as usize) < w && (gr as usize) < h {
            entities.push(PerceivedEntity {
                col: gc as u16,
                row: gr as u16,
                glyph,
                color: EntityColor::Ghost,
                is_ghost: true,
            });
        }
    }

    // Stability = how much of what they see is real.
    let visible_count = (0..h).flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| player.fov.is_visible(x, y))
        .count()
        .max(1);
    let distorted_count = (0..h).flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| player.fov.is_visible(x, y) && if double_distortion {
            is_distorted_wide(seed, x, y)
        } else {
            is_distorted(seed, x, y)
        })
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
