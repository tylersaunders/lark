use crate::board::{defs::Sides, Board};

/// Calculates an evaluation of the position from the current side to move's point of view. A
/// positive value indicates that the current side to move is better, a negative value that the
/// opponent's side is better.
///
/// Currently this is just a simple count of all the material on the board.
///
/// * `board`: The board to evaluate.
pub fn evaluate_position(board: &Board) -> i16 {
    let side = board.state.active_side as usize;

    // Start by calculating the evaluation from White's point of view.
    let mut value: i16 = (board.state.material[Sides::WHITE] - board.state.material[Sides::BLACK])
        .try_into()
        .unwrap();

    // If it is black to move, flip the value before it is returned.
    value = if side == Sides::BLACK { -value } else { value };

    value
}

