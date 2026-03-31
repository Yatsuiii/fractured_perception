/// Stage System — linear progression through 5 unique environments.
///
/// Each stage is a self-contained world with its own map, encounters, NPCs,
/// and atmosphere. The team clears one stage to unlock the gate to the next.
///
/// Stages:
///   1. The Shattered Halls  — fragmented architecture, intro
///   2. The Drowned Archive  — flooded library, sound/ink/reflections
///   3. The Hollow Garden    — organic overgrowth, shifting paths
///   4. The Mirror Vault     — reflections, symmetry, fabrication overload
///   5. The Static           — reality breakdown, final challenge

pub mod maps;

use crate::encounter::EncounterDef;

// ---------------------------------------------------------------------------
// Stage theme
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageTheme {
    ShatteredHalls,
    DrownedArchive,
    HollowGarden,
    MirrorVault,
    TheStatic,
}

impl StageTheme {
    pub fn name(self) -> &'static str {
        match self {
            StageTheme::ShatteredHalls => "THE SHATTERED HALLS",
            StageTheme::DrownedArchive => "THE DROWNED ARCHIVE",
            StageTheme::HollowGarden   => "THE HOLLOW GARDEN",
            StageTheme::MirrorVault    => "THE MIRROR VAULT",
            StageTheme::TheStatic      => "THE STATIC",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            StageTheme::ShatteredHalls => "Reality is cracked but recognizable. Learn to communicate.",
            StageTheme::DrownedArchive => "Sound warps through flooded ruins. Ink bleeds across walls.",
            StageTheme::HollowGarden   => "Paths grow and close. Nothing stays open for long.",
            StageTheme::MirrorVault    => "Every room has a twin. One is real. One is not.",
            StageTheme::TheStatic      => "Reality breaks down. Perceptions bleed into each other.",
        }
    }

    pub fn stage_number(self) -> usize {
        match self {
            StageTheme::ShatteredHalls => 1,
            StageTheme::DrownedArchive => 2,
            StageTheme::HollowGarden   => 3,
            StageTheme::MirrorVault    => 4,
            StageTheme::TheStatic      => 5,
        }
    }
}

// ---------------------------------------------------------------------------
// NPC placement data (used by engine to spawn NPCs)
// ---------------------------------------------------------------------------

pub struct NpcDef {
    pub name: &'static str,
    pub base_trust: f32,
    pub x: f32,
    pub y: f32,
}

// ---------------------------------------------------------------------------
// Stage definition — everything needed to build a stage
// ---------------------------------------------------------------------------

pub struct StageDef {
    pub theme: StageTheme,
    pub encounters: Vec<EncounterDef>,
    pub npcs: Vec<NpcDef>,
    /// Position of the exit gate in this stage.
    pub gate_position: (f32, f32),
    /// Where players spawn when entering this stage.
    pub spawn_position: (f32, f32),
    /// How many encounters must be resolved to open the gate.
    pub clear_threshold: usize,
}

// ---------------------------------------------------------------------------
// Progression — tracks the team's journey across stages
// ---------------------------------------------------------------------------

pub struct Progression {
    pub current_stage: usize,
    pub encounters_resolved: usize,
    pub gate_open: bool,
}

impl Progression {
    pub fn new() -> Self {
        Self {
            current_stage: 0,
            encounters_resolved: 0,
            gate_open: false,
        }
    }

    /// Call when an encounter is resolved. Returns true if the gate just opened.
    pub fn resolve_encounter(&mut self, threshold: usize) -> bool {
        self.encounters_resolved += 1;
        if !self.gate_open && self.encounters_resolved >= threshold {
            self.gate_open = true;
            return true;
        }
        false
    }

    /// Advance to the next stage. Returns true if there are more stages.
    pub fn advance(&mut self) -> bool {
        self.current_stage += 1;
        self.encounters_resolved = 0;
        self.gate_open = false;
        self.current_stage < STAGE_COUNT
    }

    pub fn is_final_stage(&self) -> bool {
        self.current_stage >= STAGE_COUNT - 1
    }
}

pub const STAGE_COUNT: usize = 5;

// ---------------------------------------------------------------------------
// Stage catalog — returns the definition for each stage
// ---------------------------------------------------------------------------

pub fn get_stage_def(index: usize) -> StageDef {
    match index {
        0 => stage_shattered_halls(),
        1 => stage_drowned_archive(),
        2 => stage_hollow_garden(),
        3 => stage_mirror_vault(),
        4 => stage_the_static(),
        _ => stage_shattered_halls(), // fallback
    }
}

