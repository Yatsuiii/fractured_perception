use crate::{
    events::{Event, TrustReason, thresholds::Threshold},
};

impl super::Engine {
    pub(super) fn update(&mut self) {
        self.state.apply_pending();
        self.record_positions();
        self.check_thresholds();
        self.apply_ongoing_thresholds();
        self.tick_npc_movement();
        self.tick_npc_proximity_trust();
        self.tick_companions();
        self.process_events();
    }

    fn record_positions(&mut self) {
        let t = self.time.elapsed;
        let entities = self.players.iter().map(|p| p.entity)
            .chain(self.npc_entities.iter().copied());

        for entity in entities {
            if let Some(pos) = self.world.get_position(entity) {
                self.position_history.record(entity, pos.x, pos.y, t);
            }
        }
    }

    fn apply_ongoing_thresholds(&mut self) {
        if self.threshold_tracker.is_active(Threshold::ChaosTier2) {
            let delta = self.time.delta;
            let idx = self.human_idx;
            for &npc in &self.npc_entities {
                let base = self.world.get_npc_marker(npc)
                    .map(|m| m.base_trust).unwrap_or(0.5);
                self.players[idx].adjust_trust(npc, -0.002 * delta, base);
            }
        }
    }

    fn tick_npc_movement(&mut self) {
        if self.time.elapsed - self.last_ai_tick < 0.5 {
            return;
        }
        self.last_ai_tick = self.time.elapsed;

        let player_entities = self.player_entities();

        for i in 0..self.npc_entities.len() {
            let npc = self.npc_entities[i];
            match crate::ai::decide(npc, &player_entities, &self.world) {
                crate::ai::AiAction::Move { dx, dy } => {
                    let (nx, ny) = match self.world.get_position(npc) {
                        Some(p) => (p.x as i32 + dx, p.y as i32 + dy),
                        None => continue,
                    };
                    if self.world.map.is_walkable(nx, ny) {
                        if let Some(pos) = self.world.get_position_mut(npc) {
                            pos.x = nx as f32;
                            pos.y = ny as f32;
                        }
                    }
                }
                crate::ai::AiAction::Wait => {}
            }
        }
    }

    fn tick_npc_proximity_trust(&mut self) {
        let idx = self.human_idx;
        let human_entity = self.players[idx].entity;
        let human_pos = match self.world.get_position(human_entity).copied() {
            Some(p) => p,
            None => return,
        };

        for &npc in &self.npc_entities {
            let npc_pos = match self.world.get_position(npc) {
                Some(p) => *p,
                None => continue,
            };

            let dx = human_pos.x - npc_pos.x;
            let dy = human_pos.y - npc_pos.y;
            let dist2 = dx * dx + dy * dy;

            let delta = if dist2 <= 9.0 {
                0.02
            } else if dist2 <= 36.0 {
                0.005
            } else {
                0.0
            };

            if delta > 0.0 {
                let base = self.world.get_npc_marker(npc)
                    .map(|m| m.base_trust).unwrap_or(0.5);
                self.players[idx].adjust_trust(npc, delta, base);
                self.events.emit(Event::TrustChanged {
                    npc,
                    delta,
                    reason: TrustReason::NpcProximity,
                });
            }
        }
    }

    fn tick_companions(&mut self) {
        if self.time.elapsed - self.last_companion_tick < 0.8 {
            return;
        }
        self.last_companion_tick = self.time.elapsed;

        let human_entity = self.players[self.human_idx].entity;
        let target = match self.world.get_position(human_entity).copied() {
            Some(p) => p,
            None => return,
        };

        for i in 0..3 {
            if i == self.human_idx { continue; }
            let companion = self.players[i].entity;
            let (cx, cy) = match self.world.get_position(companion) {
                Some(p) => (p.x as i32, p.y as i32),
                None => continue,
            };
            let dx = target.x as i32 - cx;
            let dy = target.y as i32 - cy;

            if dx * dx + dy * dy <= 4 { continue; }

            let (mx, my) = if dx.abs() >= dy.abs() {
                (dx.signum(), 0)
            } else {
                (0, dy.signum())
            };

            if self.world.map.is_walkable(cx + mx, cy + my) {
                if let Some(pos) = self.world.get_position_mut(companion) {
                    pos.x = (cx + mx) as f32;
                    pos.y = (cy + my) as f32;
                }
                self.world.compute_fov_into(
                    companion, super::FOV_RADIUS, &mut self.players[i].fov,
                );
            }
        }
    }

