/// NPC Dialogue System
///
/// Each NPC says completely different things depending on:
///   1. The player's role (Blind / Analyst / Hallucinating)
///   2. The player's trust level with that NPC
///
/// This is the "Mirror NPC" concept — the same character appears as a
/// different personality to each role.

use crate::player::Role;
use crate::world::entity::Entity;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// One line of NPC dialogue with optional stat effects.
#[derive(Clone)]
pub struct DialogueLine {
    pub text: &'static str,
    /// Trust change applied when this line is shown (can be 0.0).
    pub trust_delta: f32,
    /// Hidden-state nudge: (truth, chaos, illusion, balance).
    pub stat_nudge: (f32, f32, f32, f32),
}

/// Trust tiers — determines which dialogue set an NPC uses.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TrustTier {
    Low,    // trust < 0.35
    Mid,    // 0.35 <= trust < 0.7
    High,   // trust >= 0.7
}

impl TrustTier {
    fn from_trust(trust: f32) -> Self {
        if trust >= 0.7 {
            TrustTier::High
        } else if trust >= 0.35 {
            TrustTier::Mid
        } else {
            TrustTier::Low
        }
    }
}

/// Tracks an active dialogue session — which NPC, which lines, where we are.
pub struct DialogueSession {
    pub npc_entity: Entity,
    pub npc_name: &'static str,
    pub lines: Vec<DialogueLine>,
    pub current_line: usize,
}

impl DialogueSession {
    /// Returns the current line, or None if we've reached the end.
    pub fn current(&self) -> Option<&DialogueLine> {
        self.lines.get(self.current_line)
    }

    /// Advance to the next line. Returns true if there are more lines.
    pub fn advance(&mut self) -> bool {
        self.current_line += 1;
        self.current_line < self.lines.len()
    }
}

// ---------------------------------------------------------------------------
// Dialogue content — the actual lines each NPC says per role × trust tier
// ---------------------------------------------------------------------------

/// Looks up the dialogue for a given NPC name, role, and trust level.
pub fn get_dialogue(
    npc_name: &str,
    role: Role,
    trust: f32,
) -> Option<Vec<DialogueLine>> {
    let tier = TrustTier::from_trust(trust);

    let lines = match npc_name {
        "The Watcher"  => watcher_lines(role, tier),
        "The Echo"     => echo_lines(role, tier),
        "The Keeper"   => keeper_lines(role, tier),
        "The Witness"  => witness_lines(role, tier),
        _ => return None,
    };

    Some(lines)
}

// ---------------------------------------------------------------------------
// Shorthand helper
// ---------------------------------------------------------------------------

fn dl(text: &'static str, trust_delta: f32, stat_nudge: (f32, f32, f32, f32)) -> DialogueLine {
    DialogueLine { text, trust_delta, stat_nudge }
}

// ---------------------------------------------------------------------------
// The Watcher — cryptic observer, knows more than they reveal
// ---------------------------------------------------------------------------

