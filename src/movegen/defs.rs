use std::fmt::Display;

use crate::board::defs::{BitBoard, Piece, Square, PIECE_CHAR_SMALL, SQUARE_NAME};

// A list of BitBoard that represent possible attacks.
pub type AttackBoards = Vec<BitBoard>;

// A list of BitBoard that represent possible collisions/blockers.
pub type BlockerBoards = Vec<BitBoard>;

/*
Move format explanation

"data" contains all the move information, starting from LSB:

Field       :   bits     Decimal values
============================================
PIECE       :   3        0-7 (use only 0-6)
FROM        :   6        0-63
TO          :   6        0-63
CAPTURE     :   3        0-7 (captured piece)
PROMOTION   :   3        0-7 (piece promoted to)
ENPASSANT   :   1        0-1
DOUBLESTEP  :   1        0-1
CASTLING    :   1        0-1
SORTSCORE   :   32       0-65536


---------------------------------- move data -------------------------------------------
0 x32               0        0          0         000       000     000000 000000 000
SORTSCORE           CASTLING DOUBLESTEP ENPASSANT PROMOTION CAPTURE TO     FROM   PIECE
----------------------------------------------------------------------------------------

Field:      PROMOTION   CAPTURE     TO          FROM        PIECE
Bits:       3           3           6           6           3
Shift:      18 bits     15 bits     9 bits      3 bits      0 bits
& Value:    0x7 (7)     0x7 (7)     0x3F (63)   0x3F (63)   0x7 (7)

Field:      SORTSCORE   CASTLING    DOUBLESTEP  ENPASSANT
Bits:       32          1           1           1
Shift:      24 bits     23 bits     22 bits     21 bits
& Value:    0xFFFFFFFF  0x1         0x1 (1)     0x1 (1)

Get the TO field from "data" by:
    -- Shift 9 bits Right
    -- AND (&) with 0x3F

Obviously, storing information in "data" is the other way around.PIECE_NAME
Storing the "To" square: Shift LEFT 9 bits, then XOR with "data".
*/

pub struct Shift;
impl Shift {
    pub const PIECE: usize = 0;
    pub const FROM_SQ: usize = 3;
    pub const TO_SQ: usize = 9;
    pub const CAPTURE: usize = 15;
    pub const PROMOTION: usize = 18;
    pub const EN_PASSANT: usize = 21;
    pub const DOUBLE_STEP: usize = 22;
    pub const CASTLING: usize = 23;
    pub const SORTSCORE: usize = 24;
}

#[derive(Clone, Copy, PartialEq)]
pub struct Move {
    data: usize,
}

impl Move {
    pub fn new(data: usize) -> Self {
        Self { data }
    }

    /// 3 bits
    pub fn piece(&self) -> Piece {
        ((self.data >> Shift::PIECE as u64) & 0b111) as Piece
    }

    /// 6 bits
    pub fn from(&self) -> Square {
        ((self.data >> Shift::FROM_SQ as u64) & 0b111111) as Piece
    }

    /// 6 bits
    pub fn to(&self) -> Square {
        ((self.data >> Shift::TO_SQ as u64) & 0b111111) as Piece
    }

    /// 3 bits
    pub fn captured(&self) -> Piece {
        ((self.data >> Shift::CAPTURE as u64) & 0b111) as Piece
    }

    /// 3 bits
    pub fn promoted(&self) -> Piece {
        ((self.data >> Shift::PROMOTION as u64) & 0b111) as Piece
    }

    /// 1 bit
    pub fn en_passant(&self) -> Piece {
        ((self.data >> Shift::EN_PASSANT as u64) & 0b1) as Piece
    }

    /// 1 bit
    pub fn double_step(&self) -> Piece {
        ((self.data >> Shift::DOUBLE_STEP as u64) & 0b1) as Piece
    }

    /// 1 bit
    pub fn castling(&self) -> Piece {
        ((self.data >> Shift::CASTLING as u64) & 0b1) as Piece
    }

    /// 32 bits
    pub fn get_sort_score(&self) -> Piece {
        ((self.data >> Shift::SORTSCORE as u64) & 0xFFFFFFFF) as Piece
    }

    pub fn set_sort_score(&mut self, value: u32) {
        let mask: usize = 0xFFFFFFFF << Shift::SORTSCORE;
        let v: usize = (value as usize) << Shift::SORTSCORE;
        self.data = (self.data & !mask) | v;
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            PIECE_CHAR_SMALL[self.piece()],
            SQUARE_NAME[self.from()],
            SQUARE_NAME[self.to()]
        )
    }
}

// This enum holds the direction in which a ray of a slider piece can point.
#[derive(Copy, Clone)]
pub enum Direction {
    North,
    East,
    South,
    West,
    NorthWest,
    NorthEast,
    SouthEast,
    SouthWest,
}

/// A helper struct that contains methods for shifting A bitboard in a Compass Rose cardinal
/// direction.
///
/// Additionally, it also contains compass directions for Knight moves.
pub struct Compass;
impl Compass {

    pub fn northwest(bb: BitBoard) -> BitBoard {
        bb << 7
    }
    pub fn north(bb: BitBoard) -> BitBoard {
        bb << 8
    }
    pub fn northeast(bb: BitBoard) -> BitBoard {
        bb << 9
    }
    pub fn west(bb: BitBoard) -> BitBoard {
        bb >> 1
    }
    pub fn east(bb: BitBoard) -> BitBoard {
        bb << 1
    }
    pub fn southwest(bb: BitBoard) -> BitBoard {
        bb >> 9
    }
    pub fn south(bb: BitBoard) -> BitBoard {
        bb >> 8
    }
    pub fn southeast(bb: BitBoard) -> BitBoard {
        bb >> 7
    }

    // Knight Only moves

    pub fn north_north_west(bb: BitBoard) -> BitBoard {
        bb << 15
    }
    pub fn north_north_east(bb: BitBoard) -> BitBoard {
        bb << 17
    }
    pub fn north_west_west(bb: BitBoard) -> BitBoard {
        bb << 6
    }
    pub fn north_east_east(bb: BitBoard) -> BitBoard {
        bb << 10
    }
    pub fn south_west_west(bb: BitBoard) -> BitBoard {
        bb >> 10
    }
    pub fn south_east_east(bb: BitBoard) -> BitBoard {
        bb >> 6
    }
    pub fn south_south_west(bb: BitBoard) -> BitBoard {
        bb >> 17
    }
    pub fn south_south_east(bb: BitBoard) -> BitBoard {
        bb >> 15
    }
}
