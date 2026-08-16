//! Self-play demo: `search()` plays both sides, printing the board after
//! every move so you can watch it work instead of just trusting `cargo
//! test`. Run with `cargo run --release --example play [depth]`, where
//! depth is a number or one of easy/medium/hard (defaults to medium).

use ninemensmorris::position::CurrentGameState;
use ninemensmorris::search;
use ninemensmorris::types::{Color, GameResult, Square};

/// (row, col) on a 7x7 grid for each square 0..23, matching the board
/// diagram in README.md.
const SQUARE_POS: [(usize, usize); 24] = [
    (0, 0),
    (0, 3),
    (0, 6),
    (1, 1),
    (1, 3),
    (1, 5),
    (2, 2),
    (2, 3),
    (2, 4),
    (3, 0),
    (3, 1),
    (3, 2),
    (3, 4),
    (3, 5),
    (3, 6),
    (4, 2),
    (4, 3),
    (4, 4),
    (5, 1),
    (5, 3),
    (5, 5),
    (6, 0),
    (6, 3),
    (6, 6),
];

fn print_board(state: &CurrentGameState) {
    let mut grid = [[' '; 7]; 7];
    for sq in 0..24u8 {
        let (row, col) = SQUARE_POS[sq as usize];
        grid[row][col] = match state.owner(Square(sq)) {
            Some(Color::White) => 'W',
            Some(Color::Black) => 'B',
            None => '.',
        };
    }
    println!("   A  B  C  D  E  F  G");
    for (i, row) in grid.iter().enumerate() {
        print!(" {} ", i + 1);
        for c in row {
            print!("{c}  ");
        }
        println!();
    }
}

#[derive(Clone, Copy)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
    Custom(u8),
}

impl Difficulty {
    fn depth(self) -> u8 {
        match self {
            Difficulty::Easy => search::EASY_DEPTH,
            Difficulty::Medium => search::MEDIUM_DEPTH,
            Difficulty::Hard => search::HARD_DEPTH,
            Difficulty::Custom(depth) => depth,
        }
    }

    fn label(self) -> String {
        match self {
            Difficulty::Easy => "easy".to_string(),
            Difficulty::Medium => "medium".to_string(),
            Difficulty::Hard => "hard".to_string(),
            Difficulty::Custom(depth) => format!("depth {depth}"),
        }
    }

    /// Picks a move for `state`, honoring what each tier is supposed to
    /// feel like: easy plays weaker on purpose via
    /// `search_with_randomization`, not just shallower — quiescence keeps
    /// even a shallow search tactically sound, so depth alone wouldn't
    /// actually feel "easy". Medium/Hard/Custom just want the best move.
    fn pick(self, state: &CurrentGameState, seed: u64) -> search::SearchResult {
        match self {
            Difficulty::Easy => search::search_with_randomization(
                state,
                self.depth(),
                search::EASY_RANDOMIZATION_MARGIN,
                seed,
            ),
            Difficulty::Medium | Difficulty::Hard | Difficulty::Custom(_) => {
                search::search(state, self.depth())
            }
        }
    }
}

/// Accepts a raw depth number, or "easy"/"medium"/"hard".
fn parse_difficulty(arg: Option<String>) -> Difficulty {
    match arg.as_deref() {
        None => Difficulty::Medium,
        Some("easy") => Difficulty::Easy,
        Some("medium") => Difficulty::Medium,
        Some("hard") => Difficulty::Hard,
        Some(s) => Difficulty::Custom(
            s.parse().unwrap_or_else(|_| panic!("expected a depth number or easy/medium/hard, got {s:?}")),
        ),
    }
}

/// A different value each call, mixed with `ply` so consecutive calls
/// within the same clock tick still diverge. Only used to vary easy mode's
/// move choice across a game — not meant to be cryptographically anything,
/// and this kind of wall-clock seeding is exactly the platform-specific
/// randomness the core itself deliberately never generates on its own.
fn seed_for(ply: u32) -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos() as u64;
    nanos ^ (ply as u64)
}

fn main() {
    let difficulty = parse_difficulty(std::env::args().nth(1));

    let mut state = CurrentGameState::new();
    let mut ply = 0u32;

    println!("Nine Men's Morris — self-play at {}\n", difficulty.label());
    print_board(&state);

    while !state.is_game_over() {
        let side = state.side_to_move();
        let started = std::time::Instant::now();
        let result = difficulty.pick(&state, seed_for(ply));
        let elapsed = started.elapsed();

        ply += 1;
        println!(
            "\nply {ply}: {side:?} plays {:?}  (score {}, {elapsed:.2?})",
            result.best_move, result.score
        );
        state = state.make_move(result.best_move);
        print_board(&state);
    }

    match state.result() {
        GameResult::Winner(color) => println!("\nGame over: {color:?} wins."),
        GameResult::Draw => println!("\nGame over: draw (no capture within the ply limit)."),
        GameResult::Ongoing => unreachable!("loop only exits once is_game_over() is true"),
    }
}
