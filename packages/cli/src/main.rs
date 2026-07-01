use std::io;

use api::{Game, Mode, PlayerPick, State};

fn main() {
    let mode = loop {
        println!("Select mode Easy or Hard");
        let mut mode_input_line = String::new();
        io::stdin()
            .read_line(&mut mode_input_line)
            .expect("Failed to read");

        let mode = mode_input_line.trim().to_lowercase();
        match Mode::try_from(mode.as_str()) {
            Ok(mode) => break mode,
            Err(e) => println!("{}", e),
        }
    };

    let player_pick = loop {
        println!("Select X or O");
        let mut player_pick_input_line = String::new();
        io::stdin()
            .read_line(&mut player_pick_input_line)
            .expect("Failed to read");

        let player_pick = player_pick_input_line.trim().to_uppercase();
        match PlayerPick::try_from(player_pick.as_str()) {
            Ok(mode) => break mode,
            Err(e) => println!("{}", e),
        }
    };

    let mut game = Game::new(mode, player_pick);
    loop {
        println!("{}", game);

        let player_move = if game.is_player_turn() {
            let r = loop {
                println!("Pick a row...");
                let mut row_input_line = String::new();
                io::stdin()
                    .read_line(&mut row_input_line)
                    .expect("Failed to read");

                match row_input_line.trim().parse::<usize>() {
                    Ok(r) => break r,
                    Err(_) => println!("Must be a number!"),
                };
            };

            let c = loop {
                println!("Pick a column...");
                let mut column_input_line = String::new();
                io::stdin()
                    .read_line(&mut column_input_line)
                    .expect("Failed to read");

                match column_input_line.trim().parse::<usize>() {
                    Ok(c) => break c,
                    Err(_) => println!("Must be a number!"),
                };
            };

            Some((r, c))
        } else {
            None
        };

        match game.run(player_move) {
            State::GameOver(message) => {
                println!("{}", message);
                break;
            }
            State::InProgresss(message) => {
                println!("{}", message);
            }
        };

        println!()
    }
}
