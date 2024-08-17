use crate::{board::Board, utils::bits};

use super::defs::{Sides, PIECE_VALUES};

/// Counts the material for both sides on the given board.
///
/// * `board`: the Board to count material on.
pub fn count_material(board:&Board) -> (u16, u16) {

    let mut white_material: u16 = 0;
    let mut black_material: u16 = 0;

    let bb_w = board.bb_pieces[Sides::WHITE];
    let bb_b = board.bb_pieces[Sides::BLACK];

    for(piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {

        let mut white_pieces = *w;
        let mut black_pieces = *b;


        while white_pieces > 0 {
            white_material += PIECE_VALUES[piece_type];
            bits::next(&mut white_pieces);
        }

        while black_pieces > 0 {
            black_material += PIECE_VALUES[piece_type];
            bits::next(&mut black_pieces);
        }

    }

    (white_material, black_material)
}
