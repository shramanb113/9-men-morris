# Nine Men's Morris Engine

A dependency-free Rust core for Nine Men's Morris: bitboard position
representation, full rule enforcement, and Zobrist hashing, built as a
foundation for a search engine and, eventually, a bot you can play against
in the browser.

## Status

Rules engine and search (negamax + alpha-beta, quiescence search,
iterative deepening, Zobrist-backed transposition table) are complete and
tested. WASM adapter hasn't been started yet — see Roadmap.

## Board

24 points, numbered 0..23, mapped directly to bits 0..23 of a `u32`
bitboard — square `N` is always bit `N`, no translation table needed.

```text
  A  B  C  D  E  F  G
1 0        1        2
2    3     4     5
3       6  7  8
4 9 10 11    12 13 14
5      15 16 17
6   18    19    20
7 21       22       23
```

## Architecture

The core is split by concern, not by convenience — each module only knows
about the ones below it:

| Module | Owns |
|---|---|
| `types.rs` | Vocabulary: `Color`, `Square`, `Phase`, `Move`, `GameResult` |
| `board.rs` | Static topology: `BOARD_MASK`, `MOVES[24]`, `MILLS[16]`, bit helpers |
| `position.rs` | `CurrentGameState` — bitboards, hand counts, turn, draw clock, and the invariants that protect them |
| `moves.rs` | `Position + Move → Position` — applying a move (place/slide/fly + capture) |
| `rules.rs` | Phase, mill detection, legal move generation, terminal/draw/result |
| `zobrist.rs` | 64-bit position hashing, used by `search/`'s transposition table |
| `search/` | Negamax + alpha-beta + quiescence search over `rules.rs`, with its own eval, move ordering, and transposition table submodules — no UI/platform dependency |

## Position invariants

`CurrentGameState`'s fields are private; every mutation goes through a
method that keeps these true:

- White and Black never occupy the same square
- Only bits 0..23 are ever set
- Each side's hand + on-board pieces never exceed 9
- A square is exactly one of empty / White / Black

Internal mutators (`place_piece`, `make_move`, ...) check this with
`debug_assert!` — cheap, but compiled out of release builds, so they only
guard against bugs in this crate's own code. External input (a UI, a save
file, a future FFI caller) instead goes through
`CurrentGameState::from_bitboards`, which checks unconditionally and
returns a `Result<_, PositionError>` explaining what's wrong — this is the
one seam where untrusted data is allowed to become a trusted state.

## Testing

```bash
cargo test
```

Covers: invariant enforcement (including deliberately-broken states),
mill detection, full-phase move generation without panicking, Zobrist
correctness (determinism, transposition equality, and the hand-count
collision case a naive board-only hash would get wrong), and search
(terminal/mate-distance scoring, quiescence resolving hanging captures, the
transposition table, move ordering, and deterministic seeded tie-breaking).

`cargo run --release --example play [depth]` self-plays a full game and
prints the board after every move — `depth` is a number or one of
easy/medium/hard (see `search::EASY_DEPTH`/`MEDIUM_DEPTH`/`HARD_DEPTH`).

## Roadmap

1. ~~`search/` — negamax + alpha-beta, iterative deepening, Zobrist-backed
   transposition table~~ done, including quiescence search
2. Difficulty levels: depths chosen (`search::EASY_DEPTH`/`MEDIUM_DEPTH`/
   `HARD_DEPTH`); still need an adapter that pairs easy with
   `search_with_randomization` for an actually-weaker feel, not just
   shallower search
3. `wasm-bindgen` adapter crate (kept separate from the core — see below)
   and a minimal web UI (single live session per browser tab, no backend)
4. Later: a UniFFI adapter crate for a Kotlin Multiplatform client

## Design principles

- **Bitboards throughout.** `u32`, bits 0..23. No `Vec<Option<Player>>`,
  no per-square heap allocation.
- **Functional core.** `make_move` takes `&self`, returns a new
  `CurrentGameState`. No mutable shared game session inside the engine.
- **No I/O, no randomness, no UI in the core.** Keeps it portable to CLI,
  WASM, and native FFI without changes. Platform-specific code belongs in
  a separate adapter crate that depends on this one — never the reverse.
