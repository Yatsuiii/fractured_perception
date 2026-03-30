use crate::world::{entity::Entity, World};

pub enum AiAction {
    Move { dx: i32, dy: i32 },
    Attack { target: Entity },
    Wait,
}

/// Pure read-only decision — the engine applies the result.
pub fn decide(entity: Entity, player: Entity, world: &World) -> AiAction {
    let (ex, ey) = match world.get_position(entity) {
        Some(p) => (p.x as i32, p.y as i32),
        None => return AiAction::Wait,
    };

    // Only act when visible to the player (symmetric FOV).
    if !world.fov.is_visible(ex as usize, ey as usize) {
        return AiAction::Wait;
    }

    let (px, py) = match world.get_position(player) {
        Some(p) => (p.x as i32, p.y as i32),
        None => return AiAction::Wait,
    };

    let dx = (px - ex).signum();
    let dy = (py - ey).signum();

    // Diagonal preferred; fall back to cardinal if blocked.
    let candidates = [
        (dx, dy),
        (dx, 0),
        (0, dy),
    ];

    for (mx, my) in candidates {
        if mx == 0 && my == 0 {
            continue;
        }
        let nx = ex + mx;
        let ny = ey + my;

        if let Some(target) = world.entity_at(nx, ny) {
            if target == player {
                return AiAction::Attack { target };
            }
            // Another entity — skip this direction.
            continue;
        }

        if world.map.is_walkable(nx, ny) {
            return AiAction::Move { dx: mx, dy: my };
        }
    }

    AiAction::Wait
}
