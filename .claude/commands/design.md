# Command: /design

## Purpose

Design a new system or feature for the Fractured Perception project using clean, modular Rust architecture.

---

## Instructions

Given a feature idea, perform the following:

### 1. System Breakdown

- Identify core components/modules
- Define responsibilities of each module
- Ensure separation of concerns

### 2. Data Modeling

- Define structs and enums
- Identify relationships between entities
- Use strong typing (avoid loose data)

### 3. State and Flow

- Describe how state changes over time
- Define state machines if applicable
- Show how data flows between components

### 4. API Design

- Define key functions and method signatures
- Keep interfaces minimal and clear
- Avoid tight coupling

### 5. Performance Considerations

- Identify potential bottlenecks
- Avoid unnecessary allocations or cloning
- Consider ownership and borrowing

### 6. Error Handling

- Use Result and Option appropriately
- Avoid panics in core systems
- Define failure scenarios

---

## Output Format

### Feature Overview

Short description of the feature

### System Architecture

- Module 1
  - Responsibility
- Module 2
  - Responsibility

### Data Structures

```rust
// Example structs/enums
