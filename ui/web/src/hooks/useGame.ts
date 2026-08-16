import { useEffect, useMemo, useRef, useState } from 'react';
import { newGameSession, freshSeed, type Color, type Difficulty, type GameSession } from '../lib/wasmEngine';
import { recordResult } from '../lib/stats';

export interface PendingCapture {
  readonly from: number | undefined;
  readonly to: number;
  readonly targets: readonly number[];
  readonly chosen: readonly number[];
}

export interface SuggestedMove {
  readonly from: number | undefined;
  readonly to: number;
}

export interface GameFacts {
  readonly board: Uint8Array;
  readonly side: Color;
  readonly phase: 0 | 1 | 2;
  readonly handWhite: number;
  readonly handBlack: number;
  readonly moves: { from: number | undefined; to: number }[];
  readonly gameOver: boolean;
  readonly winner: Color | undefined;
}

function pieceCodeFor(color: Color): number {
  return color === 0 ? 1 : 2;
}

/** A piece with a stable id independent of its square, so Framer Motion
 * can animate it sliding from one point to another instead of popping. */
export interface PieceState {
  readonly id: number;
  readonly square: number;
  readonly color: Color;
}

// How long a captured piece flashes before it's actually removed from the
// board — long enough that the mover's own slide/placement clearly finishes
// first, so a capture always reads as two beats ("piece lands", then "that
// one goes") instead of everything changing in one instant.
const CAPTURE_FLASH_MS = 480;

