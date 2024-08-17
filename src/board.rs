mod boardstate;
pub mod defs;
mod fen;
mod material;

use std::fmt::Display;

use defs::{Side, Square, BB_SQUARES, PIECE_VALUES};

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

    /// Place a piece on the board.
    ///
    /// This will update the material count in [`BoardState`].
    ///
    /// * `side`: The [`Sides`] that owns the piece, must be oneof WHITE or BLACK.
    /// * `piece`: The [`Pieces`] type to place.
    /// * `square`: The [`Squares`] to place the piece on.
    pub fn put_piece(&mut self, side: Side, piece: usize, square: Square) {
        self.bb_pieces[side][piece] |= BB_SQUARES[square];
        self.bb_side[side] |= BB_SQUARES[square];
        self.state.material[side] += PIECE_VALUES[piece];
    }

    /// Remove a piece from the board.
    ///
    /// This will update the material count in [`BoardState`].
    ///
    /// * `side`: The [`Sides`] that owns the piece, must be oneof WHITE or BLACK.
    /// * `piece`: The [`Pieces`] type to remove.
    /// * `square`: The [`Squares`] to remove the piece from.
    pub fn remove_piece(&mut self, side: Side, piece: usize, square: Square) {
        self.bb_pieces[side][piece] ^= BB_SQUARES[square];
        self.bb_side[side] ^= BB_SQUARES[square];
        self.state.material[side] -= PIECE_VALUES[piece]
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

#[cfg(test)]
mod tests {

    use defs::{Pieces, Ranks, Squares, BB_RANKS};

    use super::*;

    #[test]
    fn test_board_new_makes_empty_board() {
        let board = Board::new();
        assert_eq!(board.bb_side[Sides::WHITE], 0);
        assert_eq!(board.bb_side[Sides::BLACK], 0);

        let bb_w = board.bb_pieces[Sides::WHITE];
        let bb_b = board.bb_pieces[Sides::BLACK];

        for (w, b) in bb_w.iter().zip(bb_b.iter()) {
            assert_eq!(*w, 0);
            assert_eq!(*b, 0);
        }
    }

    #[test]
    fn test_board_init() {
        // Create a new board and manually set up the normal starting position.
        let mut board = Board::new();

        // Pawns
        board.bb_pieces[Sides::WHITE][Pieces::PAWN] |= BB_RANKS[Ranks::R2];
        board.bb_pieces[Sides::BLACK][Pieces::PAWN] |= BB_RANKS[Ranks::R7];

        // Rooks
        board.bb_pieces[Sides::WHITE][Pieces::ROOK] |= Squares::bb_of(&[Squares::A1, Squares::H1]);
        board.bb_pieces[Sides::BLACK][Pieces::ROOK] |= Squares::bb_of(&[Squares::A8, Squares::H8]);

        // Knights
        board.bb_pieces[Sides::WHITE][Pieces::KNIGHT] |= Squares::bb_of(&[Squares::B1, Squares::G1]);
        board.bb_pieces[Sides::BLACK][Pieces::KNIGHT] |= Squares::bb_of(&[Squares::B8, Squares::G8]);

        // Bishops
        board.bb_pieces[Sides::WHITE][Pieces::BISHOP] |= Squares::bb_of(&[Squares::C1, Squares::F1]);
        board.bb_pieces[Sides::BLACK][Pieces::BISHOP] |= Squares::bb_of(&[Squares::C8, Squares::F8]);

        // Queens
        board.bb_pieces[Sides::WHITE][Pieces::QUEEN] |= BB_SQUARES[Squares::D1];
        board.bb_pieces[Sides::BLACK][Pieces::QUEEN] |= BB_SQUARES[Squares::D8];

        // Kings
        board.bb_pieces[Sides::WHITE][Pieces::KING] |= BB_SQUARES[Squares::E1];
        board.bb_pieces[Sides::BLACK][Pieces::KING] |= BB_SQUARES[Squares::E8];

        board.init();

        // Total material for the standard starting position is 3900
        assert_eq!(board.state.material[Sides::WHITE], 3900);
        assert_eq!(board.state.material[Sides::BLACK], 3900);

        // Ranks 1-2 and 7-8 should be completely full of pieces after init()
        assert_eq!(
            board.bb_side[Sides::WHITE] & BB_RANKS[Ranks::R1],
            BB_RANKS[Ranks::R1]
        );
        assert_eq!(
            board.bb_side[Sides::WHITE] & BB_RANKS[Ranks::R2],
            BB_RANKS[Ranks::R2]
        );
        assert_eq!(
            board.bb_side[Sides::BLACK] & BB_RANKS[Ranks::R7],
            BB_RANKS[Ranks::R7]
        );
        assert_eq!(
            board.bb_side[Sides::BLACK] & BB_RANKS[Ranks::R8],
            BB_RANKS[Ranks::R8]
        );
    }

    #[test]
    fn test_board_put_piece() {
        let mut board = Board::new();

        // Add a piece for white.
        assert!(board.bb_pieces[Sides::WHITE][Pieces::QUEEN] & BB_SQUARES[Squares::F1] == 0);
        board.put_piece(Sides::WHITE, Pieces::QUEEN,Squares::F1);
        assert!(board.bb_pieces[Sides::WHITE][Pieces::QUEEN] & BB_SQUARES[Squares::F1] > 0);

        assert_eq!(board.state.material[Sides::WHITE], PIECE_VALUES[Pieces::QUEEN]);

        // Add a piece for black.
        assert!(board.bb_pieces[Sides::BLACK][Pieces::ROOK] & BB_SQUARES[Squares::H1] == 0);
        board.put_piece(Sides::BLACK, Pieces::ROOK,Squares::H1);
        assert!(board.bb_pieces[Sides::BLACK][Pieces::ROOK] & BB_SQUARES[Squares::H1] > 0);

        assert_eq!(board.state.material[Sides::BLACK], PIECE_VALUES[Pieces::ROOK]);
    }

    #[test]
    #[should_panic]
    fn test_board_put_piece_invalid_side(){
        let mut board = Board::new();
        board.put_piece(Sides::BOTH, Pieces::QUEEN,Squares::F1);
    }

    #[test]
    fn test_board_remove_piece() {
        let mut board = Board::new();

        // Add a piece for white.
        board.bb_pieces[Sides::WHITE][Pieces::QUEEN] |= BB_SQUARES[Squares::F1];

        // Add a piece for black.
        board.bb_pieces[Sides::BLACK][Pieces::ROOK] |= BB_SQUARES[Squares::H1];

        board.init();

        assert_eq!(board.state.material[Sides::WHITE], PIECE_VALUES[Pieces::QUEEN]);
        assert_eq!(board.state.material[Sides::BLACK], PIECE_VALUES[Pieces::ROOK]);

        board.remove_piece(Sides::WHITE, Pieces::QUEEN, Squares::F1);
        board.remove_piece(Sides::BLACK, Pieces::ROOK, Squares::H1);

        assert_eq!(board.state.material[Sides::WHITE], 0);
        assert_eq!(board.state.material[Sides::BLACK], 0);
        assert_eq!(board.bb_side[Sides::WHITE], 0);
        assert_eq!(board.bb_side[Sides::BLACK], 0);

    }

    #[test]
    #[should_panic]
    fn test_board_remove_piece_invalid_side(){
        let mut board = Board::new();
        board.remove_piece(Sides::BOTH, Pieces::QUEEN,Squares::F1);
    }
}
