use crate::{
    events::Event,
    input::Key,
    player::Role,
    state::GameState,
};

impl super::Engine {
    /// Returns true if engine should quit.
    pub(super) fn handle_input(&mut self) -> bool {
        if self.input.is_pressed(&Key::Q) {
            return true;
        }

        match self.state.current().clone() {
            GameState::MainMenu => {
                if self.input.is_pressed(&Key::Enter) {
                    self.state.transition(GameState::RoleSelect);
                }
            }
            GameState::RoleSelect => {
                if self.input.is_pressed(&Key::One) {
                    self.select_role(Role::Blind);
                } else if self.input.is_pressed(&Key::Two) {
                    self.select_role(Role::Delayed);
                } else if self.input.is_pressed(&Key::Three) {
                    self.select_role(Role::Hallucinating);
                } else if self.input.is_pressed(&Key::Escape) {
                    self.state.transition(GameState::MainMenu);
                }
            }
            GameState::Playing => {
                if self.input.is_pressed(&Key::Escape) {
                    self.state.transition(GameState::Paused);
                } else {
                    if self.input.is_pressed(&Key::E) {
                        if !self.try_start_dialogue() && !self.try_interact_encounter() {
                            let role = self.human().role;
                            self.events.emit(Event::Ping { from_role: role });
                        }
                    }
                    self.handle_movement();
                }
            }
            GameState::Dialogue => {
                self.handle_dialogue_input();
            }
            GameState::StageTransition => {
                if self.input.is_pressed(&Key::Enter) {
                    self.state.transition(GameState::Playing);
                }
            }
            GameState::Paused => {
                if self.input.is_pressed(&Key::Escape) {
                    self.state.transition(GameState::Playing);
                }
            }
            GameState::GameOver => {
                if self.input.is_pressed(&Key::Enter) {
                    self.reset();
                    self.state.transition(GameState::MainMenu);
                }
            }
        }

        false
    }

    fn select_role(&mut self, chosen: Role) {
        let all_roles = [Role::Blind, Role::Delayed, Role::Hallucinating];
        let remaining: Vec<Role> = all_roles.iter().copied().filter(|r| *r != chosen).collect();

        self.players[0].role = chosen;
        self.players[1].role = remaining[0];
        self.players[2].role = remaining[1];
        self.human_idx = 0;
        self.chosen_role = Some(chosen);

        let text = format!("  Role selected: {}", chosen.name());
        self.session_logger.log(&text);
        self.state.transition(GameState::StageTransition);
    }
}
