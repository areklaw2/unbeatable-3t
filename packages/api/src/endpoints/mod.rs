mod create_room;
mod game;
mod join_room;

pub use create_room::create_room_ws;
pub use game::game_ws;
pub use join_room::join_room_ws;

#[cfg(feature = "server")]
pub(crate) type GameSocket = dioxus::fullstack::TypedWebsocket<
    crate::ClientEvent,
    crate::ServerEvent,
    dioxus::fullstack::CborEncoding,
>;
