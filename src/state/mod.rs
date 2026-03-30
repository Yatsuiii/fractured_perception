pub mod transitions;

#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    MainMenu,
    RoleSelect,
    Playing,
    Paused,
    GameOver,
}

pub struct StateManager {
    current: GameState,
    pending: Option<GameState>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            current: GameState::MainMenu,
            pending: None,
        }
    }

    pub fn current(&self) -> &GameState {
        &self.current
    }

    pub fn transition(&mut self, to: GameState) {
        if transitions::is_valid(&self.current, &to) {
            self.pending = Some(to);
        }
    }

    pub fn apply_pending(&mut self) {
        if let Some(next) = self.pending.take() {
            self.current = next;
        }
    }
}
