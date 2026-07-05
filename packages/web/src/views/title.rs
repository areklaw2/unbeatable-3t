use dioxus::prelude::*;

use crate::Route;
use crate::components::{Button, ButtonVariant};
use crate::state::{APP_STATE, AppStateStoreExt, GameMode};

const TITLE_CSS: Asset = asset!("/assets/styling/title.css");

#[component]
pub fn Title() -> Element {
    let nav = use_navigator();
    let store = APP_STATE.resolve();
    let mut game_mode = store.game_mode();

    rsx! {
        document::Link { rel: "stylesheet", href: TITLE_CSS }

        div { class: "card screen-title",
            div { class: "kicker", "UNBEATABLE" }
            div { class: "logo-lg",
                "3T"
                span { class: "dot", "." }
            }
            div { class: "tagline", "Tic-tac-toe, but the machine plays perfect." }

            div { class: "title-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        game_mode.set(GameMode::Single);
                        nav.push(Route::SinglePlayer {});
                    },
                    "Play the computer"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        game_mode.set(GameMode::Multiple);
                        nav.push(Route::Multiplayer {});
                    },
                    "Play a friend"
                }
            }

            div { class: "footer-mark",
                span { class: "mark-x", "X" }
                span { class: "footer-label", "FORCE THE TIE" }
                span { class: "mark-o", "O" }
            }
        }
    }
}
