use crate::{
    encounter::EncounterKind,
    events::Event,
    input::Key,
    perception::PanelColor,
    player::Role,
    stage::get_stage_def,
    world::entity::Entity,
};

impl super::Engine {
    pub(super) fn handle_movement(&mut self) {
        let mut dx = 0_i32;
        let mut dy = 0_i32;

        if self.input.is_pressed(&Key::W) || self.input.is_pressed(&Key::Up)    { dy -= 1; }
        if self.input.is_pressed(&Key::S) || self.input.is_pressed(&Key::Down)  { dy += 1; }
        if self.input.is_pressed(&Key::A) || self.input.is_pressed(&Key::Left)  { dx -= 1; }
        if self.input.is_pressed(&Key::D) || self.input.is_pressed(&Key::Right) { dx += 1; }

        if dx == 0 && dy == 0 {
            return;
        }

        let entity = self.human().entity;

        let (nx, ny) = match self.session.world.get_position(entity) {
            Some(pos) => (pos.x as i32 + dx, pos.y as i32 + dy),
            None => return,
        };

        if !self.session.world.map.is_walkable(nx, ny) {
            return;
        }

        if let Some(pos) = self.session.world.get_position_mut(entity) {
            pos.x = nx as f32;
            pos.y = ny as f32;
        }

        self.events.emit(Event::PlayerMoved { entity, x: nx as f32, y: ny as f32 });

        if self.try_activate_puzzle(nx, ny) {
            return;
        }

        self.post_move_effects(entity, nx as usize, ny as usize);
    }

    fn try_activate_puzzle(&mut self, x: i32, y: i32) -> bool {
        let puzzle_tile = match self.session.world.puzzle_tile_at_mut(x, y) {
            Some((_, tile)) => tile,
            None => return false,
        };

        if puzzle_tile.is_active {
            return false;
        }

        let seq_id = puzzle_tile.sequence_id;
        if seq_id == 0 && self.progression.gate_open {
            puzzle_tile.is_active = true;
            self.advance_stage();
            return true;
        }

        if seq_id > 0 {
            puzzle_tile.is_active = true;
            self.events.emit(Event::PuzzleActivated { sequence_id: seq_id });
            self.human_mut().hidden_state.add_truth(0.05);
        }

        false
    }

    fn post_move_effects(&mut self, entity: Entity, x: usize, y: usize) {
        let effective_fov = super::FOV_RADIUS + self.threshold_tracker.fov_bonus();
        let idx = self.human_idx;
        self.session.world.compute_fov_into(entity, effective_fov, &mut self.session.players[idx].fov);

        if self.human().role == Role::Hallucinating {
            let was_distorted = if self.threshold_tracker.double_distortion() {
                crate::perception::is_distorted_wide(self.session.world.map.seed, x, y)
            } else {
                crate::perception::is_distorted(self.session.world.map.seed, x, y)
            };
            if was_distorted {
                self.human_mut().hidden_state.add_balance(0.02);
            }
        }

        self.human_mut().hidden_state.add_truth(0.01);
    }

    pub(super) fn try_interact_encounter(&mut self) -> bool {
        let active_encounters: Vec<Entity> = self.session.encounter_entities.iter()
            .copied()
            .filter(|&e| self.session.world.get_encounter(e).map_or(false, |m| m.is_active()))
            .collect();

        let (enc_entity, _) = match self.closest_entity_in_range(&active_encounters, 4.0) {
            Some(c) => c,
            None => return false,
        };

        let (enc_name, enc_kind, role_text) = {
            let marker = self.session.world.get_encounter(enc_entity).unwrap();
            let role = self.human().role;
            (marker.name, marker.kind, marker.text_for_role(role))
        };

        self.session.event_log.push(
            format!("  [{}] {}", enc_kind.label(), enc_name),
            PanelColor::Yellow,
            self.time.elapsed,
        );
        self.session.event_log.push(
            format!("  {}", truncate_text(role_text, 24)),
            PanelColor::Grey,
            self.time.elapsed,
        );
        self.session_logger.log(&format!("  Encounter: {} ({})", enc_name, enc_kind.label()));

        if let Some(marker) = self.session.world.get_encounter_mut(enc_entity) {
            marker.resolve();
        }

        self.events.emit(Event::EncounterResolved { entity: enc_entity });

        if enc_name == "Phantom Signal" {
            self.human_mut().hidden_state.add_illusion(0.1);
            self.session.event_log.push(
                "  ...it was never real.".into(),
                PanelColor::Red,
                self.time.elapsed,
            );
        } else {
            let m = self.threshold_tracker.stat_gain_multiplier();
            let hs = &mut self.human_mut().hidden_state;
            match enc_kind {
                EncounterKind::Puzzle   => { hs.add_truth(0.08 * m); }
                EncounterKind::Enemy    => { hs.add_chaos(0.05 * m); hs.add_balance(0.03 * m); }
                EncounterKind::Obstacle => { hs.add_balance(0.05 * m); hs.add_truth(0.03 * m); }
            }
        }

        let stage_def = get_stage_def(self.progression.current_stage);
        let gate_just_opened = self.progression.resolve_encounter(stage_def.clear_threshold);
        if gate_just_opened {
            self.session.event_log.push(
                "  >>> GATE OPENED <<<".into(),
                PanelColor::Green,
                self.time.elapsed,
            );
            self.session_logger.log("  Gate opened!");
        }

        let idx = self.human_idx;
        for &npc in &self.session.npc_entities {
            let base = self.session.world.get_npc_marker(npc)
                .map(|m| m.base_trust).unwrap_or(0.5);
            self.session.players[idx].adjust_trust(npc, 0.08, base);
        }

        true
    }

    pub(super) fn closest_entity_in_range(&self, entities: &[Entity], range_sq: f32) -> Option<(Entity, f32)> {
        let player_pos = self.session.world.get_position(self.human().entity)?;

        let mut best: Option<(Entity, f32)> = None;
        for &e in entities {
            let pos = match self.session.world.get_position(e) {
                Some(p) => p,
                None => continue,
            };
            let dx = player_pos.x - pos.x;
            let dy = player_pos.y - pos.y;
            let dist2 = dx * dx + dy * dy;

            if dist2 <= range_sq && best.map_or(true, |(_, d)| dist2 < d) {
                best = Some((e, dist2));
            }
        }

        best
    }
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}
