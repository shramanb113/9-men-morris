# Nine Men's Morris

A Nine Men's Morris engine, and a browser bot built on top of it.

Play it live: **https://playmorris.vercel.app**

## Layout

- **`engine/`** — a self-contained Cargo workspace with the Rust core
  (bitboard position, rules, Zobrist hashing) and the negamax/alpha-beta
  search engine, plus a `wasm-bindgen` adapter crate that packages it for
  the browser. See [`engine/README.md`](engine/README.md) for the
  architecture, testing, and design principles.
- **`ui/web/`** — the React/Vite web UI: play against the bot, pick a
  difficulty and a side, get hints backed by the engine's own search, and
  share a win. Builds against the packaged `engine/wasm` output; nothing
  in `engine/` depends on it. See [`ui/web/README.md`](ui/web/README.md).

## Quick start

```bash
cd engine
cargo test --workspace                    # run everything
cargo run --release --example play medium # watch the engine play itself
```

```bash
cd ui/web
npm install
npm run dev # http://localhost:5173
```
