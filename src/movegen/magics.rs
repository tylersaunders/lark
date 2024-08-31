use crate::{
    board::{
        defs::{
            BitBoard, Coordinate, Files, NrOf, Piece, Pieces, RangeOf, Ranks, Square, BB_FILES,
            BB_RANKS, BB_SQUARES, EMPTY, PIECE_CHAR_CAPS, SQUARE_NAME,
        },
        Board,
    },
    movegen::{defs::Direction, MoveGenerator},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

use super::defs::{AttackBoards, BlockerBoards};

// These are the exact sizes needed for the rook and bishop moves. These
// can be calculated by adding all the possible blocker boards for a rook
// or a bishop.
pub const ROOK_TABLE_SIZE: usize = 102_400; // Total permutations of all rook blocker boards.
pub const BISHOP_TABLE_SIZE: usize = 5_248; // Total permutations of all bishop blocker boards.

/// Magics implementation
///
/// * `mask`: A Rook or Bishop mask for the square the magic belongs to.
/// * `shift`: This number creates the magic index. It's "64 - (nr. of bits set 1 in mask)"
/// * `offset`: Contains the offset where the indexing of the square's attack boards begin.
/// * `number`: The magic number itself, used to create the magic index into the attack table.
#[derive(Default, Copy, Clone)]
pub struct Magic {
    pub mask: BitBoard,
    pub shift: u8,
    pub offset: u64,
    pub number: u64,
}

impl Magic {
    /// Gets the magic index into the attack table.
    /// The attack table is a perfect hash:
    ///
    ///   - A rook on A1 has 7 squares vertical and 7 squares horizontal movement.
    ///   - This is a total of 14 bits. However, if there are no pieces on A2-A6 or B1-G1,
    ///     the rook can always see A8 and H1. This means that if there are no blockers on
    ///     the file or rank, the rook can 'see' the square at the edge of the board. Therefore,
    ///     the bits marking the edge of a ray are not counted. Thus, the rook on A1 has actually
    ///     12 bits set.
    ///   - These bits along the rank and file denote the possible position of blocking pieces.
    ///   - For 12 bits, there are 4096 possible configuration of blockers (2 ^ 12).
    ///   - Thus, square A1 has 4096 blocker boards.
    ///   - The get_index function receives a board occupancy when called.
    ///   - "occupancy & self.mask" (the mask for the piece on the square the magic belongs to)
    ///     yields a blocker board.
    ///   - Each blocker board (configuration of blockers) goes with one attack board (the
    ///     squares the piece can actually attack). This attack board is in the attack table.
    ///   - The formula calculates WHERE in the attack table the blocker board is:
    ///     (blockerboard * magic number ) >> (64 - bits in mask) + offset
    ///   - For the rook on A1 the outcome will be an index of 0-4095:
    ///     0 - 4095 because of 4096 possible blocker (and thus, attack board) permutations.
    ///     0 for offset, because A1 is the first square.
    ///   - So the index for a rook on B1 will start at 4096, and so on. (So B1's offset is 4096)
    ///   - The "magic number" is called magic because it generates a UNIQUE index for each
    ///     attack board in the attack table, without any collisions; so the entire table is
    ///     exactly filled. (A perfect hash)
    ///   - Finding the magics is a process of just trying random numbers, with the formula below,
    ///     over and over again until a number is found that generates unique indexes for all the
    ///     permutations of attacks of the piece on a particular square.
    ///
    /// * `occupancy`: The occupancy Bitboard of the current board.
    pub fn get_index(&self, occupancy: BitBoard) -> usize {
        let blockerboard = occupancy & self.mask;
        ((blockerboard.wrapping_mul(self.number) >> self.shift) + self.offset) as usize
    }
}

/// Precalculated Rook magic numbers.
/// These were generated via find_magics, and are hard coded to speed up start-up time.
#[rustfmt::skip]
pub const PRECALC_ROOK_MAGIC_NUMBERS: [u64; NrOf::SQUARES] = [
    540432557653762064u64, 432363296033685762u64, 36046389741912104u64, 16176947462358433920u64,
    216191473845534804u64, 9367489424093348100u64, 2594076686081601544u64, 36029896536367232u64,
    864831868105932800u64, 70506187329536u64, 729864752066609808u64, 4611826824660387840u64,
    9511743184862979072u64, 288793506494284944u64, 4666010693227642884u64, 9801521647616803841u64,
    13546533286985732u64, 1153626292634050816u64, 9592817289904807936u64, 4611695914574151939u64,
    4972115276532286465u64, 162412161677670422u64, 9223376434918199304u64, 162254930953113921u64,
    18860219306115072u64, 9007337768485058u64, 2342733859858153602u64, 4539887806599172u64,
    22799494588923920u64, 11547792471846052872u64, 9223478706663604776u64, 11674460540688187521u64,
    141562264682784u64, 1625817744865755200u64, 3900117862030774272u64, 306385580877432832u64,
    562984883589124u64, 1442559272838760458u64, 9385642502670778880u64, 1126176965788705u64,
    5764678993437589544u64, 4616189893469585408u64, 2882901917318709312u64, 2305993651165134881u64,
    72090579693862936u64, 4830392848425877528u64, 36592296744992832u64, 146376343418961924u64,
    289075350976080128u64, 211174954106944u64, 145204289995904u64, 7061926412516729088u64,
    4399254503552u64, 9296067347670401152u64, 4036360555987993600u64, 36169538803941504u64,
    288552535206139969u64, 13917479648142360610u64, 5197752104730562625u64, 9511690787327967754u64,
    9583941791261131281u64, 289637768221950489u64, 87973891154500u64, 90146817869676674u64
];

/// Precalculated Bishop magic numbers.
/// These were generated via find_magics, and are hard coded to speed up start-up time.
#[rustfmt::skip]
pub const PRECALC_BISHOP_MAGIC_NUMBERS: [u64; NrOf::SQUARES] = [
    9763706338607138u64, 5190495499374829568u64, 1757608938763715080u64, 5635023166382148u64,
    1441717098471489568u64, 216471892494671936u64, 71485708371456u64, 1441714970365796416u64,
    2305913413525602432u64, 1171081317891375617u64, 292760922441777154u64, 9011614524637184u64,
    6917533443404333188u64, 2342436989636452364u64, 9732894142632192u64, 9223976227102001673u64,
    4506073566151680u64, 4508306947212296u64, 10383049009760833568u64, 281544300199936u64,
    2532149008611147937u64, 5476941217843577856u64, 281974401011873u64, 1171498857374290249u64,
    2307116385683113996u64, 76710746607719424u64, 2256202859819520u64, 2815986751275520u64,
    4648735170633730u64, 110340390971785748u64, 282575093629258u64, 292804956568815633u64,
    4612847411946063361u64, 5260945541455423488u64, 1153137051845002784u64, 283676147974272u64,
    1160805561324011600u64, 6935966192667853058u64, 145315856925218816u64, 74032317458317840u64,
    1451296112752231426u64, 1198502117924880u64, 1164264067767223296u64, 1179312455686144u64,
    218429048836064256u64, 2307004385555138816u64, 7081912648232600097u64, 20275068512763968u64,
    1128134030393480u64, 71502700556416u64, 566319364718600u64, 3206599220787413332u64,
    2310508615553270400u64, 2594706738441895940u64, 2269396328513536u64, 1226179799735885824u64,
    633490563538950u64, 578730428585740288u64, 54044295744786434u64, 562952273003016u64,
    4629702650605209088u64, 705062972686464u64, 621501215379308818u64, 2832350678368384u64,
];

impl MoveGenerator {


    /// Generates a rook mask for a rook on the given square.
    ///
    /// These are squares the rook could potentially "see".
    ///
    /// * `square`: The square the rook is on.
    pub fn rook_mask(square: Square) -> BitBoard {
        let coordinate = Board::square_on_file_rank(square);
        let bb_rook_square = BB_SQUARES[square];
        let bb_edges = MoveGenerator::edges_without_piece(coordinate);
        let bb_mask = BB_FILES[coordinate.0 as usize] | BB_RANKS[coordinate.1 as usize];

        bb_mask & !bb_edges & !bb_rook_square
    }

    /// Generates a bishop mask for a bishop on the given square.
    ///
    /// These are squares the bishop could potentially "see".
    ///
    /// * `square`: The square the bishop is on.
    pub fn bishop_mask(square: Square) -> BitBoard {
        let coordinate = Board::square_on_file_rank(square);
        let bb_edges = MoveGenerator::edges_without_piece(coordinate);
        let bb_up_left = MoveGenerator::bb_ray(0, square, Direction::NorthWest);
        let bb_up_right = MoveGenerator::bb_ray(0, square, Direction::NorthEast);
        let bb_down_right = MoveGenerator::bb_ray(0, square, Direction::SouthEast);
        let bb_down_left = MoveGenerator::bb_ray(0, square, Direction::SouthWest);

        (bb_up_left | bb_up_right | bb_down_left | bb_down_right) & !bb_edges
    }

    /// A BitBoard mask of the edges of the board.
    ///
    /// NOTE: Does not include the edge the piece is currently sitting on, if the piece is
    /// currently on the edge of the board.
    ///
    /// * `coordinate`: The current board coordinate of the piece.
    pub fn edges_without_piece(coordinate: Coordinate) -> BitBoard {
        let bb_piece_file = BB_FILES[coordinate.0 as usize];
        let bb_piece_rank = BB_RANKS[coordinate.1 as usize];

        (BB_FILES[Files::A] & !bb_piece_file)
            | (BB_FILES[Files::H] & !bb_piece_file)
            | (BB_RANKS[Ranks::R1] & !bb_piece_rank)
            | (BB_RANKS[Ranks::R8] & !bb_piece_rank)
    }

    /// Generates all possible blocker permutations for the piece mask.
    ///
    /// This uses the Carry-Rippler method.
    ///
    /// * `mask`:
    pub fn blocker_boards(mask: BitBoard) -> BlockerBoards {
        let d: BitBoard = mask;

        let mut bb_blocker_boards: BlockerBoards = Vec::new();
        let mut n: BitBoard = 0;

        // Carry Rippler method:
        // See: https://www.chessprogramming.org/Traversing_Subsets_of_a_Set
        loop {
            bb_blocker_boards.push(n);
            n = n.wrapping_sub(d) & d;
            if n == 0 {
                break;
            }
        }

        bb_blocker_boards
    }

    /// Given a square, and the blocker boards belonging to that square, generate the corresponding
    /// attack boards for each blocker board.
    ///
    /// * `square`: The square the rook is on.
    /// * `blocker_boards`: The blocker boards that belong to that square.
    pub fn rook_attack_boards(square: Square, blocker_boards: &[BitBoard]) -> AttackBoards {
        let mut bb_attack_boards: AttackBoards = Vec::new();

        for b in blocker_boards.iter() {
            let bb_attacks = MoveGenerator::bb_ray(*b, square, Direction::North)
                | MoveGenerator::bb_ray(*b, square, Direction::East)
                | MoveGenerator::bb_ray(*b, square, Direction::South)
                | MoveGenerator::bb_ray(*b, square, Direction::West);
            bb_attack_boards.push(bb_attacks);
        }

        bb_attack_boards
    }

    /// Given a square, and the blocker boards belonging to that square, generate the corresponding
    /// attack boards for each blocker board.
    ///
    /// * `square`: The square the bishop is on.
    /// * `blocker_boards`: The blocker boards that belong to that square.
    pub fn bishop_attack_boards(square: Square, blocker_boards: &[BitBoard]) -> AttackBoards {
        let mut bb_attack_boards: AttackBoards = Vec::new();

        for b in blocker_boards.iter() {
            let bb_attacks = MoveGenerator::bb_ray(*b, square, Direction::NorthWest)
                | MoveGenerator::bb_ray(*b, square, Direction::NorthEast)
                | MoveGenerator::bb_ray(*b, square, Direction::SouthWest)
                | MoveGenerator::bb_ray(*b, square, Direction::SouthEast);
            bb_attack_boards.push(bb_attacks)
        }

        bb_attack_boards
    }

    /// Generates a ray for a sliding piece in the designated direction.
    ///
    /// It will then generate a ray in the direction until it either hits a piece or reaches the
    /// edge of the board.
    ///
    /// * `bb_in`: BitBoard containing any blockers.
    /// * `square`: The square to start the ray from.
    /// * `direction`: The direction the ray should be cast in.
    pub fn bb_ray(bb_in: BitBoard, square: Square, direction: Direction) -> BitBoard {
        let mut file = Board::square_on_file_rank(square).0 as usize;
        let mut rank = Board::square_on_file_rank(square).1 as usize;
        let mut bb_square = BB_SQUARES[square];
        let mut bb_ray = 0;
        let mut done = false;

        while !done {
            done = true;
            match direction {
                Direction::North => {
                    if rank != Ranks::R8 {
                        bb_square <<= 8;
                        bb_ray |= bb_square;
                        rank += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::East => {
                    if file != Files::H {
                        bb_square <<= 1;
                        bb_ray |= bb_square;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::South => {
                    if rank != Ranks::R1 {
                        bb_square >>= 8;
                        bb_ray |= bb_square;
                        rank -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::West => {
                    if file != Files::A {
                        bb_square >>= 1;
                        bb_ray |= bb_square;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::NorthWest => {
                    if (rank != Ranks::R8) && (file != Files::A) {
                        bb_square <<= 7;
                        bb_ray |= bb_square;
                        rank += 1;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::NorthEast => {
                    if (rank != Ranks::R8) && (file != Files::H) {
                        bb_square <<= 9;
                        bb_ray |= bb_square;
                        rank += 1;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::SouthEast => {
                    if (rank != Ranks::R1) && (file != Files::H) {
                        bb_square >>= 7;
                        bb_ray |= bb_square;
                        rank -= 1;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::SouthWest => {
                    if (rank != Ranks::R1) && (file != Files::A) {
                        bb_square >>= 9;
                        bb_ray |= bb_square;
                        rank -= 1;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
            };
        }
        bb_ray
    }
}

impl Board {
    pub fn square_on_file_rank(square: Square) -> Coordinate {
        let file = (square % 8) as u8; // square mod 8
        let rank = (square / 8) as u8; // square div 8
        (file, rank)
    }
}

/// Generates magic numbers & attack tables for valid pieces. (Rooks, Bishops)
///
/// This looks for a suitable number for a "perfect hash" to store attack tables for the given
/// piece. This can be used to generate precalculated magic numbers that can be used to increase
/// start-up speed.
///
/// * `piece`: The piece to generate magics & attack tables for, either a Rook or Bishop.
pub fn find_magics(piece: Piece) -> (Vec<BitBoard>, [Magic; NrOf::SQUARES]) {
    println!();
    let ok = piece == Pieces::ROOK || piece == Pieces::BISHOP;
    assert!(ok, "Illegal piece: {piece}");

    let is_rook = piece == Pieces::ROOK;

    let mut rook_table: Vec<BitBoard> = vec![EMPTY; ROOK_TABLE_SIZE];
    let mut bishop_table: Vec<BitBoard> = vec![EMPTY; BISHOP_TABLE_SIZE];

    let mut rook_magics = [Magic::default(); NrOf::SQUARES];
    let mut bishop_magics = [Magic::default(); NrOf::SQUARES];

    let mut random = ChaChaRng::from_entropy();
    let mut offset = 0;

    println!("Finding magics for: {}", PIECE_CHAR_CAPS[piece]);
    for sq in RangeOf::SQUARES {
        // Create the mask for either the rook or bishop
        let r_mask = MoveGenerator::rook_mask(sq);
        let b_mask = MoveGenerator::bishop_mask(sq);
        let mask = if is_rook { r_mask } else { b_mask };

        let bits = mask.count_ones(); // Number of set bits in mask.
        let permutations = 2u64.pow(bits); // Number of  blocker boards to be indexed.
        let end = offset + permutations - 1; // End index in the attack table.

        let blocker_boards = MoveGenerator::blocker_boards(mask);

        // Create attack boards for the current square/blocker combo (either rook or bishop)
        let r_ab = MoveGenerator::rook_attack_boards(sq, &blocker_boards);
        let b_ab = MoveGenerator::bishop_attack_boards(sq, &blocker_boards);
        let attack_boards = if is_rook { r_ab } else { b_ab };

        // Create a new magic and begin the search.
        let mut test_magic: Magic = Default::default();
        let mut found = false;
        let mut attempts = 0;

        test_magic.mask = mask;
        test_magic.shift = (64 - bits) as u8;
        test_magic.offset = offset;

        while !found {
            attempts += 1; // Next attempt to find a magic number.
            found = true; // Assume this attempt will succeed until it doesn't.

            // Create a random magic number to test.
            test_magic.number = random.gen::<u64>() & random.gen::<u64>() & random.gen::<u64>();

            // Try all the possible permutations of blocker boards on this square.
            for i in 0..permutations {
                // Get the index where the magic for this blocker board needs to go (if it works).
                let next = i as usize;
                let index = test_magic.get_index(blocker_boards[next]);

                // Use either a reference to the rook or bishop table
                let r_table = &mut rook_table[..];
                let b_table = &mut bishop_table[..];
                let table: &mut [BitBoard] = if is_rook { r_table } else { b_table };

                // If the table is empty at this index
                if table[index] == EMPTY {
                    // Check if inside the expected range
                    let fail_low = index < offset as usize;
                    let fail_high = index > end as usize;
                    assert!(!fail_low && !fail_high, "indexing error.");

                    // Found a working magic.
                    table[index] = attack_boards[next];
                } else {
                    // The table at this index is not empty, so there is a collision. This magic
                    // doesn't work, wipe the part of the table that we are currently working with
                    // and try a new number.
                    for wipe_index in offset..=end {
                        table[wipe_index as usize] = EMPTY;
                    }
                    found = false;
                    break;
                }
            }
        }

        // We got out of the loop and found a random magic number that can index all the attack
        // boards for a rook/bishop for a single square without a collision. Report this number.
        found_magic(sq, test_magic, offset, end, attempts);

        if is_rook {
            rook_magics[sq] = test_magic
        } else {
            bishop_magics[sq] = test_magic
        }

        // Set table offset for the next magic.
        offset += permutations;
    }

    // Check if the entire table is correct. The offset should now be equal to the size of the
    // table. If it is not, we skipped permutation and thus have some sort of error in the code
    // above.
    let r_ts = ROOK_TABLE_SIZE as u64;
    let b_ts = BISHOP_TABLE_SIZE as u64;
    let expected = if is_rook { r_ts } else { b_ts };
    const ERROR: &str = "Creating magics failed, expected permutations were skipped.";
    assert!(offset == expected, "{}", ERROR);

    let table = if is_rook { rook_table } else { bishop_table };
    let magics = if is_rook { rook_magics } else { bishop_magics };
    (table, magics)
}

/// Prints a report when of a Magic number to stdout.
///
/// * `square`: The square the magic number is for.
/// * `m`: the Magic that fits the square.
/// * `offset`: The current starting attack_table offset
/// * `end`: The end of the attack table for this magic.
/// * `attempts`: How many attempts were required to find this magic number.
fn found_magic(square: Square, m: Magic, offset: u64, end: u64, attempts: u64) {
    println!(
        "{}: {:24}u64 (offset: {:6}, end: {:6}, attempts: {})",
        SQUARE_NAME[square], m.number, offset, end, attempts
    );
}
