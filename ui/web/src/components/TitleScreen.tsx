import { useState } from 'react';
import { motion } from 'framer-motion';
import type { Color, Difficulty } from '../lib/wasmEngine';
import type { Stats } from '../lib/stats';

interface TitleScreenProps {
  stats: Stats;
  onPlay: (color: Color, difficulty: Difficulty) => void;
  onShowRules: () => void;
}

const DIFFICULTIES: { value: Difficulty; label: string; note: string }[] = [
  { value: 'easy', label: 'Easy', note: 'still plays fair' },
  { value: 'medium', label: 'Medium', note: 'a real fight' },
  { value: 'hard', label: 'Hard', note: 'thinks ahead, slow' },
];

const fadeUp = {
  initial: { opacity: 0, y: 14 },
  animate: { opacity: 1, y: 0 },
};

export function TitleScreen({ stats, onPlay, onShowRules }: TitleScreenProps) {
  const [color, setColor] = useState<Color>(0);
  const [difficulty, setDifficulty] = useState<Difficulty>('easy');

  const played = stats.wins + stats.losses + stats.draws;

  return (
    <div className="w-full max-w-md mx-auto flex flex-col items-center text-center gap-8 py-10 px-4">
      <motion.div {...fadeUp} transition={{ duration: 0.5 }} className="flex flex-col items-center gap-3">
        <div className="flex items-center gap-3" aria-hidden>
          <MillGlyph />
        </div>
        <h1
          className="text-5xl sm:text-6xl leading-none"
          style={{ fontFamily: 'var(--font-display)', fontWeight: 900, letterSpacing: '-0.01em', color: 'var(--ink)' }}
        >
          Morris
        </h1>
        <p className="text-base sm:text-lg" style={{ color: 'var(--ink-soft)' }}>
          Three in a row — but the board fights back.
        </p>
      </motion.div>

      <motion.div
        {...fadeUp}
        transition={{ duration: 0.5, delay: 0.08 }}
        className="w-full rounded-2xl p-5 flex flex-col gap-5"
        style={{ background: 'var(--panel)', border: '1px solid var(--line)' }}
      >
        <div>
          <div className="text-xs font-semibold uppercase tracking-wider mb-2.5" style={{ color: 'var(--ink-soft)' }}>
            Play as
          </div>
          <div className="flex gap-2">
            {([0, 1] as Color[]).map((c) => (
              <button
                key={c}
                onClick={() => setColor(c)}
                className="flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl font-medium transition-colors"
                style={{
                  background: color === c ? 'var(--brass)' : 'var(--panel-raised)',
                  color: color === c ? 'var(--brass-ink)' : 'var(--ink)',
                }}
              >
                <span
                  className="w-3.5 h-3.5 rounded-full inline-block"
                  style={{
                    background: c === 0 ? 'var(--white-piece)' : 'var(--black-piece)',
                    border: `1.5px solid ${c === 0 ? 'var(--white-piece-edge)' : 'var(--black-piece-edge)'}`,
                  }}
                />
                {c === 0 ? 'White' : 'Black'}
              </button>
            ))}
          </div>
          {color === 1 && (
            <p className="text-xs mt-2 text-left" style={{ color: 'var(--ink-soft)' }}>
              Black moves second — the bot opens.
            </p>
          )}
        </div>

        <div>
          <div className="text-xs font-semibold uppercase tracking-wider mb-2.5" style={{ color: 'var(--ink-soft)' }}>
            Bot strength
          </div>
          <div className="flex gap-2">
            {DIFFICULTIES.map((d) => (
              <button
                key={d.value}
                onClick={() => setDifficulty(d.value)}
                className="flex-1 py-2.5 rounded-xl font-medium"
                style={{
                  background: difficulty === d.value ? 'var(--brass)' : 'var(--panel-raised)',
                  color: difficulty === d.value ? 'var(--brass-ink)' : 'var(--ink)',
                }}
              >
                {d.label}
              </button>
            ))}
          </div>
          <p className="text-xs mt-2 text-left" style={{ color: 'var(--ink-soft)' }}>
            {DIFFICULTIES.find((d) => d.value === difficulty)?.note}
          </p>
        </div>
      </motion.div>

      <motion.div {...fadeUp} transition={{ duration: 0.5, delay: 0.16 }} className="w-full flex flex-col gap-3">
        <button
          onClick={() => onPlay(color, difficulty)}
          className="w-full py-4 rounded-full text-lg font-semibold shadow-sm"
          style={{ background: 'var(--brass)', color: 'var(--brass-ink)' }}
        >
          Play
        </button>
        <button onClick={onShowRules} className="text-sm underline underline-offset-4" style={{ color: 'var(--ink-soft)' }}>
          New to Nine Men's Morris? See how it works
        </button>
      </motion.div>

      {played > 0 && (
        <motion.div
          {...fadeUp}
          transition={{ duration: 0.5, delay: 0.22 }}
          className="flex gap-4 text-sm"
          style={{ fontFamily: 'var(--font-mono)', color: 'var(--ink-soft)' }}
        >
          <span>{stats.visits} visits</span>
          <span>
            {stats.wins}W {stats.losses}L {stats.draws}D
          </span>
        </motion.div>
      )}
    </div>
  );
}

function MillGlyph() {
  return (
    <svg width="72" height="24" viewBox="0 0 72 24" aria-hidden>
      <line x1="4" y1="12" x2="68" y2="12" stroke="var(--line)" strokeWidth="2" />
      {[4, 36, 68].map((x, i) => (
        <circle
          key={x}
          cx={x}
          cy="12"
          r="7"
          fill="var(--brass)"
          opacity={0.55 + i * 0.22}
        />
      ))}
    </svg>
  );
}
