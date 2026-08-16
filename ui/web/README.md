# Morris — web UI

The React/Vite web UI for the Nine Men's Morris engine. Play against the
bot in the browser — no backend, no signup, one live session per tab.

Live at **https://playmorris.vercel.app**.

## What's here

- Play/difficulty/color selection, mill capture, and phase transitions
  (placing → sliding → flying) driven entirely by the compiled
  `engine/wasm` binary in `public/wasm/` — this UI never re-implements
  game rules in JS.
- Smooth piece placement/slide/capture animations (Framer Motion).
- A hint button backed by the engine's own search (`GameSession::hint`),
  gated by a small `localStorage` credit economy: a few free hints, more
  via the share flow.
- A custom share modal (X / LinkedIn / WhatsApp intents, not the native
  OS share sheet) for the win state and for earning hint credits.
- SEO: Open Graph/Twitter meta, JSON-LD, `robots.txt`, `sitemap.xml`.

## Quick start

```bash
npm install
npm run dev      # http://localhost:5173
npm run build    # tsc -b && vite build -> dist/
npm run preview  # serve the production build locally
npm run lint      # oxlint
```

## Rebuilding the wasm binary

`public/wasm/ninemensmorris_wasm.js` and `_bg.wasm` are committed, built
from `engine/wasm`. After changing the Rust engine, regenerate them:

```bash
cd ../../engine/wasm
wasm-pack build --target web --out-dir ../../ui/web/public/wasm
```

The glue is loaded via `fetch` + a Blob-URL `import()` in
`src/lib/wasmEngine.ts` (see the comment there) rather than a normal
`import`, because Vite's dev server won't serve `/public/*.js` as an ES
module.

## Deployment

Deployed on Vercel, connected to this repo's `main` branch with **Root
Directory: `ui/web`** and **Framework Preset: Vite** (the repo also
contains the unrelated Rust engine, so both must be set explicitly —
see `vercel.json`).
