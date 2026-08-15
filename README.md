# 9-Men-Morris Engine

# Milestone 0 — Project Skeleton & Foundation

**Goal:** Create a clean, future-proof foundation for the Nine Men's Morris engine.
No game logic yet. Establish the domain vocabulary, bitboard representation, and board topology.

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

* [x] All files above exist
* [x] `lib.rs` declares the core modules:

```rust
pub mod types;
pub mod board;
pub mod position;
pub mod moves;
pub mod rules;
// pub mod search; // uncomment when search is introduced
```

* [x] Pure game core remains independent from UI and I/O
* [x] Search/AI remains separate from rules
* [x] Board topology is represented as static data

---

# 3. Fundamental Types — `types.rs`

Define the stable vocabulary of the engine.

## Player

* [x] Define `Color` or `Player`
* [x] Include `White`
* [x] Include `Black`
* [x] Provide a natural way to obtain the opponent

## Square

* [x] Define `Square`
* [x] Represent exactly 24 board points
* [x] Canonical range is `0..23`
* [x] Make conversions between `Square` and bit positions explicit

## Phase

* [x] Define `Phase`
* [x] Include:

  * [x] `Placing`
  * [x] `Moving`
  * [ ] `Flying`
* [x]Decide whether phase is stored or derived from the position

## Move

* [x] Define a structured `Move`
* [x] Support placement
* [x] Support normal movement
* [x] Leave room for flying
* [x] Keep move representation independent from UI

## GameResult

* [x] Define `GameResult`
* [x] Include:

  * [x] `Ongoing`
  * [x] `Winner(Player)`
  * [x] `Draw`

## Bitboard

* [ ] Establish `u32` as the canonical bitboard type
* [ ] Only the lower 24 bits are used
* [ ] Document this invariant clearly

Example:

```rust
pub type Bitboard = u32;

pub const BOARD_MASK: Bitboard = 0x00FF_FFFF;
```

**Core invariant:**

```text
bit N ↔ Square N
```

Therefore:

```text
Square 0 → bit 0
Square 1 → bit 1
...
Square 23 → bit 23
```

---

# 4. Board Topology — `board.rs`

The board will use **bitboards as its canonical topology representation**.

Do **not** introduce an adjacency-list representation as the primary structure.

The board module should contain immutable, precomputed data describing the topology.

---

## 4.1 Canonical Square Numbering

* [ ] Define exactly 24 squares
* [ ] Number squares from `0` to `23`
* [ ] Document the numbering with an ASCII diagram
* [ ] Ensure numbering remains stable for the lifetime of the engine
* [ ] Ensure square numbers map directly to bit positions

The fundamental mapping is:

```text
Square N
   │
   ▼
Bit N
```

This means no translation table is required between a square and its bitboard representation.

---

## 4.2 Board Mask

Define the valid board bits:

```rust
pub const BOARD_MASK: u32 = 0x00FF_FFFF;
```

This guarantees that only bits `0..23` represent actual board squares.

* [ ] `BOARD_MASK` is defined
* [ ] All board operations respect the mask
* [ ] Bits `24..31` are never used to represent squares

---

## 4.3 Adjacency as Move Bitboards

Instead of:

```text
square → list of neighboring squares
```

use:

```rust
static MOVES: [u32; 24] = [
    // bitboard of reachable adjacent squares for each square
];
```

Each entry represents the squares directly connected to that square.

For example:

```text
MOVES[sq]
```

contains a `1` for every square that can be reached from `sq` during a normal movement phase.

* [ ] Define `MOVES: [Bitboard; 24]`
* [ ] Every square has a precomputed adjacency bitboard
* [ ] No runtime graph construction is required
* [ ] No heap allocation is required
* [ ] Adjacency is represented entirely through bit operations
* [ ] Verify adjacency relationships are correct
* [ ] Verify adjacency symmetry where applicable

Example move-generation primitive:

```rust
let empty = BOARD_MASK & !(white | black);
let targets = MOVES[square as usize] & empty;
```

This should become one of the fundamental primitives used by the future rules engine.

---

