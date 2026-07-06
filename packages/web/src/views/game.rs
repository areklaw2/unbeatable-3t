use api::{ClientEvent, GameStatus, Mark, ServerEvent, engine, game_ws};
use dioxus::fullstack::{WebSocketOptions, use_websocket};
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};

use crate::Route;
use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::state::{APP_STATE, AppStateStoreExt, GameMode};

const GAME_CSS: Asset = asset!("/assets/styling/game.css");

fn mark_css(mark: Mark) -> &'static str {
    match mark {
        Mark::X => "mark-x",
        Mark::O => "mark-o",
    }
}

fn entropy_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::js_sys::Date::now().to_bits()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        1
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
fn GameView(
    board: engine::Board,
    turn_dot: Mark,
    turn_text: String,
    label_x: String,
    label_o: String,
    mode_label: String,
    on_cell: EventHandler<usize>,
    children: Element,
) -> Element {
    let nav = use_navigator();

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
                div { class: "turn-dot {mark_css(turn_dot)}" }
                div { class: "turn-text", "{turn_text}" }
            }

            div { class: "board-wrap",
                div { class: "board",
                    for i in 0..9 {
                        div {
                            key: "{i}",
                            class: {
                                match board[i] {
                                    Some(mark) => format!("cell filled {}", mark_css(mark)),
                                    None => "cell".to_string(),
                                }
                            },
                            onclick: move |_| on_cell.call(i),
                            {board[i].map(|mark| mark.label()).unwrap_or("")}
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

            {children}
        }
    }
}

#[component]
fn MultiplayerGame(room_id: String) -> Element {
    let nav = use_navigator();
    let mut board = use_signal(|| [None::<Mark>; 9]);
    let mut is_x_turn = use_signal(|| true);
    let mut status = use_signal(|| GameStatus::InProgress);
    let mut my_mark = use_signal(|| None::<Mark>);
    let mut name_x = use_signal(|| None::<String>);
    let mut name_o = use_signal(|| None::<String>);
    let mut opp_connected = use_signal(|| true);
    let mut i_want_rematch = use_signal(|| false);
    let mut opp_wants_rematch = use_signal(|| false);

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
                        if new_status == GameStatus::InProgress {
                            i_want_rematch.set(false);
                            opp_wants_rematch.set(false);
                        }
                    }
                    ServerEvent::OpponentPresence { connected } => {
                        opp_connected.set(connected);
                    }
                    ServerEvent::RematchRequested { mark } => {
                        if my_mark() == Some(mark) {
                            i_want_rematch.set(true);
                        } else {
                            opp_wants_rematch.set(true);
                        }
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

    let turn_mark = if is_x_turn() { Mark::X } else { Mark::O };
    let is_my_turn = my_mark() == Some(turn_mark);
    let in_progress = status() == GameStatus::InProgress;

    let turn_text = match status() {
        GameStatus::InProgress => {
            if is_my_turn {
                format!("{} to move — your turn", turn_mark.label())
            } else {
                format!("{} to move", turn_mark.label())
            }
        }
        GameStatus::Won { mark } => {
            if my_mark() == Some(mark) {
                format!("{} wins — you win!", mark.label())
            } else {
                format!("{} wins!", mark.label())
            }
        }
        GameStatus::Draw => "It's a draw".to_string(),
    };

    let turn_dot = match status() {
        GameStatus::Won { mark } => mark,
        _ => turn_mark,
    };

    let rematch_label = if i_want_rematch() {
        "Waiting for opponent…"
    } else if opp_wants_rematch() {
        "Accept rematch"
    } else {
        "Rematch"
    };

    rsx! {
        GameView {
            board: board(),
            turn_dot,
            turn_text,
            label_x: name_x().unwrap_or_else(|| "Player".to_string()),
            label_o: name_o().unwrap_or_else(|| "Player 2".to_string()),
            mode_label: "2P".to_string(),
            on_cell: move |cell: usize| {
                let cell_open = board()[cell].is_none();
                if is_my_turn && in_progress && cell_open {
                    spawn(async move {
                        let _ = socket.send(ClientEvent::Move { cell }).await;
                    });
                }
            },

            if !opp_connected() {
                div { class: "mp-notice", "Opponent disconnected" }
            }

            if !in_progress {
                div { class: "controls-row",
                    Button {
                        class: "rematch-button",
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: i_want_rematch(),
                        onclick: move |_| {
                            i_want_rematch.set(true);
                            spawn(async move {
                                let _ = socket.send(ClientEvent::Rematch).await;
                            });
                        },
                        "{rematch_label}"
                    }
                }
            }
        }
    }
}

#[component]
fn LocalGame() -> Element {
    let mut board = use_signal(|| [None::<Mark>; 9]);
    let mut turn = use_signal(|| Mark::X);
    let mut rng_state = use_signal(|| 0u64);

    let store = APP_STATE.resolve();
    let game_mode = store.game_mode();
    let difficulty = store.difficulty();
    let name_x = store.name_x();
    let name_o = store.name_o();

    let is_single = game_mode.cloned() == GameMode::Single;
    let difficulty_value = difficulty.cloned();

    let win = engine::winner(&board());
    let full = engine::is_full(&board());
    let game_over = win.is_some() || full;

    let cpu_pending = is_single && !game_over && {
        let b = board();
        let count = |mark| b.iter().filter(|c| **c == Some(mark)).count();
        count(Mark::X) > count(Mark::O)
    };

    let turn_text = match win {
        Some(mark) if is_single && mark == Mark::X => "You win!".to_string(),
        Some(_) if is_single => "Computer wins!".to_string(),
        Some(mark) => format!("{} wins!", mark.label()),
        None if full => "It's a draw".to_string(),
        None if cpu_pending => "Computer thinking…".to_string(),
        None if is_single => "X to move — your turn".to_string(),
        None => format!("{} to move", turn().label()),
    };

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
        GameView {
            board: board(),
            turn_dot: win.unwrap_or(if cpu_pending { Mark::O } else { turn() }),
            turn_text,
            label_x,
            label_o,
            mode_label: if is_single { "1P" } else { "2P" },
            on_cell: move |cell: usize| {
                if game_over {
                    return;
                }
                let mut b = board();
                if is_single {
                    if cpu_pending || b[cell].is_some() {
                        return;
                    }
                    let original = b;
                    b[cell] = Some(Mark::X);
                    board.set(b);

                    spawn(async move {
                        dioxus_sdk_time::sleep(std::time::Duration::from_millis(600)).await;
                        if board() != b {
                            return;
                        }
                        let mut with_reply = original;
                        let mut seed = rng_state();
                        if seed == 0 {
                            seed = entropy_seed();
                        }
                        if engine::play_vs_cpu(
                            &mut with_reply,
                            cell,
                            Mark::X,
                            difficulty_value,
                            &mut seed,
                        ) {
                            rng_state.set(seed);
                            board.set(with_reply);
                        }
                    });
                } else if b[cell].is_none() {
                    let current = turn();
                    b[cell] = Some(current);
                    board.set(b);
                    turn.set(current.other());
                }
            },

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
