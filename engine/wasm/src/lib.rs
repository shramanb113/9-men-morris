//! `wasm-bindgen` adapter for the `ninemensmorris` core. Owns everything
//! platform-specific — the JS-friendly `GameSession` API, plain-number
//! encodings of `Color`/`Phase`/`Square`, and untrusted-input validation
//! for human moves — so the core itself never has to know a browser
//! exists. See the crate-level design principle in the core's README:
//! adapters depend on the core, never the reverse.

use ninemensmorris::position::CurrentGameState;
use ninemensmorris::search;
use ninemensmorris::types::{Captures, Color, GameResult, Move, Phase, Square};
use wasm_bindgen::prelude::*;

fn color_to_u8(color: Color) -> u8 {
    match color {
        Color::White => 0,
        Color::Black => 1,
    }
}

fn u8_to_color(v: u8) -> Color {
    if v == 0 { Color::White } else { Color::Black }
}

fn phase_to_u8(phase: Phase) -> u8 {
    match phase {
        Phase::Placing => 0,
        Phase::Sliding => 1,
        Phase::Flying => 2,
    }
}

fn move_from_to(mv: Move) -> (Option<u8>, u8) {
    match mv {
        Move::Place { to, .. } => (None, to.0),
        Move::Slide { from, to, .. } | Move::Fly { from, to, .. } => (Some(from.0), to.0),
    }
}

fn move_captures(mv: Move) -> Captures {
    match mv {
        Move::Place { captures, .. }
        | Move::Slide { captures, .. }
        | Move::Fly { captures, .. } => captures,
    }
}

