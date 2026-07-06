use api::{ClientEvent, GameStatus, ServerEvent, game_ws};
use dioxus::fullstack::{WebSocketOptions, use_websocket};
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};

use crate::Route;
use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::state::{APP_STATE, AppStateStoreExt, GameMode};

const GAME_CSS: Asset = asset!("/assets/styling/game.css");

#[derive(Clone, Copy, PartialEq)]
enum Mark {
    X,
    O,
}

impl Mark {
    fn label(self) -> &'static str {
        match self {
            Mark::X => "X",
            Mark::O => "O",
        }
    }

    fn class(self) -> &'static str {
        match self {
            Mark::X => "mark-x",
            Mark::O => "mark-o",
        }
    }

    fn other(self) -> Mark {
        match self {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        }
    }
}

fn mark_class(cell: &str) -> &'static str {
    match cell {
        "X" => "mark-x",
        "O" => "mark-o",
        _ => "",
    }
}

#[component]
pub fn Game(room_id: Option<String>) -> Element {
    match room_id {
        Some(room_id) => rsx! {
            MultiplayerGame { room_id }
        },
        None => rsx! {
            LocalGame {}
        },
    }
}

#[component]
fn MultiplayerGame(room_id: String) -> Element {
    let nav = use_navigator();
    let mut board = use_signal(|| vec![vec![String::from(" "); 3]; 3]);
    let mut is_x_turn = use_signal(|| true);
    let mut status = use_signal(|| GameStatus::InProgress);
    let mut my_mark = use_signal(|| None::<String>);
    let mut name_x = use_signal(|| None::<String>);
    let mut name_o = use_signal(|| None::<String>);

    let mut socket = use_websocket({
        let room_id = room_id.clone();
        move || {
            let player_id: String = LocalStorage::get("player_id").unwrap_or_default();
            game_ws(room_id.clone(), player_id, WebSocketOptions::new())
        }
    });

    use_future(move || async move {
        loop {
            _ = socket.connect().await;

            while let Ok(msg) = socket.recv().await {
                match msg {
                    ServerEvent::GameConnected { your_mark } => {
                        my_mark.set(Some(your_mark));
                    }
                    ServerEvent::GameState {
                        board: new_board,
                        is_x_turn: new_turn,
                        status: new_status,
                        player_x_name,
                        player_o_name,
                    } => {
                        board.set(new_board);
                        is_x_turn.set(new_turn);
                        status.set(new_status);
                        name_x.set(player_x_name);
                        name_o.set(player_o_name);
                    }
                    ServerEvent::RoomNotFound | ServerEvent::Unauthorized => {
                        nav.push(Route::Title {});
                        return;
                    }
                    _ => {}
                }
            }
        }
    });

    let turn_mark = if is_x_turn() { "X" } else { "O" };
    let is_my_turn = my_mark().as_deref() == Some(turn_mark);
    let in_progress = status() == GameStatus::InProgress;

    let turn_text = match status() {
        GameStatus::InProgress => {
            if is_my_turn {
                format!("{turn_mark} to move — your turn")
            } else {
                format!("{turn_mark} to move")
            }
        }
        GameStatus::Won { ref mark } => {
            if my_mark().as_deref() == Some(mark.as_str()) {
                format!("{mark} wins — you win!")
            } else {
                format!("{mark} wins!")
            }
        }
        GameStatus::Draw => "It's a draw".to_string(),
    };

    let turn_dot_class = if is_x_turn() { "mark-x" } else { "mark-o" };
    let label_x = name_x().unwrap_or_else(|| "Player".to_string());
    let label_o = name_o().unwrap_or_else(|| "Player 2".to_string());

    rsx! {
        document::Link { rel: "stylesheet", href: GAME_CSS }

        div { class: "card screen",
            div { class: "game-header",
                button {
                    class: "quit-link",
                    onclick: move |_| {
                        nav.push(Route::Title {});
                    },
                    "← Quit"
                }
                div { class: "logo-sm",
                    "3T"
                    span { class: "dot", "." }
                }
            }

            div { class: "turn-row",
                div { class: "turn-dot {turn_dot_class}" }
                div { class: "turn-text", "{turn_text}" }
            }

            div { class: "board-wrap",
                div { class: "board",
                    for i in 0..9 {
                        div {
                            key: "{i}",
                            class: {
                                let cell = board()[i / 3][i % 3].clone();
                                if cell == " " {
                                    "cell".to_string()
                                } else {
                                    format!("cell filled {}", mark_class(&cell))
                                }
                            },
                            onclick: move |_| {
                                let cell_open = board()[i / 3][i % 3] == " ";
                                if is_my_turn && in_progress && cell_open {
                                    spawn(async move {
                                        let _ = socket
                                            .send(ClientEvent::Move {
                                                r: i / 3,
                                                c: i % 3,
                                            })
                                            .await;
                                    });
                                }
                            },
                            {
                                let cell = board()[i / 3][i % 3].clone();
                                if cell == " " { String::new() } else { cell }
                            }
                        }
                    }
                }
            }

            div { class: "score-row",
                div { class: "score-col mark-x",
                    div { class: "score-label", "{label_x} · X" }
                    div { class: "score-value", "0" }
                }
                div { class: "score-col mark-tie",
                    div { class: "score-label", "Tie" }
                    div { class: "score-value", "0" }
                }
                div { class: "score-col mark-o",
                    div { class: "score-label", "{label_o} · O" }
                    div { class: "score-value", "0" }
                }
                div { class: "mode-tag",
                    div { class: "person-icon",
                        div { class: "person-head" }
                        div { class: "person-body" }
                    }
                    div { class: "mode-label", "2P" }
                }
            }
        }
    }
}

