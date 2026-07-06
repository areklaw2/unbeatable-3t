# Unbeatable 3T

Tic-tac-toe with a CPU that never loses. The Hard-mode opponent uses the
[Minimax algorithm](https://en.wikipedia.org/wiki/Minimax) to play perfectly the best you can do is force a tie. Play it in the browser (solo or online
against a friend) or in the terminal.

## Features

- **Unbeatable CPU** — Hard mode runs a full minimax search; Easy mode plays random moves.
- **Single player** — play against the CPU in the web app or the CLI.
- **Online multiplayer** — create a room, share the 4-character room code, and a
  friend joins from another device. Games run over websockets and include:
  - reconnect support if a player drops and rejoins
  - opponent-disconnect notices
  - a rematch handshake after a finished game
  - server-side cleanup of abandoned rooms (empty rooms expire after 5 minutes)

## Workspace layout

This is a Cargo workspace with three packages under `packages/`:

| Package | Description                                                                                                                                                                                                  |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `api`   | Shared crate. `api::engine` is a dependency-free, wasm-safe game engine (board logic, win detection, minimax CPU). Server-only code — room registry, websocket endpoints — sits behind the `server` feature. |
| `web`   | [Dioxus](https://dioxuslabs.com/) fullstack web app: UI, routing, and the multiplayer client.                                                                                                                |
| `cli`   | Interactive terminal game: pick your mark, then play against the CPU.                                                                                                                                        |

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started/) (`dx`) — only for web development
- [just](https://github.com/casey/just) and [Docker](https://www.docker.com/) — optional, for the containerized web app

## Running

### Web (dev)

From the repo root:

```bash
dx serve --package web
```

### Web (Docker)

The `justfile` wraps the Docker workflow. The image bundles the web app with
`dx bundle` and serves it on port 8080:

```bash
just build   # build the Docker image
just run     # run the container on http://localhost:8080 (Ctrl+C to stop)
just logs    # tail container logs
just stop    # stop the running container
```

### CLI

```bash
cargo run -p cli
```

### Tests

```bash
cargo test -p api --features server
```
