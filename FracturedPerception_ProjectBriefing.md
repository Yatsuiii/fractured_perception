# Fractured Perception — Project Briefing

---

## What this game is

Fractured Perception is a 3-player co-op puzzle game where each player experiences a completely different version of the same world. No single player has the full picture. The only way to progress is to communicate — describe what you see, argue about what's real, and build a shared understanding from broken fragments.

The core mechanic is **communication under perceptual uncertainty**. Not reflexes. Not logic. Whether you can trust your own eyes — and each other.

---

## The three roles

**The Blind**
Navigates entirely by sound. No visuals. Hears things the others miss. The NPC's voice is their only perception of it.

**The Delayed**
Sees the real layout — but everything is shown seconds behind. Entities appear where they were, not where they are. The world has moved on; you haven't.

**The Hallucinating**
Sees everything distorted — objects doubled, walls stretching, allies sometimes looking like enemies. Every truth arrives wrapped in noise.

---

## The hidden state system (T/C/I/B)

The game silently tracks four variables per player throughout every session:

- **T — Truth:** chose honesty, verified before acting, trusted carefully
- **C — Chaos:** acted on incomplete info, communication broke down
- **I — Illusion:** accepted false signals as real, built certainty on lies
- **B — Balance:** held contradictions, never fully resolved, learned to exist in the fracture

These accumulate from every choice, every NPC interaction, every moment of trust or doubt. At the end, each player gets a different ending shaped by their own hidden state. 12–15 endings across four archetypes: Truth, Chaos, Illusion, Balance.

---

## The Mirror NPC system

NPCs are "Mirrors" — the same character appears completely differently to each role:

- **Blind:** invisible, but emits positional sound cues (direction arrow + distance intensity). The voice IS the NPC.
- **Delayed:** visible at stale position, trust level affects glyph — high trust shows `W`, low trust shows `?` with doubt color.
- **Hallucinating:** always sees a ghost duplicate at an offset. NPC rendered with ghost color.

NPC dialogue is role-specific scripted lines accessed via the dialogue system. Trust tracked per-player and modified by encounters and interactions.

NPCs per stage: **The Watcher**, **The Echo**, **The Archivist**, **The Gardener**, **The Keeper**, **The Witness** — each placed in thematically appropriate stages.

---

## The Echo Chamber puzzle

The first and primary puzzle. Tiles must be activated in sequence. The order changes over time. Each role perceives different signals about the correct order. Nobody has the full answer. Everyone has a fragment. Solved entirely through conversation.

---

## Current build status

### Completed — Rust prototype

- Full game concept and design
- Core design and perception systems documented
- Engine loop — input → update → perception → render, 16 ms frame cap
- Engine split into submodules by concern: `input.rs`, `render.rs`, `movement.rs`, `update.rs`, `stage.rs`, `dialogue.rs`
- Three-role perception system (Blind, Delayed, Hallucinating) — fully distinct views per role
- Per-role encounter perception — Blind hears sound cues, Delayed sees stale glyphs, Hallucinating sees ghost duplicates
- FOV system — Bresenham LOS, per-player reveal tracking
- World / ECS — entity spawning, position, NPC marker, puzzle tile, encounter marker components
- Map — 6-room dungeon with corridors, deterministic seed
- State machine — MainMenu → Playing ↔ Paused, Playing → GameOver
- Role-gated puzzle activation — Blind→#1, Delayed→#2, Hallucinating→#3
- Puzzle progress tracking — 3/3 triggers win screen
- Co-op team event log — shared side panel, 8-entry rolling, 4 s fade
- Ping system — E key, logged to all role panels
- Session log file — timestamped events written to `logs/session_<unix>.log` per run
- Watcher NPC — spawned, moves toward nearest player every 0.5 s, visible to all roles
- Encounter system — 3 kinds (Puzzle/Enemy/Obstacle), spawned per stage, interact with E key, resolve for T/C/I/B rewards
- 5 stages defined — Shattered Halls, Drowned Archive, Hollow Garden, Mirror Vault, The Static — each with unique encounters, NPCs, and per-role perception text
- Phantom encounter spawning — Chaos T1 threshold spawns a fake encounter that boosts Illusion on resolve
- Stage progression — encounter clear threshold opens gate, advance to next stage
- Event bus — typed events (PlayerMoved, EncounterResolved, ThresholdCrossed, etc.)
- T/C/I/B threshold system — Tier 1/2 thresholds trigger world mutations (FOV bonus, delay penalty, distortion doubling, stat dampening)
- Delayed perception — entities shown at positions from seconds ago via PositionHistory
- Hallucinating perception — tile distortion (18%/36%), ghost entity duplicates, stability meter
- Dialogue system — NPC interaction with scripted lines, role-specific dialogue
- NPC trust per-player — trust modifiers tracked, affect NPC glyph and color in perception

