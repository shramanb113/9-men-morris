use std::fmt;

use crate::board::{BOARD_MASK, BitBoard, clear, is_bb, popcount, set};
use crate::types::{Color, GameResult, Square};

/// Bitboard-backed game state: side bitboards, hand counts, turn, and the
/// draw clock. Fields are private — every mutation goes through a method
/// that ends by asserting `invariants_hold`, so the four position
/// invariants (no overlap, only bits 0..23, counts agreeing with the
/// bitboards, no state that is neither empty/White/Black) can never be
/// violated silently.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CurrentGameState {
    white_pieces: BitBoard,
    black_pieces: BitBoard,
    white_unplaced: u8,
    black_unplaced: u8,
    turn: Color,
    plies_since_capture: u16,
    game_result: GameResult,
    draw_ply_limit: u16,
}

/// Why `CurrentGameState::from_bitboards` rejected its input. Unlike the
/// `debug_assert!`s used internally — which guard against bugs in this
/// crate's own code and compile out of release builds — these checks run
/// unconditionally, because `from_bitboards` is the boundary where data
/// from outside the engine (a UI, a save file, a future FFI/Kotlin caller)
/// first becomes a trusted `CurrentGameState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionError {
    /// The same square is set in both the white and black bitboards.
    Overlap,
    /// A bit outside squares 0..23 is set in white and/or black.
    OutOfBounds,
    /// `unplaced + pieces_on_board` exceeds the starting count of 9 for this side.
    TooManyPieces(Color),
}

impl fmt::Display for PositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionError::Overlap => write!(f, "white and black occupy the same square"),
            PositionError::OutOfBounds => {
                write!(f, "a piece bitboard sets a bit outside squares 0..23")
            }
            PositionError::TooManyPieces(color) => {
                write!(f, "{color:?} has more than 9 pieces across hand and board")
            }
        }
    }
}

impl std::error::Error for PositionError {}

impl CurrentGameState {
    pub fn new() -> Self {
        Self {
            white_pieces: 0,
            black_pieces: 0,
            white_unplaced: 9,
            black_unplaced: 9,
            turn: Color::White,
            plies_since_capture: 0,
            game_result: GameResult::Ongoing,
            draw_ply_limit: 100,
        }
    }

    /// Builds a `CurrentGameState` from raw parts, rejecting anything that
    /// violates the position invariants instead of trusting the caller.
    /// This is the one place external, untrusted input is allowed to
    /// become a `CurrentGameState` — every other constructor in this crate
    /// (`new`, `make_move`) only ever produces states this crate already
    /// trusts. Checked unconditionally so a release build still rejects a
    /// malformed position rather than silently running with it.
    pub fn from_bitboards(
        white: BitBoard,
        black: BitBoard,
        white_unplaced: u8,
        black_unplaced: u8,
        turn: Color,
        plies_since_capture: u16,
        draw_ply_limit: u16,
    ) -> Result<Self, PositionError> {
        let state = Self {
            white_pieces: white,
            black_pieces: black,
            white_unplaced,
            black_unplaced,
            turn,
            plies_since_capture,
            game_result: GameResult::Ongoing,
            draw_ply_limit,
        };

        if !state.no_overlap() {
            return Err(PositionError::Overlap);
        }
        if !state.in_bounds() {
            return Err(PositionError::OutOfBounds);
        }
        if !Self::count_ok(white_unplaced, white) {
            return Err(PositionError::TooManyPieces(Color::White));
        }
        if !Self::count_ok(black_unplaced, black) {
            return Err(PositionError::TooManyPieces(Color::Black));
        }

        Ok(state)
    }

    fn no_overlap(&self) -> bool {
        (self.white_pieces & self.black_pieces) == 0
    }

    fn in_bounds(&self) -> bool {
        (self.white_pieces | self.black_pieces) & !BOARD_MASK == 0
    }

    fn count_ok(unplaced: u8, pieces: BitBoard) -> bool {
        unplaced as u32 + popcount(pieces) <= 9
    }

