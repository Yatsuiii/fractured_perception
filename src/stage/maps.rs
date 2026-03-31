/// Per-stage map generators.
///
/// Each stage has a unique layout that reinforces its theme.
/// All maps use the same Map struct and Tile enum — the visual
/// differences come from layout, not tile types.

use crate::map::{Map, Tile};

const MAP_W: usize = 80;
const MAP_H: usize = 34;

// Shared helper — carve a rectangular room.
fn carve_room(map: &mut Map, x: usize, y: usize, w: usize, h: usize) {
    for ry in y..y + h {
        for rx in x..x + w {
            if rx < map.width && ry < map.height {
                map.set(rx, ry, Tile::Floor);
            }
        }
    }
}

fn carve_h_corridor(map: &mut Map, x1: usize, x2: usize, y: usize) {
    let (lo, hi) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    for x in lo..=hi {
        if x < map.width && y < map.height {
            map.set(x, y, Tile::Floor);
        }
    }
}

fn carve_v_corridor(map: &mut Map, y1: usize, y2: usize, x: usize) {
    let (lo, hi) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
    for y in lo..=hi {
        if x < map.width && y < map.height {
            map.set(x, y, Tile::Floor);
        }
    }
}

fn place_door(map: &mut Map, x: usize, y: usize) {
    if x < map.width && y < map.height {
        map.set(x, y, Tile::Door);
    }
}

// ============================================================
// Stage 1: The Shattered Halls
// Fragmented rooms connected by broken corridors.
// Linear left-to-right with vertical branches.
// ============================================================

pub fn generate_shattered_halls() -> Map {
    let mut map = Map::new(MAP_W, MAP_H, 0xDEADBEEF_CAFE1337);

    // Rooms — broken, irregular sizes to feel "shattered"
    let rooms: &[(usize, usize, usize, usize)] = &[
        (2,  2,  10, 6),   // 0: Entrance
        (16, 2,  8,  5),   // 1: Junction
        (28, 2,  12, 7),   // 2: Observation
        (44, 2,  9,  6),   // 3: Alcove
        (58, 2,  14, 8),   // 4: Signal room
        (2,  12, 12, 7),   // 5: Storage
        (18, 14, 10, 6),   // 6: Relay
        (32, 12, 14, 8),   // 7: Central hall
        (50, 14, 10, 6),   // 8: Generator
        (2,  24, 14, 6),   // 9: Collapsed wing
        (20, 24, 10, 8),   // 10: Echo chamber
        (34, 26, 12, 6),   // 11: Den
        (50, 24, 14, 8),   // 12: Terminus
        (66, 14, 10, 7),   // 13: Tower
        (66, 26, 10, 6),   // 14: Shrine
    ];

    for &(x, y, w, h) in rooms {
        carve_room(&mut map, x, y, w, h);
    }

    // Corridors — L-shaped connections
    let connections: &[(usize, usize)] = &[
        (0, 1), (1, 2), (2, 3), (3, 4),
        (0, 5), (5, 6), (6, 7), (7, 8),
        (5, 9), (9, 10), (10, 11), (11, 12),
        (4, 13), (8, 13), (13, 14), (12, 14),
        (2, 7), (7, 11),
    ];

    let centers: Vec<(usize, usize)> = rooms.iter()
        .map(|&(x, y, w, h)| (x + w / 2, y + h / 2))
        .collect();

    for &(a, b) in connections {
        let (x1, y1) = centers[a];
        let (x2, y2) = centers[b];
        carve_h_corridor(&mut map, x1, x2, y1);
        carve_v_corridor(&mut map, y1, y2, x2);
    }

    // Doors at key transitions
    place_door(&mut map, 12, 5);
    place_door(&mut map, 7, 12);
    place_door(&mut map, 32, 15);
    place_door(&mut map, 50, 28);

    map
}

// ============================================================
// Stage 2: The Drowned Archive
// Wide flooded chambers connected by narrow passages.
// Vertical flow — top to bottom, water flows down.
// ============================================================

