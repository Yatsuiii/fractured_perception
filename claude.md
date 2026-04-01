# Fractured Perception – AI Instructions

## Project Overview

Fractured Perception is a systems-driven project built entirely in Rust.

The focus is on:

- Clean architecture
- High performance
- Predictable behavior
- Strong type safety

This is not a scripting-heavy project. All core logic is implemented in Rust.

---

## Development Philosophy

- Build systems, not scripts
- Prioritize clarity over cleverness
- Prefer deterministic and predictable logic
- Keep components modular and decoupled
- Avoid premature optimization, but be performance-aware

---

## Architecture Guidelines

- Separate logic into well-defined modules
- Each system must have a clear responsibility
- Avoid tight coupling between systems
- Use enums to represent states and transitions
- Keep data flow explicit and traceable

---

## Code Expectations

- Follow Rust ownership and borrowing rules strictly
- Minimize unnecessary allocations and cloning
- Use strong typing (structs/enums instead of primitives)
- Avoid global mutable state
- Keep functions small and focused

---

## Error Handling

- Use `Result` for fallible operations
- Use `Option` for nullable values
- Avoid `unwrap()` in core systems
- Handle errors explicitly and safely

---

## Performance Guidelines

- Avoid unnecessary heap allocations
- Prefer borrowing over cloning
- Be mindful of hot paths (loops, updates)
- Optimize only when needed and measurable

---

## Debugging Approach

- Identify the exact source of the issue before fixing
- Trace data flow step-by-step
- Do not guess — explain the root cause
- Keep debugging changes minimal and reversible

---

## AI Behavior Rules

When assisting:

- Always follow project architecture and code-style rules
- Do not generate overly complex or abstract solutions
- Prefer simple, maintainable implementations
- Explain design decisions clearly
- Highlight trade-offs when relevant

---

## Commands Usage

- `/design` → for system architecture and planning
- `/review` → for code quality and improvements
- `/fix` → for debugging and resolving issues

---

## Current Scope

- Pure Rust implementation
- No Lua or external scripting layers
- Focus on building core systems first
- Three roles: Blind, Delayed, Hallucinating — each with fully distinct perception views
- Five stages with per-role encounter perception, threshold-driven world mutations, and stage progression
- Engine split into submodules: input, render, movement, update, stage, dialogue

Future expansions may include additional layers, but not at this stage.
