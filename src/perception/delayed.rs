/// The Delayed — sees the real map but all entities are shown at their
/// positions from several seconds ago. The world has moved on; you haven't.
///
/// Map tiles render correctly (walls, floors, doors are static — they don't
/// change). But NPCs, allies, and encounters appear where they *were*, not
/// where they *are*. The side panel shows a delay indicator and warnings.

use crate::{
    player::Player,
    world::{entity::Entity, history::PositionHistory, World},
};

use super::{
    append_puzzle_progress, cell_from_tile,
    EntityColor, PanelColor, PanelLine, PerceivedEntity, PlayerView,
    stat_bar,
};

/// Base seconds behind the Delayed sees entity positions.
const BASE_DELAY: f32 = 3.0;

/// Look up an entity's position from `delay` seconds ago, falling back to
/// its current world position when no history exists yet.
fn resolve_position(
    entity: Entity,
    world: &World,
    history: &PositionHistory,
    current_time: f32,
    delay: f32,
) -> Option<(f32, f32)> {
    history
        .get_delayed(entity, current_time, delay)
        .or_else(|| world.get_position(entity).map(|p| (p.x, p.y)))
}

pub fn build(
    player: &Player,
    player_entities: &[Entity],
    world: &World,
    history: &PositionHistory,
    current_time: f32,
    delay_extra: f32,
) -> PlayerView {
    let w = world.map.width;
    let h = world.map.height;
    let delay_seconds = BASE_DELAY + delay_extra;

    // --- Map cells — real tiles, no distortion, no fabrication ---
    let mut cells = Vec::with_capacity(w * h);

    for y in 0..h {
        for x in 0..w {
            let visible  = player.fov.is_visible(x, y);
            let revealed = player.fov.is_revealed(x, y);
            let tile = world.map.get(x, y);
            cells.push(cell_from_tile(tile, visible, revealed));
        }
    }

    // --- Entities — rendered at DELAYED positions ---
    let mut entities = Vec::new();

    // Players (self + allies) — self is always current, allies are delayed.
    for &e in player_entities {
        let is_self = e == player.entity;

        let (ex, ey) = if is_self {
            match world.get_position(e) {
                Some(p) => (p.x, p.y),
                None => continue,
            }
        } else {
            match resolve_position(e, world, history, current_time, delay_seconds) {
                Some(pos) => pos,
                None => continue,
            }
        };

        let x = ex as usize;
        let y = ey as usize;
        if x >= w || y >= h { continue; }
        if !player.fov.is_visible(x, y) && !is_self { continue; }

        let color = if is_self { EntityColor::Self_ } else { EntityColor::Ally };
        entities.push(PerceivedEntity {
            col: ex as u16,
            row: ey as u16,
            glyph: '@',
            color,
            is_ghost: false,
        });
    }

    // NPCs — shown at delayed positions.
    for (npc_entity, _npc_pos, marker) in world.all_npcs() {
        let (nx, ny) = match resolve_position(npc_entity, world, history, current_time, delay_seconds) {
            Some(pos) => pos,
            None => continue,
        };

        let x = nx as usize;
        let y = ny as usize;
        if x >= w || y >= h || !player.fov.is_visible(x, y) { continue; }

        let trust = player.trust_for(npc_entity, marker.base_trust);
        let (glyph, color) = if trust >= 0.5 {
            ('W', EntityColor::Npc)
        } else {
            ('?', EntityColor::NpcDoubt)
        };

        entities.push(PerceivedEntity {
            col: nx as u16,
            row: ny as u16,
            glyph,
            color,
            is_ghost: false,
        });
    }

    // Puzzle tiles — static, no delay needed.
    for (_tile_entity, tile_pos, tile) in world.all_puzzle_tiles() {
        let x = tile_pos.x as usize;
        let y = tile_pos.y as usize;
        if x >= w || y >= h || !player.fov.is_visible(x, y) { continue; }

        let glyph = if tile.is_active { '✓' } else { '*' };
        entities.push(PerceivedEntity {
            col: tile_pos.x as u16,
            row: tile_pos.y as u16,
            glyph,
            color: if tile.is_active { EntityColor::Npc } else { EntityColor::NpcDoubt },
            is_ghost: false,
        });
    }

    // --- Side panel ---
    let mut panel = vec![
        PanelLine { text: "[ THE DELAYED ]".into(), color: PanelColor::Cyan },
        PanelLine { text: String::new(), color: PanelColor::Grey },
    ];

    // Delay indicator.
    panel.push(PanelLine {
        text: format!("  Delay: {:.1}s behind reality", delay_seconds),
        color: PanelColor::Yellow,
    });
    panel.push(PanelLine {
        text: "  Entities show old positions".into(),
        color: PanelColor::DarkGrey,
    });

    // NPC trust.
    panel.push(PanelLine { text: String::new(), color: PanelColor::Grey });
    panel.push(PanelLine { text: "NPC TRUST:".into(), color: PanelColor::White });
    for (npc_entity, _, marker) in world.all_npcs() {
        let trust = player.trust_for(npc_entity, marker.base_trust);
        let bar = stat_bar(&marker.name, trust);
        let color = if trust >= 0.7 { PanelColor::Green }
                    else if trust >= 0.4 { PanelColor::Yellow }
                    else { PanelColor::Red };
        panel.push(PanelLine { text: bar, color });
    }

    // Hidden state.
    panel.push(PanelLine { text: String::new(), color: PanelColor::Grey });
    panel.push(PanelLine { text: "─────────────────────".into(), color: PanelColor::DarkGrey });
    let hs = &player.hidden_state;
    panel.push(PanelLine { text: stat_bar("T", hs.bar(hs.truth)),    color: PanelColor::Green });
    panel.push(PanelLine { text: stat_bar("C", hs.bar(hs.chaos)),    color: PanelColor::Red });
    panel.push(PanelLine { text: stat_bar("I", hs.bar(hs.illusion)), color: PanelColor::Yellow });
    panel.push(PanelLine { text: stat_bar("B", hs.bar(hs.balance)),  color: PanelColor::Cyan });

    append_puzzle_progress(&mut panel, world);

    PlayerView {
        role: player.role,
        map_width: w,
        map_height: h,
        cells,
        entities,
        panel_lines: panel,
    }
}
