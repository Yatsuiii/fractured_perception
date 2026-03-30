#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Floor,
    Wall,
    Door,
}

impl Tile {
    pub fn is_walkable(self) -> bool {
        matches!(self, Tile::Floor | Tile::Door)
    }

    pub fn is_opaque(self) -> bool {
        matches!(self, Tile::Wall)
    }

    pub fn glyph(self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall => '#',
            Tile::Door => '+',
        }
    }
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    /// Deterministic seed used by perception system for fabrication / distortion.
    pub seed: u64,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        Self {
            width,
            height,
            seed,
            tiles: vec![Tile::Wall; width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Tile {
        if x >= self.width || y >= self.height {
            return Tile::Wall;
        }
        self.tiles[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.tiles[y * self.width + x] = tile;
    }

    pub fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.is_in_bounds(x, y) && self.get(x as usize, y as usize).is_walkable()
    }
}

// --- Map generation ---

struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Rect {
    fn center(&self) -> (usize, usize) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    fn carve(&self, map: &mut Map) {
        for y in self.y..self.y + self.h {
            for x in self.x..self.x + self.w {
                if x < map.width && y < map.height {
                    map.set(x, y, Tile::Floor);
                }
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

/// Places a door where a corridor meets a room edge.
fn place_doors(map: &mut Map, rooms: &[Rect]) {
    for room in rooms {
        // Scan the perimeter one tile outside the room.
        // If a floor tile sits just outside a room edge, put a door on the boundary.
        let x0 = room.x;
        let x1 = room.x + room.w - 1;
        let y0 = room.y;
        let y1 = room.y + room.h - 1;

        // Top and bottom edges.
        for x in x0..=x1 {
            if y0 > 0 && map.get(x, y0.wrapping_sub(1)) == Tile::Floor && map.get(x, y0) == Tile::Floor {
                map.set(x, y0, Tile::Door);
            }
            if y1 + 1 < map.height && map.get(x, y1 + 1) == Tile::Floor && map.get(x, y1) == Tile::Floor {
                map.set(x, y1, Tile::Door);
            }
        }
        // Left and right edges.
        for y in y0..=y1 {
            if x0 > 0 && map.get(x0.wrapping_sub(1), y) == Tile::Floor && map.get(x0, y) == Tile::Floor {
                map.set(x0, y, Tile::Door);
            }
            if x1 + 1 < map.width && map.get(x1 + 1, y) == Tile::Floor && map.get(x1, y) == Tile::Floor {
                map.set(x1, y, Tile::Door);
            }
        }
    }
}

pub fn generate_test_map(width: usize, height: usize) -> Map {
    let mut map = Map::new(width, height, 0xDEADBEEF_CAFE1337);

    // --- Wing A: northwest ---
    let rooms = [
        Rect { x: 2,  y: 2,  w: 10, h: 6 },   //  0  Entrance hall
        Rect { x: 16, y: 2,  w: 8,  h: 5 },    //  1  Corridor junction
        Rect { x: 28, y: 2,  w: 12, h: 7 },     //  2  Observation room
        Rect { x: 44, y: 2,  w: 9,  h: 6 },     //  3  Archive alcove
        Rect { x: 58, y: 2,  w: 14, h: 8 },     //  4  Signal chamber

        // --- Wing B: southwest ---
        Rect { x: 2,  y: 12, w: 12, h: 7 },     //  5  Storage vault
        Rect { x: 18, y: 14, w: 10, h: 6 },     //  6  Relay room
        Rect { x: 32, y: 12, w: 14, h: 8 },     //  7  Central hall (large)
        Rect { x: 50, y: 14, w: 10, h: 6 },     //  8  Generator bay

        // --- Wing C: south ---
        Rect { x: 2,  y: 24, w: 14, h: 6 },     //  9  Collapsed wing
        Rect { x: 20, y: 24, w: 10, h: 8 },     // 10  Echo chamber
        Rect { x: 34, y: 26, w: 12, h: 6 },     // 11  Keeper's den
        Rect { x: 50, y: 24, w: 14, h: 8 },     // 12  Terminus

        // --- Wing D: southeast ---
        Rect { x: 66, y: 14, w: 10, h: 7 },     // 13  Watch tower
        Rect { x: 66, y: 26, w: 10, h: 6 },     // 14  Dead-end shrine
    ];

    for room in &rooms {
        room.carve(&mut map);
    }

    // Primary corridor chain (connect rooms sequentially).
    let connections: &[(usize, usize)] = &[
        (0, 1), (1, 2), (2, 3), (3, 4),          // top row
        (0, 5), (5, 6), (6, 7), (7, 8),          // mid row
        (5, 9), (9, 10), (10, 11), (11, 12),     // bottom row
        (4, 13), (8, 13), (13, 14), (12, 14),    // east loop
        (2, 7), (7, 11),                          // vertical shortcuts
    ];

    for &(a, b) in connections {
        let (x1, y1) = rooms[a].center();
        let (x2, y2) = rooms[b].center();
        // L-shaped corridor: horizontal then vertical.
        carve_h_corridor(&mut map, x1, x2, y1);
        carve_v_corridor(&mut map, y1, y2, x2);
    }

    place_doors(&mut map, &rooms);

    map
}
