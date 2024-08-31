use crate::board::defs::{
    Files, NrOf, Pieces, RangeOf, Ranks, Sides, BB_FILES, BB_RANKS, BB_SQUARES, EMPTY,
};

use super::{
    defs::Compass,
    magics::{find_magics, Magic, BISHOP_TABLE_SIZE, ROOK_TABLE_SIZE},
    MoveGenerator,
};

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

    /// Initializes the sliding piece attack tables by generating a new set of magic numbers.
    /// This is slow, but guarantees to find a set of magic numbers.
    pub fn init_magics(&mut self) {
        (self.rook, self.rook_magics) = find_magics(Pieces::ROOK);
        (self.bishop, self.bishop_magics) = find_magics(Pieces::BISHOP);
    }

    /// Initializes the sliding piece attack tables with the provided magic numbers.
    /// binary.
    ///
    /// * `rook_magics`: A set of rook magic numbers for each square on the board.
    /// * `bishop_magics`: A set of bishop magic numbers for each square on the board.
    pub fn init_magics_with_precalc(
        &mut self,
        rook_magics: [u64; NrOf::SQUARES],
        bishop_magics: [u64; NrOf::SQUARES],
    ) {
        for piece in [Pieces::ROOK, Pieces::BISHOP] {
            let is_rook = piece == Pieces::ROOK;
            let mut offset = 0;

            for sq in RangeOf::SQUARES {
                let r_mask = MoveGenerator::rook_mask(sq);
                let b_mask = MoveGenerator::bishop_mask(sq);
                let mask = if is_rook { r_mask } else { b_mask };

                let bits = mask.count_ones();
                let permutations = 2u64.pow(bits);
                let end = offset + permutations - 1;
                let blocker_boards = MoveGenerator::blocker_boards(mask);

                let r_ab = MoveGenerator::rook_attack_boards(sq, &blocker_boards);
                let b_ab = MoveGenerator::bishop_attack_boards(sq, &blocker_boards);
                let attack_boards = if is_rook { r_ab } else { b_ab };

                let mut magic: Magic = Default::default();
                let r_magic_number = rook_magics[sq];
                let b_magic_number = bishop_magics[sq];

                magic.mask = mask;
                magic.shift = (64 - bits) as u8;
                magic.offset = offset;
                magic.number = if is_rook {
                    r_magic_number
                } else {
                    b_magic_number
                };

                for i in 0..permutations {
                    let next = i as usize;
                    let index = magic.get_index(blocker_boards[next]);
                    let rook_table = &mut self.rook[..];
                    let bishop_table = &mut self.bishop[..];
                    let table = if is_rook { rook_table } else { bishop_table };

                    if table[index] == EMPTY {
                        let fail_low = index < offset as usize;
                        let fail_high = index > end as usize;
                        assert!(
                            !fail_low && !fail_high,
                            "Indexing error, Error in Magic initialization"
                        );
                        table[index] = attack_boards[next];
                    } else {
                        panic!("Attack table index was not empty when Empty was expected. Error in Magics.");
                    }
                }

                if is_rook {
                    self.rook_magics[sq] = magic
                } else {
                    self.bishop_magics[sq] = magic
                }

                offset += permutations;
            }

            let r_ts = ROOK_TABLE_SIZE as u64;
            let b_ts = BISHOP_TABLE_SIZE as u64;
            let expectation = if is_rook { r_ts } else { b_ts };
            const ERROR: &str = "initialization of magics failed, check magic numbers.";
            assert!(offset == expectation, "{}", ERROR);
        }
    }
}
