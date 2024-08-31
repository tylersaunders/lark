mod boardstate;
pub mod defs;
mod fen;
mod material;

use std::{error::Error, fmt::Display};

use defs::{Piece, Pieces, Side, Square, Squares, BB_SQUARES, PIECE_CHAR_CAPS, PIECE_CHAR_SMALL, PIECE_VALUES, SQUARE_NAME};

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

    /// The side to play.
    pub fn current_side(&self) -> usize {
        self.state.active_side as usize
    }

    /// The side opposite the side to play.
    pub fn opponent(&self) -> usize {
        (self.state.active_side ^ 1) as usize
    }

    /// Get the piece and owner on the given square.
    ///
    /// Will always return [`Pieces::NONE`] when no piece is on the square.
    /// Otherwise, will return the Piece type and side that owns the piece.
    ///
    /// * `square`: The square to check for a piece.
    pub fn get_piece_on_square(&self, square: Square) -> Result<(Piece, Side), Piece> {
        let bb_square = BB_SQUARES[square];
        let is_square_occupied_white = self.bb_side[Sides::WHITE] & bb_square > 0;
        let is_square_occupied_black = self.bb_side[Sides::BLACK] & bb_square > 0;

        if is_square_occupied_white {
            for (piece, bb_piece) in self.bb_pieces[Sides::WHITE].iter().enumerate() {
                let exists = bb_square & bb_piece > 0;
                match exists {
                    true => return Ok((piece, Sides::WHITE)),
                    false => continue,
                }
            }
        }

        if is_square_occupied_black {
            for (piece, bb_piece) in self.bb_pieces[Sides::BLACK].iter().enumerate() {
                let exists = bb_square & bb_piece > 0;
                match exists {
                    true => return Ok((piece, Sides::BLACK)),
                    false => continue,
                }
            }
        }

        Err(Pieces::NONE)
    }

    /// Moves a piece from one [`Square`] to another.
    ///
    /// WARNING: This function will panic if the piece does not exist.
    ///
    /// * `side`: The side that owns the piece.
    /// * `piece`: The piece to move
    /// * `from`: Or, starting square.
    /// * `to`: Or, destination square.
    pub fn move_piece(&mut self, side: Side, piece: Piece, from: Square, to: Square) {
        let piece_exists = self.bb_pieces[side][piece] & BB_SQUARES[from] > 0;
        if !piece_exists {
            panic!(
                "Cannot move a piece that does not exist: {}|{}",
                PIECE_CHAR_CAPS[piece], SQUARE_NAME[from]
            );
        }
        println!("{piece_exists}");
        self.remove_piece(side, piece, from);
        self.put_piece(side, piece, to);
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
        self.state.material[side] -= PIECE_VALUES[piece];
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
        writeln!(f, "==========================")?;
        writeln!(f, "          l  a  r  k      ")?;
        for rank in (0..8).rev() {
            writeln!(f)?;

            // Write the Rank number
            write!(f, "{} |", rank + 1)?;

            for file in 0..8 {
                if file == 0 {
                    write!(f, " ")?;
                }
                let char = match self.get_piece_on_square((rank * 8 + file) as usize) {
                    Err(e) => "-",
                    Ok((piece, side)) => {
                        match side {
                            Sides::WHITE => PIECE_CHAR_CAPS[piece],
                            Sides::BLACK => PIECE_CHAR_SMALL[piece],
                            _ => panic!("invalid side")
                        }
                    }
                };
                write!(f, "{char:3}")?;
            }

            if rank == 0 {
                writeln!(f)?;
            }
        }
        write!(f, "  --------------------------")?;
        writeln!(f)?;
        write!(f, "    A  B  C  D  E  F  G  H  ")?;
        writeln!(f)?;

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
    fn test_board_current_side() {
        let mut board = Board::new();
        board.state.active_side = Sides::WHITE as u8;
        assert_eq!(board.current_side(), Sides::WHITE);
        board.state.active_side = Sides::BLACK as u8;
        assert_eq!(board.current_side(), Sides::BLACK);
    }

    #[test]
    fn test_board_opponent() {
        let mut board = Board::new();
        board.state.active_side = Sides::WHITE as u8;
        assert_eq!(board.opponent(), Sides::BLACK);
        board.state.active_side = Sides::BLACK as u8;
        assert_eq!(board.opponent(), Sides::WHITE);
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
        board.bb_pieces[Sides::WHITE][Pieces::KNIGHT] |=
            Squares::bb_of(&[Squares::B1, Squares::G1]);
        board.bb_pieces[Sides::BLACK][Pieces::KNIGHT] |=
            Squares::bb_of(&[Squares::B8, Squares::G8]);

        // Bishops
        board.bb_pieces[Sides::WHITE][Pieces::BISHOP] |=
            Squares::bb_of(&[Squares::C1, Squares::F1]);
        board.bb_pieces[Sides::BLACK][Pieces::BISHOP] |=
            Squares::bb_of(&[Squares::C8, Squares::F8]);

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
        board.put_piece(Sides::WHITE, Pieces::QUEEN, Squares::F1);
        assert!(board.bb_pieces[Sides::WHITE][Pieces::QUEEN] & BB_SQUARES[Squares::F1] > 0);

        assert_eq!(
            board.state.material[Sides::WHITE],
            PIECE_VALUES[Pieces::QUEEN]
        );

        // Add a piece for black.
        assert!(board.bb_pieces[Sides::BLACK][Pieces::ROOK] & BB_SQUARES[Squares::H1] == 0);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::H1);
        assert!(board.bb_pieces[Sides::BLACK][Pieces::ROOK] & BB_SQUARES[Squares::H1] > 0);

        assert_eq!(
            board.state.material[Sides::BLACK],
            PIECE_VALUES[Pieces::ROOK]
        );
    }

    #[test]
    #[should_panic]
    fn test_board_put_piece_invalid_side() {
        let mut board = Board::new();
        board.put_piece(Sides::BOTH, Pieces::QUEEN, Squares::F1);
    }

    #[test]
    fn test_board_remove_piece() {
        let mut board = Board::new();

        // Add a piece for white.
        board.bb_pieces[Sides::WHITE][Pieces::QUEEN] |= BB_SQUARES[Squares::F1];

        // Add a piece for black.
        board.bb_pieces[Sides::BLACK][Pieces::ROOK] |= BB_SQUARES[Squares::H1];

        board.init();

        assert_eq!(
            board.state.material[Sides::WHITE],
            PIECE_VALUES[Pieces::QUEEN]
        );
        assert_eq!(
            board.state.material[Sides::BLACK],
            PIECE_VALUES[Pieces::ROOK]
        );

        board.remove_piece(Sides::WHITE, Pieces::QUEEN, Squares::F1);
        board.remove_piece(Sides::BLACK, Pieces::ROOK, Squares::H1);

        assert_eq!(board.state.material[Sides::WHITE], 0);
        assert_eq!(board.state.material[Sides::BLACK], 0);
        assert_eq!(board.bb_side[Sides::WHITE], 0);
        assert_eq!(board.bb_side[Sides::BLACK], 0);
    }

    #[test]
    fn test_board_move_piece() {
        let mut board = Board::new();

        board.put_piece(Sides::WHITE, Pieces::PAWN, Squares::D2);

        // Play the move D4
        board.move_piece(Sides::WHITE, Pieces::PAWN, Squares::D2, Squares::D4);

        assert!(board.bb_pieces[Sides::WHITE][Pieces::PAWN] & BB_SQUARES[Squares::D4] > 0);
        assert!(board.bb_pieces[Sides::WHITE][Pieces::PAWN] & BB_SQUARES[Squares::D2] == 0);
    }

    #[test]
    #[should_panic]
    fn test_board_move_piece_that_does_not_exist() {
        let mut board = Board::new();

        // Play the move D4
        board.move_piece(Sides::WHITE, Pieces::PAWN, Squares::D2, Squares::D4);
    }

    #[test]
    #[should_panic]
    fn test_board_remove_piece_invalid_side() {
        let mut board = Board::new();
        board.remove_piece(Sides::BOTH, Pieces::QUEEN, Squares::F1);
    }
}
