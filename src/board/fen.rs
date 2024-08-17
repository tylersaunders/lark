use if_chain::if_chain;
use std::{char, fmt::Display, ops::RangeInclusive};

use crate::{
    board::Board,
    board::defs::{
        Castling, Files, Pieces, Ranks, Sides, Square, Squares, BB_SQUARES, MAX_GAME_MOVES,
        MAX_MOVE_RULE, SQUARE_NAME,
    },
};

const FEN_NR_OF_SECTIONS: usize = 6;
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const ENPASSANT_SQUARES_WHITE: RangeInclusive<Square> = Squares::A3..=Squares::H3;
const ENPASSANT_SQUARES_BLACK: RangeInclusive<Square> = Squares::A6..=Squares::H6;
const WHITE_OR_BLACK: &str = "wb";
const DELIMITER: char = '/';
const DASH: char = '-';
const EM_DASH: char = '–';
const SPACE: char = ' ';
const DEFAULT_FEN_STRING: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, PartialEq)]
/// The possible errors that can be encountered during FEN parsing.
pub enum FenError {
    IncorrectLength,
    PieceSection,
    ColorSection,
    CastlingSection,
    EnPassantSection,
    HalfMoveClockSection,
    FullMoveSection,
}

impl Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::IncorrectLength => "Error in FEN string: must be 6 parts",
            Self::PieceSection => "Error in FEN Section: Pieces or Squares",
            Self::ColorSection => "Error in FEN Section: Colors",
            Self::CastlingSection => "Error in FEN Section: Castling rights",
            Self::EnPassantSection => "Error in FEN Section: En passant field",
            Self::HalfMoveClockSection => "Error in FEN Section: Half-move clock",
            Self::FullMoveSection => "Error in FEN Section: Full-move number",
        };
        write!(f, "{error}")
    }
}

/// The result of the FEN parsing operation.
pub type FenResult = Result<(), FenError>;

/// The result of the FEN string splitting operation.
pub type SplitResult = Result<Vec<String>, FenError>;

/// A parser for a FEN part, that applies the FEN data to the board.
type FenPartParser = fn(board: &mut Board, part: &str) -> FenResult;

impl Board {
    // This function reads a provided FEN-string or uses the default position
    ///
    /// * `fen_string`: A valid FEN-style string containing a chess position.
    pub fn fen_read(&mut self, fen_string: Option<&str>) -> FenResult {
        // Split the string into parts, there should be 6 parts.

        let fen_parts = split_fen_string(fen_string)?;

        let fen_parsers = create_part_parsers();

        let mut new_board = self.clone();

        for (parser, part) in fen_parsers.iter().zip(fen_parts.iter()) {
            parser(&mut new_board, part)?;
        }

        new_board.init();
        *self = new_board;

        Ok(())
    }
}

/// Splits the incoming (optional) string into its component parts.
///
/// It also does a bit of error handling:
///     such as replacing the EM-dash with a normal dash.
///     If the FEN string is 4 parts long, the values 0 and 1 are assumed for the last two parts.
///
/// * `fen_string`: a FEN-style string to split into its component parts.
fn split_fen_string(fen_string: Option<&str>) -> SplitResult {
    const SHORT_FEN_LENGTH: usize = 4;

    // If no FEN string was provided, use the default chess starting position.
    let mut fen_sections: Vec<String> = match fen_string {
        Some(fen) => fen,
        None => DEFAULT_FEN_STRING,
    }
    .replace(EM_DASH, DASH.encode_utf8(&mut [0; 4]))
    .split(SPACE)
    .map(|s| s.to_string())
    .collect();

    if fen_sections.len() == SHORT_FEN_LENGTH {
        fen_sections.append(&mut vec![String::from("0"), String::from("1")]);
    }

    if fen_sections.len() != FEN_NR_OF_SECTIONS {
        return Err(FenError::IncorrectLength);
    }

    Ok(fen_sections)
}

/// Generates the list of parsers to use for each part of the FEN string.
fn create_part_parsers() -> [FenPartParser; FEN_NR_OF_SECTIONS] {
    [
        pieces,
        color,
        castling,
        en_passant,
        half_move_clock,
        full_move_number,
    ]
}

