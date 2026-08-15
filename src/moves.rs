use crate::position::CurrentGameState;
use crate::types::{Color, Move, Square};

/// Applies a `Move` to a `CurrentGameState`, producing the resulting state.
/// This module knows nothing about legality — `rules.rs` decides which
/// moves are generated; this module just plays one out.
impl CurrentGameState {
    fn apply_capture(&mut self, capture: Option<Square>) {
        match capture {
            Some(sq) => {
                self.remove_piece(sq);
                self.reset_capture_clock();
            }
            None => self.tick_capture_clock(),
        }
    }

    /// Places a piece on the new square and removes it from hand.
    /// Order matters: hand count must drop before the piece appears on the
    /// board, otherwise the piece is briefly counted in both places at once.
    fn apply_place(&mut self, color: Color, to: Square, capture: Option<Square>) {
        self.dec_unplaced(color);
        self.place_piece(color, to);
        self.apply_capture(capture);
    }

    /// Moves a piece from the old square to the new square and applies a capture.
    fn apply_slide(&mut self, color: Color, from: Square, to: Square, capture: Option<Square>) {
        self.move_piece(color, from, to);
        self.apply_capture(capture);
    }

    /// Moves a piece from the old square to the new square and applies a capture.
    fn apply_fly(&mut self, color: Color, from: Square, to: Square, capture: Option<Square>) {
        self.move_piece(color, from, to);
        self.apply_capture(capture);
    }

    pub fn make_move(&self, mv: Move) -> CurrentGameState {
        let mut next = self.clone();
        let color = next.side_to_move();
        match mv {
            Move::Place { to, capture } => next.apply_place(color, to, capture),
            Move::Slide { from, to, capture } => next.apply_slide(color, from, to, capture),
            Move::Fly { from, to, capture } => next.apply_fly(color, from, to, capture),
        }
        next.set_turn(color.opponent());
        next.debug_assert_fully_valid();
        next
    }
}
