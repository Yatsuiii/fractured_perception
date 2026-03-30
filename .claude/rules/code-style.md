# Rust Code Style Rules

## General Principles

- Keep code simple, readable, and modular
- Prefer clarity over cleverness
- Each module should have a single responsibility
- Avoid deep nesting and overly complex logic

---

## Project Structure

- Organize code into modules by system (e.g., state, events, entities)
- Avoid putting too much logic inside `main.rs`
- Use separate files for major systems
- Keep public interfaces minimal

---

## Naming Conventions

- Use `snake_case` for variables and functions
- Use `PascalCase` for structs and enums
- Use clear and descriptive names (avoid abbreviations)
- Boolean variables should read like conditions (e.g., `is_active`, `has_target`)

---

## Data Modeling

- Prefer `struct` over loose variables
- Use `enum` for states and variants
- Avoid using primitive types when a struct provides better meaning
- Keep data structures small and focused

---

## Ownership and Borrowing

- Prefer borrowing (`&`, `&mut`) over cloning
- Avoid unnecessary allocations
- Be explicit about ownership transfer
- Minimize use of `.clone()`

---

## Functions

- Keep functions small and focused
- Each function should do one thing
- Avoid functions longer than ~30–40 lines
- Use meaningful parameter names

---

## Error Handling

- Use `Result<T, E>` for fallible operations
- Use `Option<T>` when value may be absent
- Avoid `unwrap()` in production code
- Handle errors explicitly

---

## State Management

- Use enums to represent states
- Avoid global mutable state
- Keep state transitions predictable
- Centralize state logic where possible

---

## Performance

- Avoid unnecessary heap allocations
- Prefer stack allocation where possible
- Be mindful of loops and repeated work
- Profile before optimizing

---

## Code Smells to Avoid

- Large functions doing multiple things
- Excessive cloning
- Hidden side effects
- Tight coupling between modules
- Magic numbers (use constants instead)

---

## Testing (Future)

- Write unit tests for core systems
- Test edge cases and failure paths
- Keep tests simple and deterministic