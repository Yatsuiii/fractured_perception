use std::collections::VecDeque;

use crate::perception::PanelColor;

pub(super) struct LogEntry {
    pub text:    String,
    pub color:   PanelColor,
    pub elapsed: f32,
}

pub(super) struct EventLog {
    pub entries: VecDeque<LogEntry>,
}

impl EventLog {
    pub fn new() -> Self { Self { entries: VecDeque::new() } }

    #[allow(dead_code)]
    pub fn clear(&mut self) { self.entries.clear(); }

    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    pub fn push(&mut self, text: String, color: PanelColor, elapsed: f32) {
        if self.entries.len() >= 8 {
            self.entries.pop_front();
        }
        self.entries.push_back(LogEntry { text, color, elapsed });
    }
}
