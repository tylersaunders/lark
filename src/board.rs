mod fen;
mod boardstate;
pub mod defs;
mod material;

use std::fmt::Display;

use crate::{
    board::boardstate::BoardState,
    board::defs::{BitBoard, NrOf, Sides, EMPTY},
};

#[derive(Clone)]
/// Struct that holds a representation of a Chess board.
///
/// * `bb_pieces`: Bitboard lists of each piece type, for each side.
/// * `bb_side`: Bitboard of piece positions for each side.
/// * `state`: The current board state.
pub struct Board {
    pub bb_pieces: [[BitBoard; NrOf::PIECE_TYPES]; Sides::BOTH],
    pub bb_side: [BitBoard; Sides::BOTH],
    pub state: BoardState,
}

impl Board {
    /// Generates an empty [`Board`] with no pieces.
    pub fn new() -> Self {
        Self {
            bb_pieces: [[EMPTY; NrOf::PIECE_TYPES]; Sides::BOTH],
            bb_side: [EMPTY; Sides::BOTH],
            state: BoardState::new(),
        }
    }

    pub fn init(&mut self) {
        let pieces_per_side_bitboards = self.init_pieces_per_side_bitboards();

        self.bb_side[Sides::WHITE] = pieces_per_side_bitboards.0;
        self.bb_side[Sides::BLACK] = pieces_per_side_bitboards.1;


        let material = material::count_material(&self);
        self.state.material[Sides::WHITE] = material.0;
        self.state.material[Sides::BLACK] = material.1;
    }

    /// Generates two BitBoards ([`Sides::WHITE`], [`Sides::BLACK`]) that contain all of the piece
    /// locations for each side.
    fn init_pieces_per_side_bitboards(&self) -> (BitBoard, BitBoard) {
        let mut bb_white: BitBoard = 0;
        let mut bb_black: BitBoard = 0;

        for (bb_w, bb_b) in self.bb_pieces[Sides::WHITE]
            .iter()
            .zip(self.bb_pieces[Sides::BLACK].iter())
        {
            bb_white |= *bb_w;
            bb_black |= *bb_b;
        }

        (bb_white, bb_black)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bitboard: u64 = self.bb_side[Sides::WHITE] | self.bb_side[Sides::BLACK];
        const LAST_BIT: u64 = 63;

        for rank in 0..8 {
            writeln!(f)?;
            for file in (0..8).rev() {
                if file == 7 {
                    write!(f, "   ")?;
                }
                let mask = 1u64 << (LAST_BIT - (rank * 8) - file);
                let char = if bitboard & mask != 0 { '1' } else { '0' };
                write!(f, "{char:3}")?;
            }

            if rank == 7 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
