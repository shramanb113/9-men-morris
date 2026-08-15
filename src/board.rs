use crate::types::Square;

pub type BitBoard = u32;   // 24 bits are used
pub const BOARD_MASK: BitBoard = 0x00FF_FFFF;

    //   A  B  C  D  E  F  G
    // 1 0        1        2
    // 2    3     4     5
    // 3       6  7  8
    // 4 9 10 11    12 13 14
    // 5      15 16 17
    // 6   18    19    20
    // 7 21       22       23

    // positions of the squares


// board connections  to its different pieces  to be represented here 
/// MOVES[sq] = bitboard of all squares connected to `sq`
pub static MOVES: [BitBoard; 24] = [
    // 0  A1
    (1 << 1) | (1 << 9),
    // 1  D1
    (1 << 0) | (1 << 2) | (1 << 4),
    // 2  G1
    (1 << 1) | (1 << 14),
    // 3  B2
    (1 << 4) | (1 << 10),
    // 4  D2
    (1 << 1) | (1 << 3) | (1 << 5) | (1 << 7),
    // 5  F2
    (1 << 4) | (1 << 13),
    // 6  C3
    (1 << 7) | (1 << 11),
    // 7  D3
    (1 << 4) | (1 << 6) | (1 << 8),
    // 8  E3
    (1 << 7) | (1 << 12),
    // 9  A4
    (1 << 0) | (1 << 10) | (1 << 21),
    // 10 B4
    (1 << 3) | (1 << 9) | (1 << 11) | (1 << 18),
    // 11 C4
    (1 << 6) | (1 << 10) | (1 << 15),
    // 12 E4
    (1 << 8) | (1 << 13) | (1 << 17),
    // 13 F4
    (1 << 5) | (1 << 12) | (1 << 14) | (1 << 20),
    // 14 G4
    (1 << 2) | (1 << 13) | (1 << 23),
    // 15 C5
    (1 << 11) | (1 << 16),
    // 16 D5
    (1 << 15) | (1 << 17) | (1 << 19),
    // 17 E5
    (1 << 12) | (1 << 16),
    // 18 B6
    (1 << 10) | (1 << 19),
    // 19 D6
    (1 << 16) | (1 << 18) | (1 << 20) | (1 << 22),
    // 20 F6
    (1 << 13) | (1 << 19),
    // 21 A7
    (1 << 9) | (1 << 22),
    // 22 D7
    (1 << 19) | (1 << 21) | (1 << 23),
    // 23 G7
    (1 << 14) | (1 << 22),
];

/// All 16 possible mills as bitboards
pub static MILLS: [BitBoard; 16] = [
    // Horizontal
    (1 << 0) | (1 << 1) | (1 << 2),       // A1-D1-G1
    (1 << 3) | (1 << 4) | (1 << 5),       // B2-D2-F2
    (1 << 6) | (1 << 7) | (1 << 8),       // C3-D3-E3
    (1 << 9) | (1 << 10) | (1 << 11),     // A4-B4-C4
    (1 << 12) | (1 << 13) | (1 << 14),    // E4-F4-G4
    (1 << 15) | (1 << 16) | (1 << 17),    // C5-D5-E5
    (1 << 18) | (1 << 19) | (1 << 20),    // B6-D6-F6
    (1 << 21) | (1 << 22) | (1 << 23),    // A7-D7-G7

    // Vertical
    (1 << 0) | (1 << 9) | (1 << 21),      // A1-A4-A7
    (1 << 3) | (1 << 10) | (1 << 18),     // B2-B4-B6
    (1 << 6) | (1 << 11) | (1 << 15),     // C3-C4-C5
    (1 << 1) | (1 << 4) | (1 << 7),       // D1-D2-D3
    (1 << 16) | (1 << 19) | (1 << 22),    // D5-D6-D7
    (1 << 8) | (1 << 12) | (1 << 17),     // E3-E4-E5
    (1 << 5) | (1 << 13) | (1 << 20),     // F2-F4-F6
    (1 << 2) | (1 << 14) | (1 << 23),     // G1-G4-G7
];

pub const SQUARE_MILLS: [[BitBoard; 2]; 24] = [
    [MILLS[0], MILLS[8]],   // 0  A1
    [MILLS[0], MILLS[11]],  // 1  D1
    [MILLS[0], MILLS[15]],  // 2  G1
    [MILLS[1], MILLS[9]],   // 3  B2
    [MILLS[1], MILLS[11]],  // 4  D2
    [MILLS[1], MILLS[14]],  // 5  F2
    [MILLS[2], MILLS[10]],  // 6  C3
    [MILLS[2], MILLS[11]],  // 7  D3
    [MILLS[2], MILLS[13]],  // 8  E3
    [MILLS[3], MILLS[8]],   // 9  A4
    [MILLS[3], MILLS[9]],   // 10 B4
    [MILLS[3], MILLS[10]],  // 11 C4
    [MILLS[4], MILLS[13]],  // 12 E4
    [MILLS[4], MILLS[14]],  // 13 F4
    [MILLS[4], MILLS[15]],  // 14 G4
    [MILLS[5], MILLS[10]],  // 15 C5
    [MILLS[5], MILLS[12]],  // 16 D5
    [MILLS[5], MILLS[13]],  // 17 E5
    [MILLS[6], MILLS[9]],   // 18 B6
    [MILLS[6], MILLS[12]],  // 19 D6
    [MILLS[6], MILLS[14]],  // 20 F6
    [MILLS[7], MILLS[8]],   // 21 A7
    [MILLS[7], MILLS[12]],  // 22 D7
    [MILLS[7], MILLS[15]],  // 23 G7
];

// instead of writing bitwise opearation everywhere we write helper function or it 

// changes every square numebr to bit
#[inline]
pub fn bit(sq: Square) -> BitBoard{
    1 << sq.0
}

// checks if it is a valid sq in bitboard or not
#[inline]
pub fn is_bb(bb: BitBoard, sq: Square) -> bool {
    (bb & bit(sq)) != 0
}

// stes the bit, makes the particular position occupied
#[inline]
pub fn set(bb: BitBoard, sq: Square) -> BitBoard {
    bb | bit(sq)
}

// clears that position in the bit board 
#[inline]
pub fn clear(bb: BitBoard, sq: Square) -> BitBoard {
    bb & !bit(sq)
}

// counts the number of occupied position count_ones counts the number of ones in bit format
#[inline]
pub fn popcount(bb: BitBoard) -> u32 {
    bb.count_ones()
}