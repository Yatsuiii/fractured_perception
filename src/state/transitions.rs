use super::GameState;

pub fn is_valid(from: &GameState, to: &GameState) -> bool {
    matches!(
        (from, to),
        (GameState::MainMenu, GameState::Playing)
            | (GameState::Playing, GameState::Paused)
            | (GameState::Playing, GameState::GameOver)
            | (GameState::Paused, GameState::Playing)
            | (GameState::GameOver, GameState::MainMenu)
    )
}