### Todo — Rust prototype

#### Bugs (fix first)
- [ ] Can't quit from Paused state — Q key not handled; must resume first
- [ ] FOV ray skips out-of-bounds tiles instead of blocking — edge-case vision leak
- [ ] `map::get()` / `map::set()` have no bounds check — will panic on bad input

#### Core systems (not yet functional)
- [ ] Loss condition — no way to fail; game runs indefinitely if players can't solve puzzles
- [ ] T/C/I/B ending determination — `dominant()` method exists but endings not implemented yet
- [ ] NPC dialogue depth — scripted lines only; no branching, no trust-gated responses

#### Gameplay depth
- [ ] Echo Chamber puzzle — tiles placed but no sequence ordering or per-role signals; everyone sees the same hint
- [ ] Co-op puzzle sequencing — two-phase activation requiring two roles to coordinate per puzzle
- [ ] Doors and interactive objects — Door tile exists in map but no open/close mechanic
- [ ] Win/loss summary screen — currently just "ALL PUZZLES SOLVED"; needs per-role stats, dominant T/C/I/B, session time
- [ ] The Archivist NPC — second NPC described in design; spawned in some stages but no unique behavior

#### Polish
- [ ] Role assignment system — currently hardcoded in Engine; needs a selection screen
- [ ] Terminal resize handling — rendering breaks if terminal is resized mid-game
- [ ] Session logger graceful failure — panics on disk full / read-only fs; should degrade silently

### Deferred for now

- Roblox prototype / Lua implementation
- UE5 migration / C++ implementation

---

## Roblox prototype — build order

1. Role assignment system (Blind / Analyst / Hallucinating on join + UI card)
2. Echo Chamber puzzle (one room, tile sequence, per-role signals, win condition)
3. Mirror NPC integration (drop Lua files, place Watcher, create RemoteEvents)
4. T/C/I/B tracking wired to puzzle choices

### Studio setup checklist

Create in ReplicatedStorage — four RemoteEvents named exactly:

- `NPCDialogueEvent`
- `NPCAppearanceEvent`
- `PlayerChoiceEvent`
- `RoleAssignEvent`

File locations:

- `NPCPerceptionModule.lua` → ServerScriptService
- `NPCController.server.lua` → ServerScriptService
- `NPCClient.client.lua` → StarterPlayerScripts

Place Model in Workspace: `Watcher` with a Part named `ProximityTrigger` inside it.

---

## Full tech stack

### Current prototype

| Tool | Purpose |
|------|---------|
| Rust | Core engine implementation |
| crossterm | Terminal rendering for prototype |
| Claude API | Puzzle generation and NPC dialogue logic (design only) |
| ElevenLabs | NPC voice design inspiration |

### Deferred for now

| Tool | Purpose |
|------|---------|
| Roblox Studio + Lua | Deferred prototype engine work |
| Unreal Engine 5 | Deferred full game migration |
| C++ + Blueprints | Deferred performance and systems implementation |
| DataStoreService | Deferred persistence layer for Lua prototype |
| Epic Online Services | Deferred multiplayer integration |
| Supabase | T/C/I/B server-side persistence |

### Visual style

HD-2D — pixel sprites over 3D environments, dramatic Lumen lighting.
Reference: Octopath Traveler.
Realistic solo path: Hyper Light Drifter / Crosscode aesthetic first, full HD-2D post-launch.

---

## Roadmap

| Phase | Timeline | Goal |

|-------|----------|------|
| 1 — Rust prototype | Now → Month 2 | Validate core loop in Rust |
| 2 — Polish Rust build | Month 2–4 | Refine perception, puzzle feedback, and UI |
| 3 — Design full game | Month 3–5 | Architect future expansion, target engine choices |
| 4 — Deferred migration planning | Month 5–10 | Decide whether to port beyond Rust |
| 5 — Ship | Month 10+ | Steam Early Access |

Target audience: Phasmophobia, Keep Talking and Nobody Explodes, It Takes Two players.

---

## Developer profile

- Building Rust game engine systems (growing proficiency)
- Beginner C++
- Strong on game design and creative vision
- Solo dev (no team yet)

## How to work with Claude on this project

- **Writing code:** specify file, system, and ask for beginner-friendly comments
- **Designing systems:** describe in plain English — Claude architects before coding
- **Debugging:** paste the error + the relevant script
- **When stuck:** ask "what's the smallest thing I can build to test this?"
