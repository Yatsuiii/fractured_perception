/// Encounter System — the challenges players face in each stage.
///
/// Encounters come in three flavors:
///   - Puzzle    → coordination challenge, requires communication
///   - Enemy     → hostile entity, perceived differently per role
///   - Obstacle  → environmental block, requires role-specific insight
///
/// Every encounter is experienced differently by each role through
/// the RolePerception struct. The team must communicate to resolve them.

// ---------------------------------------------------------------------------
// Encounter types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterKind {
    Puzzle,
    Enemy,
    Obstacle,
}

impl EncounterKind {
    pub fn label(self) -> &'static str {
        match self {
            EncounterKind::Puzzle   => "PUZZLE",
            EncounterKind::Enemy    => "ENEMY",
            EncounterKind::Obstacle => "OBSTACLE",
        }
    }

    pub fn glyph(self) -> char {
        match self {
            EncounterKind::Puzzle   => '?',
            EncounterKind::Enemy    => '!',
            EncounterKind::Obstacle => '~',
        }
    }
}

// ---------------------------------------------------------------------------
// Encounter definition — placed by stage definitions
// ---------------------------------------------------------------------------

/// What each role perceives when they encounter this challenge.
#[derive(Clone)]
pub struct RolePerception {
    pub blind: &'static str,
    pub analyst: &'static str,
    pub hallucinating: &'static str,
}

/// Static definition of an encounter — where it is and what it looks like.
#[derive(Clone)]
pub struct EncounterDef {
    pub kind: EncounterKind,
    pub name: &'static str,
    pub position: (f32, f32),
    pub perception: RolePerception,
}

// ---------------------------------------------------------------------------
// Encounter runtime state — lives in World as a component
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterState {
    Active,
    Resolved,
}

/// Component attached to an encounter entity in the world.
#[derive(Clone)]
pub struct EncounterMarker {
    pub kind: EncounterKind,
    pub name: &'static str,
    pub state: EncounterState,
    pub perception: EncounterPerception,
}

/// Stored perception strings — one per role.
#[derive(Clone)]
pub struct EncounterPerception {
    pub blind: &'static str,
    pub analyst: &'static str,
    pub hallucinating: &'static str,
}

impl EncounterMarker {
    pub fn from_def(def: &EncounterDef) -> Self {
        Self {
            kind: def.kind,
            name: def.name,
            state: EncounterState::Active,
            perception: EncounterPerception {
                blind: def.perception.blind,
                analyst: def.perception.analyst,
                hallucinating: def.perception.hallucinating,
            },
        }
    }

    pub fn is_active(&self) -> bool {
        self.state == EncounterState::Active
    }

    pub fn resolve(&mut self) {
        self.state = EncounterState::Resolved;
    }

    /// Returns the perception text for the given role.
    pub fn text_for_role(&self, role: crate::player::Role) -> &'static str {
        match role {
            crate::player::Role::Blind         => self.perception.blind,
            crate::player::Role::VisualAnalyst => self.perception.analyst,
            crate::player::Role::Hallucinating => self.perception.hallucinating,
        }
    }
}
