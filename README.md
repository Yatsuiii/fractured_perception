# Fractured Perception

Experimental Rust game prototype built around asymmetric perception and co-op communication.

## Concept

Three players experience the same world differently:

- Blind: navigates by sound rather than visuals
- Delayed: sees the real layout, but seconds late
- Hallucinating: sees distorted space, ghost entities, and unreliable signals

The core mechanic is communication under uncertainty. No single player has the full picture, so progress depends on describing partial truths and coordinating around conflicting information.

## Current Prototype

The current Rust build focuses on systems work rather than polish:

- custom engine loop with input, update, perception, and render stages
- distinct per-role perception rules
- FOV and line-of-sight logic
- event logging and hidden state tracking
- multi-stage puzzle and encounter structure
- hidden T/C/I/B state model that shapes outcomes

## Status

This repo is an active prototype, not a finished game. The strongest parts today are the world/perception systems and the underlying design direction.