# 4.4 Mills as Bitboards

Represent all 16 standard mills as bitboards.

```rust
static MILLS: [Bitboard; 16] = [
    // 16 three-square mill bitboards
];
```

Each mill contains exactly three set bits.

Example conceptual representation:

```text
mill = bit(a) | bit(b) | bit(c)
```

* [ ] Define exactly 16 mills
* [ ] Each mill contains exactly 3 squares
* [ ] Every square referenced by a mill is within `0..23`
* [ ] No mill contains duplicate squares
* [ ] No standard mill is missing
* [ ] No standard mill is duplicated
* [ ] Mills are immutable static data

---

# 4.5 Mill Detection Primitive

The board representation should make mill detection a simple bitwise operation.

For a player's bitboard:

```rust
(player & mill) == mill
```

means the player occupies every square in that mill.

Therefore:

```text
Player Bitboard
      │
      ▼
   AND mill
      │
      ▼
 equals mill?
      │
   ┌──┴──┐
  yes    no
  mill   no mill
```

* [ ] Confirm this operation works for every mill
* [ ] Keep the primitive independent from game rules
* [ ] Do not implement complete mill/removal rules in Milestone 0

---

# 4.6 Useful Precomputed Board Data

The board module may contain additional static bitboard data where it simplifies future operations.

Potential examples:

```rust
pub const BOARD_MASK: Bitboard = ...;

pub static MOVES: [Bitboard; 24] = [...];

pub static MILLS: [Bitboard; 16] = [...];
```

Future derived/precomputed structures may include:

```text
square → mills containing that square
square → movement mask
mill → square mask
```

Only add them when they provide a clear advantage.

**Principle:** Board topology should be static, compact, deterministic, and allocation-free.

---

# 5. Position Representation — `position.rs`

The position representation should be designed around bitboards from the beginning.

A conceptual representation:

```rust
pub struct Position {
    pub white: Bitboard,
    pub black: Bitboard,
    // side to move
    // piece counts
    // additional state if required
}
```

The exact final structure can evolve as rules are implemented.

---

## 5.1 Occupancy

Define:

```text
occupied = white | black
```

Empty squares:

```text
empty = BOARD_MASK & !occupied
```

* [ ] White pieces represented as a bitboard
* [ ] Black pieces represented as a bitboard
* [ ] Occupied squares can be derived with OR
* [ ] Empty squares can be derived with AND/NOT + `BOARD_MASK`
* [ ] No per-square heap allocation is required

---

## 5.2 Position Invariants

Establish these invariants early:

* [ ] White and Black never overlap
* [ ] Only bits `0..23` may be set
* [ ] Piece counts agree with the corresponding bitboards
* [ ] A square is either empty, White, or Black
* [ ] No invalid board state can silently enter the core

Example invariant:

```rust
(white & black) == 0
```

and:

```rust
(white | black) & !BOARD_MASK == 0
```

---

# 6. Move Representation — `moves.rs`

Moves should operate naturally on the bitboard-based position representation.

* [ ] Define structured `Move`
* [ ] Support placement moves
* [ ] Support normal movement
* [ ] Leave room for flying
* [ ] Keep move representation independent from search
* [ ] Keep move representation independent from UI

The future engine should be able to transform:

```text
Position + Move → Position
```

without knowing anything about the interface calling it.

---

# 7. Rules Boundary — `rules.rs`

Milestone 0 establishes the boundary only.

Do **not** implement complete gameplay yet.

* [ ] Create `rules.rs`
* [ ] Define appropriate rule-engine interfaces/helpers
* [ ] Keep rules independent from search
* [ ] Keep rules independent from CLI/UI
* [ ] Make the future rules engine consume the bitboard-based `Position`
* [ ] Make the future rules engine use `MOVES` and `MILLS`

Do **not** implement yet:

* [ ] Complete legal move generation
* [ ] Complete placement rules
* [ ] Mill formation rules
* [ ] Piece removal rules
* [ ] Flying rules
* [ ] Win detection
* [ ] Draw detection

Those belong to **Milestone 1 — Perfect Rules**.

