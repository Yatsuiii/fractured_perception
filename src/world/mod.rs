pub mod components;
pub mod entity;

use std::collections::HashMap;

use components::{NpcMarker, Position, PuzzleTile};
use entity::Entity;

use crate::{encounter::EncounterMarker, fov::Fov, map::Map};

pub struct World {
    next_id: u32,
    positions: HashMap<Entity, Position>,
    npc_markers: HashMap<Entity, NpcMarker>,
    puzzle_tiles: HashMap<Entity, PuzzleTile>,
    encounter_markers: HashMap<Entity, EncounterMarker>,
    pub map: Map,
}

impl World {
    pub fn new(map: Map) -> Self {
        Self {
            next_id: 0,
            positions: HashMap::new(),
            npc_markers: HashMap::new(),
            puzzle_tiles: HashMap::new(),
            encounter_markers: HashMap::new(),
            map,
        }
    }

    // --- Entity lifecycle ---

    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        self.next_id += 1;
        entity
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.positions.remove(&entity);
        self.npc_markers.remove(&entity);
        self.puzzle_tiles.remove(&entity);
        self.encounter_markers.remove(&entity);
    }

    // --- Component setters ---

    pub fn add_position(&mut self, entity: Entity, position: Position) {
        self.positions.insert(entity, position);
    }

    pub fn add_npc_marker(&mut self, entity: Entity, marker: NpcMarker) {
        self.npc_markers.insert(entity, marker);
    }

    pub fn add_puzzle_tile(&mut self, entity: Entity, tile: PuzzleTile) {
        self.puzzle_tiles.insert(entity, tile);
    }

    pub fn add_encounter(&mut self, entity: Entity, marker: EncounterMarker) {
        self.encounter_markers.insert(entity, marker);
    }

    // --- Component getters ---

    pub fn get_position(&self, entity: Entity) -> Option<&Position> {
        self.positions.get(&entity)
    }

    pub fn get_position_mut(&mut self, entity: Entity) -> Option<&mut Position> {
        self.positions.get_mut(&entity)
    }

    pub fn get_npc_marker(&self, entity: Entity) -> Option<&NpcMarker> {
        self.npc_markers.get(&entity)
    }

    pub fn get_encounter(&self, entity: Entity) -> Option<&EncounterMarker> {
        self.encounter_markers.get(&entity)
    }

    pub fn get_encounter_mut(&mut self, entity: Entity) -> Option<&mut EncounterMarker> {
        self.encounter_markers.get_mut(&entity)
    }

    // --- Iterators ---

    pub fn all_positions(&self) -> impl Iterator<Item = (Entity, &Position)> {
        self.positions.iter().map(|(e, p)| (*e, p))
    }

    pub fn all_npcs(&self) -> impl Iterator<Item = (Entity, &Position, &NpcMarker)> {
        self.npc_markers.iter().filter_map(|(e, m)| {
            self.positions.get(e).map(|p| (*e, p, m))
        })
    }

    pub fn all_puzzle_tiles(&self) -> impl Iterator<Item = (Entity, &Position, &PuzzleTile)> {
        self.puzzle_tiles.iter().filter_map(|(e, t)| {
            self.positions.get(e).map(|p| (*e, p, t))
        })
    }

    pub fn all_encounters(&self) -> impl Iterator<Item = (Entity, &Position, &EncounterMarker)> {
        self.encounter_markers.iter().filter_map(|(e, m)| {
            self.positions.get(e).map(|p| (*e, p, m))
        })
    }

    pub fn encounter_at(&self, x: i32, y: i32) -> Option<(Entity, &EncounterMarker)> {
        self.encounter_markers.iter().find_map(|(e, marker)| {
            self.positions.get(e)
                .filter(|pos| pos.x as i32 == x && pos.y as i32 == y)
                .map(|_| (*e, marker))
        })
    }

    pub fn encounter_at_mut(&mut self, x: i32, y: i32) -> Option<(Entity, &mut EncounterMarker)> {
        self.encounter_markers.iter_mut().find_map(|(e, marker)| {
            self.positions.get(e)
                .filter(|pos| pos.x as i32 == x && pos.y as i32 == y)
                .map(|_| (*e, marker))
        })
    }

    // --- Spatial query ---

    pub fn entity_at(&self, x: i32, y: i32) -> Option<Entity> {
        self.positions
            .iter()
            .find(|(_, p)| p.x as i32 == x && p.y as i32 == y)
            .map(|(e, _)| *e)
    }

    pub fn puzzle_tile_at(&self, x: i32, y: i32) -> Option<(Entity, &PuzzleTile)> {
        self.puzzle_tiles.iter().find_map(|(e, tile)| {
            self.positions.get(e)
                .filter(|pos| pos.x as i32 == x && pos.y as i32 == y)
                .map(|_| (*e, tile))
        })
    }

    pub fn puzzle_tile_at_mut(&mut self, x: i32, y: i32) -> Option<(Entity, &mut PuzzleTile)> {
        self.puzzle_tiles.iter_mut().find_map(|(e, tile)| {
            self.positions.get(e)
                .filter(|pos| pos.x as i32 == x && pos.y as i32 == y)
                .map(|_| (*e, tile))
        })
    }

    // --- FOV ---

    /// Recomputes `fov` from the given entity's position without storing it.
    /// Each Player owns their own Fov instance.
    pub fn compute_fov_into(&self, entity: Entity, radius: f32, fov: &mut Fov) {
        if let Some(pos) = self.positions.get(&entity) {
            fov.compute(pos.x, pos.y, radius, &self.map);
        }
    }
}