#[component]
fn LocalGame() -> Element {
    let nav = use_navigator();
    let mut board = use_signal(|| [None::<Mark>; 9]);
    let mut turn = use_signal(|| Mark::X);

    let store = APP_STATE.resolve();
    let game_mode = store.game_mode();
    let name_x = store.name_x();
    let name_o = store.name_o();

    let is_single = game_mode.cloned() == GameMode::Single;
    let mode_label = if is_single { "1P" } else { "2P" };
    let label_x = {
        let value = name_x.cloned();
        if value.is_empty() {
            "Player".to_string()
        } else {
            value
        }
    };
    let label_o = if is_single {
        "Computer".to_string()
    } else {
        let value = name_o.cloned();
        if value.is_empty() {
            "Player 2".to_string()
        } else {
            value
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: GAME_CSS }

        div { class: "card screen",
            div { class: "game-header",
                button {
                    class: "quit-link",
                    onclick: move |_| {
                        nav.push(Route::Title {});
                    },
                    "← Quit"
                }
                div { class: "logo-sm",
                    "3T"
                    span { class: "dot", "." }
                }
            }

            div { class: "turn-row",
                div { class: "turn-dot {turn().class()}" }
                div { class: "turn-text", "{turn().label()} to move" }
            }

            div { class: "board-wrap",
                div { class: "board",
                    for i in 0..9 {
                        div {
                            key: "{i}",
                            class: if let Some(mark) = board()[i] { "cell filled {mark.class()}" } else { "cell" },
                            onclick: move |_| {
                                if board()[i].is_none() {
                                    let current = turn();
                                    board.write()[i] = Some(current);
                                    turn.set(current.other());
                                }
                            },
                            {board()[i].map(|mark| mark.label()).unwrap_or("")}
                        }
                    }
                }
            }

            div { class: "score-row",
                div { class: "score-col mark-x",
                    div { class: "score-label", "{label_x} · X" }
                    div { class: "score-value", "0" }
                }
                div { class: "score-col mark-tie",
                    div { class: "score-label", "Tie" }
                    div { class: "score-value", "0" }
                }
                div { class: "score-col mark-o",
                    div { class: "score-label", "{label_o} · O" }
                    div { class: "score-value", "0" }
                }
                div { class: "mode-tag",
                    div { class: "person-icon",
                        div { class: "person-head" }
                        div { class: "person-body" }
                    }
                    div { class: "mode-label", "{mode_label}" }
                }
            }

            div { class: "controls-row",
                Button {
                    class: "undo-button",
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    disabled: true,
                    "Undo"
                }
                Button {
                    class: "new-game-button",
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        board.set([None; 9]);
                        turn.set(Mark::X);
                    },
                    "New game"
                }
            }
        }
    }
}
