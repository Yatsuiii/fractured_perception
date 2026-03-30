use crate::{
    player::Player,
    world::{entity::Entity, World},
};

use super::{
    append_puzzle_progress, direction_arrow, stat_bar,
    CellColor, EntityColor, PanelColor, PanelLine, PerceivedCell, PerceivedEntity, PlayerView,
};

pub fn build(player: &Player, player_entities: &[Entity], world: &World) -> PlayerView {
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
