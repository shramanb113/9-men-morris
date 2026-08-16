use crate::board::{BitBoard, MOVES, SQUARE_MILLS, clear, popcount, set};
use crate::position::CurrentGameState;
use crate::types::{Color, GameResult, Move, Phase, Square};

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

    fn with_capture(mv: Move, target: Square) -> Move {
        match mv {
            Move::Place { to, .. } => Move::Place { to, capture: Some(target) },
            Move::Slide { from, to, .. } => Move::Slide { from, to, capture: Some(target) },
            Move::Fly { from, to, .. } => Move::Fly { from, to, capture: Some(target) },
        }
    }

    fn with_captures(&self, base_moves: Vec<Move>, color: Color) -> Vec<Move> {
        // computed once — identical for every move in this batch
        let captures = self.generate_captures(color);
        let mut result = Vec::with_capacity(base_moves.len());

        for mv in base_moves {
            let formed_mill = match mv {
                Move::Place { to, .. } => self.forms_mill(color, to, None),
                Move::Slide { from, to, .. } | Move::Fly { from, to, .. } => {
                    self.forms_mill(color, to, Some(from))
                }
            };

            if formed_mill {
                debug_assert!(
                    !captures.is_empty(),
                    "mill formed with no capture targets — opponent has {} pieces; \
                     caller must check is_terminal() before generating moves",
                    self.pieces_on_board(color.opponent())
                );
                result.extend(captures.iter().map(|&target| Self::with_capture(mv, target)));
            } else {
                result.push(mv);
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
        let mut non_mill = Vec::new();
        while remaining != 0 {
            let sq = Square(remaining.trailing_zeros() as u8);
            if !self.is_in_mill(opponent, sq) {
                non_mill.push(sq);
            }
            remaining = clear(remaining, sq);
        }
        non_mill
    }

    /// Generates all legal placing moves (Phase 1).
    fn generate_place_moves(&self, color: Color) -> Vec<Move> {
        let mut base = Vec::new();
        let mut empties = self.empty_squares();
        while empties != 0 {
            let to = Square(empties.trailing_zeros() as u8);
            base.push(Move::Place { to, capture: None });
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
                base.push(Move::Slide { from, to, capture: None });
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
        let mut own = self.pieces(color); // no movable_pieces filter — everything can fly
        while own != 0 {
            let from = Square(own.trailing_zeros() as u8);
            let mut dests = MOVES[from.0 as usize] & empties;
            while dests != 0 {
                let to = Square(dests.trailing_zeros() as u8);
                base.push(Move::Fly { from, to, capture: None });
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
