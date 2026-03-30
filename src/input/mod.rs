use std::collections::HashSet;
use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    W,
    A,
    S,
    D,
    E,
    Up,
    Down,
    Left,
    Right,
    Escape,
    Enter,
    Space,
    Q,
}

pub struct InputState {
    current: HashSet<Key>,
    previous: HashSet<Key>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            current: HashSet::new(),
            previous: HashSet::new(),
        }
    }

    pub fn capture(&mut self) {
        // Build next into a fresh set so a poll error never clears the current frame.
        let mut next = HashSet::new();

        while event::poll(Duration::ZERO).unwrap_or(false) {
            if let Ok(CEvent::Key(KeyEvent { code, kind: KeyEventKind::Press, .. })) = event::read()
            {
                if let Some(key) = map_keycode(code) {
                    next.insert(key);
                }
            }
        }

        self.previous = std::mem::replace(&mut self.current, next);
    }

    /// True only on the first frame a key is pressed.
    pub fn is_pressed(&self, key: &Key) -> bool {
        self.current.contains(key) && !self.previous.contains(key)
    }

    /// True while a key is held down (pressed or repeating).
    pub fn is_active(&self, key: &Key) -> bool {
        self.current.contains(key)
    }

    /// True on the first frame after a key is released.
    pub fn is_released(&self, key: &Key) -> bool {
        !self.current.contains(key) && self.previous.contains(key)
    }
}

fn map_keycode(code: KeyCode) -> Option<Key> {
    match code {
        KeyCode::Char('w') | KeyCode::Char('W') => Some(Key::W),
        KeyCode::Char('a') | KeyCode::Char('A') => Some(Key::A),
        KeyCode::Char('s') | KeyCode::Char('S') => Some(Key::S),
        KeyCode::Char('d') | KeyCode::Char('D') => Some(Key::D),
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Key::Q),
        KeyCode::Char('e') | KeyCode::Char('E') => Some(Key::E),
        KeyCode::Up => Some(Key::Up),
        KeyCode::Down => Some(Key::Down),
        KeyCode::Left => Some(Key::Left),
        KeyCode::Right => Some(Key::Right),
        KeyCode::Esc => Some(Key::Escape),
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::Char(' ') => Some(Key::Space),
        _ => None,
    }
}
