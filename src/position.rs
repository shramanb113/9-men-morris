use crate::{board::{BitBoard, SQUARE_MILLS, clear, is_bb, popcount, set}, types::{Color, GameResult, Phase, Square}};

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
        let [m1, m2] = SQUARE_MILLS[sq.0 as usize];
        [m1, m2].iter().filter(|&&mill| (hypothetical & mill) == mill).count() as u8
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

    
}