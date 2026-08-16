use crate::board::{MILLS, popcount};
use crate::position::CurrentGameState;
use crate::types::Color;

/// Value of one piece, whether on the board or still in hand — an unplaced
/// piece during the placing phase is just as much an asset as a placed one.
const PIECE_VALUE: i32 = 100;
/// Value of one "open" mill: two of its three squares owned, the third
/// empty, i.e. one move away from completing it.
const MILL_THREAT_VALUE: i32 = 20;

fn material(state: &CurrentGameState, color: Color) -> i32 {
    (state.pieces_on_board(color) as i32 + state.pieces_in_hand(color) as i32) * PIECE_VALUE
}

/// Number of mills where `color` owns exactly two of the three squares and
/// the third is empty. Pure bitboard counting over the static `MILLS`
/// table — no move generation involved.
fn open_mill_count(state: &CurrentGameState, color: Color) -> i32 {
    let own = state.pieces(color);
    let empty = state.empty_squares();
    MILLS
        .iter()
        .filter(|&&mill| popcount(own & mill) == 2 && (empty & mill) != 0)
        .count() as i32
}

/// Score of `state` from `side`'s point of view: positive means `side` is
/// better off, negative means the opponent is. Combines material with open
/// mill threats for each side.
pub fn evaluate(state: &CurrentGameState, side: Color) -> i32 {
    let opponent = side.opponent();
    let side_score = material(state, side) + open_mill_count(state, side) * MILL_THREAT_VALUE;
    let opponent_score =
        material(state, opponent) + open_mill_count(state, opponent) * MILL_THREAT_VALUE;
    side_score - opponent_score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::set;
    use crate::types::Square;

    #[test]
    fn empty_board_is_symmetric() {
        let state = CurrentGameState::new();
        assert_eq!(evaluate(&state, Color::White), 0);
        assert_eq!(evaluate(&state, Color::Black), 0);
    }

    #[test]
    fn material_edge_is_scored_for_the_side_ahead() {
        // White: 0 on board + 9 in hand = 9 total.
        // Black: 0 on board + 8 in hand = 8 total (down one piece).
        let state =
            CurrentGameState::from_bitboards(0, 0, 9, 8, Color::White, 0, 100).unwrap();
        assert_eq!(evaluate(&state, Color::White), PIECE_VALUE);
        assert_eq!(evaluate(&state, Color::Black), -PIECE_VALUE);
    }

    #[test]
    fn open_mill_threat_is_scored_for_the_threatening_side() {
        // White owns two of mill (0, 1, 2); square 2 is empty. Material is
        // kept equal (9 total each side) so only the threat term shows up.
        let white = set(set(0, Square(0)), Square(1));
        let state =
            CurrentGameState::from_bitboards(white, 0, 7, 9, Color::White, 0, 100).unwrap();
        assert_eq!(evaluate(&state, Color::White), MILL_THREAT_VALUE);
        assert_eq!(evaluate(&state, Color::Black), -MILL_THREAT_VALUE);
    }
}
