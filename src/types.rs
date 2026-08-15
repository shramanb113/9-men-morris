#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    Placing,
    Moving,
    Flying,
}


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameResult {
    Ongoing,
    Winner(Color),
    Draw,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Square(pub u32);

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