    fn process_events(&mut self) {
        let drained: Vec<Event> = self.events.drain().collect();
        let idx = self.human_idx;

        for event in drained {
            match event {
                Event::PlayerMoved { .. } => {}
                Event::PuzzleActivated { sequence_id } => {
                    if self.activated_puzzles.insert(sequence_id) {
                        self.puzzle_flash = Some((sequence_id, self.time.elapsed));
                        let role = self.players[idx].role;
                        let text = format!("  {} → Puzzle #{} ✓", role.name(), sequence_id);
                        self.session_logger.log(&text);
                        self.event_log.push(text, crate::perception::PanelColor::Green, self.time.elapsed);
                    }
                }
                Event::Ping { from_role } => {
                    let text = format!("  [{}] PING!", from_role.name());
                    self.session_logger.log(&text);
                    self.event_log.push(text, crate::perception::PanelColor::Cyan, self.time.elapsed);

                    let human = self.players[idx].entity;
                    if let Some(hp) = self.world.get_position(human).copied() {
                        for &npc in &self.npc_entities {
                            let npc_pos = match self.world.get_position(npc) {
                                Some(p) => p,
                                None => continue,
                            };
                            let dx = hp.x - npc_pos.x;
                            let dy = hp.y - npc_pos.y;
                            if dx * dx + dy * dy <= 25.0 {
                                let base = self.world.get_npc_marker(npc)
                                    .map(|m| m.base_trust).unwrap_or(0.5);
                                self.players[idx].adjust_trust(npc, -0.05, base);
                            }
                        }
                    }
                }
                Event::TrustChanged { delta, reason, .. } => {
                    let dir = if delta > 0.0 { "+" } else { "" };
                    let text = format!("  Trust {}{:.2} ({:?})", dir, delta, reason);
                    self.session_logger.log(&text);
                    self.event_log.push(text, crate::perception::PanelColor::Yellow, self.time.elapsed);
                }
                Event::DialogueStarted { npc } => {
                    let name = self.world.get_npc_marker(npc)
                        .map(|m| m.name)
                        .unwrap_or("???");
                    self.session_logger.log(&format!("  >> Dialogue: {}", name));
                }
                Event::DialogueEnded { npc } => {
                    let name = self.world.get_npc_marker(npc)
                        .map(|m| m.name)
                        .unwrap_or("???");
                    self.session_logger.log(&format!("  << Dialogue end: {}", name));
                }
                Event::EncounterResolved { .. } => {}
                Event::ThresholdCrossed { threshold } => {
                    let text = format!("  [THRESHOLD] {}", threshold.name());
                    self.session_logger.log(&text);
                }
                _ => {}
            }
        }
    }

    fn check_thresholds(&mut self) {
        let crossed = self.threshold_tracker.check(&self.players[self.human_idx].hidden_state);

        for threshold in crossed {
            self.event_log.push(
                format!("  >> {} <<", threshold.name()),
                crate::perception::PanelColor::Yellow,
                self.time.elapsed,
            );
            self.event_log.push(
                format!("  {}", threshold.description()),
                crate::perception::PanelColor::DarkGrey,
                self.time.elapsed,
            );
            self.session_logger.log(&format!("  Threshold: {} — {}", threshold.name(), threshold.description()));
            self.events.emit(Event::ThresholdCrossed { threshold });

            match threshold {
                Threshold::TruthTier1 => {
                    let idx = self.human_idx;
                    for &npc in &self.npc_entities {
                        let base = self.world.get_npc_marker(npc)
                            .map(|m| m.base_trust).unwrap_or(0.5);
                        self.players[idx].adjust_trust(npc, 0.15, base);
                    }
                }
                Threshold::TruthTier2 => {
                    let idx = self.human_idx;
                    let entity = self.players[idx].entity;
                    let new_radius = super::FOV_RADIUS + self.threshold_tracker.fov_bonus();
                    self.world.compute_fov_into(entity, new_radius, &mut self.players[idx].fov);
                }
                Threshold::ChaosTier1 => {
                    self.spawn_phantom_encounter();
                }
                Threshold::BalanceTier2 => {
                    let hs = &mut self.players[self.human_idx].hidden_state;
                    let mean = (hs.truth + hs.chaos + hs.illusion + hs.balance) / 4.0;
                    hs.truth   = hs.truth   + (mean - hs.truth)   * 0.3;
                    hs.chaos   = hs.chaos   + (mean - hs.chaos)   * 0.3;
                    hs.illusion = hs.illusion + (mean - hs.illusion) * 0.3;
                    hs.balance  = hs.balance  + (mean - hs.balance)  * 0.3;
                }
                Threshold::ChaosTier2
                | Threshold::IllusionTier1
                | Threshold::IllusionTier2
                | Threshold::BalanceTier1 => {}
            }
        }
    }
}
