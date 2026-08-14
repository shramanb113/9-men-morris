# 9-Men-Morris Engine

# Milestone 0 — Project Skeleton & Foundation

**Goal:** Create a clean, future-proof foundation for the Nine Men's Morris engine.
No game logic yet. Only structure, vocabulary, and board topology.

**Status:** ⬜ Not started

---

## 1. Project Initialization

* [x] Created project with `cargo init --lib`
* [x] Project folder has a clean name (e.g. `morris` or `ninemensmorris`)
* [x] `Cargo.toml` exists and is valid
* [x] Basic `src/lib.rs` exists

---

## 2. Folder & Module Structure

Create the following structure:

```text
src/
├── lib.rs
├── types.rs
├── board.rs
├── position.rs
├── moves.rs
├── rules.rs
└── search/
    └── mod.rs
```

* [ ] All files above exist
* [ ] `lib.rs` declares the core modules:

```rust
pub mod types;
pub mod board;
pub mod position;
pub mod moves;
pub mod rules;
// pub mod search; // uncomment when search is introduced
```

* [ ] Folder structure follows the principle: **pure core stays clean**
* [ ] Search/AI is isolated from the game rules
* [ ] No UI or I/O code exists inside the core

---

## 3. Fundamental Types — `types.rs`

Define the basic vocabulary of the engine.

### Player

* [ ] Define `Color` or `Player`
* [ ] Include `White`
* [ ] Include `Black`
* [ ] Provide a natural way to get the opposing player

### Square

* [ ] Define `Square`
* [ ] Represent exactly 24 board points
* [ ] Use `0..23` as the canonical representation
* [ ] Prevent accidental use of arbitrary integers where practical

### Phase

* [ ] Define `Phase`
* [ ] Include:

  * [ ] `Placing`
  * [ ] `Moving`
  * [ ] `Flying`
* [ ] Decide whether phase is stored or derived from a position

### Move

* [ ] Define a structured `Move`
* [ ] It should represent the information necessary to describe a legal action
* [ ] Avoid coupling the move representation to CLI/UI concerns

### GameResult

* [ ] Define `GameResult`
* [ ] Include:

  * [ ] `Ongoing`
  * [ ] `Winner(Player)`
  * [ ] `Draw`

### Other Small Types

* [ ] Add additional newtypes/enums only when they represent stable domain concepts
* [ ] Avoid premature abstractions
* [ ] Keep the vocabulary small and explicit

**Principle:** These names form the language of the engine. Changing them later should be expensive, so choose them deliberately.

---

## 4. Board Topology — `board.rs`

The board topology is pure immutable data.

### Numbering Scheme

* [ ] Decide the canonical numbering for all 24 points
* [ ] Number points from `0` to `23`
* [ ] Document the numbering with an ASCII diagram
* [ ] Make the numbering easy to understand when debugging positions

Example:

```text
0 -------- 1 -------- 2
|          |          |
|  3 ----- 4 ----- 5  |
|  |       |       |  |
|  |  6 -- 7 -- 8  |  |
|  |  |         |  |  |
9 -10-11       12-13-14
|  |  |         |  |  |
|  | 15-16-17-18 |  |
|  19 ----20----21  |
|          |          |
22 --------23---------24
```

> Adjust the diagram/numbering so the final representation contains exactly 24 canonical squares numbered `0..23`.

### Adjacency

* [ ] Define the adjacency relationships
* [ ] Every square knows which squares can be reached by a normal move
* [ ] Adjacency is represented as static/constant data
* [ ] No runtime graph construction is necessary
* [ ] Verify adjacency relationships manually

### Mills

* [ ] Define all 16 standard mills
* [ ] Each mill contains exactly 3 squares
* [ ] Store mills as static/constant data
* [ ] Ensure every standard mill is represented exactly once
* [ ] Document the relationship between board numbering and mill definitions

### Topology Rules

* [ ] `board.rs` contains topology only
* [ ] No move validation exists here
* [ ] No player state exists here
* [ ] No randomness exists here
* [ ] No I/O exists here

---

## 5. Position Foundation — `position.rs`

Create the foundation for representing a game position.

* [ ] Define the board occupancy representation
* [ ] Represent empty squares
* [ ] Represent White pieces
* [ ] Represent Black pieces
* [ ] Represent the player whose turn it is
* [ ] Represent enough state to determine the current phase
* [ ] Keep the representation compact and deterministic

**Important:** Do not implement complete game transitions yet.
This milestone only establishes the data model.

---

## 6. Move Representation — `moves.rs`

Create the move-domain layer without implementing complete rules.

* [ ] Define the structure of a move
* [ ] Represent placement moves
* [ ] Represent movement moves
* [ ] Leave room for flying moves
* [ ] Leave room for mill/removal information if the final design requires it
* [ ] Keep move generation separate from move validation

**Principle:**

```text
Position → Move → Rules
```

Search should eventually operate on these abstractions rather than directly manipulating UI state.

---

## 7. Rules Boundary — `rules.rs`

Create the boundary where game rules will eventually live.

For Milestone 0:

* [ ] Create `rules.rs`
* [ ] Define placeholders/interfaces where useful
* [ ] Do not implement complete legality checking
* [ ] Do not implement win detection
* [ ] Do not implement mill detection logic
* [ ] Do not implement piece removal
* [ ] Do not implement move generation

The purpose is to establish a clean architectural boundary for Milestone 1.

---

## 8. Search Boundary

