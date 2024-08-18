use crate::board::defs::{Files, RangeOf, Ranks, Sides, BB_FILES, BB_RANKS, BB_SQUARES};

use super::{defs::Compass, MoveGenerator};

impl MoveGenerator {
    /// Generates all the possible King moves for each square on the board.
    ///
    /// Generates a move in all Compass directions: [NW,N,NE,W,E,SW,S,SE]
    /// but excludes directions that would go off the edge of the board.
    pub fn init_king(&mut self) {
        // For each square on the board, generate all the move locations from this square.
        for sq in RangeOf::SQUARES {
            let bb_square = BB_SQUARES[sq];
            let bb_moves =
                Compass::northwest(bb_square & !BB_FILES[Files::A] & !BB_RANKS[Ranks::R8])
                    | Compass::north(bb_square & !BB_RANKS[Ranks::R8])
                    | Compass::northeast(bb_square & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R8])
                    | Compass::west(bb_square & !BB_FILES[Files::A])
                    | Compass::east(bb_square & !BB_FILES[Files::H])
                    | Compass::southwest(bb_square & !BB_FILES[Files::A] & !BB_RANKS[Ranks::R1])
                    | Compass::south(bb_square & !BB_RANKS[Ranks::R1])
                    | Compass::southeast(bb_square & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R1]);

            self.king[sq] = bb_moves;
        }
    }

    /// Generates all possibles Knight moves for each square on the board.
    ///
    /// Generates a move in all Knight Compass directions: [NNW, NNE, NWW, NEE, SSW, SSE, SWW, SEE]
    /// but excludes directions that would go off the edge of the board.
    pub fn init_knight(&mut self) {
        // For each square on the board, generate all the move locations from this square.
        for sq in RangeOf::SQUARES {
            let bb_square = BB_SQUARES[sq];
            let bb_moves = Compass::north_north_west(
                bb_square & !BB_RANKS[Ranks::R8] & !BB_RANKS[Ranks::R7] & !BB_FILES[Files::A],
            ) | Compass::north_north_east(
                bb_square & !BB_RANKS[Ranks::R8] & !BB_RANKS[Ranks::R7] & !BB_FILES[Files::H],
            ) | Compass::north_west_west(
                bb_square & !BB_FILES[Files::A] & !BB_FILES[Files::B] & !BB_RANKS[Ranks::R8],
            ) | Compass::north_east_east(
                bb_square & !BB_FILES[Files::G] & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R8],
            ) | Compass::south_south_west(
                bb_square & !BB_RANKS[Ranks::R1] & !BB_RANKS[Ranks::R2] & !BB_FILES[Files::A],
            ) | Compass::south_south_east(
                bb_square & !BB_RANKS[Ranks::R1] & !BB_RANKS[Ranks::R2] & !BB_FILES[Files::H],
            ) | Compass::south_west_west(
                bb_square & !BB_FILES[Files::A] & !BB_FILES[Files::B] & !BB_RANKS[Ranks::R1],
            ) | Compass::south_east_east(
                bb_square & !BB_FILES[Files::G] & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R1],
            );

            self.knight[sq] = bb_moves;
        }
    }

    /// Generates all the possible pawn capture targets for each square.
    ///
    /// For white, generate a move northwest & northeast.
    /// For black, generate a move southwest & southeast.
    pub fn init_pawns(&mut self) {
        for sq in RangeOf::SQUARES {
            let bb_square = BB_SQUARES[sq];
            let w = Compass::northwest(bb_square & !BB_FILES[Files::A])
                | Compass::northeast(bb_square & !BB_FILES[Files::H]);
            let b = Compass::southwest(bb_square & !BB_FILES[Files::A])
                | Compass::southeast(bb_square & !BB_FILES[Files::H]);
            self.pawns[Sides::WHITE][sq] = w;
            self.pawns[Sides::BLACK][sq] = b;
        }
    }
}
