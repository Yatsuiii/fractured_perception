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
