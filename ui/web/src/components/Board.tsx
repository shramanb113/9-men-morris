import { useMemo, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { BOARD_VIEWBOX_SIZE, EDGES, PIECE_RADIUS, POINT_RADIUS, pointCoord } from '../lib/boardTopology';
import type { PieceState } from '../hooks/useGame';
import type { Color, Phase } from '../lib/wasmEngine';

interface BoardProps {
  pieces: PieceState[];
  capturedFlash: readonly number[];
  moves: { from: number | undefined; to: number }[];
  phase: Phase;
  side: Color;
  playerColor: Color;
  selectedFrom: number | null;
  captureTargets: readonly number[];
  captureChosen: readonly number[];
  isCapturing: boolean;
  interactive: boolean;
  suggestedMove: { from: number | undefined; to: number } | null;
  onPointClick: (sq: number) => void;
}

export function Board({
  pieces,
  capturedFlash,
  moves,
  phase,
  side,
  playerColor,
  selectedFrom,
  captureTargets,
  captureChosen,
  isCapturing,
  interactive,
  suggestedMove,
  onPointClick,
}: BoardProps) {
  const destinations = useMemo(() => {
    if (isCapturing) return new Set<number>();
    return new Set(
      moves.filter((m) => (selectedFrom === null ? m.from === undefined : m.from === selectedFrom)).map((m) => m.to)
    );
  }, [moves, selectedFrom, isCapturing]);

  const movablePieces = useMemo(
    () => new Set(moves.filter((m) => m.from !== undefined).map((m) => m.from as number)),
    [moves]
  );

  const yourTurn = interactive && side === playerColor;

  const [hoverFrom, setHoverFrom] = useState<number | null>(null);
  const hoverDestinations = useMemo(() => {
    if (hoverFrom === null || selectedFrom !== null || isCapturing) return new Set<number>();
    return new Set(moves.filter((m) => m.from === hoverFrom).map((m) => m.to));
  }, [hoverFrom, selectedFrom, isCapturing, moves]);

  return (
    <svg
      viewBox={`0 0 ${BOARD_VIEWBOX_SIZE} ${BOARD_VIEWBOX_SIZE}`}
      className="block w-full h-auto select-none touch-manipulation"
      role="img"
      aria-label="Nine Men's Morris board"
    >
      {EDGES.map(([a, b], i) => {
        const [x1, y1] = pointCoord(a);
        const [x2, y2] = pointCoord(b);
        return <line key={i} x1={x1} y1={y1} x2={x2} y2={y2} stroke="var(--line)" strokeWidth={3} />;
      })}

      {Array.from({ length: 24 }, (_, sq) => {
        const [x, y] = pointCoord(sq);
        const capturable = captureTargets.includes(sq) && !captureChosen.includes(sq);
        const selected = selectedFrom === sq;
        const canPreviewFromHere = yourTurn && !isCapturing && selectedFrom === null && phase !== 0 && movablePieces.has(sq);
        const legal =
          destinations.has(sq) ||
          hoverDestinations.has(sq) ||
          (selectedFrom === null && !isCapturing && yourTurn && phase !== 0 && movablePieces.has(sq));
        const hinted = suggestedMove !== null && (suggestedMove.from === sq || suggestedMove.to === sq);

        return (
          <g
            key={sq}
            data-sq={sq}
            role="button"
            tabIndex={yourTurn ? 0 : -1}
            aria-label={`point ${sq}`}
            className={yourTurn ? 'cursor-pointer' : 'cursor-default'}
            onClick={() => yourTurn && onPointClick(sq)}
            onMouseEnter={() => canPreviewFromHere && setHoverFrom(sq)}
            onMouseLeave={() => setHoverFrom((h) => (h === sq ? null : h))}
            onKeyDown={(e) => {
              if (yourTurn && (e.key === 'Enter' || e.key === ' ')) {
                e.preventDefault();
                onPointClick(sq);
              }
            }}
          >
            <circle cx={x} cy={y} r={POINT_RADIUS + 8} fill="transparent" />
            <circle
              cx={x}
              cy={y}
              r={POINT_RADIUS}
              fill={capturable ? 'var(--bad)' : legal ? 'var(--brass)' : 'var(--panel-raised)'}
              fillOpacity={capturable ? 0.45 : legal ? 0.35 : 1}
              stroke={selected ? 'var(--brass)' : hinted ? 'var(--hint)' : 'var(--line)'}
              strokeWidth={selected || hinted ? 4 : 2}
              strokeDasharray={hinted && !selected ? '4 3' : undefined}
            />
          </g>
        );
      })}

      <AnimatePresence>
        {pieces.map((p) => {
          const [cx, cy] = pointCoord(p.square);
          const edgeColor = p.color === 0 ? 'var(--white-piece-edge)' : 'var(--black-piece-edge)';
          const flashing = capturedFlash.includes(p.square);
          return (
            <motion.circle
              key={p.id}
              r={PIECE_RADIUS}
              style={{
                fill: p.color === 0 ? 'var(--white-piece)' : 'var(--black-piece)',
                pointerEvents: 'none',
              }}
              initial={{ cx, cy, opacity: 0, scale: 0.3, stroke: edgeColor, strokeWidth: 2 }}
              animate={{
                cx,
                cy,
                opacity: 1,
                scale: flashing ? [1, 1.22, 1.05] : 1,
                stroke: flashing ? 'var(--bad)' : edgeColor,
                strokeWidth: flashing ? 4 : 2,
              }}
              exit={{ opacity: 0, scale: 0.15, transition: { duration: 0.32, ease: 'easeIn' } }}
              transition={{
                cx: { type: 'tween', duration: 0.42, ease: [0.32, 0.72, 0.35, 1] },
                cy: { type: 'tween', duration: 0.42, ease: [0.32, 0.72, 0.35, 1] },
                default: { type: 'spring', stiffness: 320, damping: 28 },
              }}
            />
          );
        })}
      </AnimatePresence>
    </svg>
  );
}
