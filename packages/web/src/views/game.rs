use dioxus::prelude::*;

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

#[component]
pub fn Game() -> Element {
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
