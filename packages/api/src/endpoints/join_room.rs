use dioxus::{
    fullstack::{CborEncoding, WebSocketOptions, Websocket},
    prelude::*,
};

use crate::{ClientEvent, ServerEvent};

#[cfg(feature = "server")]
use crate::{Mark, endpoints::GameSocket, registry::with_room, room::Player};

#[cfg(feature = "server")]
use tokio::sync::mpsc;

#[cfg(feature = "server")]
use ulid::Ulid;

#[get("/api/join_room_ws?room_id&name")]
pub async fn join_room_ws(
    room_id: String,
    name: Option<String>,
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |socket| handle(socket, room_id, name)))
}

#[cfg(feature = "server")]
async fn handle(mut socket: GameSocket, room_id: String, name: Option<String>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();
    let player_o_id = Ulid::new().to_string();
    let player_o = Player::new(player_o_id.clone(), name, tx);

    let joined = with_room(&room_id, |room| {
        room.player_o = Some(player_o);
        let _ = room.player_x.tx.send(ServerEvent::PlayerJoined {
            player_o_id: player_o_id.clone(),
        });
    })
    .is_some();

    if !joined {
        let _ = socket.send(ServerEvent::RoomNotFound).await;
        return;
    }

    let joined_event = ServerEvent::RoomJoined {
        player_o_id,
        room_id: room_id.clone(),
    };
    if socket.send(joined_event).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Ok(ClientEvent::SetName(name)) => {
                    with_room(&room_id, |room| room.set_name(Mark::O, name));
                }
                Ok(_) => {}
                Err(_) => break,
            },
            Some(event) = rx.recv() => {
                if socket.send(event).await.is_err() {
                    break;
                }
            }
        }
    }
}
