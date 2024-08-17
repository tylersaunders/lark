mod board;
mod engine;
mod utils;

use crate::board::Board;

fn main() {
    let mut board = Board::new();

    // Create a new Board with the default position and print it.
    match board.fen_read(None) {
        Ok(()) => (),
        Err(error) => panic!("{error}"),
    }
    print!("{board}");
    println!();
    print!("   {}", board.state);
    println!();
}
