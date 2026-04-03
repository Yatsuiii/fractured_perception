use crate::{
    perception::{self, PanelColor, PanelLine, PlayerView},
    renderer::Renderer,
    stage::{get_stage_def, get_theme},
};

impl super::Engine {
    pub(super) fn render(&mut self) -> Result<(), crate::renderer::RenderError> {
        match self.state.current() {
            crate::state::GameState::StageTransition => {
                let theme = get_theme(self.progression.current_stage);
                self.renderer.clear()?;
                self.renderer.draw_stage_transition(theme)?;
            }
            _ => {
                let mut view = self.build_gameplay_view();
                self.inject_panel_extras(&mut view);

                self.renderer.clear()?;
                self.renderer.draw_view(self.state.current(), &view)?;

                if *self.state.current() == crate::state::GameState::Dialogue {
                    if let Some(ref session) = self.session.dialogue_session {
                        self.renderer.draw_dialogue_overlay(session)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn build_gameplay_view(&self) -> PlayerView {
        let player_entities = self.player_entities();
        let overrides = crate::perception::PerceptionOverrides {
            delay_extra: self.threshold_tracker.delay_penalty(),
            double_distortion: self.threshold_tracker.double_distortion(),
        };
        perception::build_view(
            self.human(),
            &player_entities,
            &self.session.world,
            &self.session.position_history,
            self.time.elapsed,
            &overrides,
        )
    }

    fn inject_panel_extras(&self, view: &mut PlayerView) {
        if let Some((seq_id, flash_time)) = self.session.puzzle_flash {
            if self.time.elapsed - flash_time < 2.0 {
                view.panel_lines.push(PanelLine { text: String::new(), color: PanelColor::Grey });
                view.panel_lines.push(PanelLine {
                    text: format!("  * PUZZLE #{} ACTIVATED!", seq_id),
                    color: PanelColor::Green,
                });
            }
        }

        let theme = get_theme(self.progression.current_stage);
        view.panel_lines.push(PanelLine { text: String::new(), color: PanelColor::Grey });
        view.panel_lines.push(PanelLine {
            text: format!("Stage {}: {}", theme.stage_number(), theme.name()),
            color: PanelColor::White,
        });

        let stage_def = get_stage_def(self.progression.current_stage);
        let resolved = self.progression.encounters_resolved;
        let total = stage_def.clear_threshold;
        view.panel_lines.push(PanelLine {
            text: format!("Encounters: {}/{}", resolved, total),
            color: if self.progression.gate_open { PanelColor::Green } else { PanelColor::Grey },
        });

        if self.progression.gate_open {
            view.panel_lines.push(PanelLine {
                text: "  GATE OPEN — find the exit!".into(),
                color: PanelColor::Green,
            });
        }

        if !self.session.event_log.is_empty() {
            view.panel_lines.push(PanelLine { text: String::new(), color: PanelColor::Grey });
            view.panel_lines.push(PanelLine { text: "─ TEAM LOG ─".into(), color: PanelColor::DarkGrey });
            for entry in &self.session.event_log.entries {
                let age = self.time.elapsed - entry.elapsed;
                let color = if age < 4.0 { entry.color } else { PanelColor::DarkGrey };
                view.panel_lines.push(PanelLine { text: entry.text.clone(), color });
            }
        }
    }
}