pub fn get_theme(index: usize) -> StageTheme {
    match index {
        0 => StageTheme::ShatteredHalls,
        1 => StageTheme::DrownedArchive,
        2 => StageTheme::HollowGarden,
        3 => StageTheme::MirrorVault,
        4 => StageTheme::TheStatic,
        _ => StageTheme::ShatteredHalls,
    }
}

// ---------------------------------------------------------------------------
// Stage definitions
// ---------------------------------------------------------------------------

fn stage_shattered_halls() -> StageDef {
    use crate::encounter::{EncounterDef, EncounterKind, RolePerception};

    StageDef {
        theme: StageTheme::ShatteredHalls,
        spawn_position: (5.0, 4.0),
        gate_position: (70.0, 28.0),
        clear_threshold: 4,
        encounters: vec![
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Cracked Sequence",
                position: (18.0, 4.0),
                perception: RolePerception {
                    blind: "A rhythmic tapping echoes from the tiles. The pattern is incomplete.",
                    delayed: "Tiles glow in sequence — but you see the pattern from seconds ago. It's already changed.",
                    hallucinating: "Two sequences pulse at once. Only one leads forward.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Obstacle,
                name: "Collapsed Archway",
                position: (35.0, 12.0),
                perception: RolePerception {
                    blind: "Rubble blocks the path. You hear wind through a gap on the left.",
                    delayed: "The archway looks intact to you. But it collapsed seconds ago — the rubble hasn't caught up.",
                    hallucinating: "The archway rebuilds and collapses repeatedly. One frame shows the way.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Enemy,
                name: "Fragment",
                position: (50.0, 18.0),
                perception: RolePerception {
                    blind: "Something scrapes across the floor. It moves when you move.",
                    delayed: "A jagged shape stands still in the corridor. But that's where it was — not where it is.",
                    hallucinating: "Three shapes circle you. Two are echoes. One is hunting.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Echo Lock",
                position: (60.0, 25.0),
                perception: RolePerception {
                    blind: "A door hums at a pitch that changes. Match the tone to pass.",
                    delayed: "The symbols on the door are already rearranging, but you see the old arrangement. Ask what's there now.",
                    hallucinating: "The door is open and closed at the same time. Decide which is true.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Obstacle,
                name: "Shattered Bridge",
                position: (40.0, 28.0),
                perception: RolePerception {
                    blind: "The floor drops away. Stepping stones — you hear them shift underfoot.",
                    delayed: "The bridge looks whole to you. Your team says it's broken. Trust them — your view is stale.",
                    hallucinating: "The bridge is intact, then shattered. Your balance tells you which tiles hold.",
                },
            },
        ],
        npcs: vec![
            NpcDef { name: "The Watcher", base_trust: 0.6, x: 33.0, y: 5.0 },
            NpcDef { name: "The Echo",    base_trust: 0.4, x: 24.0, y: 27.0 },
        ],
    }
}

fn stage_drowned_archive() -> StageDef {
    use crate::encounter::{EncounterDef, EncounterKind, RolePerception};

    StageDef {
        theme: StageTheme::DrownedArchive,
        spawn_position: (5.0, 4.0),
        gate_position: (68.0, 28.0),
        clear_threshold: 4,
        encounters: vec![
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Ink Current",
                position: (20.0, 8.0),
                perception: RolePerception {
                    blind: "Water rushes in channels. The current carries whispers — follow the loudest.",
                    delayed: "Ink flowed across these pages moments ago. The words you read have already dissolved.",
                    hallucinating: "The ink writes two messages at once. One is a warning, one is a lie.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Enemy,
                name: "Drowned Scribe",
                position: (40.0, 15.0),
                perception: RolePerception {
                    blind: "Waterlogged breathing. Something writes on the walls in the dark.",
                    delayed: "A figure hunched over a desk. But it stood up seconds ago — you're watching a ghost of its posture.",
                    hallucinating: "The scribe splits into two. One writes truth, the other writes traps.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Obstacle,
                name: "Flooded Stacks",
                position: (55.0, 10.0),
                perception: RolePerception {
                    blind: "Water rises. You hear shelves creaking, books floating free. Climb.",
                    delayed: "The water looks low to you. Your team says it's rising. You're seeing the level from seconds ago.",
                    hallucinating: "Water rises and falls unpredictably. One rhythm is the real tide.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Catalog Cipher",
                position: (30.0, 25.0),
                perception: RolePerception {
                    blind: "A mechanism clicks in a pattern. Three tumblers — each sounds different.",
                    delayed: "The catalog shows old entries. The index has already been rearranged — ask your team what it says now.",
                    hallucinating: "The cipher shifts between two solutions. The dimmer one is correct.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Enemy,
                name: "Page Swarm",
                position: (60.0, 22.0),
                perception: RolePerception {
                    blind: "Paper rustling from every direction. They're closing in.",
                    delayed: "Pages drift lazily in the air. But that was seconds ago — they've already swarmed.",
                    hallucinating: "A storm of pages, doubled and tripled. Swat the ones that feel heavy.",
                },
            },
        ],
        npcs: vec![
            NpcDef { name: "The Archivist", base_trust: 0.3, x: 15.0, y: 20.0 },
            NpcDef { name: "The Echo",      base_trust: 0.5, x: 50.0, y: 28.0 },
        ],
    }
}

fn stage_hollow_garden() -> StageDef {
    use crate::encounter::{EncounterDef, EncounterKind, RolePerception};

    StageDef {
        theme: StageTheme::HollowGarden,
        spawn_position: (5.0, 16.0),
        gate_position: (72.0, 16.0),
        clear_threshold: 4,
        encounters: vec![
            EncounterDef {
                kind: EncounterKind::Obstacle,
                name: "Thornwall",
                position: (20.0, 10.0),
                perception: RolePerception {
                    blind: "Thorns scrape across your arms. A gap opens and closes rhythmically.",
                    delayed: "The gap was open moments ago. It's closed now but you still see it open. Ask if there's a way through.",
                    hallucinating: "The thorns bloom and retract in waves. Two patterns overlap — follow the slower one.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Enemy,
                name: "Root Stalker",
                position: (35.0, 20.0),
                perception: RolePerception {
                    blind: "Something moves underground. You feel vibrations before it surfaces.",
                    delayed: "You see roots erupting where they were seconds ago. The stalker has already moved on.",
                    hallucinating: "Roots lash from below, doubled. The ghost roots pass through you harmlessly.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Seed Sequence",
                position: (50.0, 8.0),
                perception: RolePerception {
                    blind: "Seeds drop in a musical pattern. Plant them in the right order.",
                    delayed: "You see seeds falling — but they landed seconds ago. The planting spots have already shifted.",
                    hallucinating: "Seeds split into pairs mid-air. Catch the heavier one.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Obstacle,
                name: "Living Maze",
                position: (60.0, 25.0),
                perception: RolePerception {
                    blind: "Walls of leaves shuffle around you. The wind tells you which path just opened.",
                    delayed: "The maze layout you see is outdated. The hedges moved while you were watching the old paths.",
                    hallucinating: "The maze has two configurations overlapping. Walk the intersections.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Bloom Gate",
                position: (68.0, 16.0),
                perception: RolePerception {
                    blind: "A flower hums. Others nearby answer. Create the right harmony.",
                    delayed: "The flowers already bloomed. You see their old colors — they've changed since.",
                    hallucinating: "Every flower blooms twice. The afterimage is the real color.",
                },
            },
        ],
        npcs: vec![
            NpcDef { name: "The Gardener", base_trust: 0.5, x: 40.0, y: 16.0 },
        ],
    }
}

fn stage_mirror_vault() -> StageDef {
    use crate::encounter::{EncounterDef, EncounterKind, RolePerception};

    StageDef {
        theme: StageTheme::MirrorVault,
        spawn_position: (40.0, 4.0),
        gate_position: (40.0, 30.0),
        clear_threshold: 5,
        encounters: vec![
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Reflection Test",
                position: (20.0, 8.0),
                perception: RolePerception {
                    blind: "Two identical sounds from opposite sides. One is the source, one is the echo.",
                    delayed: "Two rooms — but you see the state of both from seconds ago. One changed since. Which?",
                    hallucinating: "Four rooms. Two are reflections of reflections. Find the original.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Enemy,
                name: "Mirror Self",
                position: (60.0, 8.0),
                perception: RolePerception {
                    blind: "Your own footsteps — but delayed. Something copies your movements.",
                    delayed: "A copy of you stands where you were seconds ago. It's your own past trailing behind you.",
                    hallucinating: "Three copies. You are one of them. Which one is moving on their own?",
                },
            },
            EncounterDef {
                kind: EncounterKind::Obstacle,
                name: "Infinite Corridor",
                position: (10.0, 18.0),
                perception: RolePerception {
                    blind: "The corridor repeats. Same echo, same distance, forever. Break the loop.",
                    delayed: "You see yourself walking this corridor seconds ago. The loop feeds on your delay.",
                    hallucinating: "The corridor folds on itself. Walk backward to go forward.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Symmetry Break",
                position: (40.0, 18.0),
                perception: RolePerception {
                    blind: "Perfect symmetry in the echoes. Introduce asymmetry to unlock the way.",
                    delayed: "Both halves looked identical seconds ago. One has changed since — but you can't tell which.",
                    hallucinating: "Symmetry is broken everywhere for you. Describe what you see — it's the key.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Enemy,
                name: "The Reflection",
                position: (70.0, 18.0),
                perception: RolePerception {
                    blind: "It speaks with your voice. It says the opposite of what you mean.",
                    delayed: "It looks like a teammate — but from seconds ago. The real one has already moved.",
                    hallucinating: "Your reflection steps out of the glass. It moves before you do.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "True Glass",
                position: (40.0, 28.0),
                perception: RolePerception {
                    blind: "A wall of glass. Tap each pane — one sounds hollow. That's the exit.",
                    delayed: "The mirrors showed something different seconds ago. The exit pane keeps shifting before you react.",
                    hallucinating: "Every pane shows a different reality. The one that doesn't shimmer is true.",
                },
            },
        ],
        npcs: vec![
            NpcDef { name: "The Keeper",  base_trust: 0.3, x: 40.0, y: 14.0 },
            NpcDef { name: "The Witness", base_trust: 0.4, x: 30.0, y: 24.0 },
        ],
    }
}

fn stage_the_static() -> StageDef {
    use crate::encounter::{EncounterDef, EncounterKind, RolePerception};

    StageDef {
        theme: StageTheme::TheStatic,
        spawn_position: (40.0, 4.0),
        gate_position: (40.0, 30.0),
        clear_threshold: 5,
        encounters: vec![
            EncounterDef {
                kind: EncounterKind::Enemy,
                name: "Noise Entity",
                position: (15.0, 10.0),
                perception: RolePerception {
                    blind: "Pure static. No direction, no shape. It is everywhere and nowhere.",
                    delayed: "You see where the noise was. It's already somewhere else. Your delay is fatal here.",
                    hallucinating: "The static takes your shape. Then your ally's shape. Then nothing.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Signal Extraction",
                position: (65.0, 10.0),
                perception: RolePerception {
                    blind: "A signal buried in noise. Filter it. Your teammates hear different frequencies.",
                    delayed: "The signal already passed. You see its afterimage. Tell your team what it looked like — they see it now.",
                    hallucinating: "The signal is inverted for you. What you discard is what they need.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Obstacle,
                name: "Phase Wall",
                position: (30.0, 18.0),
                perception: RolePerception {
                    blind: "The wall is there. Then it isn't. Time your step between the pulses.",
                    delayed: "The wall phased open seconds ago. Is it still open? Your view can't tell you. Ask.",
                    hallucinating: "Every wall phases. The real ones phase slower.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Perception Merge",
                position: (50.0, 18.0),
                perception: RolePerception {
                    blind: "You hear what the Delayed sees. Describe it back to them.",
                    delayed: "You see the past of what the Hallucinating feels. Translate the old version for the Blind.",
                    hallucinating: "All three perceptions are yours now. Find the overlap — that's the truth.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Enemy,
                name: "The Collapse",
                position: (40.0, 26.0),
                perception: RolePerception {
                    blind: "Everything is sound. The final threat is silence itself.",
                    delayed: "The world already collapsed. You see the last frame before it ended. Find the exit in the memory.",
                    hallucinating: "All distortions merge into one shape. Face it together or fracture completely.",
                },
            },
            EncounterDef {
                kind: EncounterKind::Puzzle,
                name: "Final Alignment",
                position: (40.0, 30.0),
                perception: RolePerception {
                    blind: "Three tones. Three players. Sing together.",
                    delayed: "Three symbols — you see the one from seconds ago. Tell your team what was there before.",
                    hallucinating: "Three truths. Each player holds one. Speak them aloud.",
                },
            },
        ],
        npcs: vec![
            NpcDef { name: "The Witness", base_trust: 0.5, x: 40.0, y: 14.0 },
        ],
    }
}
