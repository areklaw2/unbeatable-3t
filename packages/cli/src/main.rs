use std::io::{self, BufRead, Write};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use api::engine::{self, Board};
use api::{Mark, Mode};

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let mode = loop {
        prompt("Select mode (easy/hard): ");
        match read_line(&mut lines).trim().to_lowercase().as_str() {
            "easy" => break Mode::Easy,
            "hard" => break Mode::Hard,
            _ => println!("Enter easy or hard."),
        }
    };

    let human = loop {
        prompt("Select your mark (X/O): ");
        match read_line(&mut lines).trim().to_uppercase().as_str() {
            "X" => break Mark::X,
            "O" => break Mark::O,
            _ => println!("Enter X or O."),
        }
    };

    println!(
        "You are {}, CPU is {}.",
        human.label(),
        human.other().label()
    );

    let mut board: Board = [None; 9];
    let mut rng = seed();

    if human == Mark::O {
        let opening = match mode {
            Mode::Easy => xorshift(&mut rng) as usize % 9,
            Mode::Hard => 4,
        };
        board[opening] = Some(Mark::X);
        println!("CPU (X) opens.");
    }

    loop {
        render(&board);
        let cell = loop {
            let row = read_coord(&mut lines, "Row (0-2): ");
            let col = read_coord(&mut lines, "Col (0-2): ");
            let cell = row * 3 + col;
            if board[cell].is_some() {
                println!("That cell is taken.");
            } else {
                break cell;
            }
        };

        engine::play_vs_cpu(&mut board, cell, human, mode, &mut rng);

        if let Some(w) = engine::winner(&board) {
            render(&board);
            if w == human {
                println!("You win!");
            } else {
                println!("CPU wins!");
            }
            break;
        }
        if engine::is_full(&board) {
            render(&board);
            println!("Draw!");
            break;
        }
    }
}

fn seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        .max(1)
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn prompt(text: &str) {
    print!("{text}");
    io::stdout().flush().expect("Failed to flush stdout");
}

fn read_line(lines: &mut impl Iterator<Item = io::Result<String>>) -> String {
    match lines.next() {
        Some(line) => line.expect("Failed to read input"),
        None => process::exit(0),
    }
}

fn read_coord(lines: &mut impl Iterator<Item = io::Result<String>>, label: &str) -> usize {
    loop {
        prompt(label);
        match read_line(lines).trim().parse::<usize>() {
            Ok(n) if n <= 2 => break n,
            _ => println!("Enter a number from 0 to 2."),
        }
    }
}

fn render(board: &Board) {
    for row in 0..3 {
        let cells: Vec<&str> = (0..3)
            .map(|col| board[row * 3 + col].map_or(".", Mark::label))
            .collect();
        println!("{}", cells.join(" "));
    }
}
