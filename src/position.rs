use crate::{board::{BOARD_MASK, BitBoard, MOVES, SQUARE_MILLS, clear, is_bb, popcount, set}, types::{Color, GameResult, Phase, Square, Move}};

#[derive(Debug, Clone , PartialEq, Hash)]
pub struct CurrentGameState {
    pub white_pieces: BitBoard,
    pub black_pieces: BitBoard,
    pub white_unplaced: u8,
    pub black_unplaced: u8,
    pub turn: Color,
    pub plies_since_capture: u16,
    pub game_result: GameResult,
}


impl CurrentGameState {
    pub fn new() -> Self {  // initializes a new game state with all pieces in their starting positions
        Self {
            white_pieces: 0,
            black_pieces: 0,
            white_unplaced: 9,
            black_unplaced: 9,
            turn: Color::White,
            plies_since_capture: 0,
            game_result: GameResult::Ongoing,
        }
    }

    pub fn phase(&self, side: Color) -> Phase {
        let (unplaced, pieces) = match side {
            Color::Black => (self.black_unplaced, self.black_pieces),
            Color::White => (self.white_unplaced, self.white_pieces)
        };

        if unplaced > 0 {
            return Phase::Placing;
        } else if popcount(pieces) <= 3 {
            return Phase::Flying
        } else {
            return Phase::Sliding;
        }
    }

    pub fn is_white(&self, sq: Square) -> bool {
        is_bb(self.white_pieces, sq)
    }

    pub fn is_black(&self, sq: Square) -> bool {
        is_bb(self.black_pieces, sq)
    }

    pub fn is_empty(&self, sq: Square) -> bool {
        !self.is_white(sq) && !self.is_black(sq)
    }

    pub fn owner(&self, sq: Square) -> Option<Color> {
        if self.is_white(sq) {
            Some(Color::White)
        } else if self.is_black(sq) {
            Some(Color::Black)
        } else {
            None
        }
    }

    pub fn pieces_on_board(&self, side: Color) -> u8 {
        match side {
            Color::White => popcount(self.white_pieces) as u8,
            Color::Black => popcount(self.black_pieces) as u8,
        }
    }

    pub fn pieces_in_hand(&self, side: Color) -> u8 {
        match side {
            Color::White => self.white_unplaced,
            Color::Black => self.black_unplaced,
        }
    }

    pub fn total_pieces_on_board(&self) -> u8 {
        self.pieces_on_board(Color::White) + self.pieces_on_board(Color::Black)
    }

    pub fn side_to_move(&self) -> Color {
        self.turn
    }

    pub fn current_phase(&self) -> Phase {
        self.phase(self.turn)
    }

    pub fn can_fly(&self, side: Color) -> bool {
        self.phase(side) == Phase::Flying
    }

    pub fn pieces(&self, side: Color) -> BitBoard {
        match side {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
        }
    }

    // Mill detection 
    /// Returns true if the given color completely owns this mill bitboard.
    pub fn is_mill(&self, side: Color, mill: BitBoard) -> bool {
        let pieces = self.pieces(side);
        pieces & mill == mill
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
        if (hypothetical & m1) == m1 { mills += 1; }
        if (hypothetical & m2) == m2 { mills += 1; }
        mills
    }


    /// Returns true if every piece the given color still has on the board
    /// is currently part of a mill.
    /// Needed for the capture rule exception.
    pub fn all_pieces_in_mills(&self, color: Color) -> bool {
        let remaining = self.pieces(color);
        while remaining != 0 {
            let sq = Square(remaining.trailing_zeros() as u8);
            if !self.is_in_mill(color, sq) {
                return false;
            }
        }
        true
    }

    // Move generation

    /// True if either side already has fewer than 3 pieces on the board.
    /// A position in this state is terminal — generate_moves (and therefore
    /// with_captures) should never be called on it.
    pub fn is_terminal(&self) -> bool {
        let lost = |side: Color| self.pieces_in_hand(side) == 0 && self.pieces_on_board(side) < 3;
        lost(Color::White) || lost(Color::Black)
    }

    /// Returns a bitboard of all empty squares on the board.
    pub fn empty_squares(&self) -> BitBoard {
        BOARD_MASK & !(self.white_pieces | self.black_pieces)
    }

    /// Returns all pieces of the given color that can currently move
    /// (used mainly for slide/fly).
    fn movable_pieces(&self, color: Color) -> BitBoard {
        let empties: BitBoard = self.empty_squares();
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
        // computed once — identical for every move in this batch, see prior turn
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
    #[warn(unused_mut)]
    fn generate_captures(&self, color: Color) -> Vec<Square> {
        let opponent = color.opponent();
        let mut opp_pieces = self.pieces(opponent);

        let mut non_mill = Vec::new();
        let mut remaining = opp_pieces;
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
            !self.is_terminal(),
            "generate_moves called on terminal position: white={}, black={}",
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