Search/AI must remain separate from the rules engine.

```text
┌───────────────────────────────┐
│           Search / AI         │
│   Minimax / Alpha-Beta / etc. │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│          Game Rules            │
│  legality / mills / win state │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│       Position + Board         │
│     pure domain structures     │
└───────────────────────────────┘
```

* [ ] Search does not live inside `rules.rs`
* [ ] Rules do not depend on search
* [ ] Board topology does not depend on search
* [ ] Core types do not depend on UI
* [ ] Search can be added later without restructuring the core

---

## 9. Core Design Principles

### Pure Core

* [ ] No `println!`
* [ ] No CLI handling
* [ ] No filesystem access
* [ ] No network access
* [ ] No randomness
* [ ] No wall-clock/time dependencies
* [ ] No global mutable state

### Dependency Direction

Dependencies should flow inward:

```text
CLI / WASM / FFI
       │
       ▼
    Search
       │
       ▼
     Rules
       │
       ▼
Position + Board + Types
```

Never reverse this dependency direction.

### Future Interfaces

The architecture should leave clean doors open for:

* [ ] Native CLI
* [ ] WASM/web interface
* [ ] Future FFI
* [ ] Kotlin/JVM integration
* [ ] External AI/search implementations
* [ ] Game replay/import/export

None of these need to be implemented in Milestone 0.

---

## 10. Testing Foundation

Create the testing foundation before implementing rules.

* [ ] `tests/` directory exists
* [ ] Unit testing strategy is decided
* [ ] Board topology will be tested
* [ ] Position representation will be tested
* [ ] Rules correctness will be heavily tested
* [ ] Search correctness will eventually be tested independently

Initial topology tests should eventually verify:

* [ ] Exactly 24 squares exist
* [ ] Every square has valid neighbors
* [ ] Adjacency is symmetric where appropriate
* [ ] Exactly 16 mills exist
* [ ] Every mill contains 3 distinct squares
* [ ] Every referenced square is within `0..23`

---

## 11. Cargo Configuration

Configure `Cargo.toml` cleanly.

* [ ] Package name is set
* [ ] Package description is set
* [ ] Rust edition is set to `2021` or `2024`
* [ ] No unnecessary dependencies are added
* [ ] Library target is configured correctly
* [ ] Metadata is clean
* [ ] Future dependencies are added only when required

The project should ideally begin with **zero external dependencies**.

---

## 12. Documentation

* [ ] Add a project-level README
* [ ] Explain what the engine is
* [ ] Explain the architecture
* [ ] Document the board numbering
* [ ] Document the core modules
* [ ] Explain where search will live
* [ ] Explain how external interfaces will eventually connect

A new contributor should be able to understand the architecture without reading the implementation.

---

## 13. Compilation Checkpoint

Run:

```bash
cargo check
```

* [ ] Compilation succeeds
* [ ] No warnings caused by unfinished architecture
* [ ] All modules are correctly connected
* [ ] No unnecessary dependencies are present

Then run:

```bash
cargo test
```

* [ ] Test suite starts successfully
* [ ] Foundation tests pass
* [ ] No game-rule tests are required yet

---

# 14. Architecture Checkpoint

Before moving forward, answer these questions without looking at the code:

### Where is the board topology?

```text
src/board.rs
```

### Where are domain types?

```text
src/types.rs
```

### Where is a complete game position represented?

```text
src/position.rs
```

### Where will moves be represented?

```text
src/moves.rs
```

### Where will game legality live?

```text
src/rules.rs
```

### Where will AI/search live?

```text
src/search/
```

### Where will CLI/web/FFI live?

```text
Outside the pure core.
```

* [ ] All answers are clear
* [ ] No module has an unclear responsibility
* [ ] No circular architectural dependency exists

---

# 15. Definition of Done

Milestone 0 is complete only when **all** of the following are true:

* [ ] Project compiles with `cargo check`
* [ ] `cargo test` runs successfully
* [ ] Module structure is clean
* [ ] Fundamental domain types exist
* [ ] Board numbering is documented
* [ ] 24 squares are represented
* [ ] Adjacency is defined
* [ ] 16 standard mills are defined
* [ ] Position representation exists
* [ ] Move representation exists
* [ ] Rules boundary exists
* [ ] Search boundary exists
* [ ] Core contains no I/O
* [ ] Core contains no randomness
* [ ] Core contains no UI concerns
* [ ] No real game logic has been implemented
* [ ] Architecture can support CLI
* [ ] Architecture can support WASM
* [ ] Architecture can support future FFI/Kotlin
* [ ] You can explain the architecture in 60 seconds
* [ ] The project feels small, boring, and extremely easy to extend

---

# Milestone 0 Checkpoint

**Expected outcome:**

```text
9-Men-Morris Engine
        │
        ├── Types
        │    └── Domain vocabulary
        │
        ├── Board
        │    └── Static topology
        │
        ├── Position
        │    └── Game state representation
        │
        ├── Moves
        │    └── Move representation
        │
        ├── Rules
        │    └── Future rule engine
        │
        └── Search
             └── Future AI
```

**No gameplay yet.**

The only achievement of Milestone 0 is that the engine now has a stable language, a known board topology, and clean architectural boundaries.

---

## Next Milestone

Once every checkbox above is complete:

# Milestone 1 — Perfect Rules

Implement the complete Nine Men's Morris rules engine with exhaustive tests.

**Do not begin Milestone 1 until Milestone 0 passes its Definition of Done.**
