# PvP Web Tic-Tac-Toe — Design Note

Two-player realtime web version, over websockets. Captured 2026-06-30.

## How it plays

- Server is the source of truth. It holds each game's board and whose turn it is.
- A player's browser just sends moves and renders whatever board the server sends back.
- Both players' screens update in realtime off the same server-held game.

## Matchmaking

- **Room code / shareable link.** No accounts, no lobby.
- Player 1 creates a game and gets links to share.
- **Per-player links:** a host link and a guest link, each carrying its own player token.
  Whoever opens which link is that player (X or O).
- **Rejoin:** room state lives in memory keyed by the room code, so a refresh or dropped
  connection reopens the same link and resumes as the same player.

## Out of scope (for now)

- Chat between players — later, reuses the same connection.
