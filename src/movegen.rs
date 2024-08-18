use std::vec;
mod init;

use defs::{Move, Shift};

use crate::{
    board::{
        defs::{BitBoard, NrOf, Piece, Pieces, Ranks, Sides, Square, BB_RANKS, BB_SQUARES, EMPTY},
        Board,
    },
    utils::bits,
};

pub mod defs;

// Set of pieces that a PAWN can promote to, used for generating a corresponding promotion move.
const PROMOTION_PIECES: [usize; 4] = [Pieces::QUEEN, Pieces::ROOK, Pieces::BISHOP, Pieces::KNIGHT];

/// A generate that holds attack tables for each piece, and contains methods for creating and
/// generating possible pseudo-legal moves.
///
/// * `king`: The king's attack table.
/// * `knight`: The knight's attack table.
/// * `pawns`: The pawn's attack table.
/// * `rook`: TODO: Magic numbers table.
/// * `bishop`: TODO Magic numbers table.
pub struct MoveGenerator {
    king: [BitBoard; NrOf::SQUARES],
    knight: [BitBoard; NrOf::SQUARES],
    pawns: [[BitBoard; NrOf::SQUARES]; Sides::BOTH],
    rook: Vec<BitBoard>,
    bishop: Vec<BitBoard>,
}

impl MoveGenerator {
    /// Create and initialize a new [`MoveGenerator`].
    ///
    /// This will initialize and construct move data for all piece types.
    pub fn new() -> Self {
        let mut mg = Self {
            king: [EMPTY; NrOf::SQUARES],
            knight: [EMPTY; NrOf::SQUARES],
            pawns: [[EMPTY; NrOf::SQUARES]; Sides::BOTH],
            // TODO: replace with rook table squares
            rook: vec![EMPTY; NrOf::SQUARES],
            // TODO: replace with bishop table squares
            bishop: vec![EMPTY; NrOf::SQUARES],
        };
        mg.init_king();
        mg.init_knight();
        mg.init_pawns();
        mg
    }

    /// Generates moves for the side that is to move.
    ///
    /// * `board`: The current board to generate moves for
    /// * `move_list`: A list where the generated moves will be appended.
    pub fn generate_moves(&self, board: &Board, move_list: &mut Vec<Move>) {
        self.piece(board, Pieces::KING, move_list);
        self.piece(board, Pieces::KNIGHT, move_list);
        self.pawns(board, move_list);
    }

    /// Generate all pseudo-legal moves for the particular piece type. This generates
    /// all moves by all pieces matching this piece type on the board.
    ///
    /// NOTE: Not all moves are actually legal; they do not consider things such as pins.
    ///
    /// * `board`: The current board
    /// * `piece`: the [`Pieces`] to generate moves for.
    /// * `list`: the move list to append all pseudo-legal moves.
    pub fn piece(&self, board: &Board, piece: Piece, list: &mut Vec<Move>) {
        let player = board.current_side();
        let opponent = board.opponent();
        let bb_occupied = board.bb_side[Sides::WHITE] | board.bb_side[Sides::BLACK];
        let bb_empty = !bb_occupied;

        let bb_own_pieces = board.bb_side[player];
        let bb_opponent_pieces = board.bb_side[opponent];

        let mut bb_pieces = board.bb_pieces[player][piece];

        while bb_pieces > 0 {
            let from = bits::next(&mut bb_pieces);
            let bb_target = match piece {
                Pieces::KING => self.get_non_slider_attacks(piece, from),
                Pieces::KNIGHT => self.get_non_slider_attacks(piece, from),
                _ => panic!("Not a piece: {piece}"),
            };

            let bb_moves = bb_target & !bb_own_pieces;
            self.add_moves(board, piece, from, bb_moves, list);
        }
    }

