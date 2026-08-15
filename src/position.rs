use crate::{board::{BitBoard, is_bb, popcount}, types::{Color, GameResult, Phase, Square}};

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
}