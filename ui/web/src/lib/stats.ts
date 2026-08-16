// Per-browser play stats. There's no backend, so this is a per-device
// count (localStorage), not a real per-IP count — good enough for a
// "welcome back" nudge, not meant to be a hardened analytics signal.

const KEY = 'morris-bench:stats:v1';

export interface Stats {
  visits: number;
  wins: number;
  losses: number;
  draws: number;
}

const EMPTY: Stats = { visits: 0, wins: 0, losses: 0, draws: 0 };

function read(): Stats {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...EMPTY };
    return { ...EMPTY, ...JSON.parse(raw) };
  } catch {
    return { ...EMPTY };
  }
}

function write(stats: Stats): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(stats));
  } catch {
    // Storage unavailable (private mode, quota) — stats just won't persist.
  }
}

/** Call once per page load. Returns the updated stats. */
export function recordVisit(): Stats {
  const stats = read();
  stats.visits += 1;
  write(stats);
  return stats;
}

export function recordResult(outcome: 'win' | 'loss' | 'draw'): Stats {
  const stats = read();
  if (outcome === 'win') stats.wins += 1;
  else if (outcome === 'loss') stats.losses += 1;
  else stats.draws += 1;
  write(stats);
  return stats;
}

export function getStats(): Stats {
  return read();
}