    /// Generates all pseudo-legal pawn moves.
    ///
    /// This does consider possible en-passant captures.
    ///
    /// NOTE: Not all moves are actually legal; they do not consider things such as pins.
    ///
    /// * `board`: The current board
    /// * `list`: the move list to append all pseudo-legal pawn moves.
    pub fn pawns(&self, board: &Board, list: &mut Vec<Move>) {
        const NORTH: i8 = 8;
        const SOUTH: i8 = -8;

        let player = board.current_side();
        let bb_opponent_pieces = board.bb_side[board.opponent()];
        let bb_empty = !(board.bb_side[Sides::WHITE] | board.bb_side[Sides::BLACK]);

        let bb_fourth = match player {
            Sides::WHITE => BB_RANKS[Ranks::R4],
            Sides::BLACK => BB_RANKS[Ranks::R5],
            _ => panic!("Unexpected side"),
        };

        let direction = match player {
            Sides::WHITE => NORTH,
            Sides::BLACK => SOUTH,
            _ => panic!("Unexpected side"),
        };

        let rotation_count = (NrOf::SQUARES as i8 + direction) as u32;
        let mut bb_pawns = board.bb_pieces[player][Pieces::PAWN];

        while bb_pawns > 0 {
            let from = bits::next(&mut bb_pawns);
            let to = (from as i8 + direction) as usize;
            let mut bb_moves = 0;

            // Generate pawn pushes
            let bb_push = BB_SQUARES[to];
            let bb_one_step = bb_push & bb_empty;
            let bb_two_step = bb_one_step.rotate_left(rotation_count) & bb_empty & bb_fourth;
            bb_moves |= bb_one_step | bb_two_step;

            // Generate pawn captures
            let bb_targets = self.pawns[player][from];
            let bb_captures = bb_targets & bb_opponent_pieces;
            let bb_ep_capture = match board.state.en_passant {
                Some(ep) => bb_targets & BB_SQUARES[ep as usize],
                None => 0,
            };

            bb_moves |= bb_captures | bb_ep_capture;

            self.add_moves(board, Pieces::PAWN, from, bb_moves, list);
        }
    }

    /// Creates and adds new [`Move`]s to the provided move list.
    ///
    /// This will iterate the Bitboard provided in `to` and create new moves for each target square
    /// in the Bitboard.
    ///
    /// * `board`: The current board
    /// * `piece`: The piece this move is for.
    /// * `from`: The starting square
    /// * `to`: A [`BitBoard`] of all the possible destination squares.
    /// * `move_list`: The move list to append this move to.
    pub fn add_moves(
        &self,
        board: &Board,
        piece: Piece,
        from: Square,
        to: BitBoard,
        move_list: &mut Vec<Move>,
    ) {
        let mut bb_to = to;

        let is_pawn = piece == Pieces::PAWN;

        while bb_to > 0 {
            let to_square = bits::next(&mut bb_to);
            let capture = 0;
            let en_passant = match board.state.en_passant {
                Some(square) => is_pawn && (square as usize == to_square),
                None => false,
            };
            let promotion = false;
            let double_step = false;
            let castling = false;

            let move_data = (piece)
                | from << Shift::FROM_SQ
                | to_square << Shift::TO_SQ
                | capture << Shift::CAPTURE
                | (en_passant as usize) << Shift::EN_PASSANT
                | (double_step as usize) << Shift::DOUBLE_STEP
                | (castling as usize) << Shift::CASTLING;

            if !promotion {
                move_list.push(Move::new(move_data));
            } else {
                PROMOTION_PIECES.iter().for_each(|piece| {
                    let promotion_piece = *piece << Shift::PROMOTION;
                    move_list.push(Move::new(move_data | promotion_piece));
                })
            }
        }
    }

