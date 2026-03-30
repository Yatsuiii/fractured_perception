pub mod terminal;

use crate::{perception::PlayerView, state::GameState};

#[derive(Debug)]
pub enum RenderError {
    Io(std::io::Error),
}

impl From<std::io::Error> for RenderError {
    fn from(e: std::io::Error) -> Self {
        RenderError::Io(e)
    }
}

/// The renderer knows nothing about World or roles.
/// It only draws what a PlayerView says — enforcing the perceptual contract at the type level.
pub trait Renderer {
    fn init(&mut self) -> Result<(), RenderError>;
    fn clear(&mut self) -> Result<(), RenderError>;
    fn draw_view(&mut self, state: &GameState, view: &PlayerView) -> Result<(), RenderError>;
    fn shutdown(&mut self) -> Result<(), RenderError>;
}
