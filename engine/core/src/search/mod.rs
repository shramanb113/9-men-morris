mod eval;
mod negamax;
mod ordering;
mod tt;

use crate::position::CurrentGameState;
use crate::types::Move;
use crate::zobrist::splitmix64;
use negamax::{WIN_SCORE, negamax};
use ordering::order_moves;
use tt::TranspositionTable;

/// Table size is a tuning knob, not a correctness concern — a collision
/// just costs a cache miss, it never produces a wrong answer. 65,536 slots
/// is a reasonable default for the depths this engine searches today.
const DEFAULT_TT_CAPACITY: usize = 1 << 16;

/// Recommended `depth` values for a three-tier difficulty selector, measured
/// on the opening position (native release build) — the worst case for
/// search time, since every other reachable position has fewer legal moves
/// per side and resolves faster:
///
/// | depth        | first-move time (opening, worst case) |
/// |--------------|-----------------------------------------|
/// | `EASY_DEPTH`   (4) | ~40ms   |
/// | `MEDIUM_DEPTH` (6) | ~800ms  |
/// | `HARD_DEPTH`   (8) | ~6.4s   |
///
/// These are just depth numbers, not a policy for how to use them — that's
/// a caller/adapter decision (see the crate-level design principle that the
/// core stays free of platform/UI policy). In particular, an "easy" feel
/// should come from pairing `EASY_DEPTH` with `search_with_randomization`
/// and `EASY_RANDOMIZATION_MARGIN`, not from shallow search alone —
/// quiescence means even a depth-4 search won't blunder a hanging capture,
/// so weakness has to come from picking a worse-but-still-safe move on
/// purpose. `Medium`/`Hard` should just call `search` directly.
pub const EASY_DEPTH: u8 = 4;
pub const MEDIUM_DEPTH: u8 = 6;
pub const HARD_DEPTH: u8 = 8;

/// Recommended `margin` for `search_with_randomization` at `EASY_DEPTH`:
/// roughly one piece's value (see `PIECE_VALUE` in `eval`), so easy mode
/// picks randomly among moves that give up at most about a piece's worth
/// of score relative to the best move, never something wildly weaker.
pub const EASY_RANDOMIZATION_MARGIN: i32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub best_move: Move,
    pub score: i32,
}

/// Every root move together with its fully-searched negamax score, from the
/// root side's point of view. Shared by `search` (picks the max) and
/// `search_with_randomization` (picks among near-max moves).
fn root_search(state: &CurrentGameState, depth: u8, tt: &mut TranspositionTable) -> Vec<(Move, i32)> {
    let mut moves = state.generate_moves();
    let tt_move = tt.probe(state.zobrist()).and_then(|entry| entry.best_move);
    order_moves(&mut moves, tt_move);

    let alpha = -WIN_SCORE - 1;
    let beta = WIN_SCORE + 1;
    let mut scored = Vec::with_capacity(moves.len());

    for mv in moves {
        let next = state.make_move(mv);
        // Deliberately the same full (alpha, beta) window for every
        // sibling, not narrowed by the best score found so far. Narrowing
        // here would let a later move's search get cut off (e.g. by
        // quiescence's stand-pat check) and return the window's bound
        // itself rather than a real value — fine for interior nodes, which
        // only ever compare scores with strict `>` and so never mistake a
        // cutoff bound for an exact tie, but wrong here: `search` and
        // `search_with_randomization` both compare every root move's score
        // directly against its siblings, so each one has to be exact.
        let score = -negamax(&next, depth - 1, 1, -beta, -alpha, tt);
        scored.push((mv, score));
    }

    scored
}

/// Finds the strongest move for the side to move, via iterative deepening
/// up to `depth` plies. `state` must not already be a finished game — there
/// is no move to find for one.
pub fn search(state: &CurrentGameState, depth: u8) -> SearchResult {
    debug_assert!(!state.is_game_over(), "search called on a finished game");
    debug_assert!(depth >= 1, "search depth must be at least 1");

    let mut tt = TranspositionTable::new(DEFAULT_TT_CAPACITY);
    let mut scored = Vec::new();
    for d in 1..=depth {
        scored = root_search(state, d, &mut tt);
    }

    let (best_move, score) = *scored
        .iter()
        .max_by_key(|(_, score)| *score)
        .expect("a non-game-over state always has at least one legal move");
    SearchResult { best_move, score }
}

/// Like `search`, but among root moves within `margin` of the best score,
/// picks one deterministically from `seed` instead of always the single
/// strongest — a weaker, still-reproducible "easy mode". `margin` is in the
/// same units as `eval::evaluate` (see `PIECE_VALUE` there). The core never
/// generates its own randomness; `seed` is entirely the caller's choice.
pub fn search_with_randomization(
    state: &CurrentGameState,
    depth: u8,
    margin: i32,
    seed: u64,
) -> SearchResult {
    debug_assert!(!state.is_game_over(), "search called on a finished game");
    debug_assert!(depth >= 1, "search depth must be at least 1");

    let mut tt = TranspositionTable::new(DEFAULT_TT_CAPACITY);
    let mut scored = Vec::new();
    for d in 1..=depth {
        scored = root_search(state, d, &mut tt);
    }

    let best_score = scored
        .iter()
        .map(|(_, score)| *score)
        .max()
        .expect("a non-game-over state always has at least one legal move");
    let candidates: Vec<_> =
        scored.into_iter().filter(|&(_, score)| best_score - score <= margin).collect();

    let (_, pick) = splitmix64(seed);
    let idx = (pick as usize) % candidates.len();
    let (best_move, score) = candidates[idx];
    SearchResult { best_move, score }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::set;
    use crate::types::{Color, Square};

    #[test]
    fn finds_an_immediate_winning_capture() {
        // White slides 14 -> 2, completing mill (0, 1, 2) and capturing one
        // of Black's 3 pieces (none in a mill, so all are capturable),
        // dropping Black to 2 on board with 0 in hand: an immediate loss.
        let white = set(set(set(0, Square(0)), Square(1)), Square(14));
        let white = set(white, Square(22)); // 4th piece: keeps White in Sliding, not Flying
        let black = set(set(set(0, Square(6)), Square(12)), Square(18));
        let state =
            CurrentGameState::from_bitboards(white, black, 0, 0, Color::White, 0, 100).unwrap();

        let result = search(&state, 1);
        assert_eq!(result.score, WIN_SCORE - 1);
        match result.best_move {
            Move::Slide { from: Square(14), to: Square(2), captures } if !captures.is_empty() => {}
            other => panic!("expected a winning slide from 14 to 2, got {other:?}"),
        }
    }

    #[test]
    fn self_play_runs_without_panicking() {
        let mut state = CurrentGameState::new();
        for _ in 0..12 {
            if state.is_game_over() {
                break;
            }
            let result = search(&state, 2);
            state = state.make_move(result.best_move);
            assert!(state.invariants_hold());
        }
    }

    #[test]
    fn search_with_randomization_is_deterministic_for_a_given_seed() {
        let state = CurrentGameState::new();
        let a = search_with_randomization(&state, 1, 0, 42);
        let b = search_with_randomization(&state, 1, 0, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn search_with_randomization_stays_within_margin_of_best() {
        let state = CurrentGameState::new();
        let best = search(&state, 1);
        let picked = search_with_randomization(&state, 1, 10, 7);
        assert!(best.score - picked.score <= 10);
    }
}
