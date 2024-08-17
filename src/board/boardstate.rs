use std::fmt::Display;

use crate::board::defs::{Castling, Sides, SQUARE_NAME};

#[derive(Clone, Copy)]
/// Struct that represents the state of the game.
///
/// * `active_side`: which [`Sides`] is next to move.
/// * `castling`: the [`Castling`] permissions
/// * `en_passant`: the square where an en-passant move can be played.
/// * `half_move_clock`: Halfmove Clock for enforcing the fifty-move rule.
/// * `full_move_number`: The total number of complete moves. (starts at 1, is incremented after
///                       each move by [`Sides::BLACK`])
/// * `material`: The total material count for each side.
pub struct BoardState {
    pub active_side: u8,
    pub castling: u8,
    pub en_passant: Option<u8>,
    pub half_move_clock: u8,
    pub full_move_number: u16,
    pub material: [u16; Sides::BOTH],
}

impl BoardState {
    pub fn new() -> Self {
        Self {
            active_side: 0,
            castling: 0,
            en_passant: None,
            half_move_clock: 0,
            full_move_number: 0,
            material: [0; Sides::BOTH],
        }
    }

    fn castling_as_string(permissions: u8) -> String {
        let mut castling_as_string: String = String::from("");
        let p = permissions;

        castling_as_string += if p & Castling::WK > 0 { "K" } else { "" };
        castling_as_string += if p & Castling::WQ > 0 { "Q" } else { "" };
        castling_as_string += if p & Castling::BK > 0 { "k" } else { "" };
        castling_as_string += if p & Castling::BQ > 0 { "q" } else { "" };

        if castling_as_string.is_empty() {
            castling_as_string = String::from("-");
        }

        castling_as_string
    }
}

impl Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let en_passant = if let Some(x) = self.en_passant {
            SQUARE_NAME[x as usize]
        } else {
            "-"
        };
        write!(
            f,
            "ac: {} mat: {}/{} castling: {}, en_passant: {}, hmc: {}, fmn: {}",
            self.active_side,
            self.material[Sides::WHITE],
            self.material[Sides::BLACK],
            BoardState::castling_as_string(self.castling),
            en_passant,
            self.half_move_clock,
            self.full_move_number,
        )
    }
}
