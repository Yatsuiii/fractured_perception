#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Blind,
    VisualAnalyst,
    Hallucinating,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Blind => "THE BLIND",
            Role::VisualAnalyst => "THE VISUAL ANALYST",
            Role::Hallucinating => "THE HALLUCINATING",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Role::Blind => "You navigate by sound alone.",
            Role::VisualAnalyst => "You see everything. Half is a lie.",
            Role::Hallucinating => "Reality bends. Trust nothing.",
        }
    }
}
