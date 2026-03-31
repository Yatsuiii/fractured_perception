use crate::{
    encounter::{EncounterMarker, EncounterState, EncounterKind, EncounterPerception},
    player::{Player, Role},
    stage::{get_stage_def, get_theme, maps, StageDef, Progression},
    world::{components::{NpcMarker, Position, PuzzleTile}, entity::Entity, World},
};

pub(super) fn build_stage(stage_def: &StageDef) -> (World, [Entity; 3], Vec<Entity>, Vec<Entity>) {
    let map = maps::generate_stage_map(stage_def.theme.stage_number() - 1);
    let mut world = World::new(map);

    let (sx, sy) = stage_def.spawn_position;
    let e0 = world.spawn(); world.add_position(e0, Position { x: sx, y: sy });
    let e1 = world.spawn(); world.add_position(e1, Position { x: sx + 1.0, y: sy });
    let e2 = world.spawn(); world.add_position(e2, Position { x: sx, y: sy + 1.0 });

    let mut npc_entities = Vec::new();
    for npc_def in &stage_def.npcs {
        let e = world.spawn();
        world.add_position(e, Position { x: npc_def.x, y: npc_def.y });
        world.add_npc_marker(e, NpcMarker { name: npc_def.name, base_trust: npc_def.base_trust });
        npc_entities.push(e);
    }

    let mut encounter_entities = Vec::new();
    for enc_def in &stage_def.encounters {
        let e = world.spawn();
        let (ex, ey) = enc_def.position;
        world.add_position(e, Position { x: ex, y: ey });
        world.add_encounter(e, EncounterMarker::from_def(enc_def));
        encounter_entities.push(e);
    }

    let gate = world.spawn();
    let (gx, gy) = stage_def.gate_position;
    world.add_position(gate, Position { x: gx, y: gy });
    world.add_puzzle_tile(gate, PuzzleTile { sequence_id: 0, is_active: false });

    (world, [e0, e1, e2], npc_entities, encounter_entities)
}

impl super::Engine {
    pub(super) fn spawn_phantom_encounter(&mut self) {
        let player_pos = match self.world.get_position(self.human().entity) {
            Some(p) => *p,
            None => return,
        };

        let offsets: [(i32, i32); 8] = [
            (3, 0), (-3, 0), (0, 3), (0, -3),
            (2, 2), (-2, 2), (2, -2), (-2, -2),
        ];

        for (dx, dy) in offsets {
            let nx = player_pos.x as i32 + dx;
            let ny = player_pos.y as i32 + dy;
            if self.world.map.is_walkable(nx, ny) {
                let e = self.world.spawn();
                self.world.add_position(e, Position { x: nx as f32, y: ny as f32 });
                self.world.add_encounter(e, EncounterMarker {
                    kind: EncounterKind::Puzzle,
                    name: "Phantom Signal",
                    state: EncounterState::Active,
                    perception: EncounterPerception {
                        blind: "You hear... nothing. It was never there.",
                        delayed: "The signal was here — seconds ago. Or was it?",
                        hallucinating: "It shimmers and splits. Which one is real?",
                    },
                });
                self.encounter_entities.push(e);
                return;
            }
        }
    }

    pub(super) fn advance_stage(&mut self) {
        let theme = get_theme(self.progression.current_stage);
        self.session_logger.log(&format!("  Stage cleared: {}", theme.name()));

        if self.progression.is_final_stage() {
            self.state.transition(crate::state::GameState::GameOver);
            return;
        }

        self.progression.advance();
        self.load_current_stage();
        self.state.transition(crate::state::GameState::StageTransition);
    }

    fn install_stage(
        &mut self,
        world: World,
        players: [Player; 3],
        npc_entities: Vec<Entity>,
        encounter_entities: Vec<Entity>,
    ) {
        self.world = world;
        self.players = players;
        self.npc_entities = npc_entities;
        self.encounter_entities = encounter_entities;
        self.activated_puzzles.clear();
        self.puzzle_flash = None;
        self.event_log.clear();
        self.position_history.clear();
        self.last_ai_tick = 0.0;
        self.last_companion_tick = 0.0;
        self.dialogue_session = None;
    }

    fn load_current_stage(&mut self) {
        let stage_def = get_stage_def(self.progression.current_stage);
        let (world, entities, npc_entities, encounter_entities) = build_stage(&stage_def);
        let (w, h) = (world.map.width, world.map.height);

        let hidden_states: Vec<_> = self.players.iter().map(|p| p.hidden_state.clone()).collect();
        let roles: Vec<_> = self.players.iter().map(|p| p.role).collect();

        let mut players = std::array::from_fn(|i| {
            let mut p = Player::new(entities[i], roles[i], w, h);
            p.hidden_state = hidden_states[i].clone();
            p
        });

        for player in &mut players {
            self.world.compute_fov_into(player.entity, super::FOV_RADIUS, &mut player.fov);
        }

        self.install_stage(world, players, npc_entities, encounter_entities);

        let theme = get_theme(self.progression.current_stage);
        self.session_logger.log(&format!("  Entering: {}", theme.name()));
    }

    pub(super) fn reset(&mut self) {
        self.progression = Progression::new();
        let stage_def = get_stage_def(0);
        let (world, entities, npc_entities, encounter_entities) = build_stage(&stage_def);
        let (w, h) = (world.map.width, world.map.height);

        let default_roles = [Role::Blind, Role::Delayed, Role::Hallucinating];
        let mut players = std::array::from_fn(|i| Player::new(entities[i], default_roles[i], w, h));

        for player in &mut players {
            world.compute_fov_into(player.entity, super::FOV_RADIUS, &mut player.fov);
        }

        self.install_stage(world, players, npc_entities, encounter_entities);
        self.human_idx = 0;
        self.threshold_tracker.clear();
        self.chosen_role = None;
        self.session_logger.log("--- NEW GAME ---");
    }
}