    fn piece_counts_ok(&self) -> bool {
        Self::count_ok(self.white_unplaced, self.white_pieces)
            && Self::count_ok(self.black_unplaced, self.black_pieces)
    }

    /// True if this state satisfies every position invariant:
    /// White/Black never overlap, only bits 0..23 are ever set, and each
    /// side's hand + on-board pieces never exceed the starting count of 9.
    /// Holds for any state a caller can observe from outside this crate —
    /// i.e. anything returned by `make_move` or `new`.
    pub fn invariants_hold(&self) -> bool {
        self.no_overlap() && self.in_bounds() && self.piece_counts_ok()
    }

    /// Checked after every primitive bitboard mutation. Only the bitboard
    /// shape must hold at this granularity — the hand/board count
    /// invariant is a property of a *completed* move (see
    /// `debug_assert_fully_valid`, called from `moves.rs`), since a
    /// placement briefly has the piece counted in both hand and board
    /// between its two constituent primitive calls.
    fn debug_assert_valid(&self) {
        debug_assert!(
            self.no_overlap() && self.in_bounds(),
            "CurrentGameState bitboard invariant violated: {:#?}",
            self
        );
    }

    /// Checked once a full move has been applied (placement, slide, fly,
    /// including any capture). Used by `moves.rs::make_move`.
    pub(crate) fn debug_assert_fully_valid(&self) {
        debug_assert!(
            self.invariants_hold(),
            "CurrentGameState invariant violated: {:#?}",
            self
        );
    }

    // --- queries ---

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

    pub fn pieces(&self, side: Color) -> BitBoard {
        match side {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
        }
    }

