use crate::board::clear;
use crate::position::CurrentGameState;
use crate::types::{Color, Square};

/// Zobrist hashing: XOR together one random 64-bit number per independent
/// "fact" that is true about a `CurrentGameState`, so equal states always
/// collapse to the same key and (with overwhelming probability) different
/// states don't. This is what a search transposition table indexes on.
///
/// Unlike chess, the board bitboards alone are *not* enough facts: two
/// states can have identical `white_pieces`/`black_pieces` but different
/// `pieces_in_hand`, and that difference changes `Phase` and therefore
/// which moves are legal. So `pieces_in_hand` per side is hashed too.
///
/// `plies_since_capture` is deliberately excluded — it changes almost every
/// ply, so hashing it would defeat transposition sharing entirely. This is
/// the standard "graph-history-interaction" tradeoff every engine makes;
/// it means a cached score should not be trusted as an exact draw score
/// without extra care, but plain minimax/negamax value lookups are fine.
const fn color_index(color: Color) -> usize {
    match color {
        Color::White => 0,
        Color::Black => 1,
    }
}

/// splitmix64: a small, fast, well-distributed PRNG, used only to fill the
/// static tables below at compile time. Chosen over `rand` to keep the
/// crate at zero external dependencies. Deterministic seed -> deterministic
/// tables -> reproducible hashes across builds/runs, which matters for
/// debugging a transposition table.
const fn splitmix64(seed: u64) -> (u64, u64) {
    let seed = seed.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    (seed, z ^ (z >> 31))
}

struct ZobristTables {
    /// [color][square] -> random key for "this color occupies this square"
    piece: [[u64; 24]; 2],
    /// [color][count] -> random key for "this color has `count` pieces in hand" (0..=9)
    unplaced: [[u64; 10]; 2],
    /// XOR'd in whenever Black is to move
    side_to_move: u64,
}

const fn build_tables() -> ZobristTables {
    // Arbitrary fixed seed (first digits of e and pi interleaved) — any
    // constant works, it just needs to be fixed so the tables are stable.
    let mut seed: u64 = 0x2718281828459045 ^ 0x3141592653589793;

    let mut piece = [[0u64; 24]; 2];
    let mut color = 0;
    while color < 2 {
        let mut sq = 0;
        while sq < 24 {
            let (next_seed, value) = splitmix64(seed);
            seed = next_seed;
            piece[color][sq] = value;
            sq += 1;
        }
        color += 1;
    }

    let mut unplaced = [[0u64; 10]; 2];
    let mut color = 0;
    while color < 2 {
        let mut n = 0;
        while n < 10 {
            let (next_seed, value) = splitmix64(seed);
            seed = next_seed;
            unplaced[color][n] = value;
            n += 1;
        }
        color += 1;
    }

    let (_, side_to_move) = splitmix64(seed);

    ZobristTables { piece, unplaced, side_to_move }
}

static TABLES: ZobristTables = build_tables();

impl CurrentGameState {
    /// Full-recompute Zobrist hash of this state. Not incrementally
    /// maintained: with only 24 squares, recomputing from scratch is a
    /// couple dozen XORs (cheap enough that incremental upkeep during
    /// search would add bug surface without a measurable win). Revisit
    /// only if profiling shows this is actually a hot spot.
    pub fn zobrist(&self) -> u64 {
        let mut hash = 0u64;

        for &color in &[Color::White, Color::Black] {
            let idx = color_index(color);
            let mut remaining = self.pieces(color);
            while remaining != 0 {
                let sq = Square(remaining.trailing_zeros() as u8);
                hash ^= TABLES.piece[idx][sq.0 as usize];
                remaining = clear(remaining, sq);
            }
            hash ^= TABLES.unplaced[idx][self.pieces_in_hand(color) as usize];
        }

        if self.side_to_move() == Color::Black {
            hash ^= TABLES.side_to_move;
        }

        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Move;

    #[test]
    fn deterministic_for_equal_states() {
        let a = CurrentGameState::new();
        let b = CurrentGameState::new();
        assert_eq!(a.zobrist(), b.zobrist());
    }

    #[test]
    fn placing_a_piece_changes_the_hash() {
        let state = CurrentGameState::new();
        let before = state.zobrist();
        let after = state.make_move(Move::Place { to: Square(0), capture: None });
        assert_ne!(before, after.zobrist());
    }

    #[test]
    fn side_to_move_affects_the_hash() {
        let state = CurrentGameState::new();
        let after_white_moves =
            state.make_move(Move::Place { to: Square(0), capture: None });
        let mut same_board_white_to_move = after_white_moves.clone();
        same_board_white_to_move.set_turn(Color::White);
        assert_ne!(after_white_moves.zobrist(), same_board_white_to_move.zobrist());
    }

    /// The property that actually justifies bothering with Zobrist at all:
    /// two different move orders that land on the identical position must
    /// hash identically. Turns strictly alternate White/Black, so a real
    /// transposition has to keep each square assigned to the same color in
    /// both orderings — e.g. White plays {0,1} and Black plays {9,10}
    /// regardless of which of their own two squares each side plays first.
    #[test]
    fn transposition_same_position_hashes_equal() {
        let start = CurrentGameState::new();

        let order_a = start
            .make_move(Move::Place { to: Square(0), capture: None }) // White
            .make_move(Move::Place { to: Square(9), capture: None }) // Black
            .make_move(Move::Place { to: Square(1), capture: None }) // White
            .make_move(Move::Place { to: Square(10), capture: None }); // Black

        let order_b = start
            .make_move(Move::Place { to: Square(1), capture: None }) // White
            .make_move(Move::Place { to: Square(10), capture: None }) // Black
            .make_move(Move::Place { to: Square(0), capture: None }) // White
            .make_move(Move::Place { to: Square(9), capture: None }); // Black

        assert_eq!(order_a, order_b);
        assert_eq!(order_a.zobrist(), order_b.zobrist());
    }

    /// Same board occupancy, different hand counts, must NOT collide — this
    /// is the case naive board-only hashing (i.e. copying a chess scheme
    /// verbatim) would silently get wrong.
    #[test]
    fn identical_board_different_hand_count_hashes_differently() {
        let mut fewer_captures = CurrentGameState::new();
        fewer_captures.place_piece(Color::White, Square(0));
        fewer_captures.dec_unplaced(Color::White);

        let mut more_captures = CurrentGameState::new();
        more_captures.place_piece(Color::White, Square(0));
        more_captures.dec_unplaced(Color::White);
        more_captures.dec_unplaced(Color::White);
        more_captures.dec_unplaced(Color::White);
        more_captures.place_piece(Color::White, Square(1));
        more_captures.remove_piece(Square(1));

        // Same occupied squares for White (only Square(0)), same side to
        // move, but different `pieces_in_hand` — must hash differently.
        assert_eq!(fewer_captures.pieces(Color::White), more_captures.pieces(Color::White));
        assert_ne!(fewer_captures.pieces_in_hand(Color::White), more_captures.pieces_in_hand(Color::White));
        assert_ne!(fewer_captures.zobrist(), more_captures.zobrist());
    }

    #[test]
    fn single_piece_on_each_square_hashes_are_pairwise_distinct() {
        let mut hashes = Vec::new();
        for sq in 0..24u8 {
            let state = CurrentGameState::new()
                .make_move(Move::Place { to: Square(sq), capture: None });
            hashes.push(state.zobrist());
        }
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "collision between square {i} and {j}");
            }
        }
    }
}
