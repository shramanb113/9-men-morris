use super::eval::evaluate;
use super::ordering::{is_capture, order_moves};
use super::tt::{Bound, TTEntry, TranspositionTable};
use crate::position::CurrentGameState;

/// Score of a position that is a forced win for the side to move "right
/// now" (ply 0). Actual win/loss scores are offset by ply so the engine
/// prefers a faster win and a slower loss over an equally-certain one
/// further away (mate-distance scoring).
pub const WIN_SCORE: i32 = 100_000;

/// Negamax search with alpha-beta pruning and a transposition table.
/// Returns a score from the perspective of `state.side_to_move()`.
///
/// Every node checked into `make_move` by this crate's own move generation
/// always has the side to move be the loser at a terminal node (a side only
/// discovers it has lost — by piece count or by having no legal moves — on
/// what has become its own turn), so a terminal leaf here is always scored
/// as a loss for the side to move, never a win.
pub fn negamax(
    state: &CurrentGameState,
    depth: u8,
    ply: u8,
    mut alpha: i32,
    mut beta: i32,
    tt: &mut TranspositionTable,
) -> i32 {
    if state.is_terminal() {
        return -(WIN_SCORE - ply as i32);
    }
    if state.is_draw_by_plies() {
        return 0;
    }
    if depth == 0 {
        return quiescence(state, ply, alpha, beta);
    }

    let key = state.zobrist();
    let original_alpha = alpha;
    let mut tt_move = None;

    if let Some(entry) = tt.probe(key) {
        tt_move = entry.best_move;
        if entry.depth >= depth {
            match entry.bound {
                Bound::Exact => return entry.score,
                Bound::Lower => alpha = alpha.max(entry.score),
                Bound::Upper => beta = beta.min(entry.score),
            }
            if alpha >= beta {
                return entry.score;
            }
        }
    }

    let mut moves = state.generate_moves();
    order_moves(&mut moves, tt_move);

    let mut best_score = -WIN_SCORE - 1;
    let mut best_move = moves[0];

    for mv in moves {
        let next = state.make_move(mv);
        let score = -negamax(&next, depth - 1, ply + 1, -beta, -alpha, tt);
        if score > best_score {
            best_score = score;
            best_move = mv;
        }
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    let bound = if best_score <= original_alpha {
        Bound::Upper
    } else if best_score >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };
    tt.store(key, TTEntry { key, depth, score: best_score, bound, best_move: Some(best_move) });

    best_score
}

/// Extends search past the nominal depth limit through capturing moves
/// only, so `negamax` never evaluates a position mid-capture-exchange —
/// the classic "horizon effect", and an especially sharp one here since
/// captures are the entire tactical layer of this game. Bounded for free:
/// each recursive step removes a piece, and the game is already over well
/// before either side could run out, so this always terminates quickly
/// without an explicit depth cap.
///
/// No transposition table here — quiescence nodes don't have a stable
/// "depth" to key entries on, and the position count reachable this way is
/// small enough that it isn't worth the complexity.
fn quiescence(state: &CurrentGameState, ply: u8, mut alpha: i32, beta: i32) -> i32 {
    if state.is_terminal() {
        return -(WIN_SCORE - ply as i32);
    }
    if state.is_draw_by_plies() {
        return 0;
    }

    // Stand pat: the side to move isn't forced to capture, so "decline and
    // take the static score" is always a legal option and a valid lower
    // bound on what this node is worth.
    let stand_pat = evaluate(state, state.side_to_move());
    if stand_pat >= beta {
        return beta;
    }
    alpha = alpha.max(stand_pat);

    let mut captures: Vec<_> = state.generate_moves().into_iter().filter(|&mv| is_capture(mv)).collect();
    order_moves(&mut captures, None);

    for mv in captures {
        let next = state.make_move(mv);
        let score = -quiescence(&next, ply + 1, -beta, -alpha);
        if score >= beta {
            return beta;
        }
        alpha = alpha.max(score);
    }

    alpha
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::set;
    use crate::types::{Color, Square};

    #[test]
    fn terminal_loss_is_scored_relative_to_ply() {
        // Same blocked construction as rules.rs's own regression test for
        // this case: White to move has no legal moves (each of its 4
        // pieces is boxed in by Black on every neighboring square), even
        // though both sides have well above 3 pieces on board.
        let white = set(set(set(set(0, Square(0)), Square(2)), Square(21)), Square(23));
        let black = set(set(set(set(0, Square(1)), Square(9)), Square(14)), Square(22));
        let state =
            CurrentGameState::from_bitboards(white, black, 0, 0, Color::White, 0, 100).unwrap();

        let mut tt = TranspositionTable::new(16);
        let score = negamax(&state, 3, 0, -WIN_SCORE - 1, WIN_SCORE + 1, &mut tt);
        assert_eq!(score, -WIN_SCORE);
    }

    #[test]
    fn quiescence_search_looks_through_a_hanging_capture() {
        // White can slide 14 -> 2, completing mill (0, 1, 2) and capturing
        // one of Black's 4 pieces (none in a mill). Black stays at 3 after
        // the capture — non-terminal — so this is a pure material swing, of
        // exactly the kind a depth-0 leaf would miss without quiescence:
        // the position "looks" balanced (both sides have two open threats,
        // material is even) until you look one ply further.
        let white = set(set(set(0, Square(0)), Square(1)), Square(14));
        let white = set(white, Square(9));
        let black = set(set(set(0, Square(5)), Square(6)), Square(18));
        let black = set(black, Square(20));
        let state =
            CurrentGameState::from_bitboards(white, black, 0, 0, Color::White, 0, 100).unwrap();

        let mut tt = TranspositionTable::new(16);
        let stand_pat = evaluate(&state, Color::White);
        let quiescent_score = negamax(&state, 0, 0, -WIN_SCORE - 1, WIN_SCORE + 1, &mut tt);

        assert!(
            quiescent_score > stand_pat,
            "quiescence should find White's capture and score above the static eval \
             (stand_pat={stand_pat}, quiescent_score={quiescent_score})"
        );
    }
}