fn watcher_lines(role: Role, tier: TrustTier) -> Vec<DialogueLine> {
    match (role, tier) {
        // -- Blind --
        (Role::Blind, TrustTier::Low) => vec![
            dl("...footsteps. You walk loudly for someone with no eyes.", 0.0, (0.01, 0.0, 0.0, 0.0)),
            dl("I see everything here. You hear everything. We are not so different.", 0.02, (0.02, 0.0, 0.0, 0.0)),
        ],
        (Role::Blind, TrustTier::Mid) => vec![
            dl("The tiles hum when activated in the right order. Listen carefully.", 0.0, (0.03, 0.0, 0.0, 0.0)),
            dl("Your companions see things I cannot describe. Ask them about the walls.", 0.03, (0.02, 0.0, 0.0, 0.0)),
        ],
        (Role::Blind, TrustTier::High) => vec![
            dl("You are closer to the truth than the others. Sound does not lie.", 0.05, (0.05, 0.0, 0.0, 0.0)),
            dl("The last tile is in the terminus. The echo there is... different.", 0.03, (0.03, 0.0, 0.0, 0.0)),
            dl("Trust what you hear. Always.", 0.0, (0.02, 0.0, 0.0, 0.02)),
        ],

        // -- Delayed --
        (Role::Delayed, TrustTier::Low) => vec![
            dl("You're looking at where I was, not where I am.", 0.0, (0.0, 0.0, 0.02, 0.0)),
            dl("By the time you react, the moment has already passed.", -0.02, (0.0, 0.0, 0.03, 0.0)),
        ],
        (Role::Delayed, TrustTier::Mid) => vec![
            dl("Three seconds. That's how far behind you are. Count them.", 0.0, (0.02, 0.0, 0.02, 0.0)),
            dl("I watch you chase shadows of where things were. It's... poetic.", 0.02, (0.01, 0.0, 0.01, 0.0)),
        ],
        (Role::Delayed, TrustTier::High) => vec![
            dl("Learn to predict, not react. The delay is your teacher.", 0.05, (0.05, 0.0, 0.0, 0.0)),
            dl("The Blind one hears the present. You see the past. Together — clarity.", 0.03, (0.03, 0.0, 0.0, 0.02)),
            dl("Few who live in the past learn to see the future. You might.", 0.0, (0.02, 0.0, 0.0, 0.0)),
        ],

        // -- Hallucinating --
        (Role::Hallucinating, TrustTier::Low) => vec![
            dl("Am I real? Are you? Does the question matter?", 0.0, (0.0, 0.02, 0.0, 0.0)),
            dl("You see two of me, don't you. Only one is watching.", -0.02, (0.0, 0.03, 0.0, 0.0)),
        ],
        (Role::Hallucinating, TrustTier::Mid) => vec![
            dl("The ghosts you see — they follow a pattern. The real one stays still.", 0.0, (0.02, 0.0, 0.0, 0.02)),
            dl("Walk through what looks solid. Sometimes the distortion is the door.", 0.03, (0.0, 0.0, 0.0, 0.03)),
        ],
        (Role::Hallucinating, TrustTier::High) => vec![
            dl("Your perception is broken, but your instinct is not. Trust the feeling.", 0.05, (0.03, 0.0, 0.0, 0.05)),
            dl("The others see a cleaner world. Cleaner is not truer.", 0.03, (0.05, 0.0, 0.0, 0.0)),
            dl("Balance is your gift. The distortion teaches what clarity cannot.", 0.0, (0.0, 0.0, 0.0, 0.03)),
        ],
    }
}

// ---------------------------------------------------------------------------
// The Echo — repeats fragments of truth, sometimes garbled
// ---------------------------------------------------------------------------

fn echo_lines(role: Role, tier: TrustTier) -> Vec<DialogueLine> {
    match (role, tier) {
        (Role::Blind, TrustTier::Low) => vec![
            dl("...echo... echo... can you hear me?", 0.0, (0.01, 0.01, 0.0, 0.0)),
            dl("Words bounce here. Meaning gets lost in the walls.", 0.0, (0.0, 0.02, 0.0, 0.0)),
        ],
        (Role::Blind, TrustTier::Mid) => vec![
            dl("I repeat what the rooms whisper. The chamber to the south hums loudest.", 0.02, (0.03, 0.0, 0.0, 0.0)),
            dl("Your ears are sharper than their eyes. The echo proves it.", 0.02, (0.02, 0.0, 0.0, 0.01)),
        ],
        (Role::Blind, TrustTier::High) => vec![
            dl("The tiles speak a sequence. I have heard it: follow the hum south, then east.", 0.05, (0.05, 0.0, 0.0, 0.0)),
            dl("You are the only one who truly listens. The others just look.", 0.03, (0.03, 0.0, 0.0, 0.02)),
        ],

        (Role::Delayed, TrustTier::Low) => vec![
            dl("...delayed... delayed... you hear me now, but I spoke ages ago...", 0.0, (0.0, 0.0, 0.02, 0.0)),
            dl("The echo does not wait for you to catch up.", -0.01, (0.0, 0.02, 0.0, 0.0)),
        ],
        (Role::Delayed, TrustTier::Mid) => vec![
            dl("I echo the truth, but by the time it reaches you... things have moved.", 0.02, (0.02, 0.0, 0.01, 0.0)),
            dl("Listen for where the echo is going, not where it was.", 0.03, (0.03, 0.0, 0.0, 0.0)),
        ],
        (Role::Delayed, TrustTier::High) => vec![
            dl("The delay is a gift. You see trajectories where others see positions.", 0.05, (0.05, 0.0, 0.0, 0.0)),
            dl("Track the pattern of movement. Where things were tells you where they'll be.", 0.03, (0.03, 0.0, 0.0, 0.02)),
        ],

        (Role::Hallucinating, TrustTier::Low) => vec![
            dl("...echo... or is it... echo echo echo...", 0.0, (0.0, 0.03, 0.0, 0.0)),
            dl("You hear me twice because you see me twice. Neither is wrong.", 0.0, (0.0, 0.02, 0.0, 0.01)),
        ],
        (Role::Hallucinating, TrustTier::Mid) => vec![
            dl("The echo and the distortion — they are cousins. Both bend reality.", 0.02, (0.02, 0.0, 0.0, 0.02)),
            dl("Walk where the ghosts don't follow. That path is real.", 0.03, (0.0, 0.0, 0.0, 0.03)),
        ],
        (Role::Hallucinating, TrustTier::High) => vec![
            dl("Your doubled vision is not madness. It is depth perception for a fractured world.", 0.05, (0.05, 0.0, 0.0, 0.03)),
            dl("The echo chamber puzzle — you will see two sequences. Follow the quieter one.", 0.03, (0.03, 0.0, 0.0, 0.02)),
        ],
    }
}

