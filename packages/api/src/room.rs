use tokio::sync::mpsc::UnboundedSender;

use crate::{GameError, GameStatus, ServerEvent};

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
    pub board: Vec<Vec<String>>,
    pub player_x: Player,
    pub player_o: Option<Player>,
    pub is_x_turn: bool,
}

impl Room {
    pub fn new(player_x: Player) -> Self {
        Self {
            board: vec![vec![String::from(" "); 3]; 3],
            player_x,
            player_o: None,
            is_x_turn: true,
        }
    }

    pub fn mark_of(&self, player_id: &str) -> Option<&'static str> {
        if player_id == self.player_x.id {
            return Some("X");
        }

        if let Some(player_o) = &self.player_o
            && player_o.id == player_id
        {
            return Some("O");
        }

        None
    }

    // swap stored tx for that player (reconnect + first game-page connect, same path)
    pub fn attach(&mut self, mark: &str, tx: UnboundedSender<ServerEvent>) {
        match mark {
            "X" => self.player_x.tx = tx,
            _ => {
                if let Some(player_o) = self.player_o.as_mut() {
                    player_o.tx = tx;
                }
            }
        }
    }

    pub fn set_name(&mut self, mark: &str, name: String) {
        match mark {
            "X" => self.player_x.name = Some(name),
            _ => {
                if let Some(player_o) = self.player_o.as_mut() {
                    player_o.name = Some(name);
                }
            }
        }
    }

    pub fn apply_move(&mut self, mark: &str, r: usize, c: usize) -> Result<(), GameError> {
        if self.status() != GameStatus::InProgress {
            return Err(GameError::InvalidMove(r, c));
        }

        if self.is_x_turn != (mark == "X") {
            return Err(GameError::InvalidMove(r, c));
        }

        if r >= 3 || c >= 3 {
            return Err(GameError::InvalidMove(r, c));
        }

        if self.board[r][c] != " " {
            return Err(GameError::InvalidMove(r, c));
        }

        self.board[r][c] = mark.to_string();
        self.is_x_turn = !self.is_x_turn;
        Ok(())
    }

    pub fn status(&self) -> GameStatus {
        if self.win("X") {
            return GameStatus::Won {
                mark: "X".to_string(),
            };
        }

        if self.win("O") {
            return GameStatus::Won {
                mark: "O".to_string(),
            };
        }

        if self.is_full() {
            return GameStatus::Draw;
        }

        GameStatus::InProgress
    }

    fn is_full(&self) -> bool {
        for r in 0..3 {
            for c in 0..3 {
                if self.board[r][c] == " " {
                    return false;
                }
            }
        }
        true
    }

    pub fn win(&self, value: &str) -> bool {
        [
            // Columns
            [&self.board[0][0], &self.board[1][0], &self.board[2][0]],
            [&self.board[0][1], &self.board[1][1], &self.board[2][1]],
            [&self.board[0][2], &self.board[1][2], &self.board[2][2]],
            // Rows
            [&self.board[0][0], &self.board[0][1], &self.board[0][2]],
            [&self.board[1][0], &self.board[1][1], &self.board[1][2]],
            [&self.board[2][0], &self.board[2][1], &self.board[2][2]],
            // Diagonals
            [&self.board[0][0], &self.board[1][1], &self.board[2][2]],
            [&self.board[0][2], &self.board[1][1], &self.board[2][0]],
        ]
        .iter()
        .any(|combo| combo.iter().all(|&cell| cell == value))
    }

    pub fn snapshot(&self) -> ServerEvent {
        ServerEvent::GameState {
            board: self.board.clone(),
            is_x_turn: self.is_x_turn,
            status: self.status(),
            player_x_name: self.player_x.name.clone(),
            player_o_name: self.player_o.as_ref().and_then(|p| p.name.clone()),
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

    fn set_board(room: &mut Room, rows: [[&str; 3]; 3]) {
        for r in 0..3 {
            for c in 0..3 {
                room.board[r][c] = rows[r][c].to_string();
            }
        }
    }

    #[test]
    fn mark_of_finds_members_and_rejects_stranger() {
        let (room, _rx_x, _rx_o) = full_room();
        assert_eq!(room.mark_of("xid"), Some("X"));
        assert_eq!(room.mark_of("oid"), Some("O"));
        assert_eq!(room.mark_of("nope"), None);
    }

    #[test]
    fn apply_move_places_mark_and_flips_turn() {
        let (mut room, _rx_x, _rx_o) = full_room();
        assert!(room.apply_move("X", 0, 0).is_ok());
        assert_eq!(room.board[0][0], "X");
        assert!(!room.is_x_turn);
        assert!(room.apply_move("O", 1, 1).is_ok());
        assert_eq!(room.board[1][1], "O");
        assert!(room.is_x_turn);
    }

    #[test]
    fn apply_move_rejects_out_of_turn() {
        let (mut room, _rx_x, _rx_o) = full_room();
        assert!(room.apply_move("O", 0, 0).is_err());
        assert_eq!(room.board[0][0], " ");
        assert!(room.apply_move("X", 0, 0).is_ok());
        assert!(room.apply_move("X", 0, 1).is_err());
    }

    #[test]
    fn apply_move_rejects_occupied_and_out_of_bounds() {
        let (mut room, _rx_x, _rx_o) = full_room();
        assert!(room.apply_move("X", 3, 0).is_err());
        assert!(room.apply_move("X", 0, 3).is_err());
        assert!(room.apply_move("X", 0, 0).is_ok());
        assert!(room.apply_move("O", 0, 0).is_err());
    }

    #[test]
    fn apply_move_rejects_after_game_over() {
        let (mut room, _rx_x, _rx_o) = full_room();
        // X wins top row
        assert!(room.apply_move("X", 0, 0).is_ok());
        assert!(room.apply_move("O", 1, 0).is_ok());
        assert!(room.apply_move("X", 0, 1).is_ok());
        assert!(room.apply_move("O", 1, 1).is_ok());
        assert!(room.apply_move("X", 0, 2).is_ok());
        assert_eq!(
            room.status(),
            GameStatus::Won {
                mark: "X".to_string()
            }
        );
        assert!(room.apply_move("O", 2, 2).is_err());
    }

    #[test]
    fn status_reports_win_on_full_board() {
        let (mut room, _rx_x, _rx_o) = full_room();
        set_board(
            &mut room,
            [["X", "X", "X"], ["O", "O", "X"], ["O", "X", "O"]],
        );
        assert_eq!(
            room.status(),
            GameStatus::Won {
                mark: "X".to_string()
            }
        );
    }

    #[test]
    fn status_draw_on_full_board_without_winner() {
        let (mut room, _rx_x, _rx_o) = full_room();
        set_board(
            &mut room,
            [["X", "O", "X"], ["X", "O", "O"], ["O", "X", "X"]],
        );
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
        room.attach("X", tx_new);
        room.player_x.tx.send(ServerEvent::InvalidMove).unwrap();
        assert!(matches!(rx_new.try_recv(), Ok(ServerEvent::InvalidMove)));
        assert!(rx_x_old.try_recv().is_err());
    }

    #[test]
    fn set_name_updates_right_player() {
        let (mut room, _rx_x, _rx_o) = full_room();
        room.set_name("X", "Xena".to_string());
        room.set_name("O", "Omar".to_string());
        assert_eq!(room.player_x.name.as_deref(), Some("Xena"));
        assert_eq!(
            room.player_o.as_ref().unwrap().name.as_deref(),
            Some("Omar")
        );
    }

    #[test]
    fn snapshot_contains_board_turn_status_and_names() {
        let (mut room, _rx_x, _rx_o) = full_room();
        room.apply_move("X", 1, 1).unwrap();
        match room.snapshot() {
            ServerEvent::GameState {
                board,
                is_x_turn,
                status,
                player_x_name,
                player_o_name,
            } => {
                assert_eq!(board[1][1], "X");
                assert!(!is_x_turn);
                assert_eq!(status, GameStatus::InProgress);
                assert_eq!(player_x_name.as_deref(), Some("Ana"));
                assert_eq!(player_o_name.as_deref(), Some("Bo"));
            }
            other => panic!("expected GameState, got {other:?}"),
        }
    }

    #[test]
    fn broadcast_sends_to_both_players() {
        let (room, mut rx_x, mut rx_o) = full_room();
        room.broadcast(ServerEvent::InvalidMove);
        assert!(matches!(rx_x.try_recv(), Ok(ServerEvent::InvalidMove)));
        assert!(matches!(rx_o.try_recv(), Ok(ServerEvent::InvalidMove)));
    }
}
