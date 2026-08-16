import { motion } from 'framer-motion';

interface RulesPanelProps {
  onClose: () => void;
}

const STEPS = [
  {
    title: 'Placing',
    body: "You and the bot take turns placing your 9 pieces on any open point. Nobody moves anything yet — just claim ground.",
  },
  {
    title: 'Moving',
    body: 'Once all pieces are down, slide one piece at a time along a line to an empty point right next to it.',
  },
  {
    title: 'Flying',
    body: "Drop to your last 3 pieces and that piece can jump anywhere on the board, not just next door — your one comeback move.",
  },
];

export function RulesPanel({ onClose }: RulesPanelProps) {
  return (
    <motion.div
      className="fixed inset-0 z-50 flex items-start sm:items-center justify-center overflow-y-auto p-4 sm:p-8"
      style={{ background: 'color-mix(in srgb, var(--bg) 78%, transparent)', backdropFilter: 'blur(6px)' }}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
    >
      <motion.div
        className="w-full max-w-lg rounded-2xl p-6 sm:p-8 my-8"
        style={{ background: 'var(--panel)', border: '1px solid var(--line)', color: 'var(--ink)' }}
        initial={{ opacity: 0, y: 16, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: 12, scale: 0.97 }}
        transition={{ type: 'spring', stiffness: 340, damping: 30 }}
      >
        <div className="flex items-start justify-between gap-4 mb-1">
          <h2 className="text-2xl sm:text-3xl" style={{ fontFamily: 'var(--font-display)', fontWeight: 600 }}>
            How to play
          </h2>
          <button
            onClick={onClose}
            aria-label="Close rules"
            className="shrink-0 w-9 h-9 rounded-full grid place-items-center text-lg"
            style={{ border: '1px solid var(--line)', color: 'var(--ink-soft)' }}
          >
            ×
          </button>
        </div>

        <p className="mb-6" style={{ color: 'var(--ink-soft)' }}>
          Three-in-a-row, but the board fights back. Here's the whole game in one minute.
        </p>

        <ol className="flex flex-col gap-4 mb-6">
          {STEPS.map((step, i) => (
            <li key={step.title} className="flex gap-4">
              <span
                className="shrink-0 w-8 h-8 rounded-full grid place-items-center font-semibold"
                style={{ background: 'var(--brass)', color: 'var(--brass-ink)', fontFamily: 'var(--font-mono)' }}
              >
                {i + 1}
              </span>
              <div>
                <div className="font-semibold mb-0.5">{step.title}</div>
                <p style={{ color: 'var(--ink-soft)' }}>{step.body}</p>
              </div>
            </li>
          ))}
        </ol>

        <div className="rounded-xl p-4 mb-6" style={{ background: 'var(--panel-raised)' }}>
          <div className="font-semibold mb-1">Mills capture</div>
          <p style={{ color: 'var(--ink-soft)' }}>
            Line up 3 of your pieces in a straight line — that's a <em>mill</em>. Forming one lets you remove one
            opposing piece from the board immediately. Keep making and re-forming mills to grind your opponent down.
          </p>
        </div>

        <div className="rounded-xl p-4 mb-6" style={{ background: 'var(--panel-raised)' }}>
          <div className="font-semibold mb-1">You win by...</div>
          <p style={{ color: 'var(--ink-soft)' }}>
            Trapping your opponent down to 2 pieces, or leaving them with no legal move on their turn.
          </p>
        </div>

        <button
          onClick={onClose}
          className="w-full py-3 rounded-full font-semibold"
          style={{ background: 'var(--brass)', color: 'var(--brass-ink)' }}
        >
          Got it — let's play
        </button>
      </motion.div>
    </motion.div>
  );
}
