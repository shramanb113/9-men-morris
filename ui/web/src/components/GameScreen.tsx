import { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { useGame } from '../hooks/useGame';
import { Board } from './Board';
import { ShareModal } from './ShareModal';
import {
  HINT_CREDITS_PER_SHARE,
  SHARE_TEXT,
  consumeHintCredit,
  grantHintCredits,
  hintCredits,
  winShareText,
  type SharePlatform,
} from '../lib/share';
import type { Color, Difficulty } from '../lib/wasmEngine';

interface GameScreenProps {
  playerColor: Color;
  difficulty: Difficulty;
  onExit: () => void;
  onShowRules: () => void;
}

const PHASE_LABEL = ['placing', 'sliding', 'flying'] as const;
const DIFFICULTY_LABEL: Record<Difficulty, string> = { easy: 'Easy', medium: 'Medium', hard: 'Hard' };

// Long enough that the capture-flash sequence (see useGame's
// CAPTURE_FLASH_MS) has fully played out before the outcome modal covers
// the board, so a losing player actually gets to see how it happened.
const OUTCOME_MODAL_DELAY_MS = 1600;

export function GameScreen({ playerColor, difficulty, onExit, onShowRules }: GameScreenProps) {
  const {
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
  } = useGame(playerColor, difficulty);

  const [shareContext, setShareContext] = useState<'hints' | 'win' | null>(null);
  const [showOutcome, setShowOutcome] = useState(false);
  const [credits, setCredits] = useState(() => hintCredits());

  const gameOver = facts?.gameOver ?? false;
  const outcome = gameOver
    ? facts!.winner === undefined
      ? 'draw'
      : facts!.winner === playerColor
        ? 'win'
        : 'loss'
    : null;

  useEffect(() => {
    if (!gameOver) {
      setShowOutcome(false);
      return;
    }
    const t = setTimeout(() => setShowOutcome(true), OUTCOME_MODAL_DELAY_MS);
    return () => clearTimeout(t);
  }, [gameOver]);

  const canHint = ready && !busy && !gameOver && !hintLoading && pendingCapture === null && facts?.side === playerColor;

  function handleHintClick() {
    if (!canHint) return;
    if (credits > 0 && consumeHintCredit()) {
      setCredits((c) => c - 1);
      void requestHint();
    } else {
      setShareContext('hints');
    }
  }

  function handleSharePlatform(_platform: SharePlatform) {
    if (shareContext !== 'hints') return; // win-share carries no reward — stays open for more platforms
    grantHintCredits(HINT_CREDITS_PER_SHARE);
    setShareContext(null);
    if (consumeHintCredit()) {
      setCredits(hintCredits());
      void requestHint();
    } else {
      setCredits(hintCredits());
    }
  }

  const statusText = (() => {
    if (!ready || !facts) return 'Loading engine…';
    if (gameOver) return '';
    if (pendingCapture) return message;
    if (busy) return 'Bot is thinking…';
    if (suggestedMove) return 'Hint shown below — try the outlined point.';
    const whoseTurn = facts.side === playerColor ? 'Your move' : "Bot's move";
    return `${whoseTurn} — ${PHASE_LABEL[facts.phase]}`;
  })();

  return (
    <div className="w-full max-w-md mx-auto flex flex-col gap-4 py-6 px-4">
      <div className="flex items-center justify-between gap-2">
        <button onClick={onExit} className="text-sm font-medium" style={{ color: 'var(--ink-soft)' }}>
          ← Menu
        </button>
        <div className="flex items-center gap-2">
          <span
            className="text-xs font-semibold uppercase tracking-wider px-2.5 py-1 rounded-full"
            style={{ background: 'var(--panel-raised)', color: 'var(--ink-soft)' }}
          >
            {DIFFICULTY_LABEL[difficulty]}
          </span>
          <button
            onClick={handleHintClick}
            disabled={!canHint}
            className="text-xs font-semibold uppercase tracking-wider px-2.5 py-1 rounded-full disabled:opacity-40"
            style={{ background: 'var(--panel-raised)', color: 'var(--hint)' }}
          >
            {hintLoading ? 'Thinking…' : credits > 0 ? `Hint (${credits})` : 'Hint'}
          </button>
          <button
            onClick={onShowRules}
            aria-label="How to play"
            className="w-7 h-7 rounded-full grid place-items-center text-sm font-semibold"
            style={{ border: '1px solid var(--line)', color: 'var(--ink-soft)' }}
          >
            ?
          </button>
        </div>
      </div>

      <div className="rounded-2xl p-4" style={{ background: 'var(--panel)', border: '1px solid var(--line)' }}>
        <Board
          pieces={pieces}
          capturedFlash={capturedFlash}
          moves={facts?.moves ?? []}
          phase={facts?.phase ?? 0}
          side={facts?.side ?? 0}
          playerColor={playerColor}
          selectedFrom={selectedFrom}
          captureTargets={pendingCapture?.targets ?? []}
          captureChosen={pendingCapture?.chosen ?? []}
          isCapturing={pendingCapture !== null}
          interactive={ready && !busy && !gameOver}
          suggestedMove={suggestedMove}
          onPointClick={onPointClick}
        />
      </div>

      <div className="flex items-center justify-between text-sm" style={{ fontFamily: 'var(--font-mono)' }}>
        <HandPip color={0} count={facts?.handWhite ?? 9} label="White" />
        <span className="min-h-[1.2em] text-center" style={{ color: 'var(--ink-soft)' }}>
          {statusText}
        </span>
        <HandPip color={1} count={facts?.handBlack ?? 9} label="Black" />
      </div>

      <AnimatePresence>
        {shareContext && (
          <ShareModal
            headline={shareContext === 'hints' ? 'Out of hints' : 'Nice win!'}
            subline={
              shareContext === 'hints'
                ? `Share Morris and get ${HINT_CREDITS_PER_SHARE} more hints on this device.`
                : 'Let people know you beat the bot.'
            }
            text={shareContext === 'hints' ? SHARE_TEXT : winShareText(difficulty)}
            onShare={handleSharePlatform}
            onClose={() => setShareContext(null)}
          />
        )}
      </AnimatePresence>

      <AnimatePresence>
        {showOutcome && outcome && (
          <motion.div
            className="fixed inset-0 z-40 flex items-center justify-center p-6"
            style={{ background: 'color-mix(in srgb, var(--bg) 82%, transparent)', backdropFilter: 'blur(6px)' }}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <motion.div
              className="w-full max-w-sm rounded-2xl p-7 text-center"
              style={{ background: 'var(--panel)', border: '1px solid var(--line)' }}
              initial={{ opacity: 0, y: 16, scale: 0.96 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 12, scale: 0.96 }}
              transition={{ type: 'spring', stiffness: 320, damping: 28 }}
            >
              <div
                className="text-3xl mb-2"
                style={{
                  fontFamily: 'var(--font-display)',
                  fontWeight: 700,
                  color: outcome === 'win' ? 'var(--good)' : outcome === 'loss' ? 'var(--bad)' : 'var(--ink)',
                }}
              >
                {outcome === 'win' ? 'You win' : outcome === 'loss' ? 'Bot wins' : 'Draw'}
              </div>
              <p className="mb-6" style={{ color: 'var(--ink-soft)' }}>
                {outcome === 'win'
                  ? 'The bot ran out of room to move.'
                  : outcome === 'loss'
                    ? 'No captures left within the move limit.'
                    : 'No decisive result within the move limit.'}
              </p>
              <div className="flex flex-col gap-2">
                {outcome === 'win' && (
                  <button
                    onClick={() => setShareContext('win')}
                    className="w-full py-3 rounded-full font-semibold"
                    style={{ background: 'var(--hint)', color: 'var(--brass-ink)' }}
                  >
                    Share your win
                  </button>
                )}
                <button
                  onClick={onExit}
                  className="w-full py-3 rounded-full font-semibold"
                  style={{ background: 'var(--brass)', color: 'var(--brass-ink)' }}
                >
                  Play again
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function HandPip({ color, count, label }: { color: Color; count: number; label: string }) {
  return (
    <span className="flex items-center gap-1.5" style={{ color: 'var(--ink-soft)' }} aria-label={`${label} in hand`}>
      <span
        className="w-3 h-3 rounded-full inline-block"
        style={{
          background: color === 0 ? 'var(--white-piece)' : 'var(--black-piece)',
          border: `1.5px solid ${color === 0 ? 'var(--white-piece-edge)' : 'var(--black-piece-edge)'}`,
        }}
      />
      {count}
    </span>
  );
}
