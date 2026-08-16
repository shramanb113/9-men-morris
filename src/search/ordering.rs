use crate::types::Move;

/// Orders `moves` in place to maximize alpha-beta cutoffs: the
/// transposition table's remembered best move first (if present among
/// `moves`), then capturing moves, then everything else in generation
/// order. Capture info is already attached to each `Move` by
/// `rules::generate_moves`, so no board lookups are needed here.
pub fn order_moves(moves: &mut [Move], tt_move: Option<Move>) {
    moves.sort_by_key(|&mv| move_priority(mv, tt_move));
}

fn move_priority(mv: Move, tt_move: Option<Move>) -> u8 {
    if Some(mv) == tt_move {
        0
    } else if is_capture(mv) {
        1
    } else {
        2
    }
}

fn is_capture(mv: Move) -> bool {
    match mv {
        Move::Place { captures, .. }
        | Move::Slide { captures, .. }
        | Move::Fly { captures, .. } => !captures.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Captures, Square};

    #[test]
    fn tt_move_sorts_first() {
        let a = Move::Place { to: Square(0), captures: Captures::NONE };
        let b = Move::Place { to: Square(1), captures: Captures::NONE };
        let c = Move::Place { to: Square(2), captures: Captures::one(Square(5)) };
        let mut moves = vec![a, b, c];
        order_moves(&mut moves, Some(b));
        assert_eq!(moves[0], b);
    }

    #[test]
    fn captures_sort_before_quiet_moves_when_no_tt_move() {
        let quiet = Move::Place { to: Square(0), captures: Captures::NONE };
        let capture = Move::Place { to: Square(1), captures: Captures::one(Square(5)) };
        let mut moves = vec![quiet, capture];
        order_moves(&mut moves, None);
        assert_eq!(moves[0], capture);
    }
}
