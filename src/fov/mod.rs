use crate::map::Map;

pub struct Fov {
    visible: Vec<bool>,
    revealed: Vec<bool>,
    width: usize,
    height: usize,
}

impl Fov {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            visible: vec![false; size],
            revealed: vec![false; size],
            width,
            height,
        }
    }

    pub fn is_visible(&self, x: usize, y: usize) -> bool {
        self.visible.get(y * self.width + x).copied().unwrap_or(false)
    }

    pub fn is_revealed(&self, x: usize, y: usize) -> bool {
        self.revealed.get(y * self.width + x).copied().unwrap_or(false)
    }

    /// Recompute visibility from (origin_x, origin_y) within `radius` tiles.
    pub fn compute(&mut self, origin_x: f32, origin_y: f32, radius: f32, map: &Map) {
        for v in self.visible.iter_mut() {
            *v = false;
        }

        let ox = origin_x as i32;
        let oy = origin_y as i32;
        let r = radius as i32;

        for y in (oy - r)..=(oy + r) {
            for x in (ox - r)..=(ox + r) {
                if !map.is_in_bounds(x, y) {
                    continue;
                }
                let dx = x - ox;
                let dy = y - oy;
                if (dx * dx + dy * dy) as f32 > radius * radius {
                    continue;
                }
                if self.has_los(ox, oy, x, y, map) {
                    self.mark(x, y);
                }
            }
        }
    }

    fn mark(&mut self, x: i32, y: i32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let idx = y as usize * self.width + x as usize;
            self.visible[idx] = true;
            self.revealed[idx] = true;
        }
    }

    /// Bresenham line-of-sight: returns true if (x1,y1) is visible from (x0,y0).
    /// Opaque tiles along the path block visibility; the destination tile itself is
    /// always considered visible (you see the face of a wall).
    fn has_los(&self, x0: i32, y0: i32, x1: i32, y1: i32, map: &Map) -> bool {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx: i32 = if x0 < x1 { 1 } else { -1 };
        let sy: i32 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            if x == x1 && y == y1 {
                return true;
            }
            // Intermediate (non-origin) tiles block the ray if opaque or out of bounds.
            if x != x0 || y != y0 {
                if !map.is_in_bounds(x, y) {
                    return false;
                }
                if map.get(x as usize, y as usize).is_opaque() {
                    return false;
                }
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }
}
