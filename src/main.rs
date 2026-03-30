mod engine;
mod events;
mod fov;
mod input;
mod map;
mod perception;
mod player;
mod renderer;
mod state;
mod world;

use engine::Engine;

fn main() {
    let mut engine = Engine::new();
    if let Err(e) = engine.run() {
        eprintln!("Engine error: {:?}", e);
    }
}
