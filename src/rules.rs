use crate::board::{BitBoard, MOVES, SQUARE_MILLS, clear, popcount, set};
use crate::position::CurrentGameState;
use crate::types::{Captures, Color, GameResult, Move, Phase, Square};

impl CurrentGameState {
    // --- phase ---

    pub fn phase(&self, side: Color) -> Phase {
        if self.pieces_in_hand(side) > 0 {
            Phase::Placing
        } else if popcount(self.pieces(side)) <= 3 {
            Phase::Flying
        } else {
            Phase::Sliding
        }
    }

    pub fn current_phase(&self) -> Phase {
        self.phase(self.side_to_move())
    }

    pub fn can_fly(&self, side: Color) -> bool {
        self.phase(side) == Phase::Flying
    }

    // --- mill detection ---

    /// Returns true if the given color completely owns this mill bitboard.
    pub fn is_mill(&self, side: Color, mill: BitBoard) -> bool {
        self.pieces(side) & mill == mill
    }

    /// Returns true if the piece on `sq` is currently part of any mill for `color`.
    pub fn is_in_mill(&self, color: Color, sq: Square) -> bool {
        let [m1, m2]: [BitBoard; 2] = SQUARE_MILLS[sq.0 as usize];
        self.is_mill(color, m1) || self.is_mill(color, m2)
    }

    /// Returns true if placing/moving a piece of `color` to `sq` would complete
    /// one or more mills.
    pub fn forms_mill(&self, color: Color, sq: Square, from: Option<Square>) -> bool {
        self.mills_created_by(color, sq, from) > 0
    }

    /// Returns how many mills would be completed by landing on `sq` with `color`.
    /// Useful for evaluation and debugging (normally 0, 1, or 2).
    pub fn mills_created_by(&self, color: Color, sq: Square, from: Option<Square>) -> u8 {
        let mut hypothetical = self.pieces(color);
        if let Some(from) = from {
            hypothetical = clear(hypothetical, from);
        }
        hypothetical = set(hypothetical, sq);
        let [m1, m2]: [BitBoard; 2] = SQUARE_MILLS[sq.0 as usize];
        let mut mills = 0u8;
        if (hypothetical & m1) == m1 {
            mills += 1;
        }
        if (hypothetical & m2) == m2 {
            mills += 1;
        }
        mills
    }

    /// Returns true if every piece the given color still has on the board
    /// is currently part of a mill. Needed for the capture rule exception.
    pub fn all_pieces_in_mills(&self, color: Color) -> bool {
        let mut remaining = self.pieces(color);
        while remaining != 0 {
            let sq = Square(remaining.trailing_zeros() as u8);
            if !self.is_in_mill(color, sq) {
                return false;
            }
            remaining = clear(remaining, sq);
        }
        true
    }

    // --- terminal / draw / result ---

    /// Returns true if either side already has fewer than 3 pieces with none
    /// left in hand — losing by piece count alone, no move generation needed.
    pub fn is_terminal_by_pieces(&self) -> bool {
        let lost = |side: Color| self.pieces_in_hand(side) == 0 && self.pieces_on_board(side) < 3;
        lost(Color::White) || lost(Color::Black)
    }

    pub fn has_legal_moves(&self) -> bool {
        !self.generate_moves().is_empty()
    }

    pub fn is_terminal(&self) -> bool {
        self.is_terminal_by_pieces() || !self.has_legal_moves()
    }

    pub fn is_draw_by_plies(&self) -> bool {
        self.plies_since_capture() >= self.draw_ply_limit()
    }

    pub fn is_game_over(&self) -> bool {
        self.is_terminal() || self.is_draw_by_plies()
    }

    pub fn result(&self) -> GameResult {
        if !self.is_terminal() {
            return GameResult::Ongoing;
        }
        let lost = |side: Color| self.pieces_in_hand(side) == 0 && self.pieces_on_board(side) < 3;
        if lost(Color::White) {
            return GameResult::Winner(Color::Black);
        }
        if lost(Color::Black) {
            return GameResult::Winner(Color::White);
        }
        // Terminal but not by piece count: the side to move has no legal moves.
        GameResult::Winner(self.side_to_move().opponent())
    }

    // --- move generation ---

    /// Returns all pieces of the given color that can currently move
    /// (used for slide; flying pieces can always move if a square is free).
    fn movable_pieces(&self, color: Color) -> BitBoard {
        let empties = self.empty_squares();
        let mut own = self.pieces(color);
        let mut movable = 0;
        while own != 0 {
            let sq = Square(own.trailing_zeros() as u8);
            if MOVES[sq.0 as usize] & empties != 0 {
                movable = set(movable, sq);
            }
            own = clear(own, sq);
        }
        movable
    }

    fn with_captures_applied(mv: Move, captures: Captures) -> Move {
        match mv {
            Move::Place { to, .. } => Move::Place { to, captures },
            Move::Slide { from, to, .. } => Move::Slide { from, to, captures },
            Move::Fly { from, to, .. } => Move::Fly { from, to, captures },
        }
    }

