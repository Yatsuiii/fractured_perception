use crate::{
    dialogue::{self, DialogueSession},
    events::{Event, TrustReason},
    input::Key,
    perception::PanelColor,
    state::GameState,
};

impl super::Engine {
    pub(super) fn try_start_dialogue(&mut self) -> bool {
        let (npc_entity, _) = match self.closest_entity_in_range(&self.npc_entities.clone(), 4.0) {
            Some(c) => c,
            None => return false,
        };

        let npc_name = match self.world.get_npc_marker(npc_entity) {
            Some(m) => m.name,
            None => return false,
        };

        let role = self.human().role;
        let trust = self.human().trust_for(npc_entity, self.npc_base_trust(npc_entity));

        let lines = match dialogue::get_dialogue(npc_name, role, trust) {
            Some(l) => l,
            None => return false,
        };

        self.dialogue_session = Some(DialogueSession {
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

            let finished = match self.dialogue_session.as_mut() {
                Some(session) => !session.advance(),
                None => true,
            };

            if finished {
                self.end_dialogue();
            }
        }
    }

    fn apply_current_dialogue_line(&mut self) {
        let (npc_entity, trust_delta, stat_nudge) = match &self.dialogue_session {
            Some(session) => match session.current() {
                Some(line) => (session.npc_entity, line.trust_delta, line.stat_nudge),
                None => return,
            },
            None => return,
        };

        if trust_delta.abs() > f32::EPSILON {
            let base = self.npc_base_trust(npc_entity);
            self.human_mut().adjust_trust(npc_entity, trust_delta, base);
            self.events.emit(Event::TrustChanged {
                npc: npc_entity,
                delta: trust_delta,
                reason: TrustReason::Dialogue,
            });
        }

        let hs = &mut self.human_mut().hidden_state;
        let (truth, chaos, illusion, balance) = stat_nudge;
        if truth > 0.0    { hs.add_truth(truth); }
        if chaos > 0.0    { hs.add_chaos(chaos); }
        if illusion > 0.0 { hs.add_illusion(illusion); }
        if balance > 0.0  { hs.add_balance(balance); }
    }

    fn end_dialogue(&mut self) {
        if let Some(session) = self.dialogue_session.take() {
            let npc_name = session.npc_name;
            self.events.emit(Event::DialogueEnded { npc: session.npc_entity });
            self.session_logger.log(&format!("  Dialogue ended: {}", npc_name));
            self.event_log.push(
                format!("  Spoke with {}", npc_name),
                PanelColor::Cyan,
                self.time.elapsed,
            );
        }
        self.state.transition(GameState::Playing);
    }
}
