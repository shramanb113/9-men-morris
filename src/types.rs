#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Color {
    #[default]
    White,
    Black,
}

impl Color {
    pub fn opponent(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    Placing,
    Sliding,
    Flying,
}


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameResult {
    #[default]
    Ongoing,
    Winner(Color),
    Draw,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Square(pub u8);


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Move {
    Place {
        to: Square,
        capture: Option<Square>,
    },
    Slide {
        from: Square,
        to: Square,
        capture: Option<Square>,
    },
    Fly {
        from: Square,
        to: Square,
        capture: Option<Square>,
    },
}

