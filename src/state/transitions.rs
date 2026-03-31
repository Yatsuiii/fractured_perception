use super::GameState;

pub fn is_valid(from: &GameState, to: &GameState) -> bool {
    matches!(
        (from, to),
        (GameState::MainMenu, GameState::RoleSelect)
            | (GameState::RoleSelect, GameState::Playing)
            | (GameState::RoleSelect, GameState::StageTransition)
            | (GameState::RoleSelect, GameState::MainMenu)
            | (GameState::Playing, GameState::Paused)
            | (GameState::Playing, GameState::GameOver)
            | (GameState::Playing, GameState::Dialogue)
            | (GameState::Playing, GameState::StageTransition)
            | (GameState::Dialogue, GameState::Playing)
            | (GameState::StageTransition, GameState::Playing)
            | (GameState::StageTransition, GameState::GameOver)
            | (GameState::Paused, GameState::Playing)
            | (GameState::GameOver, GameState::MainMenu)
    )
}
