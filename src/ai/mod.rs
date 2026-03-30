use crate::world::{entity::Entity, World};

pub enum AiAction {
    Move { dx: i32, dy: i32 },
    Wait,
}

/// Pure read-only decision — the engine applies the result.
///
/// The Watcher drifts toward the nearest player within 10 tiles.
/// Outside that radius it stays still — cryptic, not aggressive.
pub fn decide(entity: Entity, players: &[Entity], world: &World) -> AiAction {
    let (ex, ey) = match world.get_position(entity) {
        Some(p) => (p.x as i32, p.y as i32),
        None    => return AiAction::Wait,
    };

    // Find the nearest player by squared distance.
    let nearest = players
        .iter()
        .filter_map(|&p| {
            world.get_position(p).map(|pos| {
                let dx = pos.x as i32 - ex;
                let dy = pos.y as i32 - ey;
                (dx, dy, dx * dx + dy * dy)
            })
        })
        .min_by_key(|&(_, _, dist2)| dist2);

    let (dx, dy, dist2) = match nearest {
        Some(n) => n,
        None    => return AiAction::Wait,
    };

    // Stay still when no player is within 10 tiles.
    if dist2 > 100 {
        return AiAction::Wait;
    }

    // Prefer the axis with the greater gap; try the other as fallback.
    let (primary, fallback) = if dx.abs() >= dy.abs() {
        ((dx.signum(), 0_i32), (0_i32, dy.signum()))
    } else {
        ((0_i32, dy.signum()), (dx.signum(), 0_i32))
    };

    for (mx, my) in [primary, fallback] {
        if mx == 0 && my == 0 {
            continue;
        }
        if world.map.is_walkable(ex + mx, ey + my) {
            return AiAction::Move { dx: mx, dy: my };
        }
    }

    AiAction::Wait
}
