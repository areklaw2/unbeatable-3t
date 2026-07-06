use std::time::Instant;

use tokio::sync::mpsc::UnboundedSender;

use crate::engine::{self, Board};
use crate::{GameError, GameStatus, Mark, ServerEvent};

pub struct Player {
    pub id: String,
    pub name: Option<String>,
    pub tx: UnboundedSender<ServerEvent>,
}

impl Player {
    pub fn new(id: String, name: Option<String>, tx: UnboundedSender<ServerEvent>) -> Self {
        Self { id, name, tx }
    }
}

pub struct Room {
    pub board: Board,
    pub player_x: Player,
    pub player_o: Option<Player>,
    pub is_x_turn: bool,
    pub connected: u32,
    pub last_active: Instant,
    pub wins_x: u32,
    pub wins_o: u32,
    pub ties: u32,
    rematch_x: bool,
    rematch_o: bool,
}

impl Room {
    pub fn new(player_x: Player) -> Self {
        Self {
            board: [None; 9],
            player_x,
            player_o: None,
            is_x_turn: true,
            connected: 1,
            last_active: Instant::now(),
            wins_x: 0,
            wins_o: 0,
            ties: 0,
            rematch_x: false,
            rematch_o: false,
        }
    }

    pub fn player(&self, mark: Mark) -> Option<&Player> {
        match mark {
            Mark::X => Some(&self.player_x),
            Mark::O => self.player_o.as_ref(),
        }
    }

    pub fn mark_of(&self, player_id: &str) -> Option<Mark> {
        if player_id == self.player_x.id {
            return Some(Mark::X);
        }

        if let Some(player_o) = &self.player_o
            && player_o.id == player_id
        {
            return Some(Mark::O);
        }

        None
    }

    // swap stored tx for that player (reconnect + first game-page connect, same path)
    pub fn attach(&mut self, mark: Mark, tx: UnboundedSender<ServerEvent>) {
        match mark {
            Mark::X => self.player_x.tx = tx,
            Mark::O => {
                if let Some(player_o) = self.player_o.as_mut() {
                    player_o.tx = tx;
                }
            }
        }
    }

    pub fn set_name(&mut self, mark: Mark, name: String) {
        match mark {
            Mark::X => self.player_x.name = Some(name),
            Mark::O => {
                if let Some(player_o) = self.player_o.as_mut() {
                    player_o.name = Some(name);
                }
            }
        }
    }

    pub fn send_to(&self, mark: Mark, event: ServerEvent) {
        if let Some(player) = self.player(mark) {
            let _ = player.tx.send(event);
        }
    }

    pub fn request_rematch(&mut self, mark: Mark) -> bool {
        if self.status() == GameStatus::InProgress {
            return false;
        }

        match mark {
            Mark::X => self.rematch_x = true,
            Mark::O => self.rematch_o = true,
        }

        if self.rematch_x && self.rematch_o {
            self.board = [None; 9];
            self.is_x_turn = true;
            self.rematch_x = false;
            self.rematch_o = false;
            return true;
        }

        false
    }

    pub fn apply_move(&mut self, mark: Mark, cell: usize) -> Result<(), GameError> {
        if self.status() != GameStatus::InProgress {
            return Err(GameError::InvalidMove(cell / 3, cell % 3));
        }

        if self.is_x_turn != (mark == Mark::X) {
            return Err(GameError::InvalidMove(cell / 3, cell % 3));
        }

        if cell >= 9 || self.board[cell].is_some() {
            return Err(GameError::InvalidMove(cell / 3, cell % 3));
        }

        self.board[cell] = Some(mark);
        self.is_x_turn = !self.is_x_turn;
        match self.status() {
            GameStatus::Won { mark: Mark::X } => self.wins_x += 1,
            GameStatus::Won { mark: Mark::O } => self.wins_o += 1,
            GameStatus::Draw => self.ties += 1,
            GameStatus::InProgress => {}
        }
        Ok(())
    }

    pub fn status(&self) -> GameStatus {
        if let Some(mark) = engine::winner(&self.board) {
            return GameStatus::Won { mark };
        }

        if engine::is_full(&self.board) {
            return GameStatus::Draw;
        }

        GameStatus::InProgress
    }

    pub fn snapshot(&self) -> ServerEvent {
        ServerEvent::GameState {
            board: self.board,
            is_x_turn: self.is_x_turn,
            status: self.status(),
            player_x_name: self.player_x.name.clone(),
            player_o_name: self.player_o.as_ref().and_then(|p| p.name.clone()),
            wins_x: self.wins_x,
            wins_o: self.wins_o,
            ties: self.ties,
        }
    }