/// Parses the pieces section of the FEN string to position the pieces on the board.
///
/// Returns Err(FenError::PieceSection) when invalid characters in the part are detected, or a rank
/// doesn't have 8 files. Each piece in the string is added to the corresponding sides piece
/// bitboard.
///
/// * `board`: The board to add the pieces to.
/// * `section`: Section 1 of the FEN strings that contains piece positions.
fn pieces(board: &mut Board, section: &str) -> FenResult {
    let mut rank = Ranks::R8 as u8;
    let mut file = Files::A as u8;

    // Parse each character; it should be a piece, square count or DELIMITER
    for c in section.chars() {
        let square = ((rank * 8) + file) as usize;
        match c {
            'k' => board.bb_pieces[Sides::BLACK][Pieces::KING] |= BB_SQUARES[square],
            'q' => board.bb_pieces[Sides::BLACK][Pieces::QUEEN] |= BB_SQUARES[square],
            'r' => board.bb_pieces[Sides::BLACK][Pieces::ROOK] |= BB_SQUARES[square],
            'b' => board.bb_pieces[Sides::BLACK][Pieces::BISHOP] |= BB_SQUARES[square],
            'n' => board.bb_pieces[Sides::BLACK][Pieces::KNIGHT] |= BB_SQUARES[square],
            'p' => board.bb_pieces[Sides::BLACK][Pieces::PAWN] |= BB_SQUARES[square],
            'K' => board.bb_pieces[Sides::WHITE][Pieces::KING] |= BB_SQUARES[square],
            'Q' => board.bb_pieces[Sides::WHITE][Pieces::QUEEN] |= BB_SQUARES[square],
            'R' => board.bb_pieces[Sides::WHITE][Pieces::ROOK] |= BB_SQUARES[square],
            'B' => board.bb_pieces[Sides::WHITE][Pieces::BISHOP] |= BB_SQUARES[square],
            'N' => board.bb_pieces[Sides::WHITE][Pieces::KNIGHT] |= BB_SQUARES[square],
            'P' => board.bb_pieces[Sides::WHITE][Pieces::PAWN] |= BB_SQUARES[square],

            // Skip ahead N number of files
            '1'..='8' => {
                if let Some(x) = c.to_digit(10) {
                    file += x as u8;
                }
            }

            // If we've reached a delimiter, check that we ended on the last file.
            DELIMITER => {
                if file != 8 {
                    return Err(FenError::PieceSection);
                }

                // move to the next rank, and start over at file 0.
                rank -= 1;
                file = 0;
            }
            _ => return Err(FenError::PieceSection),
        }

        // If a piece is found, advance to the next file.
        // (So that this doesn't need to be done in each arm of the match.)
        if LIST_OF_PIECES.contains(c) {
            file += 1;
        }
    }

    Ok(())
}

/// Parses the ColorSection of the FEN string to determine which color is the
/// active color.
///
/// * `board`: The board the game state will be updated on.
/// * `section`: Section 2 of the FEN strings that contains the active color.
fn color(board: &mut Board, section: &str) -> FenResult {
    if_chain! {
        if section.len() == 1;
        if let Some(c) = section.chars().next();
        if WHITE_OR_BLACK.contains(c);
        then {
            match c {
                'w' => board.state.active_side = Sides::WHITE as u8,
                'b' => board.state.active_side = Sides::BLACK as u8,
                _ => (),
            }
            return Ok(())
        }
    }
    Err(FenError::ColorSection)
}

/// Parses the CastlingSection of the FEN string to determine which, if any castling rights remain
/// for each color.
///
/// * `board`: The board the game state will be updated on.
/// * `section`: Section 3 of the FEN strings that contains the castling rights.
fn castling(board: &mut Board, section: &str) -> FenResult {
    // There should be up a minimum of 1 and a maximum of 4 castling rights. If no player has castling
    // rights, the character in the FEN string should be '-'.
    if (1..=4).contains(&section.len()) {
        for c in section.chars() {
            match c {
                // White
                'K' => board.state.castling |= Castling::WK,
                'Q' => board.state.castling |= Castling::WQ,
                // Black
                'k' => board.state.castling |= Castling::BK,
                'q' => board.state.castling |= Castling::BQ,
                // No castling rights
                '-' => (),
                // Any other character here is an error.
                _ => return Err(FenError::CastlingSection),
            }
        }
    }

    return Ok(());
}

/// Parses the EnPassantSection of the FEN string to determine if an en passant move exists
/// in the current position.
///
/// * `board`: The board the game state will be updated on.
/// * `section`: Section 4 of the FEN strings that contains the en passant square information.
fn en_passant(board: &mut Board, section: &str) -> FenResult {
    // No en-passant square if the length of the pat is 1. The character should be a DASH.
    if_chain! {
       if section.len() == 1;
       if let Some(x) = section.chars().next();
       if x == DASH;
       then {
            return Ok(());
       }
    }

    if section.len() == 2 {
        let square = SQUARE_NAME.iter().position(|&element| element == section);
        match square {
            Some(sq)
                if ENPASSANT_SQUARES_WHITE.contains(&sq)
                    || ENPASSANT_SQUARES_BLACK.contains(&sq) =>
            {
                board.state.en_passant = Some(sq as u8);
                return Ok(());
            }
            _ => return Err(FenError::EnPassantSection),
        }
    }

    Err(FenError::EnPassantSection)
}

