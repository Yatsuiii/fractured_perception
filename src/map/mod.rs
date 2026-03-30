#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Floor,
    Wall,
}

impl Tile {
    pub fn is_walkable(self) -> bool {
        matches!(self, Tile::Floor)
    }

    pub fn is_opaque(self) -> bool {
        matches!(self, Tile::Wall)
    }

    pub fn glyph(self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall => '#',
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
        self.tiles[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
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

pub fn generate_test_map(width: usize, height: usize) -> Map {
    let mut map = Map::new(width, height, 0xDEADBEEF_CAFE1337);

    let rooms = [
        Rect { x: 2,  y: 2,  w: 10, h: 7  },
        Rect { x: 18, y: 2,  w: 12, h: 5  },
        Rect { x: 36, y: 2,  w: 9,  h: 9  },
        Rect { x: 2,  y: 12, w: 14, h: 6  },
        Rect { x: 22, y: 11, w: 10, h: 7  },
        Rect { x: 38, y: 12, w: 12, h: 6  },
    ];

    for room in &rooms {
        room.carve(&mut map);
    }

    for i in 0..rooms.len() - 1 {
        let (x1, y1) = rooms[i].center();
        let (x2, y2) = rooms[i + 1].center();
        carve_h_corridor(&mut map, x1, x2, y1);
        carve_v_corridor(&mut map, y1, y2, x2);
    }

    map
}
