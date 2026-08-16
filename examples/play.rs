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

/// Accepts a raw depth number, or "easy"/"medium"/"hard" mapped to
/// `search::EASY_DEPTH`/`MEDIUM_DEPTH`/`HARD_DEPTH`.
fn parse_depth(arg: Option<String>) -> u8 {
    match arg.as_deref() {
        None => search::MEDIUM_DEPTH,
        Some("easy") => search::EASY_DEPTH,
        Some("medium") => search::MEDIUM_DEPTH,
        Some("hard") => search::HARD_DEPTH,
        Some(s) => s.parse().unwrap_or_else(|_| panic!("expected a depth number or easy/medium/hard, got {s:?}")),
    }
}

fn main() {
    let depth = parse_depth(std::env::args().nth(1));

    let mut state = CurrentGameState::new();
    let mut ply = 0u32;

    println!("Nine Men's Morris — self-play at depth {depth}\n");
    print_board(&state);

    while !state.is_game_over() {
        let side = state.side_to_move();
        let started = std::time::Instant::now();
        let result = search::search(&state, depth);
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
