use crate::{Mark, Mode};

/// Cells indexed 0..9, row-major: `i / 3` = row, `i % 3` = col.
pub type Board = [Option<Mark>; 9];

const LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

pub fn winner(board: &Board) -> Option<Mark> {
    LINES
        .iter()
        .find_map(|line| board[line[0]].filter(|&m| line.iter().all(|&i| board[i] == Some(m))))
}

pub fn is_full(board: &Board) -> bool {
    board.iter().all(|cell| cell.is_some())
}

pub fn best_move(board: &Board, cpu: Mark) -> Option<usize> {
    if winner(board).is_some() {
        return None;
    }

    let mut best: Option<(usize, i32)> = None;
    for i in 0..9 {
        if board[i].is_none() {
            let mut next = *board;
            next[i] = Some(cpu);
            let score = minimax(&next, cpu, cpu.other(), 1);
            if best.is_none_or(|(_, s)| score > s) {
                best = Some((i, score));
            }
        }
    }
    best.map(|(i, _)| i)
}

fn minimax(board: &Board, cpu: Mark, to_move: Mark, depth: i32) -> i32 {
    if let Some(w) = winner(board) {
        return if w == cpu { 10 - depth } else { depth - 10 };
    }
    if is_full(board) {
        return 0;
    }

    let mut best = if to_move == cpu { i32::MIN } else { i32::MAX };
    for i in 0..9 {
        if board[i].is_none() {
            let mut next = *board;
            next[i] = Some(to_move);
            let score = minimax(&next, cpu, to_move.other(), depth + 1);
            best = if to_move == cpu {
                best.max(score)
            } else {
                best.min(score)
            };
        }
    }
    best
}

pub fn play_vs_cpu(board: &mut Board, cell: usize, human: Mark, mode: Mode, rng: &mut u64) -> bool {
    if cell >= 9 || board[cell].is_some() || winner(board).is_some() {
        return false;
    }

    board[cell] = Some(human);
    if winner(board).is_some() || is_full(board) {
        return true;
    }

    let cpu = human.other();
    let reply = match mode {
        Mode::Easy => random_move(board, rng),
        Mode::Hard => best_move(board, cpu),
    };
    if let Some(reply) = reply {
        board[reply] = Some(cpu);
    }
    true
}

