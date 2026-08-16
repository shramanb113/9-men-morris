// Matches engine/core/src/board.rs exactly — 24 points on a 7x7 grid,
// connected along the three nested squares plus their four spokes.
export const POINT_GRID: readonly [number, number][] = [
  [0, 0], [0, 3], [0, 6],
  [1, 1], [1, 3], [1, 5],
  [2, 2], [2, 3], [2, 4],
  [3, 0], [3, 1], [3, 2], [3, 4], [3, 5], [3, 6],
  [4, 2], [4, 3], [4, 4],
  [5, 1], [5, 3], [5, 5],
  [6, 0], [6, 3], [6, 6],
];

export const EDGES: readonly [number, number][] = [
  [0, 1], [0, 9], [1, 2], [1, 4], [2, 14], [3, 4], [3, 10], [4, 5], [4, 7],
  [5, 13], [6, 7], [6, 11], [7, 8], [8, 12], [9, 10], [9, 21], [10, 11],
  [10, 18], [11, 15], [12, 13], [12, 17], [13, 14], [13, 20], [14, 23],
  [15, 16], [16, 17], [16, 19], [18, 19], [19, 20], [19, 22], [21, 22], [22, 23],
];

const CELL = 100;
const MARGIN = 60;
export const POINT_RADIUS = 20;
export const PIECE_RADIUS = 16;

export function pointCoord(sq: number): [number, number] {
  const [row, col] = POINT_GRID[sq];
  return [MARGIN + col * CELL, MARGIN + row * CELL];
}

export const BOARD_VIEWBOX_SIZE = MARGIN * 2 + CELL * 6;
