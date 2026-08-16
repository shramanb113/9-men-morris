/* @ts-self-types="./ninemensmorris_wasm.d.ts" */

/**
 * One live game, mutated in place — the "single session per browser tab"
 * model. Wraps a `CurrentGameState`, which is itself immutable/functional
 * (`make_move` returns a new state); this struct is where that gets
 * turned into something a JS caller can hold onto and update.
 */
export class GameSession {
    static __wrap(ptr) {
        const obj = Object.create(GameSession.prototype);
        obj.__wbg_ptr = ptr;
        GameSessionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GameSessionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gamesession_free(ptr, 0);
    }
    /**
     * 24 entries, one per square: 0 empty, 1 white, 2 black.
     * @returns {Uint8Array}
     */
    board() {
        const ret = wasm.gamesession_board(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Runs the bot's move at the given difficulty ("easy"/"medium"/"hard")
     * and applies it. `seed` is supplied by the caller — this crate, like
     * the core, never generates its own randomness; JS owns real entropy
     * (`Date.now()`, `Math.random()`, ...) and passes a fresh seed each
     * call. Runs synchronously and can take several seconds at "hard" —
     * call this from a Web Worker, not the main thread.
     * @param {string} difficulty
     * @param {bigint} seed
     * @returns {MoveResult}
     */
    bot_move(difficulty, seed) {
        const ptr0 = passStringToWasm0(difficulty, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.gamesession_bot_move(this.__wbg_ptr, ptr0, len0, seed);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return MoveResult.__wrap(ret[0]);
    }
    /**
     * Legal individual capture squares for the move `(from, to)`, before
     * it's actually played. Empty means that move doesn't form a mill and
     * can be played via `play_move` with no captures. Non-empty means the
     * human must choose (1 square normally, 2 distinct ones for a move
     * that completes two mills at once) before calling `play_move`.
     * @param {number | null | undefined} from
     * @param {number} to
     * @returns {Uint8Array}
     */
    capture_targets(from, to) {
        const ret = wasm.gamesession_capture_targets(this.__wbg_ptr, isLikeNone(from) ? 0xFFFFFF : from, to);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Suggests the strongest move for the side to move, searched at the
     * same depth tier as `difficulty`, without applying it — the session
     * is untouched, so the player can take it or ignore it. Same
     * synchronous blocking-time caveat as `bot_move` applies at "hard".
     * @param {string} difficulty
     * @returns {MoveResult}
     */
    hint(difficulty) {
        const ptr0 = passStringToWasm0(difficulty, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.gamesession_hint(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return MoveResult.__wrap(ret[0]);
    }
    /**
     * @returns {boolean}
     */
    is_game_over() {
        const ret = wasm.gamesession_is_game_over(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Every currently-legal `(from, to)` pair, deduplicated across
     * capture-choice variants — capture selection is a separate step, see
     * `capture_targets`.
     * @returns {LegalMove[]}
     */
    legal_moves() {
        const ret = wasm.gamesession_legal_moves(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]);
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {GameSession}
     */
    static new_game() {
        const ret = wasm.gamesession_new_game();
        return GameSession.__wrap(ret);
    }
    /**
     * 0 placing, 1 sliding, 2 flying.
     * @returns {number}
     */
    phase() {
        const ret = wasm.gamesession_phase(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} color
     * @returns {number}
     */
    pieces_in_hand(color) {
        const ret = wasm.gamesession_pieces_in_hand(this.__wbg_ptr, color);
        return ret;
    }
    /**
     * Applies a human move. This is the untrusted-input boundary — same
     * role `CurrentGameState::from_bitboards` plays for the core — so it
     * looks the request up against `generate_moves()`'s own legality
     * rather than re-deriving them, and rejects anything that doesn't
     * match instead of trusting the caller.
     * @param {number | null | undefined} from
     * @param {number} to
     * @param {Uint8Array} captures
     */
    play_move(from, to, captures) {
        const ptr0 = passArray8ToWasm0(captures, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.gamesession_play_move(this.__wbg_ptr, isLikeNone(from) ? 0xFFFFFF : from, to, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {number}
     */
    side_to_move() {
        const ret = wasm.gamesession_side_to_move(this.__wbg_ptr);
        return ret;
    }
    /**
     * Winning color, if the game has a decisive result (`None` for an
     * ongoing game or a draw).
     * @returns {number | undefined}
     */
    winner() {
        const ret = wasm.gamesession_winner(this.__wbg_ptr);
        return ret === 0xFFFFFF ? undefined : ret;
    }
}
if (Symbol.dispose) GameSession.prototype[Symbol.dispose] = GameSession.prototype.free;

export class LegalMove {
    static __wrap(ptr) {
        const obj = Object.create(LegalMove.prototype);
        obj.__wbg_ptr = ptr;
        LegalMoveFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LegalMoveFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_legalmove_free(ptr, 0);
    }
    /**
     * @returns {number | undefined}
     */
    get from() {
        const ret = wasm.legalmove_from(this.__wbg_ptr);
        return ret === 0xFFFFFF ? undefined : ret;
    }
    /**
     * @returns {number}
     */
    get to() {
        const ret = wasm.legalmove_to(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) LegalMove.prototype[Symbol.dispose] = LegalMove.prototype.free;

export class MoveResult {
    static __wrap(ptr) {
        const obj = Object.create(MoveResult.prototype);
        obj.__wbg_ptr = ptr;
        MoveResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MoveResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_moveresult_free(ptr, 0);
    }
    /**
     * @returns {Uint8Array}
     */
    get captures() {
        const ret = wasm.moveresult_captures(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number | undefined}
     */
    get from() {
        const ret = wasm.moveresult_from(this.__wbg_ptr);
        return ret === 0xFFFFFF ? undefined : ret;
    }
    /**
     * @returns {number}
     */
    get score() {
        const ret = wasm.moveresult_score(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get to() {
        const ret = wasm.moveresult_to(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) MoveResult.prototype[Symbol.dispose] = MoveResult.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_bb96b2010945f0bc: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_legalmove_new: function(arg0) {
            const ret = LegalMove.__wrap(arg0);
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./ninemensmorris_wasm_bg.js": import0,
    };
}

const GameSessionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gamesession_free(ptr, 1));
const LegalMoveFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_legalmove_free(ptr, 1));
const MoveResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_moveresult_free(ptr, 1));

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('ninemensmorris_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