export function useGame(playerColor: Color, difficulty: Difficulty) {
  const sessionRef = useRef<GameSession | null>(null);
  const outcomeRecorded = useRef(false);
  const nextPieceId = useRef(0);
  const [ready, setReady] = useState(false);
  const [tick, setTick] = useState(0);
  const [pieces, setPieces] = useState<PieceState[]>([]);
  const [capturedFlash, setCapturedFlash] = useState<readonly number[]>([]);
  const [selectedFrom, setSelectedFrom] = useState<number | null>(null);
  const [pendingCapture, setPendingCapture] = useState<PendingCapture | null>(null);
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const [suggestedMove, setSuggestedMove] = useState<SuggestedMove | null>(null);
  const [hintLoading, setHintLoading] = useState(false);
  const bump = () => {
    setSuggestedMove(null);
    setTick((t) => t + 1);
  };

  async function applyMove(from: number | undefined, to: number, color: Color, captures: readonly number[]) {
    setPieces((prev) =>
      from === undefined
        ? [...prev, { id: nextPieceId.current++, square: to, color }]
        : prev.map((p) => (p.square === from ? { ...p, square: to } : p))
    );
    if (captures.length === 0) return;
    setCapturedFlash(captures);
    await new Promise((r) => setTimeout(r, CAPTURE_FLASH_MS));
    setPieces((prev) => prev.filter((p) => !captures.includes(p.square)));
    setCapturedFlash((prev) => (prev === captures ? [] : prev));
  }

  useEffect(() => {
    let cancelled = false;
    newGameSession().then((session) => {
      if (cancelled) return;
      sessionRef.current = session;
      setReady(true);
      bump();
    });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const facts: GameFacts | null = useMemo(() => {
    const s = sessionRef.current;
    if (!s || !ready) return null;
    const gameOver = s.is_game_over();
    return {
      board: s.board(),
      side: s.side_to_move(),
      phase: s.phase(),
      handWhite: s.pieces_in_hand(0),
      handBlack: s.pieces_in_hand(1),
      moves: s.legal_moves().map((m) => ({ from: m.from, to: m.to })),
      gameOver,
      winner: gameOver ? s.winner() : undefined,
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [ready, tick]);

  // Bot's turn: fire automatically whenever it isn't the human's turn.
  useEffect(() => {
    if (!ready || !facts || facts.gameOver) return;
    if (facts.side === playerColor) return;
    let cancelled = false;
    const moverColor = facts.side;
    setBusy(true);
    setMessage('');
    (async () => {
      await new Promise((r) => setTimeout(r, 350));
      if (cancelled) return;
      try {
        const result = sessionRef.current!.bot_move(difficulty, freshSeed());
        await applyMove(result.from, result.to, moverColor, Array.from(result.captures));
        if (!cancelled) bump();
      } catch (e) {
        if (!cancelled) setMessage(`Bot move failed: ${String(e)}`);
      } finally {
        if (!cancelled) setBusy(false);
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [ready, facts?.side, facts?.gameOver, playerColor, difficulty]);

  // Record the outcome exactly once, from the human's point of view.
  useEffect(() => {
    if (!facts?.gameOver || outcomeRecorded.current) return;
    outcomeRecorded.current = true;
    if (facts.winner === undefined) recordResult('draw');
    else recordResult(facts.winner === playerColor ? 'win' : 'loss');
  }, [facts?.gameOver, facts?.winner, playerColor]);

  function tryPlay(from: number | undefined, to: number, captures: number[]): boolean {
    try {
      sessionRef.current!.play_move(from, to, new Uint8Array(captures));
      void applyMove(from, to, playerColor, captures);
      return true;
    } catch {
      return false;
    }
  }

  function attemptMove(from: number | undefined, to: number) {
    const s = sessionRef.current!;
    const targets = Array.from(s.capture_targets(from, to));
    if (targets.length === 0) {
      const played = tryPlay(from, to, []);
      if (played) {
        setSelectedFrom(null);
        setMessage('');
        bump();
      } else {
        setMessage("That's not a legal move.");
      }
      return;
    }
    setPendingCapture({ from, to, targets, chosen: [] });
    setSelectedFrom(null);
    setMessage('Mill! Tap a red piece to capture it.');
  }

  function onPointClick(sq: number) {
    if (!ready || busy || !facts || facts.gameOver) return;
    if (facts.side !== playerColor) return;

    if (pendingCapture) {
      if (!pendingCapture.targets.includes(sq) || pendingCapture.chosen.includes(sq)) return;
      const chosen = [...pendingCapture.chosen, sq];
      const played = tryPlay(pendingCapture.from, pendingCapture.to, chosen);
      if (played) {
        setPendingCapture(null);
        setSelectedFrom(null);
        setMessage('');
        bump();
      } else {
        setPendingCapture({ ...pendingCapture, chosen });
      }
      return;
    }

    const myPiece = pieceCodeFor(playerColor);

    if (selectedFrom === null) {
      if (facts.phase === 0) {
        if (facts.board[sq] !== 0) {
          setMessage('That point is occupied.');
          return;
        }
        attemptMove(undefined, sq);
      } else if (facts.board[sq] === myPiece && facts.moves.some((m) => m.from === sq)) {
        setSelectedFrom(sq);
        setMessage('Pick a highlighted destination.');
      }
      return;
    }

    if (sq === selectedFrom) {
      setSelectedFrom(null);
      setMessage('');
      return;
    }

    // A different one of the player's own movable pieces — switch the
    // selection instead of falling through to an illegal-move attempt.
    if (facts.board[sq] === myPiece && facts.moves.some((m) => m.from === sq)) {
      setSelectedFrom(sq);
      setMessage('Pick a highlighted destination.');
      return;
    }

    attemptMove(selectedFrom, sq);
  }

  async function requestHint() {
    if (!ready || !facts || facts.gameOver || busy || hintLoading) return;
    if (facts.side !== playerColor) return;
    setHintLoading(true);
    await new Promise((r) => setTimeout(r, 0)); // let the loading state paint before the (possibly slow) search
    try {
      const result = sessionRef.current!.hint(difficulty);
      setSuggestedMove({ from: result.from, to: result.to });
    } catch (e) {
      setMessage(`Hint failed: ${String(e)}`);
    } finally {
      setHintLoading(false);
    }
  }

  return {
    ready,
    facts,
    pieces,
    capturedFlash,
    selectedFrom,
    pendingCapture,
    message,
    busy,
    onPointClick,
    suggestedMove,
    hintLoading,
    requestHint,
  };
}
