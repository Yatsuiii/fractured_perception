# Fractured Perception — Project Briefing

*Paste this as the first message in every new Claude Project chat to restore full context.*

---

## What this game is

Fractured Perception is a 3-player co-op puzzle game where each player experiences a completely different version of the same world. No single player has the full picture. The only way to progress is to communicate — describe what you see, argue about what's real, and build a shared understanding from broken fragments.

The core mechanic is **communication under perceptual uncertainty**. Not reflexes. Not logic. Whether you can trust your own eyes — and each other.

---

## The three roles

**The Blind**
Navigates entirely by sound. No visuals. Hears things the others miss. The NPC's voice is their only perception of it.

**The Visual Analyst**
Sees the full layout — but half of it is fabricated. False doors, phantom traps, paths that lead nowhere. Real and fake are indistinguishable.

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

- **Blind:** invisible, but emits positional sound. The voice IS the NPC.
- **Visual Analyst:** visible, but trust level affects whether it seems real. Low trust = flickers, label shows "?"
- **Hallucinating:** always sees a ghost duplicate. Hostile red tint if trust drops below 0.3.

Every NPC interaction goes through a single function: `getNPCDialogue(player, npcName)`.
Right now it returns scripted lines. Swapping the inside to call Claude API enables live AI dialogue — nothing else changes.

Two NPCs: **The Watcher** (stands in corner, cryptic) and **The Archivist** (knows pre-player history).

---

## The Echo Chamber puzzle

The first and primary puzzle. Tiles must be activated in sequence. The order changes over time. Each role perceives different signals about the correct order. Nobody has the full answer. Everyone has a fragment. Solved entirely through conversation.

---

## Current build status

### Completed

- Full game concept and design
- Core design and perception systems documented

### Building now

- Rust engine prototype
- Role-based perception, movement, and puzzle interaction in Rust

### Deferred for now

- Roblox prototype / Lua implementation
- UE5 migration / C++ implementation

### Not started

- Echo Chamber puzzle logic in Rust
- Role assignment system in Rust

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

- Beginner Rust, focusing on game engine systems
- Beginner C++
- Strong on game design and creative vision
- Solo dev (no team yet)

## How to work with Claude on this project

- **Writing code:** specify file, system, and ask for beginner-friendly comments
- **Designing systems:** describe in plain English — Claude architects before coding
- **Debugging:** paste the error + the relevant script
- **When stuck:** ask "what's the smallest thing I can build to test this?"
