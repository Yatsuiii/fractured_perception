/// Position history — tracks where entities were over time.
///
/// Used by the Delayed role to render entities at stale positions.
/// Stores a ring buffer of snapshots: (entity, x, y, timestamp).

use std::collections::VecDeque;

use super::entity::Entity;

/// One recorded position at a specific time.
#[derive(Clone, Copy)]
struct Snapshot {
    entity: Entity,
    x: f32,
    y: f32,
    time: f32,
}

/// Stores position history for all entities.
/// The Delayed role queries this to get positions from N seconds ago.
pub struct PositionHistory {
    snapshots: VecDeque<Snapshot>,
    /// How many seconds of history to keep.
    retention: f32,
}

impl PositionHistory {
    pub fn new(retention: f32) -> Self {
        Self {
            snapshots: VecDeque::new(),
            retention,
        }
    }

    /// Record the current position of an entity.
    pub fn record(&mut self, entity: Entity, x: f32, y: f32, time: f32) {
        self.snapshots.push_back(Snapshot { entity, x, y, time });
        // Evict old snapshots beyond retention window.
        while let Some(front) = self.snapshots.front() {
            if time - front.time > self.retention + 1.0 {
                self.snapshots.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get the position of an entity from `delay` seconds ago.
    /// Returns the closest snapshot at or before (current_time - delay).
    /// Falls back to the entity's current position if no history exists.
    pub fn get_delayed(&self, entity: Entity, current_time: f32, delay: f32) -> Option<(f32, f32)> {
        let target_time = current_time - delay;

        // Walk backward through snapshots to find the latest one at or before target_time.
        let mut best: Option<(f32, f32)> = None;
        for snap in self.snapshots.iter().rev() {
            if snap.entity == entity && snap.time <= target_time {
                best = Some((snap.x, snap.y));
                break;
            }
        }
        best
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
    }
}