    /// All ways to choose `n` distinct capture squares from `available`
    /// (`n` is always 1 or 2 here — at most two mills can form from a
    /// single move, so there's never a need for a general k-combinations
    /// routine).
    fn capture_combinations(available: &[Square], n: usize) -> Vec<Captures> {
        match n {
            1 => available.iter().map(|&sq| Captures::one(sq)).collect(),
            2 => {
                let mut combos = Vec::new();
                for i in 0..available.len() {
                    for &b in &available[i + 1..] {
                        combos.push(Captures::two(available[i], b));
                    }
                }
                combos
            }
            _ => unreachable!("a single move can form at most two mills"),
        }
    }

    fn with_captures(&self, base_moves: Vec<Move>, color: Color) -> Vec<Move> {
        // computed once — identical for every move in this batch
        let available = self.generate_captures(color);
        let mut result = Vec::with_capacity(base_moves.len());

        for mv in base_moves {
            let mill_count = match mv {
                Move::Place { to, .. } => self.mills_created_by(color, to, None),
                Move::Slide { from, to, .. } | Move::Fly { from, to, .. } => {
                    self.mills_created_by(color, to, Some(from))
                }
            };

            if mill_count == 0 {
                result.push(mv);
                continue;
            }

            // Capped by how many targets are actually legal to capture: a
            // double mill against an opponent with only one non-mill piece
            // (and not all their pieces milled) still only yields one
            // capture, not two — you can't capture more than exists.
            let n = (mill_count as usize).min(available.len());
            debug_assert!(
                n > 0,
                "mill formed with no capture targets — opponent has {} pieces; \
                 caller must check is_terminal() before generating moves",
                self.pieces_on_board(color.opponent())
            );
            for captures in Self::capture_combinations(&available, n) {
                result.push(Self::with_captures_applied(mv, captures));
            }
        }
        result
    }

    /// Returns all squares that the given color is currently allowed to capture.
    /// Respects the rule: cannot capture a piece in a mill unless all opponent
    /// pieces are in mills.
    fn generate_captures(&self, color: Color) -> Vec<Square> {
        let opponent = color.opponent();
        let mut remaining = self.pieces(opponent);
        let mut squares = Vec::new();
        while remaining != 0 {
            let sq = Square(remaining.trailing_zeros() as u8);
            squares.push(sq);
            remaining = clear(remaining, sq);
        }

        if self.all_pieces_in_mills(opponent) {
            // Exception: once every remaining piece is in a mill, mill
            // membership no longer protects any of them from capture.
            return squares;
        }

        squares.retain(|&sq| !self.is_in_mill(opponent, sq));
        squares
    }

    /// Generates all legal placing moves (Phase 1).
    fn generate_place_moves(&self, color: Color) -> Vec<Move> {
        let mut base = Vec::new();
        let mut empties = self.empty_squares();
        while empties != 0 {
            let to = Square(empties.trailing_zeros() as u8);
            base.push(Move::Place { to, captures: Captures::NONE });
            empties = clear(empties, to);
        }
        self.with_captures(base, color)
    }

    /// Generates all legal sliding moves (Phase 2).
    fn generate_slide_moves(&self, color: Color) -> Vec<Move> {
        let mut base = Vec::new();
        let empties = self.empty_squares();
        let mut movable = self.movable_pieces(color);

        while movable != 0 {
            let from = Square(movable.trailing_zeros() as u8);
            let mut dests = MOVES[from.0 as usize] & empties;
            while dests != 0 {
                let to = Square(dests.trailing_zeros() as u8);
                base.push(Move::Slide { from, to, captures: Captures::NONE });
                dests = clear(dests, to);
            }
            movable = clear(movable, from);
        }
        self.with_captures(base, color)
    }

    /// Generates all legal flying moves (Phase 3).
    fn generate_fly_moves(&self, color: Color) -> Vec<Move> {
        let mut base = Vec::new();
        let empties = self.empty_squares();
        let mut own = self.pieces(color); // no adjacency restriction — flying reaches any empty square
        while own != 0 {
            let from = Square(own.trailing_zeros() as u8);
            let mut dests = empties;
            while dests != 0 {
                let to = Square(dests.trailing_zeros() as u8);
                base.push(Move::Fly { from, to, captures: Captures::NONE });
                dests = clear(dests, to);
            }
            own = clear(own, from);
        }
        self.with_captures(base, color)
    }

    /// Generates all legal moves for the side to move in the current position.
    pub fn generate_moves(&self) -> Vec<Move> {
        debug_assert!(
            !self.is_terminal_by_pieces(),
            "generate_moves called on a position already lost by piece count: white={}, black={}",
            self.pieces_on_board(Color::White),
            self.pieces_on_board(Color::Black)
        );

        let color = self.side_to_move();
        match self.current_phase() {
            Phase::Placing => self.generate_place_moves(color),
            Phase::Sliding => self.generate_slide_moves(color),
            Phase::Flying => self.generate_fly_moves(color),
        }
    }
}

