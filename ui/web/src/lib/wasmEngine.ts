// Thin typed boundary around the generated wasm-bindgen glue in
// /public/wasm — loaded from public/ (not bundled) so the glue's own
// relative `new URL(..., import.meta.url)` fetch of the .wasm binary
// resolves correctly in both dev and a static production build.

export type Color = 0 | 1; // 0 white, 1 black
export type Phase = 0 | 1 | 2; // placing, sliding, flying

export interface LegalMove {
  readonly from: number | undefined;
  readonly to: number;
}

export interface MoveResult {
  readonly from: number | undefined;
  readonly to: number;
  readonly captures: Uint8Array;
  readonly score: number;
}

export interface GameSession {
  board(): Uint8Array;
  side_to_move(): Color;
  phase(): Phase;
  pieces_in_hand(color: Color): number;
  legal_moves(): LegalMove[];
  capture_targets(from: number | undefined, to: number): Uint8Array;
  play_move(from: number | undefined, to: number, captures: Uint8Array): void;
  bot_move(difficulty: Difficulty, seed: bigint): MoveResult;
  hint(difficulty: Difficulty): MoveResult;
  is_game_over(): boolean;
  winner(): Color | undefined;
}

export type Difficulty = 'easy' | 'medium' | 'hard';

interface WasmModule {
  default: (input: string) => Promise<unknown>;
  GameSession: { new_game(): GameSession };
}

let modulePromise: Promise<WasmModule> | null = null;

/**
 * Loads and initializes the wasm module exactly once per page load.
 *
 * The generated glue lives in /public so it (and the .wasm binary next to
 * it) ship untouched by the bundler. Vite's dev server refuses to serve
 * files under /public through its module-transform pipeline though — even
 * a `@vite-ignore`d dynamic import 404s with "Cannot import non-asset
 * file" — so it's fetched as text and imported from a Blob URL instead.
 * `import.meta.url` inside a blob-sourced module doesn't resolve to
 * anything meaningful, so the .wasm path is passed explicitly.
 */
function loadModule(): Promise<WasmModule> {
  if (!modulePromise) {
    modulePromise = (async () => {
      const glueSource = await fetch('/wasm/ninemensmorris_wasm.js').then((r) => r.text());
      const blobUrl = URL.createObjectURL(new Blob([glueSource], { type: 'text/javascript' }));
      const mod = (await import(/* @vite-ignore */ blobUrl)) as WasmModule;
      await mod.default('/wasm/ninemensmorris_wasm_bg.wasm');
      return mod;
    })();
  }
  return modulePromise;
}

export async function newGameSession(): Promise<GameSession> {
  const mod = await loadModule();
  return mod.GameSession.new_game();
}

/** Fresh entropy for the bot's seeded difficulty randomization — the wasm layer never generates its own. */
export function freshSeed(): bigint {
  return BigInt(Date.now()) ^ BigInt(Math.floor(Math.random() * 1e9));
}