/// Does `mv`'s capture set match the exact squares JS asked for, as an
/// unordered set? Used to find which (if any) legal move `play_move`'s
/// untrusted `(from, to, captures)` triple refers to.
fn captures_match(mv: Move, requested: &[u8]) -> bool {
    let actual = move_captures(mv);
    actual.len() == requested.len() && requested.iter().all(|&sq| actual.contains(Square(sq)))
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct LegalMove {
    from: Option<u8>,
    to: u8,
}

#[wasm_bindgen]
impl LegalMove {
    #[wasm_bindgen(getter)]
    pub fn from(&self) -> Option<u8> {
        self.from
    }

    #[wasm_bindgen(getter)]
    pub fn to(&self) -> u8 {
        self.to
    }
}

#[wasm_bindgen]
pub struct MoveResult {
    from: Option<u8>,
    to: u8,
    captures: Vec<u8>,
    score: i32,
}

#[wasm_bindgen]
impl MoveResult {
    #[wasm_bindgen(getter)]
    pub fn from(&self) -> Option<u8> {
        self.from
    }

    #[wasm_bindgen(getter)]
    pub fn to(&self) -> u8 {
        self.to
    }

    #[wasm_bindgen(getter)]
    pub fn captures(&self) -> Vec<u8> {
        self.captures.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn score(&self) -> i32 {
        self.score
    }
}

/// One live game, mutated in place — the "single session per browser tab"
/// model. Wraps a `CurrentGameState`, which is itself immutable/functional
/// (`make_move` returns a new state); this struct is where that gets
/// turned into something a JS caller can hold onto and update.
#[wasm_bindgen]
pub struct GameSession {
    state: CurrentGameState,
}

#[wasm_bindgen]
impl GameSession {
    pub fn new_game() -> GameSession {
        GameSession { state: CurrentGameState::new() }
    }

    /// 24 entries, one per square: 0 empty, 1 white, 2 black.
    pub fn board(&self) -> Vec<u8> {
        (0..24u8)
            .map(|sq| match self.state.owner(Square(sq)) {
                None => 0,
                Some(Color::White) => 1,
                Some(Color::Black) => 2,
            })
            .collect()
    }

    pub fn side_to_move(&self) -> u8 {
        color_to_u8(self.state.side_to_move())
    }

    /// 0 placing, 1 sliding, 2 flying.
    pub fn phase(&self) -> u8 {
        phase_to_u8(self.state.current_phase())
    }

    pub fn pieces_in_hand(&self, color: u8) -> u8 {
        self.state.pieces_in_hand(u8_to_color(color))
    }

    /// Every currently-legal `(from, to)` pair, deduplicated across
    /// capture-choice variants — capture selection is a separate step, see
    /// `capture_targets`.
    pub fn legal_moves(&self) -> Vec<LegalMove> {
        let mut pairs: Vec<(Option<u8>, u8)> = Vec::new();
        for mv in self.state.generate_moves() {
            let pair = move_from_to(mv);
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
        pairs.into_iter().map(|(from, to)| LegalMove { from, to }).collect()
    }

    /// Legal individual capture squares for the move `(from, to)`, before
    /// it's actually played. Empty means that move doesn't form a mill and
    /// can be played via `play_move` with no captures. Non-empty means the
    /// human must choose (1 square normally, 2 distinct ones for a move
    /// that completes two mills at once) before calling `play_move`.
    pub fn capture_targets(&self, from: Option<u8>, to: u8) -> Vec<u8> {
        let mut targets: Vec<u8> = Vec::new();
        for mv in self.state.generate_moves() {
            if move_from_to(mv) == (from, to) {
                for sq in move_captures(mv).iter() {
                    if !targets.contains(&sq.0) {
                        targets.push(sq.0);
                    }
                }
            }
        }
        targets
    }

    /// Applies a human move. This is the untrusted-input boundary — same
    /// role `CurrentGameState::from_bitboards` plays for the core — so it
    /// looks the request up against `generate_moves()`'s own legality
    /// rather than re-deriving them, and rejects anything that doesn't
    /// match instead of trusting the caller.
    pub fn play_move(&mut self, from: Option<u8>, to: u8, captures: Vec<u8>) -> Result<(), JsValue> {
        self.try_play_move(from, to, captures).map_err(|e| JsValue::from_str(&e))
    }

    /// Runs the bot's move at the given difficulty ("easy"/"medium"/"hard")
    /// and applies it. `seed` is supplied by the caller — this crate, like
    /// the core, never generates its own randomness; JS owns real entropy
    /// (`Date.now()`, `Math.random()`, ...) and passes a fresh seed each
    /// call. Runs synchronously and can take several seconds at "hard" —
    /// call this from a Web Worker, not the main thread.
    pub fn bot_move(&mut self, difficulty: &str, seed: u64) -> Result<MoveResult, JsValue> {
        self.try_bot_move(difficulty, seed).map_err(|e| JsValue::from_str(&e))
    }

    pub fn is_game_over(&self) -> bool {
        self.state.is_game_over()
    }

    /// Winning color, if the game has a decisive result (`None` for an
    /// ongoing game or a draw).
    pub fn winner(&self) -> Option<u8> {
        match self.state.result() {
            GameResult::Winner(color) => Some(color_to_u8(color)),
            GameResult::Draw | GameResult::Ongoing => None,
        }
    }
}

/// The actual fallible logic, kept separate from the `#[wasm_bindgen]`
/// surface above and returning plain `Result<_, String>` instead of
/// `Result<_, JsValue>`. This split exists because constructing a
/// `JsValue` calls into `wasm_bindgen`'s JS-interop runtime, which isn't
/// present outside an actual wasm32 + JS host — doing so in a native
/// `cargo test` run doesn't just fail the assertion, it aborts the whole
/// test process. Keeping the real logic JsValue-free means it stays
/// natively testable; the wasm-exposed methods above are one-line
/// `.map_err(...)` wrappers around these, thin enough not to need their
/// own tests.
impl GameSession {
    fn try_play_move(&mut self, from: Option<u8>, to: u8, captures: Vec<u8>) -> Result<(), String> {
        let matching = self
            .state
            .generate_moves()
            .into_iter()
            .find(|&mv| move_from_to(mv) == (from, to) && captures_match(mv, &captures));

        match matching {
            Some(mv) => {
                self.state = self.state.make_move(mv);
                Ok(())
            }
            None => Err("illegal move".to_string()),
        }
    }

    fn try_bot_move(&mut self, difficulty: &str, seed: u64) -> Result<MoveResult, String> {
        let result = match difficulty {
            "easy" => search::search_with_randomization(
                &self.state,
                search::EASY_DEPTH,
                search::EASY_RANDOMIZATION_MARGIN,
                seed,
            ),
            "medium" => search::search(&self.state, search::MEDIUM_DEPTH),
            "hard" => search::search(&self.state, search::HARD_DEPTH),
            other => {
                return Err(format!("unknown difficulty {other:?}, expected easy/medium/hard"));
            }
        };

        self.state = self.state.make_move(result.best_move);
        let (from, to) = move_from_to(result.best_move);
        let captures = move_captures(result.best_move).iter().map(|sq| sq.0).collect();
        Ok(MoveResult { from, to, captures, score: result.score })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_starts_with_empty_board_and_white_to_move() {
        let session = GameSession::new_game();
        assert_eq!(session.board(), vec![0u8; 24]);
        assert_eq!(session.side_to_move(), 0);
        assert_eq!(session.phase(), 0);
    }

    #[test]
    fn legal_moves_from_start_has_24_placements() {
        let session = GameSession::new_game();
        let moves = session.legal_moves();
        assert_eq!(moves.len(), 24);
        assert!(moves.iter().all(|mv| mv.from().is_none()));
    }

    #[test]
    fn play_move_applies_a_legal_placement() {
        let mut session = GameSession::new_game();
        session.try_play_move(None, 0, vec![]).expect("placement should be legal");
        assert_eq!(session.board()[0], 1); // White
        assert_eq!(session.side_to_move(), 1); // Black
    }

    #[test]
    fn play_move_rejects_an_illegal_move() {
        let mut session = GameSession::new_game();
        session.try_play_move(None, 0, vec![]).unwrap();
        // Square 0 is occupied now — placing there again isn't legal.
        assert!(session.try_play_move(None, 0, vec![]).is_err());
    }

    #[test]
    fn capture_targets_and_play_move_agree_on_a_mill() {
        let mut session = GameSession::new_game();
        // White: 0, 1 (will complete mill 0,1,2). Black: 10, 11 (neither in
        // a mill, both legal capture targets).
        session.try_play_move(None, 0, vec![]).unwrap(); // White
        session.try_play_move(None, 10, vec![]).unwrap(); // Black
        session.try_play_move(None, 1, vec![]).unwrap(); // White
        session.try_play_move(None, 11, vec![]).unwrap(); // Black

        let targets = session.capture_targets(None, 2);
        assert_eq!(targets, vec![10, 11]);

        // A mill formed, so playing it with zero captures must be rejected.
        // Tried on a cloned-state throwaway session so it doesn't disturb
        // `session`'s real progression toward the success case below.
        let mut without_capture = GameSession { state: session.state.clone() };
        assert!(without_capture.try_play_move(None, 2, vec![]).is_err());

        session.try_play_move(None, 2, vec![10]).expect("capturing move should be legal");
        assert_eq!(session.board()[10], 0); // captured, now empty
        assert_eq!(session.board()[2], 1); // White completed the mill
    }

    #[test]
    fn bot_move_applies_a_move_and_reports_a_score() {
        let mut session = GameSession::new_game();
        let result = session.try_bot_move("medium", 42).expect("bot move should succeed");
        assert!(session.board().iter().any(|&sq| sq != 0));
        assert_eq!(result.from(), None);
    }

    #[test]
    fn bot_move_rejects_an_unknown_difficulty() {
        let mut session = GameSession::new_game();
        assert!(session.try_bot_move("nightmare", 1).is_err());
    }
}