#[cfg(test)]
mod verification {
    use super::*;

    #[test]
    fn generate_moves_does_not_recurse() {
        let state = CurrentGameState::new();
        assert!(!state.is_terminal());
        let moves = state.generate_moves();
        assert_eq!(moves.len(), 24); // empty board, phase Placing, no mills possible yet
    }

    #[test]
    fn result_reports_winner_when_side_to_move_is_blocked() {
        // White pieces sit on four degree-2 squares (0, 2, 21, 23); Black
        // occupies every one of their neighbors (1, 9, 14, 22), so White has
        // zero movable pieces even though both sides have 4 pieces on board
        // (well above the <3-piece loss threshold). This is terminal only
        // via "no legal moves", not via piece count.
        let white = set(set(set(set(0, Square(0)), Square(2)), Square(21)), Square(23));
        let black = set(set(set(set(0, Square(1)), Square(9)), Square(14)), Square(22));
        let state = CurrentGameState::from_bitboards(white, black, 0, 0, Color::White, 0, 100)
            .expect("valid position");

        assert!(!state.has_legal_moves());
        assert!(state.is_terminal());
        assert_eq!(state.result(), GameResult::Winner(Color::Black));
    }

    #[test]
    fn completing_two_mills_at_once_offers_two_captures() {
        // White already owns 0, 2 (two-thirds of mill (0,1,2)) and 4, 7
        // (two-thirds of mill (1,4,7)). Placing on square 1 completes both
        // simultaneously. Black has 4 pieces, none in a mill, so there are
        // 4 legal capture targets and every distinct pair should appear.
        let white =
            set(set(set(set(0, Square(0)), Square(2)), Square(4)), Square(7));
        let black =
            set(set(set(set(0, Square(3)), Square(5)), Square(6)), Square(8));
        let state = CurrentGameState::from_bitboards(white, black, 5, 5, Color::White, 0, 100)
            .expect("valid position");

        assert_eq!(state.mills_created_by(Color::White, Square(1), None), 2);

        let moves = state.generate_moves();
        let double_captures: Vec<_> = moves
            .iter()
            .filter_map(|mv| match mv {
                Move::Place { to: Square(1), captures } if captures.len() == 2 => Some(*captures),
                _ => None,
            })
            .collect();

        // C(4, 2) = 6 distinct pairs from {3, 5, 6, 8}.
        assert_eq!(double_captures.len(), 6);
        for &(a, b) in &[(3, 5), (3, 6), (3, 8), (5, 6), (5, 8), (6, 8)] {
            assert!(
                double_captures
                    .iter()
                    .any(|c| c.contains(Square(a)) && c.contains(Square(b))),
                "missing capture pair ({a}, {b})"
            );
        }

        // No single-capture variant should exist for this move — only the
        // full double-capture is legal once two mills form together.
        assert!(
            !moves
                .iter()
                .any(|mv| matches!(mv, Move::Place { to: Square(1), captures } if captures.len() == 1))
        );
    }

    #[test]
    fn generate_captures_allows_capturing_from_a_mill_when_all_pieces_are_milled() {
        let white = set(set(0, Square(0)), Square(1));
        let black = set(set(set(0, Square(3)), Square(4)), Square(5)); // mill (3,4,5)
        let state = CurrentGameState::from_bitboards(white, black, 7, 6, Color::White, 0, 100)
            .expect("valid position");

        // Placing on square 2 completes mill (0,1,2); every Black piece is
        // already in a mill, so all three become valid capture targets.
        let moves = state.generate_moves();
        let capturing_moves = moves
            .iter()
            .filter(|mv| matches!(mv, Move::Place { to: Square(2), captures } if !captures.is_empty()))
            .count();
        assert_eq!(capturing_moves, 3);
    }

    #[test]
    fn flying_reaches_non_adjacent_empty_squares() {
        // Flying phase (3 pieces, hand empty): unlike sliding, a flying
        // piece may land on any empty square, not just an adjacent one.
        // Square 0's only neighbors are 1 and 9, so a Fly to square 16
        // exists only if adjacency is correctly ignored here.
        let white = set(set(set(0, Square(0)), Square(5)), Square(20));
        let black = set(set(set(0, Square(8)), Square(11)), Square(17));
        let state = CurrentGameState::from_bitboards(white, black, 0, 0, Color::White, 0, 100)
            .expect("valid position");

        assert_eq!(state.current_phase(), Phase::Flying);
        let moves = state.generate_moves();
        assert!(
            moves.contains(&Move::Fly { from: Square(0), to: Square(16), captures: Captures::NONE })
        );
    }

    #[test]
    fn full_placing_phase_plays_out_without_panicking() {
        let mut state = CurrentGameState::new();
        let mut rng_seed = 7u64;
        for _ in 0..18 {
            if state.is_terminal() {
                break;
            }
            let moves = state.generate_moves();
            assert!(!moves.is_empty());
            rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = (rng_seed >> 33) as usize % moves.len();
            state = state.make_move(moves[idx]);
            assert!(state.invariants_hold());
        }
    }
}
