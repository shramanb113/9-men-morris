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

/// Zero, one, or two squares captured by a single move. Usually zero or
/// one, but completing two mills at once (e.g. placing on a square shared
/// by two mill lines that are both already two-thirds owned) captures one
/// piece per mill formed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Captures([Option<Square>; 2]);

impl Captures {
    pub const NONE: Captures = Captures([None, None]);

    pub fn one(sq: Square) -> Self {
        Captures([Some(sq), None])
    }

    pub fn two(a: Square, b: Square) -> Self {
        Captures([Some(a), Some(b)])
    }

    pub fn is_empty(&self) -> bool {
        self.0[0].is_none()
    }

    pub fn len(&self) -> usize {
        self.0.iter().filter(|sq| sq.is_some()).count()
    }

    pub fn contains(&self, sq: Square) -> bool {
        self.0.contains(&Some(sq))
    }

    pub fn iter(&self) -> impl Iterator<Item = Square> + '_ {
        self.0.iter().filter_map(|sq| *sq)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Move {
    Place {
        to: Square,
        captures: Captures,
    },
    Slide {
        from: Square,
        to: Square,
        captures: Captures,
    },
    Fly {
        from: Square,
        to: Square,
        captures: Captures,
    },
}