---

# 8. Search Boundary

Search will operate on the same compact position representation.

Architecture:

```text
┌─────────────────────────────┐
│          Search / AI        │
│ Minimax / Alpha-Beta / etc. │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│            Rules            │
│ legality / generation / win │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│          Position           │
│     White / Black u32       │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│            Board            │
│ MOVES[24] + MILLS[16]       │
└─────────────────────────────┘
```

* [ ] Search does not know about UI
* [ ] Search does not own board topology
* [ ] Search does not implement game rules
* [ ] Rules do not depend on search
* [ ] Board does not depend on rules
* [ ] Position does not depend on search

---

# 9. Bitboard Design Principles

These principles are now part of the engine architecture.

## Principle 1 — One Bit, One Square

```text
bit 0  → square 0
bit 1  → square 1
...
bit 23 → square 23
```

No unnecessary mapping layer.

---

## Principle 2 — Board State Is Compact

A position should be representable primarily through:

```text
White   → u32
Black   → u32
Turn    → small enum
Metadata → small values if necessary
```

Avoid representing the board as:

```text
Vec<Option<Player>>
HashMap<Square, Player>
Vec<Piece>
```

unless a future requirement proves such a representation necessary.

---

## Principle 3 — Static Topology

The board itself never changes.

Therefore:

```text
MOVES
MILLS
BOARD_MASK
```

should be compile-time/static data.

---

## Principle 4 — Bit Operations First

When a board operation can naturally be expressed as a bitwise operation, prefer that representation.

Examples:

```rust
occupied = white | black;
empty = BOARD_MASK & !occupied;
targets = MOVES[square] & empty;
mill = (player & mill_mask) == mill_mask;
```

---

# 10. Testing Foundation

Create the testing foundation before implementing complete rules.

* [ ] `tests/` directory exists
* [ ] Unit testing strategy is established
* [ ] Board topology will be tested independently
* [ ] Bitboard invariants will be tested
* [ ] Position invariants will be tested
* [ ] Rules correctness will be heavily tested

---

## 10.1 Board Tests

Eventually verify:

* [ ] Exactly 24 squares exist
* [ ] `BOARD_MASK` contains exactly 24 bits
* [ ] No adjacency mask references an invalid square
* [ ] Every adjacency mask uses only bits `0..23`
* [ ] Adjacency relationships are correct
* [ ] Adjacency is symmetric where appropriate
* [ ] Exactly 16 mills exist
* [ ] Every mill contains exactly 3 bits
* [ ] Every mill references valid squares
* [ ] No duplicate mills exist

---

## 10.2 Bitboard Tests

Verify:

```text
Square N → bit N
```

for all 24 squares.

Also verify:

* [ ] Empty board has `white == 0`
* [ ] Empty board has `black == 0`
* [ ] White/Black overlap is rejected
* [ ] Bits outside the board are rejected
* [ ] Occupancy calculation is correct
* [ ] Empty-square calculation is correct

---

## 10.3 Mill Tests

For every mill:

```rust
(player & mill) == mill
```

must correctly identify a completed mill.

Test:

* [ ] Exact mill occupancy
* [ ] Partial mill occupancy
* [ ] Extra pieces outside the mill
* [ ] Opponent occupying one square
* [ ] Empty square in the mill

---

# 11. Cargo Configuration

* [ ] Package name is set
* [ ] Description is set
* [ ] Rust edition is `2021` or `2024`
* [ ] Library target is configured correctly
* [ ] No unnecessary dependencies
* [ ] Metadata is clean

Prefer starting with:

```text
External dependencies: 0
```

The engine should initially rely entirely on the Rust standard library.

---

# 12. Documentation

* [ ] Add a project README
* [ ] Explain the engine architecture
* [ ] Explain the 24-square numbering
* [ ] Explain the bitboard representation
* [ ] Explain `MOVES`
* [ ] Explain `MILLS`
* [ ] Explain `Position`
* [ ] Explain dependency direction
* [ ] Explain where search will eventually live

A contributor should understand the board representation without reading the implementation.

---

