use std::vec;
mod init;
pub mod magics;

use defs::{Move, Shift};
use magics::{
    Magic, BISHOP_TABLE_SIZE, PRECALC_BISHOP_MAGIC_NUMBERS, PRECALC_ROOK_MAGIC_NUMBERS,
    ROOK_TABLE_SIZE,
};

use crate::{
    board::{
        defs::{
            BitBoard, Castling, NrOf, Piece, Pieces, Ranks, Side, Sides, Square, Squares, BB_RANKS,
            BB_SQUARES, EMPTY,
        },
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
/// * `rook`: The rook's attack table.
/// * `bishop`: The bishop's attack table.
/// * `rook_magics`: The per square Rook Magic numbers.
/// * `bishop_magics`: The per square Bishop Magic numbers.
pub struct MoveGenerator {
    king: [BitBoard; NrOf::SQUARES],
    knight: [BitBoard; NrOf::SQUARES],
    pawns: [[BitBoard; NrOf::SQUARES]; Sides::BOTH],
    rook: Vec<BitBoard>,
    bishop: Vec<BitBoard>,
    rook_magics: [Magic; NrOf::SQUARES],
    bishop_magics: [Magic; NrOf::SQUARES],
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
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [Magic::default(); NrOf::SQUARES],
            bishop_magics: [Magic::default(); NrOf::SQUARES],
        };
        mg.init_king();
        mg.init_knight();
        mg.init_pawns();
        mg.init_magics_with_precalc(PRECALC_ROOK_MAGIC_NUMBERS, PRECALC_BISHOP_MAGIC_NUMBERS);
        mg
    }

    /// Create and initialize a new [`MoveGenerator`].
    ///
    /// This will calculate new magic numbers for the sliding attack tables.
    pub fn new_find_magics() -> Self {
        let mut mg = Self {
            king: [EMPTY; NrOf::SQUARES],
            knight: [EMPTY; NrOf::SQUARES],
            pawns: [[EMPTY; NrOf::SQUARES]; Sides::BOTH],
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [Magic::default(); NrOf::SQUARES],
            bishop_magics: [Magic::default(); NrOf::SQUARES],
        };
        mg.init_king();
        mg.init_knight();
        mg.init_pawns();
        mg.init_magics();
        mg
    }

    /// Generates moves for the side that is to move.
    ///
    /// * `board`: The current board to generate moves for
    /// * `move_list`: A list where the generated moves will be appended.
    pub fn generate_moves(&self, board: &Board, move_list: &mut Vec<Move>) {
        self.piece(board, Pieces::KING, move_list);
        self.piece(board, Pieces::KNIGHT, move_list);
        self.piece(board, Pieces::QUEEN, move_list);
        self.piece(board, Pieces::ROOK, move_list);
        self.piece(board, Pieces::BISHOP, move_list);
        self.pawns(board, move_list);
        self.castling(board, move_list);
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
        let bb_occupied = board.bb_side[Sides::WHITE] | board.bb_side[Sides::BLACK];
        let bb_own_pieces = board.bb_side[player];

        let mut bb_pieces = board.bb_pieces[player][piece];

        while bb_pieces > 0 {
            let from = bits::next(&mut bb_pieces);
            let bb_target = match piece {
                Pieces::KING | Pieces::KNIGHT => self.get_non_slider_attacks(piece, from),
                Pieces::QUEEN | Pieces::ROOK | Pieces::BISHOP => {
                    self.get_slider_attacks(piece, from, bb_occupied)
                }
                _ => panic!("Not a piece: {piece}"),
            };

            let bb_moves = bb_target & !bb_own_pieces;
            self.add_moves(board, piece, from, bb_moves, list);
        }
    }

    /// Generate all castling moves for the current side.
    ///
    /// * `board`: The current board.
    /// * `list`: The current move list.
    pub fn castling(&self, board: &Board, list: &mut Vec<Move>) {
        let player = board.current_side();
        let opponent = board.opponent();

        let castle_permissions_white = (board.state.castling & (Castling::WK | Castling::WQ)) > 0;
        let castle_permissions_black = (board.state.castling & (Castling::BK | Castling::BQ)) > 0;

        let bb_occupancy = board.bb_side[Sides::WHITE] | board.bb_side[Sides::BLACK];
        let mut bb_king = board.bb_pieces[player][Pieces::KING];

        // If there is no king on the board, don't proceed.
        // This is not really legal state, but some tests don't always put a king piece on the
        // board.
        if bb_king == 0 {
            return
        }

        let from = bits::next(&mut bb_king);

        // Generate castling moves for white.
        if player == Sides::WHITE && castle_permissions_white {
            // King side
            if board.state.castling & Castling::WK > 0 {
                let bb_kingside_blockers = BB_SQUARES[Squares::F1] | BB_SQUARES[Squares::G1];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::F1)
                    && !self.square_attacked(board, opponent, Squares::E1)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_moves(board, Pieces::KING, from, to, list)
                }
            }

            // Queen side
            if board.state.castling & Castling::WQ > 0 {
                let bb_queenside_blockers =
                    BB_SQUARES[Squares::B1] | BB_SQUARES[Squares::C1] | BB_SQUARES[Squares::D1];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E1)
                    && !self.square_attacked(board, opponent, Squares::D1)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_moves(board, Pieces::KING, from, to, list);
                }
            }
        }

        // Generate castling moves for black.
        if player == Sides::BLACK && castle_permissions_black {
            // King side
            if board.state.castling & Castling::BK > 0 {
                let bb_kingside_blockers = BB_SQUARES[Squares::F8] | BB_SQUARES[Squares::G8];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::F8)
                    && !self.square_attacked(board, opponent, Squares::E8)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_moves(board, Pieces::KING, from, to, list)
                }
            }

            // Queen side
            if board.state.castling & Castling::BQ > 0 {
                let bb_queenside_blockers =
                    BB_SQUARES[Squares::B8] | BB_SQUARES[Squares::C8] | BB_SQUARES[Squares::D8];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E8)
                    && !self.square_attacked(board, opponent, Squares::D8)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_moves(board, Pieces::KING, from, to, list);
                }
            }
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

    /// Get the attacks table for the sliding piece.
    ///
    /// * `piece`: must be a BISHOP, ROOK or QUEEN, or this function will panic.
    /// * `square`: The square the piece is currently attacking from.
    /// * `occupancy`: The current occupied squares on the board, for both sides.
    fn get_slider_attacks(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard {
        match piece {
            Pieces::ROOK => {
                let index = self.rook_magics[square].get_index(occupancy);
                self.rook[index]
            }
            Pieces::BISHOP => {
                let index = self.bishop_magics[square].get_index(occupancy);
                self.bishop[index]
            }
            Pieces::QUEEN => {
                let r_index = self.rook_magics[square].get_index(occupancy);
                let b_index = self.bishop_magics[square].get_index(occupancy);
                self.rook[r_index] ^ self.bishop[b_index]
            }
            _ => panic!("Not a sliding piece: {piece}"),
        }
    }

    /// Determines if the given side is attacking the given square.
    ///
    /// * `board`: The board to evaluate.
    /// * `attacker`: The side that is attacking.
    /// * `square`: The square to check if it is attacked.
    pub fn square_attacked(&self, board: &Board, attacker: Side, square: Square) -> bool {
        let attackers = board.bb_pieces[attacker];
        let bb_occupied = board.bb_side[Sides::WHITE] | board.bb_side[Sides::BLACK];

        // Use the super-piece method: get the moves for each piece, starting from the given
        // square. This provides the squares where a piece has to be, to be able to reach the given
        // square.
        let bb_king = self.get_non_slider_attacks(Pieces::KING, square);
        let bb_rook = self.get_slider_attacks(Pieces::ROOK, square, bb_occupied);
        let bb_bishop = self.get_slider_attacks(Pieces::BISHOP, square, bb_occupied);
        let bb_knight = self.get_non_slider_attacks(Pieces::KNIGHT, square);
        let bb_pawns = self.pawns[attacker ^ 1][square];
        let bb_queen = bb_rook | bb_bishop;

        // Then determine if such a piece is actually there: see if a rook is on one of the squares
        // a rook has to be on to reach the given square. Same for queen, knight, etc. As soon as
        // any pieces are found, the square can be considered attacked.
        (bb_king & attackers[Pieces::KING] > 0)
            || (bb_rook & attackers[Pieces::ROOK] > 0)
            || (bb_bishop & attackers[Pieces::BISHOP] > 0)
            || (bb_queen & attackers[Pieces::QUEEN] > 0)
            || (bb_knight & attackers[Pieces::KNIGHT] > 0)
            || (bb_pawns & attackers[Pieces::PAWN] > 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{
        defs::{Castling, Pieces, Sides, Square, Squares},
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
        rook_moves_white: generate_rook_moves, Sides::WHITE;
        rook_moves_with_collisions_white: generate_rook_moves_with_collisions, Sides::WHITE;
        rook_moves_with_captures_white: generate_rook_moves_with_captures, Sides::WHITE;
        bishop_moves_white: generate_bishop_moves, Sides::WHITE;
        bishop_moves_with_collisions_white: generate_bishop_moves_with_collisions, Sides::WHITE;
        bishop_moves_with_captures_white: generate_bishop_moves_with_captures, Sides::WHITE;
        queen_moves_white: generate_queen_moves, Sides::WHITE;
        queen_moves_with_collisions_white: generate_queen_moves_with_collisions, Sides::WHITE;
        queen_moves_with_captures_white: generate_queen_moves_with_captures, Sides::WHITE;

        king_moves_black: generate_king_moves, Sides::BLACK;
        king_moves_edge_of_board_black: generate_king_moves_edge_of_board, Sides::BLACK;
        knight_moves_black: generate_knight_moves, Sides::BLACK;
        knight_moves_edge_of_board_black: generate_knight_moves_edge_of_board, Sides::BLACK;
        rook_moves_black: generate_rook_moves, Sides::BLACK;
        rook_moves_with_collisions_black: generate_rook_moves_with_collisions, Sides::BLACK;
        rook_moves_with_captures_black: generate_rook_moves_with_captures, Sides::BLACK;
        bishop_moves_black: generate_bishop_moves, Sides::BLACK;
        bishop_moves_with_collisions_black: generate_bishop_moves_with_collisions, Sides::BLACK;
        bishop_moves_with_captures_black: generate_bishop_moves_with_captures, Sides::BLACK;
        queen_moves_black: generate_queen_moves, Sides::BLACK;
        queen_moves_with_collisions_black: generate_queen_moves_with_collisions, Sides::BLACK;
        queen_moves_with_captures_black: generate_queen_moves_with_captures, Sides::BLACK;
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

    #[test]
    fn test_generate_castling_moves_white() {
        let mut board = Board::new();
        board.state.castling = Castling::WK | Castling::WQ;

        board.put_piece(Sides::WHITE, Pieces::KING, Squares::E1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::H1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::A1);

        board.state.active_side = Sides::WHITE as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(destinations.contains(&Squares::C1));
        assert!(destinations.contains(&Squares::G1));
    }

    #[test]
    fn test_generate_castling_moves_white_no_permissions() {
        let mut board = Board::new();
        board.state.castling = 0;

        board.put_piece(Sides::WHITE, Pieces::KING, Squares::E1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::H1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::A1);

        board.state.active_side = Sides::WHITE as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(!destinations.contains(&Squares::C1));
        assert!(!destinations.contains(&Squares::G1));
    }

    #[test]
    fn test_generate_castling_moves_white_kingside_attacker() {
        let mut board = Board::new();
        board.state.castling = Castling::WK | Castling::WQ;

        board.put_piece(Sides::WHITE, Pieces::KING, Squares::E1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::H1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::A1);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::F8);

        board.state.active_side = Sides::WHITE as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(destinations.contains(&Squares::C1));
        assert!(!destinations.contains(&Squares::G1));
    }

    #[test]
    fn test_generate_castling_moves_white_queenside_attacker() {
        let mut board = Board::new();
        board.state.castling = Castling::WK | Castling::WQ;

        board.put_piece(Sides::WHITE, Pieces::KING, Squares::E1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::H1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::A1);
        board.put_piece(Sides::BLACK, Pieces::QUEEN, Squares::D8);

        board.state.active_side = Sides::WHITE as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(!destinations.contains(&Squares::C1));
        assert!(destinations.contains(&Squares::G1));
    }

    #[test]
    fn test_generate_castling_moves_white_king_in_check() {
        let mut board = Board::new();
        board.state.castling = Castling::WK | Castling::WQ;

        board.put_piece(Sides::WHITE, Pieces::KING, Squares::E1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::H1);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::A1);
        board.put_piece(Sides::BLACK, Pieces::QUEEN, Squares::E8);

        board.state.active_side = Sides::WHITE as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(!destinations.contains(&Squares::C1));
        assert!(!destinations.contains(&Squares::G1));
    }

    #[test]
    fn test_generate_castling_moves_black() {
        let mut board = Board::new();
        board.state.castling = Castling::BK | Castling::BQ;

        board.put_piece(Sides::BLACK, Pieces::KING, Squares::E8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::H8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::A8);

        board.state.active_side = Sides::BLACK as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(destinations.contains(&Squares::C8));
        assert!(destinations.contains(&Squares::G8));
    }

    #[test]
    fn test_generate_castling_moves_black_kingside_attacker() {
        let mut board = Board::new();
        board.state.castling = Castling::BK | Castling::BQ;

        board.put_piece(Sides::BLACK, Pieces::KING, Squares::E8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::H8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::A8);
        board.put_piece(Sides::WHITE, Pieces::ROOK, Squares::F1);

        board.state.active_side = Sides::BLACK as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(destinations.contains(&Squares::C8));
        assert!(!destinations.contains(&Squares::G8));
    }

    #[test]
    fn test_generate_castling_moves_black_queenside_attacker() {
        let mut board = Board::new();
        board.state.castling = Castling::BK | Castling::BQ;

        board.put_piece(Sides::BLACK, Pieces::KING, Squares::E8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::H8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::A8);
        board.put_piece(Sides::WHITE, Pieces::QUEEN, Squares::D1);

        board.state.active_side = Sides::BLACK as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(!destinations.contains(&Squares::C8));
        assert!(destinations.contains(&Squares::G8));
    }

    #[test]
    fn test_generate_castling_moves_black_king_in_check() {
        let mut board = Board::new();
        board.state.castling = Castling::BK | Castling::BQ;

        board.put_piece(Sides::BLACK, Pieces::KING, Squares::E8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::H8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::A8);
        board.put_piece(Sides::WHITE, Pieces::QUEEN, Squares::E1);

        board.state.active_side = Sides::BLACK as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(!destinations.contains(&Squares::C8));
        assert!(!destinations.contains(&Squares::G8));
    }

    #[test]
    fn test_generate_castling_moves_black_no_permissions() {
        let mut board = Board::new();
        board.state.castling = 0;

        board.put_piece(Sides::BLACK, Pieces::KING, Squares::E8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::H8);
        board.put_piece(Sides::BLACK, Pieces::ROOK, Squares::A8);

        board.state.active_side = Sides::BLACK as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        // Discard all non-king moves.
        move_list.retain(|mv| mv.piece() == Pieces::KING);
        let destinations: Vec<Square> = move_list.iter().map(|mv| mv.to()).collect();

        assert!(!destinations.contains(&Squares::C8));
        assert!(!destinations.contains(&Squares::G8));
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

    fn generate_rook_moves(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::ROOK, Squares::B2);
        board.state.active_side = side as u8;
        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            Squares::A2,
            Squares::C2,
            Squares::D2,
            Squares::E2,
            Squares::F2,
            Squares::G2,
            Squares::H2,
            Squares::B1,
            Squares::B3,
            Squares::B4,
            Squares::B5,
            Squares::B6,
            Squares::B7,
            Squares::B8,
        ]);

        // There should be 14 total moves
        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert_eq!(mv.from(), Squares::B2);
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }

    fn generate_rook_moves_with_collisions(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::ROOK, Squares::A1);
        board.put_piece(side, Pieces::KING, Squares::A6);
        board.put_piece(side, Pieces::KNIGHT, Squares::C1);
        board.state.active_side = side as u8;
        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            Squares::B1,
            Squares::A2,
            Squares::A3,
            Squares::A4,
            Squares::A5,
        ]);

        // Keep only the moves for the rook.
        move_list.retain(|mv| mv.piece() == Pieces::ROOK);

        // There should be 5 total moves
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

    fn generate_rook_moves_with_captures(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::ROOK, Squares::A1);

        board.state.active_side = side as u8;

        board.put_piece(board.opponent(), Pieces::KING, Squares::A6);
        board.put_piece(board.opponent(), Pieces::KNIGHT, Squares::C1);

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            Squares::B1,
            Squares::C1,
            Squares::A2,
            Squares::A3,
            Squares::A4,
            Squares::A5,
            Squares::A6,
        ]);

        // Keep only the moves for the rook.
        move_list.retain(|mv| mv.piece() == Pieces::ROOK);

        // There should be 7 total moves
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

    fn generate_bishop_moves(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::BISHOP, Squares::B2);
        board.state.active_side = side as u8;
        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            Squares::A1,
            Squares::A3,
            Squares::C1,
            Squares::C3,
            Squares::D4,
            Squares::E5,
            Squares::F6,
            Squares::G7,
            Squares::H8,
        ]);

        // There should be 9 total moves
        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert_eq!(mv.from(), Squares::B2);
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }

    fn generate_bishop_moves_with_collisions(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::BISHOP, Squares::D4);
        board.put_piece(side, Pieces::KING, Squares::B2);
        board.put_piece(side, Pieces::KNIGHT, Squares::B6);
        board.state.active_side = side as u8;
        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            Squares::C3,
            Squares::C5,
            Squares::E3,
            Squares::E5,
            Squares::F2,
            Squares::F6,
            Squares::G1,
            Squares::G7,
            Squares::H8,
        ]);

        // Keep only the moves for the bishop.
        move_list.retain(|mv| mv.piece() == Pieces::BISHOP);

        // There should be 9 total moves
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

    fn generate_bishop_moves_with_captures(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::BISHOP, Squares::A1);

        board.state.active_side = side as u8;

        board.put_piece(board.opponent(), Pieces::KING, Squares::F6);

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            Squares::B2,
            Squares::C3,
            Squares::D4,
            Squares::E5,
            Squares::F6,
        ]);

        // Keep only the moves for the bishop.
        move_list.retain(|mv| mv.piece() == Pieces::BISHOP);

        // There should be 5 total moves
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

    fn generate_queen_moves(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::QUEEN, Squares::B2);
        board.state.active_side = side as u8;
        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            // Diagonals
            Squares::A1,
            Squares::A3,
            Squares::C1,
            Squares::C3,
            Squares::D4,
            Squares::E5,
            Squares::F6,
            Squares::G7,
            Squares::H8,
            // Horizontal
            Squares::A2,
            Squares::C2,
            Squares::D2,
            Squares::E2,
            Squares::F2,
            Squares::G2,
            Squares::H2,
            // Vertical
            Squares::B1,
            Squares::B3,
            Squares::B4,
            Squares::B5,
            Squares::B6,
            Squares::B7,
            Squares::B8,
        ]);

        // There should be 23 total moves
        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert_eq!(mv.from(), Squares::B2);
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }

    fn generate_queen_moves_with_collisions(side: usize) {
        let mut board = Board::new();

        board.put_piece(side, Pieces::QUEEN, Squares::B2);
        board.put_piece(side, Pieces::KING, Squares::F6);

        board.state.active_side = side as u8;

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            // Diagonals
            Squares::A1,
            Squares::A3,
            Squares::C1,
            Squares::C3,
            Squares::D4,
            Squares::E5,
            // Horizontal
            Squares::A2,
            Squares::C2,
            Squares::D2,
            Squares::E2,
            Squares::F2,
            Squares::G2,
            Squares::H2,
            // Vertical
            Squares::B1,
            Squares::B3,
            Squares::B4,
            Squares::B5,
            Squares::B6,
            Squares::B7,
            Squares::B8,
        ]);

        // Keep just the queen's moves.
        move_list.retain(|mv| mv.piece() == Pieces::QUEEN);

        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert_eq!(mv.from(), Squares::B2);
            assert!(expected_sq.contains(&mv.to()));

            // Remove the square from the expected_sq
            expected_sq.retain(|&x| x != mv.to());
        }

        // By now expected_sq should be empty.
        assert_eq!(expected_sq.len(), 0);
    }

    fn generate_queen_moves_with_captures(side: usize) {
        let mut board = Board::new();
        board.put_piece(side, Pieces::QUEEN, Squares::B2);

        board.state.active_side = side as u8;

        board.put_piece(board.opponent(), Pieces::KING, Squares::F6);

        let mg = MoveGenerator::new();
        let mut move_list: Vec<Move> = Vec::new();
        mg.generate_moves(&board, &mut move_list);

        let mut expected_sq = Vec::from([
            // Diagonals
            Squares::A1,
            Squares::A3,
            Squares::C1,
            Squares::C3,
            Squares::D4,
            Squares::E5,
            Squares::F6,
            // Horizontal
            Squares::A2,
            Squares::C2,
            Squares::D2,
            Squares::E2,
            Squares::F2,
            Squares::G2,
            Squares::H2,
            // Vertical
            Squares::B1,
            Squares::B3,
            Squares::B4,
            Squares::B5,
            Squares::B6,
            Squares::B7,
            Squares::B8,
        ]);

        // Keep just the queen's moves.
        move_list.retain(|mv| mv.piece() == Pieces::QUEEN);

        assert_eq!(move_list.len(), expected_sq.len());

        for mv in move_list {
            assert_eq!(mv.from(), Squares::B2);
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
