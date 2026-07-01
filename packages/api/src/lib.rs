//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

mod game;
pub use game::*;

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}