# 13. Compilation Checkpoint

Run:

```bash
cargo check
```

* [ ] Compilation succeeds
* [ ] No architectural warnings remain
* [ ] All modules compile
* [ ] No unnecessary dependencies exist

Then:

```bash
cargo test
```

* [ ] Test suite executes successfully
* [ ] Bitboard tests pass
* [ ] Board topology tests pass
* [ ] No complete game-rule tests are required yet

---

# 14. Architecture Checkpoint

Before moving to Milestone 1, you should be able to answer:

### What represents a square?

```text
Square 0..23
```

### What represents board occupancy?

```text
u32 bitboards
```

### What represents White?

```text
Position.white
```

### What represents Black?

```text
Position.black
```

### How do you calculate empty squares?

```rust
BOARD_MASK & !(white | black)
```

### How do you find movement targets?

```rust
MOVES[square] & empty
```

### How are mills represented?

```text
16 × Bitboard
```

### How do you check a mill?

```rust
(player & mill) == mill
```

### Where does game legality live?

```text
rules.rs
```

### Where does search live?

```text
search/
```

### Where does UI live?

```text
Outside the pure core
```

* [ ] All answers are clear
* [ ] Bitboards are understood
* [ ] Board topology is understood
* [ ] Module responsibilities are clear

---

# 15. Definition of Done

Milestone 0 is complete when all of the following are true:

* [ ] Project compiles with `cargo check`
* [ ] `cargo test` succeeds
* [ ] Clean module structure exists
* [ ] Fundamental domain types exist
* [ ] `Bitboard = u32` is established
* [ ] Exactly 24 board bits are used
* [ ] Square `N` maps directly to bit `N`
* [ ] `BOARD_MASK` is defined
* [ ] `MOVES[24]` is defined
* [ ] All adjacency relationships are encoded as bitboards
* [ ] `MILLS[16]` is defined
* [ ] All mills are encoded as bitboards
* [ ] Position uses White/Black bitboards
* [ ] Position invariants are established
* [ ] Move representation exists
* [ ] Rules boundary exists
* [ ] Search boundary exists
* [ ] Board topology contains no gameplay logic
* [ ] Core contains no I/O
* [ ] Core contains no randomness
* [ ] Core contains no UI concerns
* [ ] No complete game logic has been implemented
* [ ] Bitboard topology tests pass
* [ ] Architecture can support CLI
* [ ] Architecture can support WASM
* [ ] Architecture can support future FFI/Kotlin
* [ ] The entire board can be mentally understood as a small collection of `u32`s

---

# Milestone 0 Checkpoint

The final architecture should conceptually look like:

```text
                    9-Men-Morris Engine
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
        Types              Board             Position
          │                  │                  │
          │           ┌──────┴──────┐           │
          │           │             │           │
          │        MOVES[24]     MILLS[16]      │
          │           │             │           │
          └───────────┴─────────────┴───────────┘
                             │
                             ▼
                           Rules
                             │
                             ▼
                          Search
```

The key architectural decision is:

```text
                 BOARD
                   │
        ┌──────────┴──────────┐
        │                     │
   MOVES[24]              MILLS[16]
    Bitboards              Bitboards
        │                     │
        └──────────┬──────────┘
                   │
                   ▼
               POSITION
          ┌────────┴────────┐
          │                 │
      white: u32        black: u32
```

**No gameplay yet.**

Milestone 0 only establishes a compact, deterministic, bitboard-native foundation that the rules engine and eventual search engine can build upon.

---

# Next Milestone

Once every checkbox above is complete:

# Milestone 1 — Perfect Rules

Implement the complete Nine Men's Morris rules engine using the bitboard foundation.

The rules engine should eventually provide:

```text
Position
   │
   ├── Generate legal moves
   │
   ├── Apply move
   │
   ├── Detect mills
   │
   ├── Handle removals
   │
   ├── Handle placing
   │
   ├── Handle moving
   │
   ├── Handle flying
   │
   └── Detect win/draw
```

**Do not begin Milestone 1 until Milestone 0 passes its Definition of Done.**
