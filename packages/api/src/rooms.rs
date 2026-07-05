#[cfg(feature = "server")]
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

#[cfg(feature = "server")]
use tokio::sync::mpsc::UnboundedSender;

#[cfg(feature = "server")]
use crate::ServerEvent;

#[cfg(feature = "server")]
pub struct Player {
    pub id: String,
    pub name: Option<String>,
    pub tx: UnboundedSender<ServerEvent>,
}

#[cfg(feature = "server")]
impl Player {
    pub fn new(id: String, name: Option<String>, tx: UnboundedSender<ServerEvent>) -> Self {
        Self { id, name, tx }
    }
}

#[cfg(feature = "server")]
pub struct Room {
    pub board: Vec<Vec<String>>,
    pub player_x: Player,
    pub player_o: Option<Player>,
    pub is_x_turn: bool,
}

#[cfg(feature = "server")]
impl Room {
    pub fn new(player_x: Player) -> Self {
        Self {
            board: vec![vec![String::from(" "); 3]; 3],
            player_x,
            player_o: None,
            is_x_turn: true,
        }
    }
}

#[cfg(feature = "server")]
pub static ROOMS: LazyLock<Mutex<HashMap<String, Room>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(feature = "server")]
const ROOM_CODE_CHARS: &[u8] = b"123456789ABCDEFGHJKMNPQRSTUVWXYZ";

#[cfg(feature = "server")]
const ROOM_CODE_LEN: usize = 4;

#[cfg(feature = "server")]
fn generate_room_code() -> String {
    (0..ROOM_CODE_LEN)
        .map(|_| ROOM_CODE_CHARS[rand::random_range(0..ROOM_CODE_CHARS.len())] as char)
        .collect()
}

#[cfg(feature = "server")]
pub fn unique_room_code(rooms: &HashMap<String, Room>) -> String {
    loop {
        let code = generate_room_code();
        if !rooms.contains_key(&code) {
            return code;
        }
    }
}