    /// Get the attacks table for the non-slider piece.
    ///
    /// * `piece`: must be a KING or KNIGHT, or this function will panic.
    /// * `square`: The square the piece is currently attacking from.
    fn get_non_slider_attacks(&self, piece: Piece, square: Square) -> BitBoard {
        match piece {
            Pieces::KING => self.king[square],
            Pieces::KNIGHT => self.knight[square],
            _ => panic!("Not a king or a knight: {piece}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{
        defs::{Pieces, Sides, Squares},
        Board,
    };

    use super::{defs::Move, MoveGenerator};

    /// Parameterize a set of test cases for a particular side
    ///
    /// * `label`: test case name
    /// * `eval`:  test case parameterized function
    /// * `side`:  test case side to pass as parameter.
    macro_rules! test_cases_by_side {
    ( $($label:ident : $eval:ident, $side:expr);* $(;)? ) => {

        $(
            #[test]
            fn $label() {
                $eval($side)
            }
        )*

        }
    }

    // Generate test cases for each side for pieces that have the same move structures.
    test_cases_by_side! {
        king_moves_white: generate_king_moves, Sides::WHITE;
        king_moves_edge_of_board_white: generate_king_moves_edge_of_board, Sides::WHITE;
        knight_moves_white: generate_knight_moves, Sides::WHITE;
        knight_moves_edge_of_board_white: generate_knight_moves_edge_of_board, Sides::WHITE;

        king_moves_black: generate_king_moves, Sides::BLACK;
        king_moves_edge_of_board_black: generate_king_moves_edge_of_board, Sides::BLACK;
        knight_moves_black: generate_knight_moves, Sides::BLACK;
        knight_moves_edge_of_board_black: generate_knight_moves_edge_of_board, Sides::BLACK;
    }

    // Need to manually test pawns for each side since they move in different directions and
    // capture in different directions.

    #[test]
    fn test_generate_pawn_moves_white() {
        let mut board = Board::new();
        board.put_piece(Sides::WHITE, Pieces::PAWN, Squares::D2);
        board.put_piece(Sides::WHITE, Pieces::PAWN, Squares::E4);

        board.put_piece(Sides::BLACK, Pieces::PAWN, Squares::D5);

        board.state.active_side = Sides::WHITE as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([Squares::D3, Squares::D4, Squares::E5, Squares::D5]);

        // There should be 8 total moves
        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert!(Vec::from([Squares::D2, Squares::E4]).contains(&mv.from()));
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }

    #[test]
    fn test_generate_pawn_moves_black() {
        let mut board = Board::new();
        board.put_piece(Sides::BLACK, Pieces::PAWN, Squares::E7);
        board.put_piece(Sides::BLACK, Pieces::PAWN, Squares::D5);

        board.put_piece(Sides::WHITE, Pieces::PAWN, Squares::E4);

        board.state.active_side = Sides::BLACK as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([Squares::E6, Squares::E5, Squares::D4, Squares::E4]);

        // There should be 8 total moves
        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert!(Vec::from([Squares::E7, Squares::D5]).contains(&mv.from()));
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }

    fn generate_king_moves(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::KING, Squares::D4);
        board.state.active_side = side as u8;
        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            Squares::C3,
            Squares::C4,
            Squares::C5,
            Squares::D3,
            Squares::D5,
            Squares::E3,
            Squares::E4,
            Squares::E5,
        ]);

        // There should be 8 total moves
        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert_eq!(mv.from(), Squares::D4);
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }

    fn generate_king_moves_edge_of_board(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::KING, Squares::A1);
        board.state.active_side = side as u8;
        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([Squares::A2, Squares::B1, Squares::B2]);

        // There should be 8 total moves
        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert_eq!(mv.from(), Squares::A1);
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }

    fn generate_knight_moves(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::KNIGHT, Squares::D4);
        board.state.active_side = side as u8;
        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            Squares::C2,
            Squares::C6,
            Squares::E2,
            Squares::E6,
            Squares::B3,
            Squares::B5,
            Squares::F3,
            Squares::F5,
        ]);

        // There should be 8 total moves
        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert_eq!(mv.from(), Squares::D4);
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }

    fn generate_knight_moves_edge_of_board(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::KNIGHT, Squares::A1);
        board.state.active_side = side as u8;
        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([Squares::C2, Squares::B3]);

        // There should be 8 total moves
        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert_eq!(mv.from(), Squares::A1);
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }
}
