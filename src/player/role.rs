#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Blind,
    Delayed,
    Hallucinating,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Blind => "THE BLIND",
            Role::Delayed => "THE DELAYED",
            Role::Hallucinating => "THE HALLUCINATING",
        }
    }

    #[allow(dead_code)]
    pub fn hint(self) -> &'static str {
        match self {
            Role::Blind => "You navigate by sound alone.",
            Role::Delayed => "You see the past. Reality has moved on.",
            Role::Hallucinating => "Reality bends. Trust nothing.",
        }
    }
}
