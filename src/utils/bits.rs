use crate::board::defs::{BitBoard, Square};

/// Get the next set bit from a Bitboard and unset it.
/// Returns the corresponding square on the Bitboard of the set bit.
pub fn next(bitboard: &mut BitBoard) -> Square {
    let square = bitboard.trailing_zeros() as Square;
    *bitboard ^= 1u64 << square;
    square
}
