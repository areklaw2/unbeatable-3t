use api::Mode;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    Single,
    Multiple,
}

#[derive(Store)]
pub struct AppState {
    pub name_x: String,
    pub name_o: String,
    pub difficulty: Mode,
    pub game_mode: GameMode,
}

pub static APP_STATE: GlobalStore<AppState> = Global::new(|| AppState {
    name_x: String::new(),
    name_o: String::new(),
    difficulty: Mode::Easy,
    game_mode: GameMode::Single,
});