    pub fn broadcast(&self, event: ServerEvent) {
        let _ = self.player_x.tx.send(event.clone());
        if let Some(player_o) = &self.player_o {
            let _ = player_o.tx.send(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

    fn player(id: &str, name: Option<&str>) -> (Player, UnboundedReceiver<ServerEvent>) {
        let (tx, rx) = unbounded_channel();
        (
            Player::new(id.to_string(), name.map(str::to_string), tx),
            rx,
        )
    }

    fn full_room() -> (
        Room,
        UnboundedReceiver<ServerEvent>,
        UnboundedReceiver<ServerEvent>,
    ) {
        let (x, rx_x) = player("xid", Some("Ana"));
        let (o, rx_o) = player("oid", Some("Bo"));
        let mut room = Room::new(x);
        room.player_o = Some(o);
        (room, rx_x, rx_o)
    }

    fn set_board(room: &mut Room, cells: &str) {
        for (i, ch) in cells.chars().enumerate() {
            room.board[i] = match ch {
                'X' => Some(Mark::X),
                'O' => Some(Mark::O),
                _ => None,
            };
        }
    }

    #[test]
    fn mark_of_finds_members_and_rejects_stranger() {
        let (room, _rx_x, _rx_o) = full_room();
        assert_eq!(room.mark_of("xid"), Some(Mark::X));
        assert_eq!(room.mark_of("oid"), Some(Mark::O));
        assert_eq!(room.mark_of("nope"), None);
    }

    #[test]
    fn apply_move_places_mark_and_flips_turn() {
        let (mut room, _rx_x, _rx_o) = full_room();
        assert!(room.apply_move(Mark::X, 0).is_ok());
        assert_eq!(room.board[0], Some(Mark::X));
        assert!(!room.is_x_turn);
        assert!(room.apply_move(Mark::O, 4).is_ok());
        assert_eq!(room.board[4], Some(Mark::O));
        assert!(room.is_x_turn);
    }

    #[test]
    fn apply_move_rejects_out_of_turn() {
        let (mut room, _rx_x, _rx_o) = full_room();
        assert!(room.apply_move(Mark::O, 0).is_err());
        assert_eq!(room.board[0], None);
        assert!(room.apply_move(Mark::X, 0).is_ok());
        assert!(room.apply_move(Mark::X, 1).is_err());
    }

    #[test]
    fn apply_move_rejects_occupied_and_out_of_bounds() {
        let (mut room, _rx_x, _rx_o) = full_room();
        assert!(room.apply_move(Mark::X, 9).is_err());
        assert!(room.apply_move(Mark::X, 0).is_ok());
        assert!(room.apply_move(Mark::O, 0).is_err());
    }

    #[test]
    fn apply_move_rejects_after_game_over() {
        let (mut room, _rx_x, _rx_o) = full_room();
        // X wins top row
        assert!(room.apply_move(Mark::X, 0).is_ok());
        assert!(room.apply_move(Mark::O, 3).is_ok());
        assert!(room.apply_move(Mark::X, 1).is_ok());
        assert!(room.apply_move(Mark::O, 4).is_ok());
        assert!(room.apply_move(Mark::X, 2).is_ok());
        assert_eq!(room.status(), GameStatus::Won { mark: Mark::X });
        assert!(room.apply_move(Mark::O, 8).is_err());
    }

    #[test]
    fn status_reports_win_on_full_board() {
        let (mut room, _rx_x, _rx_o) = full_room();
        set_board(&mut room, "XXXOOXOXO");
        assert_eq!(room.status(), GameStatus::Won { mark: Mark::X });
    }

    #[test]
    fn status_draw_on_full_board_without_winner() {
        let (mut room, _rx_x, _rx_o) = full_room();
        set_board(&mut room, "XOXXOOOXX");
        assert_eq!(room.status(), GameStatus::Draw);
    }

    #[test]
    fn status_in_progress_on_fresh_board() {
        let (room, _rx_x, _rx_o) = full_room();
        assert_eq!(room.status(), GameStatus::InProgress);
    }

    #[test]
    fn attach_swaps_sender() {
        let (mut room, mut rx_x_old, _rx_o) = full_room();
        let (tx_new, mut rx_new) = unbounded_channel();
        room.attach(Mark::X, tx_new);
        room.player_x.tx.send(ServerEvent::InvalidMove).unwrap();
        assert!(matches!(rx_new.try_recv(), Ok(ServerEvent::InvalidMove)));
        assert!(rx_x_old.try_recv().is_err());
    }

    #[test]
    fn set_name_updates_right_player() {
        let (mut room, _rx_x, _rx_o) = full_room();
        room.set_name(Mark::X, "Xena".to_string());
        room.set_name(Mark::O, "Omar".to_string());
        assert_eq!(room.player_x.name.as_deref(), Some("Xena"));
        assert_eq!(
            room.player_o.as_ref().unwrap().name.as_deref(),
            Some("Omar")
        );
    }

    #[test]
    fn snapshot_contains_board_turn_status_and_names() {
        let (mut room, _rx_x, _rx_o) = full_room();
        room.apply_move(Mark::X, 4).unwrap();
        match room.snapshot() {
            ServerEvent::GameState {
                board,
                is_x_turn,
                status,
                player_x_name,
                player_o_name,
                ..
            } => {
                assert_eq!(board[4], Some(Mark::X));
                assert!(!is_x_turn);
                assert_eq!(status, GameStatus::InProgress);
                assert_eq!(player_x_name.as_deref(), Some("Ana"));
                assert_eq!(player_o_name.as_deref(), Some("Bo"));
            }
            other => panic!("expected GameState, got {other:?}"),
        }
    }

    #[test]
    fn request_rematch_needs_both_players_then_resets() {
        let (mut room, _rx_x, _rx_o) = full_room();
        set_board(&mut room, "XXXOOXOXO");
        assert!(!room.request_rematch(Mark::X));
        assert_eq!(room.status(), GameStatus::Won { mark: Mark::X });
        assert!(room.request_rematch(Mark::O));
        assert_eq!(room.board, [None; 9]);
        assert!(room.is_x_turn);
        assert_eq!(room.status(), GameStatus::InProgress);
    }

    fn play_x_win_top_row(room: &mut Room) {
        room.apply_move(Mark::X, 0).unwrap();
        room.apply_move(Mark::O, 3).unwrap();
        room.apply_move(Mark::X, 1).unwrap();
        room.apply_move(Mark::O, 4).unwrap();
        room.apply_move(Mark::X, 2).unwrap();
    }

    fn play_draw(room: &mut Room) {
        for (mark, cell) in [
            (Mark::X, 0),
            (Mark::O, 1),
            (Mark::X, 2),
            (Mark::O, 4),
            (Mark::X, 3),
            (Mark::O, 5),
            (Mark::X, 7),
            (Mark::O, 6),
            (Mark::X, 8),
        ] {
            room.apply_move(mark, cell).unwrap();
        }
    }

    #[test]
    fn win_increments_winner_score() {
        let (mut room, _rx_x, _rx_o) = full_room();
        assert_eq!((room.wins_x, room.wins_o, room.ties), (0, 0, 0));
        play_x_win_top_row(&mut room);
        assert_eq!((room.wins_x, room.wins_o, room.ties), (1, 0, 0));
    }

    #[test]
    fn draw_increments_tie_score() {
        let (mut room, _rx_x, _rx_o) = full_room();
        play_draw(&mut room);
        assert_eq!(room.status(), GameStatus::Draw);
        assert_eq!((room.wins_x, room.wins_o, room.ties), (0, 0, 1));
    }

    #[test]
    fn scores_persist_across_rematch_and_keep_counting() {
        let (mut room, _rx_x, _rx_o) = full_room();
        play_x_win_top_row(&mut room);
        room.request_rematch(Mark::X);
        assert!(room.request_rematch(Mark::O));
        assert_eq!((room.wins_x, room.wins_o, room.ties), (1, 0, 0));
        play_draw(&mut room);
        assert_eq!((room.wins_x, room.wins_o, room.ties), (1, 0, 1));
    }

    #[test]
    fn snapshot_contains_scores() {
        let (mut room, _rx_x, _rx_o) = full_room();
        play_x_win_top_row(&mut room);
        match room.snapshot() {
            ServerEvent::GameState {
                wins_x,
                wins_o,
                ties,
                ..
            } => {
                assert_eq!((wins_x, wins_o, ties), (1, 0, 0));
            }
            other => panic!("expected GameState, got {other:?}"),
        }
    }

    #[test]
    fn request_rematch_rejected_mid_game() {
        let (mut room, _rx_x, _rx_o) = full_room();
        assert!(!room.request_rematch(Mark::X));
        assert!(!room.request_rematch(Mark::O));
        assert_eq!(room.status(), GameStatus::InProgress);
    }

    #[test]
    fn rematch_flags_clear_after_reset() {
        let (mut room, _rx_x, _rx_o) = full_room();
        set_board(&mut room, "XXXOOXOXO");
        room.request_rematch(Mark::X);
        assert!(room.request_rematch(Mark::O));
        set_board(&mut room, "XXXOOXOXO");
        assert!(!room.request_rematch(Mark::X));
    }

    #[test]
    fn send_to_targets_single_player() {
        let (room, mut rx_x, mut rx_o) = full_room();
        room.send_to(Mark::O, ServerEvent::InvalidMove);
        assert!(rx_x.try_recv().is_err());
        assert!(matches!(rx_o.try_recv(), Ok(ServerEvent::InvalidMove)));
    }

    #[test]
    fn broadcast_sends_to_both_players() {
        let (room, mut rx_x, mut rx_o) = full_room();
        room.broadcast(ServerEvent::InvalidMove);
        assert!(matches!(rx_x.try_recv(), Ok(ServerEvent::InvalidMove)));
        assert!(matches!(rx_o.try_recv(), Ok(ServerEvent::InvalidMove)));
    }
}