// ---------------------------------------------------------------------------
// The Keeper — guards the central hall, slow to trust, blunt
// ---------------------------------------------------------------------------

fn keeper_lines(role: Role, tier: TrustTier) -> Vec<DialogueLine> {
    match (role, tier) {
        (Role::Blind, TrustTier::Low) => vec![
            dl("You stumble into my hall. State your purpose.", 0.0, (0.0, 0.0, 0.0, 0.0)),
            dl("I guard the center. The center holds everything together.", -0.02, (0.01, 0.0, 0.0, 0.0)),
        ],
        (Role::Blind, TrustTier::Mid) => vec![
            dl("You cannot see, yet you find your way. That takes nerve.", 0.03, (0.02, 0.0, 0.0, 0.0)),
            dl("The puzzles branch from here. North, south, east. Listen for the hum.", 0.02, (0.03, 0.0, 0.0, 0.0)),
        ],
        (Role::Blind, TrustTier::High) => vec![
            dl("I was wrong to doubt you. The sightless one sees the clearest path.", 0.05, (0.05, 0.0, 0.0, 0.0)),
            dl("Take the eastern corridor. The last puzzle waits there.", 0.03, (0.03, 0.0, 0.0, 0.02)),
        ],

        (Role::Delayed, TrustTier::Low) => vec![
            dl("You live three seconds behind. That makes you predictable.", 0.0, (0.0, 0.0, 0.03, 0.0)),
            dl("I guard the present. You inhabit the past.", -0.02, (0.0, 0.0, 0.02, 0.0)),
        ],
        (Role::Delayed, TrustTier::Mid) => vec![
            dl("You are learning to anticipate. Good. The delay sharpens instinct.", 0.03, (0.02, 0.0, 0.0, 0.0)),
            dl("The central hall connects all wings. Memorize the layout — it won't move.", 0.02, (0.02, 0.0, 0.0, 0.0)),
        ],
        (Role::Delayed, TrustTier::High) => vec![
            dl("The map is honest with you. Only the living things deceive.", 0.05, (0.05, 0.0, 0.0, 0.0)),
            dl("Remember paths, not positions. The walls stay true.", 0.03, (0.03, 0.0, 0.0, 0.02)),
        ],

        (Role::Hallucinating, TrustTier::Low) => vec![
            dl("You twitch. You see things. I do not trust the unstable.", 0.0, (0.0, 0.02, 0.0, 0.0)),
            dl("Stay away from my post.", -0.03, (0.0, 0.03, 0.0, 0.0)),
        ],
        (Role::Hallucinating, TrustTier::Mid) => vec![
            dl("Still here? Perhaps there is method to your madness.", 0.02, (0.01, 0.0, 0.0, 0.02)),
            dl("The distortions thin near doorways. Doors are anchors to reality.", 0.03, (0.02, 0.0, 0.0, 0.03)),
        ],
        (Role::Hallucinating, TrustTier::High) => vec![
            dl("I was the unstable one, once. Before I learned to guard instead of wander.", 0.05, (0.03, 0.0, 0.0, 0.05)),
            dl("Your balance is growing. The distortion bends to those who accept it.", 0.03, (0.02, 0.0, 0.0, 0.03)),
        ],
    }
}