/// Parses the HalfMoveClockSection of the FEN string to determine the state of the half move clock
/// for determining drawn positions.
///
/// * `board`: The board the game state will be updated on.
/// * `section`: Section 5 of the FEN strings that contains the half move clock.
fn half_move_clock(board: &mut Board, section: &str) -> FenResult {
    if_chain! {
        if (1..=3).contains(&section.len());
        if let Ok(x) = section.parse::<u8>();
        if x <= MAX_MOVE_RULE;
        then {
            board.state.half_move_clock = x;
            return Ok(());
        }

    }

    Err(FenError::HalfMoveClockSection)
}

/// Parses the FullMoveSection of the FEN string to determine the total move counter.
///
/// A full move is one move by white and one move by black. The counter begins at 1.
///
/// * `board`: The board the game state will be updated on.
/// * `section`: Section 6 of the FEN strings that contains the move counter.
fn full_move_number(board: &mut Board, section: &str) -> FenResult {
    if_chain! {
        if !section.is_empty() && section.len() <= 4;
        if let Ok(x) = section.parse::<u16>();
        if x <= (MAX_GAME_MOVES as u16);
        then {
            board.state.full_move_number = x;
            return Ok(());
        }

    }

    Err(FenError::FullMoveSection)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fen_read_none_default_position() {
        let mut board = Board::new();
        let result = board.fen_read(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fen_read_default_position() {
        let mut board = Board::new();
        let result = board.fen_read(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fen_read_default_position_castling() {
        let mut board = Board::new();
        let result = board.fen_read(None);
        assert!(result.is_ok());
        assert_eq!(board.state.castling, Castling::ALL);
    }

    #[test]
    fn test_fen_read_default_position_color() {
        let mut board = Board::new();
        let result = board.fen_read(None);
        assert!(result.is_ok());
        assert_eq!(board.state.active_side, Sides::WHITE as u8);
    }

    #[test]
    fn test_fen_read_default_position_en_passant() {
        let mut board = Board::new();
        let result = board.fen_read(None);
        assert!(result.is_ok());
        assert_eq!(board.state.en_passant, None);
    }

    #[test]
    fn test_fen_read_default_position_half_move_clock() {
        let mut board = Board::new();
        let result = board.fen_read(None);
        assert!(result.is_ok());
        assert_eq!(board.state.half_move_clock, 0);
    }

    #[test]
    fn test_fen_read_default_position_full_move_counter() {
        let mut board = Board::new();
        let result = board.fen_read(None);
        assert!(result.is_ok());
        assert_eq!(board.state.full_move_number, 1);
    }

    #[test]
    fn test_fen_read_color_invalid() {
        let mut board = Board::new();
        let result = board.fen_read(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR a kq - 0 1",
        ));
        assert!(result.is_err());
        assert_eq!(result.err(), Some(FenError::ColorSection))
    }

    #[test]
    fn test_fen_read_color_black() {
        let mut board = Board::new();
        _ = board.fen_read(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b kq - 0 1",
        ));
        assert_eq!(board.state.active_side, Sides::BLACK as u8);
    }

    #[test]
    fn test_fen_read_black_only_castling() {
        let mut board = Board::new();
        _ = board.fen_read(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kq - 0 1",
        ));
        assert_eq!(board.state.castling, Castling::BK | Castling::BQ);
    }

    #[test]
    fn test_fen_read_white_only_castling() {
        let mut board = Board::new();
        _ = board.fen_read(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1",
        ));
        assert_eq!(board.state.castling, Castling::WK | Castling::WQ);
    }

    #[test]
    fn test_fen_read_mixed_castling() {
        let mut board = Board::new();
        _ = board.fen_read(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kq - 0 1",
        ));
        assert_eq!(board.state.castling, Castling::WK | Castling::BQ);
    }

    #[test]
    fn test_fen_read_en_passant() {
        let mut board = Board::new();
        _ = board.fen_read(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b kq a3 0 1",
        ));
        assert_eq!(board.state.en_passant, Some(Squares::A3 as u8));
    }

    #[test]
    fn test_fen_read_en_passant_invalid() {
        let mut board = Board::new();
        let result = board.fen_read(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b kq d5 0 1",
        ));
        assert!(result.is_err());
    }

    #[test]
    fn test_fen_read_half_move_clock() {
        let mut board = Board::new();
        _ = board.fen_read(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b kq - 25 1",
        ));
        assert_eq!(board.state.half_move_clock, 25);
    }

    #[test]
    fn test_fen_read_full_move_counter() {
        let mut board = Board::new();
        _ = board.fen_read(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b kq - 0 37",
        ));
        assert_eq!(board.state.full_move_number, 37);
    }
}
