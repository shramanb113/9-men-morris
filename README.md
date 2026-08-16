# Nine Men's Morris

A Nine Men's Morris engine and, eventually, a browser bot built on top of it.

## Layout

- **`engine/`** — the actual project: a self-contained Cargo workspace with
  the Rust core (bitboard position, rules, Zobrist hashing) and the
  negamax/alpha-beta search engine, plus a `wasm-bindgen` adapter crate
  that packages it for the browser. See [`engine/README.md`](engine/README.md)
  for the architecture, testing, and design principles.
- **`ui/`** — reserved, empty. Nothing in `engine/` depends on it; it's
  where a UI gets built against the packaged `engine/wasm` output, later.

## Quick start

```bash
cd engine
cargo test --workspace                    # run everything
cargo run --release --example play medium # watch the engine play itself
```