// ---------------------------------------------------------------------------
// The Witness — silent observer near the terminus, knows the endgame
// ---------------------------------------------------------------------------

fn witness_lines(role: Role, tier: TrustTier) -> Vec<DialogueLine> {
    match (role, tier) {
        (Role::Blind, TrustTier::Low) => vec![
            dl("...", 0.0, (0.0, 0.0, 0.0, 0.0)),
            dl("I witness. I do not speak. You will have to earn words from me.", 0.0, (0.01, 0.0, 0.0, 0.0)),
        ],
        (Role::Blind, TrustTier::Mid) => vec![
            dl("You came far without seeing. The terminus is close.", 0.02, (0.03, 0.0, 0.0, 0.0)),
            dl("The final tile echoes differently. A deeper sound.", 0.03, (0.02, 0.0, 0.0, 0.01)),
        ],
        (Role::Blind, TrustTier::High) => vec![
            dl("I have witnessed every ending. Yours... could be different.", 0.05, (0.05, 0.0, 0.0, 0.0)),
            dl("The hidden state shapes the exit. Truth, Chaos, Illusion, Balance — they decide.", 0.03, (0.03, 0.0, 0.0, 0.03)),
            dl("Listen to the final tile. It will tell you what you've become.", 0.0, (0.02, 0.0, 0.0, 0.02)),
        ],

        (Role::Delayed, TrustTier::Low) => vec![
            dl("...", 0.0, (0.0, 0.0, 0.0, 0.0)),
            dl("You chase ghosts of the past. I witness the present.", 0.0, (0.0, 0.0, 0.01, 0.0)),
        ],
        (Role::Delayed, TrustTier::Mid) => vec![
            dl("The terminus holds the last puzzle. Trust the walls — they don't move.", 0.02, (0.02, 0.0, 0.02, 0.0)),
            dl("Ask the Blind one where things are now. Your eyes show you where they were.", 0.03, (0.03, 0.0, 0.0, 0.0)),
        ],
        (Role::Delayed, TrustTier::High) => vec![
            dl("Every ending I've witnessed began with someone reacting too late.", 0.05, (0.05, 0.0, 0.0, 0.0)),
            dl("Your delay is being measured. Anticipate, don't react.", 0.03, (0.03, 0.0, 0.0, 0.02)),
            dl("The game ends. Whether you caught up to reality — that's what matters.", 0.0, (0.02, 0.0, 0.0, 0.02)),
        ],

        (Role::Hallucinating, TrustTier::Low) => vec![
            dl("...", 0.0, (0.0, 0.0, 0.0, 0.0)),
            dl("I see you. Both of you. The real and the ghost.", 0.0, (0.0, 0.02, 0.0, 0.0)),
        ],
        (Role::Hallucinating, TrustTier::Mid) => vec![
            dl("The terminus shifts for you more than the others. Steel yourself.", 0.02, (0.01, 0.0, 0.0, 0.02)),
            dl("Your ghost follows three steps behind. It cannot enter the final room.", 0.03, (0.02, 0.0, 0.0, 0.03)),
        ],
        (Role::Hallucinating, TrustTier::High) => vec![
            dl("I have witnessed the fractured ending. It is... beautiful, in its way.", 0.05, (0.03, 0.0, 0.0, 0.05)),
            dl("Balance is the rarest stat. You are building something few achieve.", 0.03, (0.02, 0.0, 0.0, 0.03)),
            dl("When you activate the last tile, the distortion will clear. Briefly.", 0.0, (0.03, 0.0, 0.0, 0.02)),
        ],
    }
}
