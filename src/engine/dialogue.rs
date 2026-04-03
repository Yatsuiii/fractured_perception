use crate::{
    dialogue::{self, DialogueSession},
    events::{Event, TrustReason},
    input::Key,
    perception::PanelColor,
    state::GameState,
    world::entity::Entity,
};

impl super::Engine {
    pub(super) fn try_start_dialogue(&mut self) -> bool {
        let Some((npc_entity, _)) = self.closest_entity_in_range(&self.session.npc_entities, 4.0) else {
            return false;
        };

        let Some(m) = self.session.world.get_npc_marker(npc_entity) else {
            return false;
        };
        let npc_name = m.name;

        let role = self.human().role;
        let trust = self.human().trust_for(npc_entity, self.npc_base_trust(npc_entity));

        let Some(lines) = dialogue::get_dialogue(npc_name, role, trust) else {
            return false;
        };

        self.session.dialogue_session = Some(DialogueSession {
            npc_entity,
            npc_name,
            lines,
            current_line: 0,
        });

        self.events.emit(Event::DialogueStarted { npc: npc_entity });
        self.session_logger.log(&format!("  Dialogue started: {}", npc_name));
        self.state.transition(GameState::Dialogue);
        true
    }

    pub(super) fn handle_dialogue_input(&mut self) {
        if self.input.is_pressed(&Key::Escape) {
            self.end_dialogue();
            return;
        }

        if self.input.is_pressed(&Key::E) || self.input.is_pressed(&Key::Enter) {
            self.apply_current_dialogue_line();
            self.advance_dialogue();
        }
    }

    fn advance_dialogue(&mut self) {
        let finished = match self.session.dialogue_session.as_mut() {
            Some(session) => !session.advance(),
            None => true,
        };

        if finished {
            self.end_dialogue();
        }
    }

    fn apply_current_dialogue_line(&mut self) {
        let Some(session) = &self.session.dialogue_session else { return; };
        let Some(line) = session.current() else { return; };

        let npc_entity = session.npc_entity;
        let trust_delta = line.trust_delta;
        let stat_nudge = line.stat_nudge;

        self.apply_trust_from_line(npc_entity, trust_delta);
        self.apply_stat_nudge_from_line(stat_nudge);
    }

    fn apply_trust_from_line(&mut self, npc_entity: Entity, trust_delta: f32) {
        if trust_delta.abs() > f32::EPSILON {
            let base = self.npc_base_trust(npc_entity);
            self.human_mut().adjust_trust(npc_entity, trust_delta, base);
            self.events.emit(Event::TrustChanged {
                npc: npc_entity,
                delta: trust_delta,
                reason: TrustReason::Dialogue,
            });
        }
    }

    fn apply_stat_nudge_from_line(&mut self, stat_nudge: (f32, f32, f32, f32)) {
        let hs = &mut self.human_mut().hidden_state;
        let (truth, chaos, illusion, balance) = stat_nudge;
        
        if truth.abs() > f32::EPSILON    { hs.add_truth(truth); }
        if chaos.abs() > f32::EPSILON    { hs.add_chaos(chaos); }
        if illusion.abs() > f32::EPSILON { hs.add_illusion(illusion); }
        if balance.abs() > f32::EPSILON  { hs.add_balance(balance); }
    }

    fn end_dialogue(&mut self) {
        if let Some(session) = self.session.dialogue_session.take() {
            let npc_name = session.npc_name;
            self.events.emit(Event::DialogueEnded { npc: session.npc_entity });
            self.session_logger.log(&format!("  Dialogue ended: {}", npc_name));
            self.session.event_log.push(
                format!("  Spoke with {}", npc_name),
                PanelColor::Cyan,
                self.time.elapsed,
            );
        }
        self.state.transition(GameState::Playing);
    }
}