/// Uniform-ish random empty cell using a caller-owned xorshift64 state.
/// No `rand` dependency so the wasm build stays lean.
pub fn random_move(board: &Board, state: &mut u64) -> Option<usize> {
    let empties: Vec<usize> = (0..9).filter(|&i| board[i].is_none()).collect();
    if empties.is_empty() {
        return None;
    }

    // xorshift64 fixes on 0, so nudge a zero seed onto the cycle
    if *state == 0 {
        *state = 0x9E37_79B9_7F4A_7C15;
    }
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;

    Some(empties[(*state % empties.len() as u64) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(s: &str) -> Board {
        let mut b: Board = [None; 9];
        for (i, ch) in s.chars().enumerate() {
            b[i] = match ch {
                'X' => Some(Mark::X),
                'O' => Some(Mark::O),
                _ => None,
            };
        }
        b
    }

    #[test]
    fn winner_detects_row_col_diag_and_none() {
        assert_eq!(winner(&board("XXX      ")), Some(Mark::X));
        assert_eq!(winner(&board("O  O  O  ")), Some(Mark::O));
        assert_eq!(winner(&board("X   X   X")), Some(Mark::X));
        assert_eq!(winner(&board("  O O O  ")), Some(Mark::O));
        assert_eq!(winner(&board("XOX      ")), None);
        assert_eq!(winner(&board("         ")), None);
    }

    #[test]
    fn is_full_only_when_no_empty_cells() {
        assert!(is_full(&board("XOXXOXOXO")));
        assert!(!is_full(&board("XOXXOXOX ")));
        assert!(!is_full(&board("         ")));
    }

    #[test]
    fn best_move_takes_immediate_win() {
        let b = board("XX OO    ");
        assert_eq!(best_move(&b, Mark::X), Some(2));
    }

    #[test]
    fn best_move_blocks_opponent_win() {
        let b = board("XX  O    ");
        assert_eq!(best_move(&b, Mark::O), Some(2));
    }

    #[test]
    fn best_move_none_when_game_over_or_full() {
        assert_eq!(best_move(&board("XXXOO    "), Mark::O), None);
        assert_eq!(best_move(&board("XOXXOXOXO"), Mark::X), None);
    }

    #[test]
    fn best_move_prefers_faster_win() {
        let b = board("XX OOX   ");
        assert_eq!(best_move(&b, Mark::X), Some(2));
    }

    #[test]
    fn cpu_never_loses_exhaustively() {
        fn play(b: Board, cpu: Mark, to_move: Mark) {
            if let Some(w) = winner(&b) {
                assert_ne!(w, cpu.other(), "cpu lost:\n{b:?}");
                return;
            }
            if is_full(&b) {
                return;
            }
            if to_move == cpu {
                let m = best_move(&b, cpu).expect("cpu has a move");
                let mut next = b;
                assert!(next[m].is_none(), "cpu picked occupied cell {m}");
                next[m] = Some(cpu);
                play(next, cpu, cpu.other());
            } else {
                for i in 0..9 {
                    if b[i].is_none() {
                        let mut next = b;
                        next[i] = Some(cpu.other());
                        play(next, cpu, cpu);
                    }
                }
            }
        }

        play([None; 9], Mark::X, Mark::X);
        play([None; 9], Mark::O, Mark::X);
    }

    #[test]
    fn play_vs_cpu_applies_human_move_and_cpu_reply() {
        let mut b = board("         ");
        let mut rng = 7u64;
        assert!(play_vs_cpu(&mut b, 4, Mark::X, Mode::Hard, &mut rng));
        assert_eq!(b[4], Some(Mark::X));
        assert_eq!(b.iter().filter(|c| **c == Some(Mark::O)).count(), 1);
    }

    #[test]
    fn play_vs_cpu_hard_blocks_immediate_threat() {
        let mut b = board("XX    O  ");
        let mut rng = 7u64;
        assert!(play_vs_cpu(&mut b, 3, Mark::X, Mode::Hard, &mut rng));
        assert_eq!(b[2], Some(Mark::O));
    }

    #[test]
    fn play_vs_cpu_rejects_illegal_moves() {
        let mut b = board("X        ");
        let before = b;
        let mut rng = 7u64;
        assert!(!play_vs_cpu(&mut b, 0, Mark::X, Mode::Hard, &mut rng)); // occupied
        assert!(!play_vs_cpu(&mut b, 9, Mark::X, Mode::Hard, &mut rng)); // out of range
        assert_eq!(b, before, "board must be untouched on rejection");

        let mut over = board("OOO XX   ");
        let before_over = over;
        assert!(!play_vs_cpu(&mut over, 5, Mark::X, Mode::Hard, &mut rng)); // game already won
        assert_eq!(over, before_over);
    }

    #[test]
    fn play_vs_cpu_no_reply_when_human_ends_game() {
        let mut b = board("XX OO    ");
        let mut rng = 7u64;
        assert!(play_vs_cpu(&mut b, 2, Mark::X, Mode::Hard, &mut rng));
        assert_eq!(winner(&b), Some(Mark::X));
        assert_eq!(b.iter().filter(|c| **c == Some(Mark::O)).count(), 2);

        let mut d = board("XOXXOOOX ");
        assert!(play_vs_cpu(&mut d, 8, Mark::X, Mode::Easy, &mut rng));
        assert!(is_full(&d));
    }

    #[test]
    fn play_vs_cpu_easy_replies_on_empty_cell() {
        let mut b = board("         ");
        let mut rng = 0u64; // zero seed must still work
        assert!(play_vs_cpu(&mut b, 0, Mark::X, Mode::Easy, &mut rng));
        assert_eq!(b[0], Some(Mark::X));
        assert_eq!(b.iter().filter(|c| **c == Some(Mark::O)).count(), 1);
        assert_ne!(rng, 0, "rng state must advance");
    }

    #[test]
    fn random_move_returns_empty_cell_and_none_on_full() {
        let b = board("XOXXOXOX ");
        let mut state = 42u64;
        assert_eq!(random_move(&b, &mut state), Some(8));
        assert_eq!(random_move(&board("XOXXOXOXO"), &mut state), None);
    }

    #[test]
    fn random_move_varies_and_recovers_from_zero_seed() {
        let b = board("         ");
        let mut state = 0u64;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..100 {
            let m = random_move(&b, &mut state).unwrap();
            assert!(b[m].is_none());
            seen.insert(m);
        }
        assert!(seen.len() > 1, "rng stuck on one cell");
    }
}
