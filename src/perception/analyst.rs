use crate::{
    map::Tile,
    player::Player,
    world::{entity::Entity, World},
};

use super::{
    append_puzzle_progress, build_entities_standard, hidden_cell, is_fabricated,
    CellColor, EntityColor, PanelColor, PanelLine, PerceivedCell, PerceivedEntity, PlayerView,
};

pub fn build(player: &Player, player_entities: &[Entity], world: &World) -> PlayerView {
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
                // Doors are never fabricated — they're structural landmarks.
                let fabricated = true_tile == Tile::Floor && is_fabricated(seed, x, y);
                let color = if fabricated { CellColor::Fabricated } else {
                    match true_tile {
                        Tile::Floor => CellColor::Floor,
                        Tile::Wall  => CellColor::Wall,
                        Tile::Door  => CellColor::Door,
                    }
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
