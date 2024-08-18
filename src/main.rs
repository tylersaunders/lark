mod board;
mod engine;
mod evaluation;
mod movegen;
mod utils;

use movegen::{defs::Move, MoveGenerator};

use crate::board::Board;

fn main() {
    let mut board = Board::new();

    _ = board.fen_read(None);

    println!("{board}");
    println!("   {}", board.state);
    println!("   evaluation {}", evaluation::evaluate_position(&board));

    let mut move_list: Vec<Move> = Vec::new();
    let move_gen = MoveGenerator::new();
    move_gen.generate_moves(&board, &mut move_list);

    println!("Possible Moves for White:");

    for mv in move_list.iter() {
        println!();
        print!("{mv}");
    }
}
