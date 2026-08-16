use crate::position::CurrentGameState;
use crate::types::{Captures, Color, Move, Square};

/// Applies a `Move` to a `CurrentGameState`, producing the resulting state.
/// This module knows nothing about legality — `rules.rs` decides which
/// moves are generated; this module just plays one out.
impl CurrentGameState {
    fn apply_capture(&mut self, captures: Captures) {
        if captures.is_empty() {
            self.tick_capture_clock();
            return;
        }
        for sq in captures.iter() {
            self.remove_piece(sq);
        }
        self.reset_capture_clock();
    }

    /// Places a piece on the new square and removes it from hand.
    /// Order matters: hand count must drop before the piece appears on the
    /// board, otherwise the piece is briefly counted in both places at once.
    fn apply_place(&mut self, color: Color, to: Square, captures: Captures) {
        self.dec_unplaced(color);
        self.place_piece(color, to);
        self.apply_capture(captures);
    }

    /// Moves a piece from the old square to the new square and applies captures.
    fn apply_slide(&mut self, color: Color, from: Square, to: Square, captures: Captures) {
        self.move_piece(color, from, to);
        self.apply_capture(captures);
    }

    /// Moves a piece from the old square to the new square and applies captures.
    fn apply_fly(&mut self, color: Color, from: Square, to: Square, captures: Captures) {
        self.move_piece(color, from, to);
        self.apply_capture(captures);
    }

    pub fn make_move(&self, mv: Move) -> CurrentGameState {
        let mut next = self.clone();
        let color = next.side_to_move();
        match mv {
            Move::Place { to, captures } => next.apply_place(color, to, captures),
            Move::Slide { from, to, captures } => next.apply_slide(color, from, to, captures),
            Move::Fly { from, to, captures } => next.apply_fly(color, from, to, captures),
        }
        next.set_turn(color.opponent());
        next.debug_assert_fully_valid();
        next
    }
}
