use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex, Once},
    time::{Duration, Instant},
};

use crate::room::{Player, Room};

pub static ROOMS: LazyLock<Mutex<HashMap<String, Room>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const ROOM_CODE_CHARS: &[u8] = b"123456789ABCDEFGHJKMNPQRSTUVWXYZ";

const ROOM_CODE_LEN: usize = 4;

const EMPTY_ROOM_TTL: Duration = Duration::from_secs(5 * 60);

const SWEEP_INTERVAL: Duration = Duration::from_secs(60);

static SWEEPER: Once = Once::new();

fn generate_room_code() -> String {
    (0..ROOM_CODE_LEN)
        .map(|_| ROOM_CODE_CHARS[rand::random_range(0..ROOM_CODE_CHARS.len())] as char)
        .collect()
}

fn unique_room_code(rooms: &HashMap<String, Room>) -> String {
    loop {
        let code = generate_room_code();
        if !rooms.contains_key(&code) {
            return code;
        }
    }
}

fn sweep() {
    let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
    rooms.retain(|_, room| room.connected > 0 || room.last_active.elapsed() < EMPTY_ROOM_TTL);
}

fn spawn_sweeper() {
    SWEEPER.call_once(|| {
        tokio::spawn(async {
            loop {
                tokio::time::sleep(SWEEP_INTERVAL).await;
                sweep();
            }
        });
    });
}

pub fn insert_room(player_x: Player) -> String {
    spawn_sweeper();
    let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
    let code = unique_room_code(&rooms);
    rooms.insert(code.clone(), Room::new(player_x));
    code
}

// run `f` against the room, or None if the room doesn't exist
pub fn with_room<T>(room_id: &str, f: impl FnOnce(&mut Room) -> T) -> Option<T> {
    let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
    rooms.get_mut(room_id).map(|room| {
        room.last_active = Instant::now();
        f(room)
    })
}