pub fn generate_drowned_archive() -> Map {
    let mut map = Map::new(MAP_W, MAP_H, 0xA4C1E_D400E001);

    // Large chambers — archives are spacious
    carve_room(&mut map, 2, 2, 20, 8);     // Reading hall (entrance)
    carve_room(&mut map, 30, 2, 18, 8);    // Catalog room
    carve_room(&mut map, 55, 2, 20, 10);   // Upper stacks

    carve_room(&mut map, 2, 14, 15, 10);   // Flooded wing
    carve_room(&mut map, 25, 13, 20, 9);   // Central reservoir
    carve_room(&mut map, 55, 16, 18, 8);   // Dry shelf

    carve_room(&mut map, 2, 28, 18, 5);    // Drainage
    carve_room(&mut map, 28, 26, 22, 7);   // Deep archive
    carve_room(&mut map, 58, 28, 16, 5);   // Exit passage

    // Narrow connecting passages — feels like water channels
    carve_h_corridor(&mut map, 22, 30, 6);
    carve_h_corridor(&mut map, 48, 55, 6);
    carve_v_corridor(&mut map, 10, 14, 10);
    carve_v_corridor(&mut map, 10, 13, 35);
    carve_v_corridor(&mut map, 10, 16, 65);
    carve_v_corridor(&mut map, 24, 28, 10);
    carve_v_corridor(&mut map, 22, 26, 35);
    carve_v_corridor(&mut map, 24, 28, 65);
    carve_h_corridor(&mut map, 20, 28, 30);
    carve_h_corridor(&mut map, 50, 58, 30);

    place_door(&mut map, 22, 6);
    place_door(&mut map, 10, 14);
    place_door(&mut map, 35, 13);
    place_door(&mut map, 35, 22);
    place_door(&mut map, 10, 24);

    map
}

// ============================================================
// Stage 3: The Hollow Garden
// Organic, winding layout. Rooms are irregular "clearings"
// connected by curving paths. East-west flow.
// ============================================================

pub fn generate_hollow_garden() -> Map {
    let mut map = Map::new(MAP_W, MAP_H, 0x6A4DE0_0110001);

    // Clearings — organic, slightly offset
    carve_room(&mut map, 2,  12, 10, 10);  // Entry clearing
    carve_room(&mut map, 16, 6,  8,  8);   // Thorn patch
    carve_room(&mut map, 16, 20, 8,  8);   // Root hollow
    carve_room(&mut map, 30, 4,  12, 7);   // Canopy hall
    carve_room(&mut map, 30, 18, 12, 10);  // Seed bed
    carve_room(&mut map, 48, 10, 10, 8);   // Vine crossing
    carve_room(&mut map, 48, 22, 10, 8);   // Fungal grotto
    carve_room(&mut map, 64, 8,  12, 10);  // Bloom chamber
    carve_room(&mut map, 64, 22, 12, 10);  // Decay ring
    carve_room(&mut map, 68, 14, 8,  8);   // Gate clearing (exit)

    // Winding paths between clearings
    carve_h_corridor(&mut map, 12, 16, 17);
    carve_v_corridor(&mut map, 14, 17, 20);
    carve_v_corridor(&mut map, 10, 20, 20);

    carve_h_corridor(&mut map, 24, 30, 10);
    carve_h_corridor(&mut map, 24, 30, 24);

    carve_h_corridor(&mut map, 42, 48, 14);
    carve_h_corridor(&mut map, 42, 48, 26);

    carve_v_corridor(&mut map, 11, 18, 36);

    carve_h_corridor(&mut map, 58, 64, 14);
    carve_h_corridor(&mut map, 58, 64, 26);

    carve_v_corridor(&mut map, 18, 22, 70);

    place_door(&mut map, 12, 17);
    place_door(&mut map, 42, 14);
    place_door(&mut map, 58, 14);
    place_door(&mut map, 68, 14);

    map
}

// ============================================================
// Stage 4: The Mirror Vault
// Perfectly symmetrical layout — left and right halves mirror
// each other. Central axis runs vertically down the middle.
// ============================================================

