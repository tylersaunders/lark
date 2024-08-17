use std::u8;

use crate::board::defs::{BitBoard,Square};

/// Extension trait for accessing individual bits in integers.
pub trait Bits {
    /// The unsigned type that represents bits of this type.
    type Bits;

    /// Get a specific bit.
    ///
    /// Panics if the index is out of range.
    fn bit<I>(self, i: I) -> bool
    where
        I: BitsIndex<Self>,
        Self: Sized;
}

/// Trait for types that can be used to index the bits of <T>.
pub trait BitsIndex<T> {
    /// See [`Bits::bit`]
    fn bit(v: T, i: Self) -> bool;
}

macro_rules! bits {
    ($t:tt, $ut:tt, $n:tt, $i:tt) => {
        impl BitsIndex<$t> for $i {
            #[inline]
            fn bit(v: $t, i: Self) -> bool {
                if i > $n {
                    panic!("bit index out of range");
                }
                v >> i & 1 != 0
            }
        }
    };

    ($t:tt, $ut:tt, $n:tt) => {
        impl Bits for $t {
            type Bits = $ut;

            #[inline]
            fn bit<I>(self, i: I) -> bool
            where
                I: BitsIndex<Self>,
            {
                I::bit(self, i)
            }

        }

        bits!($t, $ut, $n, i8);
        bits!($t, $ut, $n, u8);
        bits!($t, $ut, $n, i16);
        bits!($t, $ut, $n, u16);
        bits!($t, $ut, $n, i32);
        bits!($t, $ut, $n, u32);
        bits!($t, $ut, $n, i64);
        bits!($t, $ut, $n, u64);
        bits!($t, $ut, $n, i128);
        bits!($t, $ut, $n, u128);
        bits!($t, $ut, $n, isize);
        bits!($t, $ut, $n, usize);
    };
}

// Generate implementations for common integer types
bits!(i8, u8, 7);
bits!(u8, u8, 7);
bits!(i16, u16, 15);
bits!(u16, u16, 15);
bits!(i32, u32, 31);
bits!(u32, u32, 31);
bits!(i64, u64, 63);
bits!(u64, u64, 63);
bits!(i128, u128, 127);
bits!(u128, u128, 127);

/// Get the next set bit from a Bitboard and unset it.
/// Returns the corresponding square on the Bitboard of the set bit.
pub fn next(bitboard: &mut BitBoard) -> Square {
    let square = bitboard.trailing_zeros() as Square;
    *bitboard ^= 1u64 << square;
    square
}
