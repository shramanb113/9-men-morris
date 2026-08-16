import { useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';

interface RulesPanelProps {
  onClose: () => void;
}

interface Page {
  readonly kicker: string;
  readonly title: string;
  readonly body: readonly string[];
}

const PAGES: readonly Page[] = [
  {
    kicker: 'Cover',
    title: "Nine Men's Morris",
    body: [
      "Diagrams for this game have turned up scratched into Bronze Age ruins and ancient temple roofs — it's one of the oldest strategy games still played today.",
      'Three-in-a-row, but the board fights back. This book is the whole game, in eight short pages.',
    ],
  },
  {
    kicker: 'Chapter 1',
    title: 'The board',
    body: [
      '24 points, laid out as three nested squares joined by four spokes. Pieces only ever sit on points, and only ever move along the lines connecting them.',
      'Those lines matter more than they look — every mill you form or block happens along one of them.',
    ],
  },
  {
    kicker: 'Chapter 2',
    title: 'Placing',
    body: [
      'You and the bot take turns placing your 9 pieces, one at a time, on any open point.',
      "Nobody moves anything yet — this phase is about claiming ground and quietly setting up mills for the phase after it.",
    ],
  },
  {
    kicker: 'Chapter 3',
    title: 'Moving',
    body: [
      'Once all 18 pieces are down, the game shifts: on your turn, slide one piece along a line to an empty point right next to it.',
      'No jumps, no diagonals off the drawn lines — just one careful step at a time.',
    ],
  },
  {
    kicker: 'Chapter 4',
    title: 'Flying',
    body: [
      "Drop to your last 3 pieces and the rules loosen: that piece can jump anywhere on the board, not just next door.",
      "It's a genuine comeback mechanic — a single well-timed flight can rebuild a mill out of nowhere.",
    ],
  },
  {
    kicker: 'Chapter 5',
    title: 'Mills & capturing',
    body: [
      'Line up 3 of your pieces in a straight line — that’s a mill. Forming one lets you remove one opposing piece from the board immediately.',
      'Break a mill and re-form it later and it captures again — a "swinging mill" is one of the strongest patterns in the whole game.',
    ],
  },
  {
    kicker: 'Chapter 6',
    title: 'Winning',
    body: [
      'Trap your opponent down to 2 pieces — with only 2 left, they can never form a mill again, so it’s an automatic loss.',
      'You can also win by leaving them with no legal move at all on their turn. Long games that resolve neither way end in a draw.',
    ],
  },
  {
    kicker: 'Chapter 7',
    title: 'Difficulty & hints',
    body: [
      'Easy, Medium, and Hard change how deep the bot searches ahead — Easy also plays a little imperfectly on purpose, so it never feels unbeatable on turn one.',
      "Stuck? The Hint button asks the same search engine the bot itself uses for its best move — not a separate, weaker helper.",
    ],
  },
];

export function RulesPanel({ onClose }: RulesPanelProps) {
  const [page, setPage] = useState(0);
  const [direction, setDirection] = useState(1);
  const lastPage = page === PAGES.length - 1;
  const current = PAGES[page];

  function go(to: number) {
    if (to < 0 || to >= PAGES.length) return;
    setDirection(to > page ? 1 : -1);
    setPage(to);
  }

  return (
    <motion.div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      style={{ background: 'color-mix(in srgb, var(--bg) 78%, transparent)', backdropFilter: 'blur(6px)' }}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
    >
      <motion.div
        className="w-full max-w-lg rounded-2xl flex flex-col"
        style={{ background: 'var(--panel)', border: '1px solid var(--line)', color: 'var(--ink)', maxHeight: '88vh' }}
        initial={{ opacity: 0, y: 16, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: 12, scale: 0.97 }}
        transition={{ type: 'spring', stiffness: 340, damping: 30 }}
      >
        {/* Pinned header — never scrolls, so the title can't get cut off */}
        <div className="flex items-start justify-between gap-4 p-6 sm:p-8 pb-4 sm:pb-4 shrink-0">
          <div>
            <div
              className="text-xs font-semibold uppercase tracking-wider mb-1"
              style={{ color: 'var(--hint)', fontFamily: 'var(--font-mono)' }}
            >
              {current.kicker}
            </div>
            <h2 className="text-2xl sm:text-3xl" style={{ fontFamily: 'var(--font-display)', fontWeight: 600 }}>
              {current.title}
            </h2>
          </div>
          <button
            onClick={onClose}
            aria-label="Close rules"
            className="shrink-0 w-9 h-9 rounded-full grid place-items-center text-lg"
            style={{ border: '1px solid var(--line)', color: 'var(--ink-soft)' }}
          >
            ×
          </button>
        </div>

        {/* Scrollable page body — bounded, so only this (never the header/footer) can overflow */}
        <div className="flex-1 overflow-y-auto px-6 sm:px-8 min-h-[9rem]">
          <AnimatePresence mode="wait" custom={direction}>
            <motion.div
              key={page}
              custom={direction}
              initial={{ opacity: 0, x: direction * 24 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: direction * -24 }}
              transition={{ duration: 0.22, ease: 'easeOut' }}
              className="flex flex-col gap-3 pb-2"
            >
              {current.body.map((para, i) => (
                <p key={i} style={{ color: 'var(--ink-soft)' }}>
                  {para}
                </p>
              ))}
            </motion.div>
          </AnimatePresence>
        </div>

        {/* Pinned footer — page dots + navigation, always reachable */}
        <div className="p-6 sm:p-8 pt-4 sm:pt-4 shrink-0">
          <div className="flex items-center justify-center gap-1.5 mb-4">
            {PAGES.map((p, i) => (
              <button
                key={p.title}
                onClick={() => go(i)}
                aria-label={`Page ${i + 1}: ${p.title}`}
                className="rounded-full"
                style={{
                  width: i === page ? 18 : 6,
                  height: 6,
                  background: i === page ? 'var(--brass)' : 'var(--line)',
                  transition: 'width 0.2s ease',
                }}
              />
            ))}
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={() => go(page - 1)}
              disabled={page === 0}
              className="py-3 px-4 rounded-full font-semibold disabled:opacity-30"
              style={{ border: '1px solid var(--line)', color: 'var(--ink-soft)' }}
            >
              Back
            </button>
            {lastPage ? (
              <button
                onClick={onClose}
                className="flex-1 py-3 rounded-full font-semibold"
                style={{ background: 'var(--brass)', color: 'var(--brass-ink)' }}
              >
                Got it — let's play
              </button>
            ) : (
              <button
                onClick={() => go(page + 1)}
                className="flex-1 py-3 rounded-full font-semibold"
                style={{ background: 'var(--brass)', color: 'var(--brass-ink)' }}
              >
                Next
              </button>
            )}
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
}