    pub fn pieces_on_board(&self, side: Color) -> u8 {
        popcount(self.pieces(side)) as u8
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

    pub fn plies_since_capture(&self) -> u16 {
        self.plies_since_capture
    }

    pub fn draw_ply_limit(&self) -> u16 {
        self.draw_ply_limit
    }

    pub fn game_result(&self) -> GameResult {
        self.game_result
    }

    pub fn occupied(&self) -> BitBoard {
        self.white_pieces | self.black_pieces
    }

    /// Returns a bitboard of all empty squares on the board.
    pub fn empty_squares(&self) -> BitBoard {
        BOARD_MASK & !self.occupied()
    }

    // --- primitive mutators ---
    // Crate-private: `moves.rs` composes these into full Place/Slide/Fly
    // application. Keeping them out of the public API means a
    // `CurrentGameState` can only ever be mutated along paths that end in
    // `debug_assert_valid`.

    pub(crate) fn place_piece(&mut self, color: Color, sq: Square) {
        debug_assert!(
            self.is_empty(sq),
            "attempting to place piece on non-empty square: color={:#?}, sq={:#?}",
            color,
            sq
        );
        match color {
            Color::White => self.white_pieces = set(self.white_pieces, sq),
            Color::Black => self.black_pieces = set(self.black_pieces, sq),
        }
        self.debug_assert_valid();
    }

    pub(crate) fn remove_piece(&mut self, sq: Square) {
        self.white_pieces = clear(self.white_pieces, sq);
        self.black_pieces = clear(self.black_pieces, sq);
        self.debug_assert_valid();
    }

    pub(crate) fn move_piece(&mut self, color: Color, from: Square, to: Square) {
        debug_assert!(
            self.owner(from) == Some(color),
            "move_piece source {:?} not owned by {:?}",
            from,
            color
        );
        self.remove_piece(from);
        self.place_piece(color, to);
    }

    pub(crate) fn dec_unplaced(&mut self, color: Color) {
        match color {
            Color::White => {
                debug_assert!(self.white_unplaced > 0, "no white pieces left to place");
                self.white_unplaced -= 1;
            }
            Color::Black => {
                debug_assert!(self.black_unplaced > 0, "no black pieces left to place");
                self.black_unplaced -= 1;
            }
        }
        self.debug_assert_valid();
    }

    pub(crate) fn set_turn(&mut self, color: Color) {
        self.turn = color;
    }

    pub(crate) fn reset_capture_clock(&mut self) {
        self.plies_since_capture = 0;
    }

    pub(crate) fn tick_capture_clock(&mut self) {
        self.plies_since_capture += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_is_valid_and_empty() {
        let state = CurrentGameState::new();
        assert!(state.invariants_hold());
        assert_eq!(state.occupied(), 0);
        assert_eq!(state.pieces_in_hand(Color::White), 9);
        assert_eq!(state.pieces_in_hand(Color::Black), 9);
    }

    #[test]
    fn placing_keeps_state_valid() {
        let state = CurrentGameState::new();
        let next = state.make_move(crate::types::Move::Place { to: Square(0), capture: None });
        assert!(next.invariants_hold());
        assert!(next.is_white(Square(0)));
        assert!(!next.is_black(Square(0)));
        assert!(!next.is_empty(Square(0)));
        assert_eq!(next.pieces_in_hand(Color::White), 8);
    }

    #[test]
    fn detects_white_black_overlap() {
        let mut state = CurrentGameState::new();
        state.white_pieces = set(state.white_pieces, Square(5));
        state.black_pieces = set(state.black_pieces, Square(5));
        assert!(!state.invariants_hold());
    }

    #[test]
    fn detects_bits_outside_board_mask() {
        let mut state = CurrentGameState::new();
        state.white_pieces |= 1 << 24;
        assert!(!state.invariants_hold());
    }

    #[test]
    fn detects_piece_count_exceeding_starting_total() {
        let mut state = CurrentGameState::new();
        // 9 in hand plus a 10th already on the board is impossible.
        state.white_pieces = set(state.white_pieces, Square(0));
        assert!(!state.invariants_hold());
    }

    #[test]
    fn square_is_exactly_one_of_empty_white_black() {
        let mut state = CurrentGameState::new();
        state.place_piece(Color::Black, Square(10));
        assert!(state.is_black(Square(10)));
        assert!(!state.is_white(Square(10)));
        assert!(!state.is_empty(Square(10)));
        assert!(state.is_empty(Square(11)));
    }

    #[test]
    fn from_bitboards_accepts_a_valid_position() {
        let state = CurrentGameState::from_bitboards(
            set(0, Square(0)),
            set(0, Square(1)),
            8,
            8,
            Color::White,
            0,
            100,
        )
        .expect("valid position should be accepted");
        assert!(state.is_white(Square(0)));
        assert!(state.is_black(Square(1)));
        assert_eq!(state.pieces_in_hand(Color::White), 8);
    }

    #[test]
    fn from_bitboards_rejects_overlap() {
        let overlapping = set(0, Square(3));
        let err = CurrentGameState::from_bitboards(overlapping, overlapping, 8, 8, Color::White, 0, 100)
            .unwrap_err();
        assert_eq!(err, PositionError::Overlap);
    }

    #[test]
    fn from_bitboards_rejects_bits_outside_board() {
        let err = CurrentGameState::from_bitboards(1 << 24, 0, 9, 9, Color::White, 0, 100)
            .unwrap_err();
        assert_eq!(err, PositionError::OutOfBounds);
    }

    #[test]
    fn from_bitboards_rejects_too_many_pieces() {
        // 9 already on the board plus a 10th claimed to still be in hand.
        let nine_white = (0..9).fold(0u32, |bb, sq| set(bb, Square(sq)));
        let err = CurrentGameState::from_bitboards(nine_white, 0, 1, 9, Color::White, 0, 100)
            .unwrap_err();
        assert_eq!(err, PositionError::TooManyPieces(Color::White));
    }

    #[test]
    fn from_bitboards_error_messages_are_readable() {
        assert_eq!(
            PositionError::Overlap.to_string(),
            "white and black occupy the same square"
        );
        assert_eq!(
            PositionError::TooManyPieces(Color::Black).to_string(),
            "Black has more than 9 pieces across hand and board"
        );
    }
}