pub fn generate_mirror_vault() -> Map {
    let mut map = Map::new(MAP_W, MAP_H, 0x1EE0E_AA01D001);

    let cx: usize = MAP_W / 2; // center axis = column 40

    // Build the left half, then mirror it.
    // Left rooms:
    let left_rooms: &[(usize, usize, usize, usize)] = &[
        (5,  2,  12, 6),   // Upper left chamber
        (5,  12, 10, 8),   // Mid left chamber
        (5,  24, 14, 8),   // Lower left chamber
        (22, 6,  8,  6),   // Inner left upper
        (20, 16, 10, 6),   // Inner left mid
        (22, 26, 8,  5),   // Inner left lower
    ];

    // Central spine rooms (not mirrored)
    carve_room(&mut map, 36, 2,  8, 5);    // Top center
    carve_room(&mut map, 35, 14, 10, 6);   // Center vault
    carve_room(&mut map, 36, 28, 8, 5);    // Bottom center (gate)

    // Carve left rooms and their mirrors
    for &(x, y, w, h) in left_rooms {
        carve_room(&mut map, x, y, w, h);
        // Mirror: reflect x across center axis
        let mirror_x = cx + (cx - x - w);
        carve_room(&mut map, mirror_x, y, w, h);
    }

    // Left corridors + mirrors
    let left_corridors: &[(usize, usize, usize, usize, usize, usize)] = &[
        // (hx1, hx2, hy, vx, vy1, vy2) — H then V
        (17, 22, 5,  26, 5,  9),
        (15, 20, 16, 25, 16, 19),
        (19, 22, 28, 26, 26, 28),
        (30, 36, 4,  36, 4,  4),
        (30, 35, 17, 35, 17, 17),
        (30, 36, 30, 36, 30, 30),
    ];

    for &(hx1, hx2, hy, vx, vy1, vy2) in left_corridors {
        // Left side
        carve_h_corridor(&mut map, hx1, hx2, hy);
        carve_v_corridor(&mut map, vy1, vy2, vx);
        // Mirror
        let m_hx1 = cx + (cx - hx1);
        let m_hx2 = cx + (cx - hx2);
        let m_vx = cx + (cx - vx);
        carve_h_corridor(&mut map, m_hx1, m_hx2, hy);
        carve_v_corridor(&mut map, vy1, vy2, m_vx);
    }

    // Central vertical spine
    carve_v_corridor(&mut map, 7, 14, 40);
    carve_v_corridor(&mut map, 20, 28, 40);

    place_door(&mut map, 36, 4);
    place_door(&mut map, 44, 4);
    place_door(&mut map, 40, 14);
    place_door(&mut map, 40, 20);
    place_door(&mut map, 36, 30);

    map
}

// ============================================================
// Stage 5: The Static
// Chaotic, fragmented layout. Rooms are scattered with minimal
// connection — the world is falling apart.
// ============================================================

pub fn generate_the_static() -> Map {
    let mut map = Map::new(MAP_W, MAP_H, 0x57A71C_001E001);

    // Scattered fragments — deliberately disorienting
    carve_room(&mut map, 35, 2,  10, 5);   // Entry fragment (top center)
    carve_room(&mut map, 5,  5,  8,  7);    // Isolated west
    carve_room(&mut map, 65, 5,  10, 7);    // Isolated east
    carve_room(&mut map, 20, 10, 12, 6);    // Mid-left
    carve_room(&mut map, 50, 10, 12, 6);    // Mid-right
    carve_room(&mut map, 35, 14, 10, 6);    // Core
    carve_room(&mut map, 8,  20, 14, 6);    // Lower west
    carve_room(&mut map, 60, 20, 14, 6);    // Lower east
    carve_room(&mut map, 25, 22, 10, 6);    // Lower mid-left
    carve_room(&mut map, 45, 22, 10, 6);    // Lower mid-right
    carve_room(&mut map, 35, 28, 10, 5);    // Exit fragment (bottom center)

    // Minimal corridors — barely connected, feels fragile
    carve_h_corridor(&mut map, 13, 20, 12);
    carve_h_corridor(&mut map, 32, 35, 16);
    carve_h_corridor(&mut map, 45, 50, 14);
    carve_v_corridor(&mut map, 7, 10, 40);
    carve_v_corridor(&mut map, 16, 20, 26);
    carve_h_corridor(&mut map, 62, 65, 12);
    carve_v_corridor(&mut map, 12, 20, 67);
    carve_h_corridor(&mut map, 22, 25, 24);
    carve_h_corridor(&mut map, 55, 60, 24);
    carve_v_corridor(&mut map, 20, 22, 40);
    carve_v_corridor(&mut map, 28, 35, 30);
    carve_h_corridor(&mut map, 35, 45, 25);
    carve_v_corridor(&mut map, 28, 45, 50);

    place_door(&mut map, 40, 7);
    place_door(&mut map, 40, 14);
    place_door(&mut map, 40, 20);
    place_door(&mut map, 40, 28);

    map
}

/// Returns the map for the given stage index.
pub fn generate_stage_map(stage_index: usize) -> Map {
    match stage_index {
        0 => generate_shattered_halls(),
        1 => generate_drowned_archive(),
        2 => generate_hollow_garden(),
        3 => generate_mirror_vault(),
        4 => generate_the_static(),
        _ => generate_shattered_halls(),
    }
}
