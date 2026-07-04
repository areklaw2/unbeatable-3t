# Unbeatable 3T — UI Component & Style Breakdown

Source: the design export in `Unbeatable 3T.html`. That file is a visual-builder
export (its own `sc-if` / `sc-for` / `DCLogic` templating, not Dioxus). This doc
translates it into **components + styles** so you can build the RSX yourself.

Nothing here is Dioxus code on purpose — it's the map, not the territory.

---

## 1. Design tokens

Pull these into one place (a `tokens.css`, a Tailwind theme extension, or Rust
`const`s). Every screen reuses them.

### Colors

| Token           | Value                   | Used for                                         |
| --------------- | ----------------------- | ------------------------------------------------ |
| `paper`         | `#efece2`               | app background, filled-button text               |
| `ink`           | `#1a1913`               | primary text, dark/filled buttons, outlines      |
| `surface`       | `#faf8f2`               | input & cell backgrounds                         |
| `border`        | `#ded9cb`               | input / cell / code-box borders                  |
| `x-blue`        | `#7fa8d9`               | player X (mark, score, code) — **prop-driven**   |
| `o-red`         | `#e08f8f`               | player O + the "." in the logo — **prop-driven** |
| `muted`         | `#8f8b7c`               | labels, subtitles, back links                    |
| `muted-2`       | `#57544a`               | share-link text, turn text                       |
| `placeholder`   | `#b7b3a6`               | input placeholders, faint accents                |
| `disabled-text` | `#c4c0b2`               | disabled Undo text                               |
| `hover-surface` | `#f3eee1`               | cell hover background                            |
| `hover-border`  | `#c9c3b2`               | cell hover border                                |
| `mark-dimmed`   | `rgba(26,25,19,.18)`    | losing/other marks after game over               |
| `overlay`       | `rgba(239,236,226,.93)` | result overlay scrim                             |

X and O colors are **configurable props** with preset options (see §5):

- `xColor`: `#7fa8d9` (default), `#6fb7a6`, `#9a8fd4`
- `oColor`: `#e08f8f` (default), `#e0a878`, `#d488b0`

### Typography

Font: **Space Grotesk**, weights 400/500/600/700 (Google Fonts), fallback `sans-serif`.

| Role                | Size / weight / tracking                     |
| ------------------- | -------------------------------------------- |
| Title "3T."         | 92px / 700 / letter-spacing -3px             |
| Kicker "UNBEATABLE" | 11px / 600 / letter-spacing 5px              |
| Screen heading      | 22px / 700                                   |
| Section label       | 11px / 600 / letter-spacing 2px (uppercase)  |
| Subtitle / body     | 13–15px / 500 / `muted`                      |
| Button text         | 13–14px / 600                                |
| Name input          | 18px / 500                                   |
| Room-code input     | 32px / 700 / letter-spacing 16px (uppercase) |
| Code box char       | 28px / 700                                   |
| Cell mark           | `clamp(40px, 13vw, 58px)` / 700              |
| Score number        | 36px / 700                                   |
| Score / tag label   | 12px / 600                                   |

### Radii

pill button `40px` · name input `14px` · room-code input `16px` · cell `16px` ·
code box `14px` · copy button `10px` · overlay `20px`.

### Layout

- **App shell**: `min-height:100vh; display:grid; place-items:center; padding:40px 20px;` on `paper`.
- **Card**: `width: min(370px, 92vw)` — every screen lives inside this one width.
- **Board**: `width: min(80vw, 320px)`, 3×3 grid, `gap:10px`, cells `aspect-ratio:1`.

### Motion

| Name        | Keyframes                                   | Applied to           |
| ----------- | ------------------------------------------- | -------------------- |
| `fade`      | opacity 0→1, translateY 4px→0, ~.3–.4s ease | each screen on enter |
| `pop`       | scale .7→1.06→1 + fade, .32s ease-out       | result overlay       |
| transitions | `.12–.14s` on buttons / inputs / cells      | hover, focus, active |

### Interaction rules (from the mockup's inline JS handlers)

- **Input focus**: border → `ink`, background → `#fff`; on blur revert to `border` / `surface`.
- **Button focus**: `outline: 2px solid ink; outline-offset: 2px;` (keyboard a11y).
- **Hover** in the mockup used a custom `style-hover` attr → in Dioxus use real
  CSS `:hover` classes (or Tailwind `hover:`). Effects: cell lifts (`scale(1.04)`),
  buttons darken/lift.

---

## 2. Screens (the top-level state machine)

One card, six mutually-exclusive screens, chosen by a single `screen` value.
In the mockup these are `sc-if` blocks; in Dioxus this is a `match` on an enum
signal (you do **not** need the URL Router for this — it's in-memory state).

| Screen enum | Mockup flag  | Title text      |
| ----------- | ------------ | --------------- |
| `Title`     | `isTitle`    | "3T."           |
| `SpSetup`   | `isSpSetup`  | "One player"    |
| `MpChoose`  | `isMpChoose` | "Two players"   |
| `MpCreate`  | `isMpCreate` | "Create a room" |
| `MpJoin`    | `isMpJoin`   | "Join a room"   |
| `Game`      | `isGame`     | (the board)     |

Navigation edges:

```
Title ──"Play the computer"──> SpSetup ──"Start game"──> Game
Title ──"Play a friend"──────> MpChoose
MpChoose ──"Create a room"───> MpCreate ──"Start game"─> Game
MpChoose ──"Join with a code"> MpJoin   ──"Join game"──> Game
(every non-Title screen has "← Back"; Game has "← Quit" → Title)
```

---

## 3. Component tree

```
App                         (owns all state; matches on `screen`)
├─ Title
│  ├─ Kicker "UNBEATABLE"
│  ├─ Logo (big)
│  ├─ tagline text
│  ├─ SecondaryButton "Play the computer"   → SpSetup
│  ├─ SecondaryButton "Play a friend"       → MpChoose
│  └─ FooterMark (X · "FORCE THE TIE" · O)
│
├─ SinglePlayer
│  ├─ ScreenHeading "One player"
│  ├─ SectionLabel "YOUR NAME" + TextInput(nameX)
│  ├─ SectionLabel "DIFFICULTY" + DifficultyToggle(Easy | Unbeatable)
│  ├─ PrimaryButton "Start game"
│  └─ BackLink → Title
│
├─ Multiplayer
│  ├─ ScreenHeading "Two players" + subtitle
│  ├─ PrimaryButton "Create a room"         → MpCreate
│  ├─ SecondaryButton "Join with a code"    → MpJoin
│  └─ BackLink → Title
│
├─ CreateRoom
│  ├─ ScreenHeading "Create a room"
│  ├─ SectionLabel "YOUR NAME" + TextInput(nameX)
│  ├─ SectionLabel "ROOM CODE" + CodeBoxes(×4, from roomCode)
│  ├─ SectionLabel "SHAREABLE LINK" + ShareLinkRow(link, Copy)
│  ├─ "Waiting for player 2…" text
│  ├─ PrimaryButton "Start game"
│  └─ BackLink → MpChoose
│
├─ JoinRoom
│  ├─ ScreenHeading "Join a room"
│  ├─ SectionLabel "ROOM CODE" + RoomCodeInput(codeInput)
│  ├─ SectionLabel "YOUR NAME" + TextInput(nameO)
│  ├─ PrimaryButton "Join game"
│  └─ BackLink → MpChoose
│
└─ Game
   ├─ GameHeader (QuitLink + Logo small)
   ├─ TurnIndicator (dot + text)          [only when showHints && no winner]
   ├─ Board
   │  ├─ Cell ×9
   │  └─ ResultOverlay                     [only when winner]
   │     └─ PrimaryButton "Play again"
   ├─ Scoreboard
   │  ├─ ScoreColumn X · ScoreColumn Tie · ScoreColumn O
   │  └─ ModeTag (person icon + "1P"/"2P")
   └─ GameControls (Undo + "New game")
```

---

## 4. Reusable components (atoms) — the styling reference

Build these once; the screens are mostly composition of them. Styles below are
the exact values from the export.

### Buttons

**`PrimaryButton`** (filled) — Start / Create / Join / Play again

```
width:100%; border:none; background:#1a1913; color:#efece2;
font:600 14px 'Space Grotesk'; border-radius:40px; padding:17px;
cursor:pointer; transition:all .14s;
```

**`SecondaryButton`** (outline) — Play computer/friend, Join with a code, New game

```
width:100%; border:1.5px solid #1a1913; background:transparent; color:#1a1913;
font:600 14px 'Space Grotesk'; border-radius:40px; padding:17px;
cursor:pointer; transition:all .14s;
```

(the compact "New game" variant uses `padding:11px 24px; font-size:13px`.)

**`BackLink` / `TextButton`** — "← Back", "← Quit"

```
background:none; border:none; color:#8f8b7c; font:500 13px 'Space Grotesk';
cursor:pointer; transition:color .14s;  /* hover → #1a1913 */
```

Prop idea for all three: `label`, `variant` (Primary/Secondary/Text), `onclick`,
optional `disabled`. All share the focus-outline rule from §1.

### Inputs

**`TextInput`** (name fields)

```
width:100%; background:#faf8f2; border:1.5px solid #ded9cb; border-radius:14px;
color:#1a1913; font:500 18px 'Space Grotesk'; padding:13px 15px; outline:none;
transition:border-color .14s, background .14s;  /* maxlength 12 */
```

**`RoomCodeInput`** (join screen — one big field)

```
…same base… border-radius:16px; color:#7fa8d9; font:700 32px; text-align:center;
text-transform:uppercase; letter-spacing:16px; padding:16px 0 16px 16px;  /* maxlength 4 */
```

Input sanitizer (from `onCode`): uppercase, strip non-`[A-Z0-9]`, clamp to 4 chars.

Focus/blur behavior for both: see §1 interaction rules.

### Labels & headings

**`Kicker`** — `11px/600, letter-spacing 5px, color muted` (the "UNBEATABLE").
**`SectionLabel`** — `11px/600, letter-spacing 2px, color muted`, `margin-bottom ~9–12px`.
**`ScreenHeading`** — `22px/700, text-align center`.

### `Logo` / wordmark

"3T" in `ink` + a "." span in `o-red`. Two sizes: **92px/700 / -3px** (title) and
**16px/700** (game header). Make size a prop.

### `DifficultyToggle` (segmented control)

Row of two buttons, `gap:10px`. Shared base:

```
flex:1; font:600 14px; border-radius:14px; padding:14px 4px;
cursor:pointer; transition:all .12s;
```

- **active**: `background:#1a1913; color:#efece2; border:1.5px solid #1a1913;`
- **inactive**: `background:transparent; color:#1a1913; border:1.5px solid #1a1913;`

Driven by `difficulty` (`easy` | `hard`). Same active/inactive pattern is reusable
for any two-way toggle.

### `CodeBox` (create screen, ×4)

```
width:56px; height:66px; display:flex; center; font:700 28px;
background:#faf8f2; border:1.5px solid #ded9cb; border-radius:14px; color:#7fa8d9;
```

Rendered by iterating the 4 chars of `roomCode` (a loop, one box per char).

### `ShareLinkRow`

Surface pill: `background:#faf8f2; border:1.5px solid #ded9cb; border-radius:14px;`
containing:

- link text: `flex:1; 14px; color:#57544a;` truncated (`overflow:hidden; white-space:nowrap; text-overflow:ellipsis`).
- **Copy button**: small, `border-radius:10px; padding:9px 14px; 12px/600`.
  Label + colors toggle on click: `Copy`→`Copied`, bg `ink`→`x-blue`, for ~1.4s.
  (Uses `navigator.clipboard.writeText`.)

### `Cell` (the interesting one — fully dynamic)

Base:

```
aspect-ratio:1; display:flex; center; font:700 clamp(40px,13vw,58px);
border-radius:16px; user-select:none; transition:all .12s;
```

State-driven overrides:

- **empty & playable**: bg `surface`, border `border`, `cursor:pointer`, hover →
  bg `hover-surface`, border `hover-border`, `transform:scale(1.04)`.
- **filled**: color = that mark's color (`x-blue` / `o-red`).
- **on winning line**: bg = mark color, border = mark color, text = `surface` (inverted).
- **game over, not on win line**: color dimmed to `mark-dimmed`.
- **not clickable** (occupied / game over / thinking / CPU's turn): `cursor:default`, no hover.

Props: `mark` (`""`/`X`/`O`), `is_win`, `dimmed`, `clickable`, `onclick`, plus the
two player colors so it can pick its own color.

### `TurnIndicator`

`dot (8px circle, background = turnColor)` + text `12px/600, color muted-2`.
Shown only when `show_hints` is on and there's no winner. Text:

- single, CPU turn → "Computer is thinking…"
- single, your turn → "Your move"
- multiplayer → `"{player name} to move"`

### `ResultOverlay`

Absolute cover of the board (`inset:-8px`), scrim `overlay`, `border-radius:20px`,
`pop` animation, centered column `gap:22px`:

- result text `26px/700`, color = winner's color (or `muted-2` for a tie).
- `PrimaryButton "Play again"` (compact padding `14px 30px`).

Result text: single → "You win!" / "Computer wins"; multiplayer → `"{name} wins!"`;
draw → "Tie game".

### `Scoreboard`

Row, `gap:26px`, three `ScoreColumn`s + a `ModeTag`.

- **`ScoreColumn`**: label `12px/600` + number `36px/700`, both tinted
  (X→`x-blue`, Tie→`muted`, O→`o-red`). Labels: single → "Player · X" /
  "Computer · O"; multiplayer → `"{name} · X/O"`.
- **`ModeTag`**: tiny CSS "person" glyph (a `6px` circle head + a `12×7`
  rounded-top body, both `#b7b3a6`) above the text `"1P"` / `"2P"`.

### `GameControls`

Row, `gap:14px`, centered:

- **Undo** — outline pill (`padding:11px 24px; 13px/600`). Enabled only when
  `history` non-empty && no winner && not thinking; disabled → border `border`,
  text `disabled-text`, `cursor:default`.
- **New game** — `SecondaryButton` compact.

---

## 5. State model

App-level state (each a signal). Names mirror the mockup so you can cross-check.

```
screen      : Screen enum   { Title, SpSetup, MpChoose, MpCreate, MpJoin, Game }
mode        : Option<Mode>  { Single, Local }        // Local = two humans, same device
difficulty  : Difficulty    { Easy, Hard }
name_x      : String                                  // maxlen 12
name_o      : String                                  // maxlen 12
code_input  : String                                  // join field, sanitized, maxlen 4
room_code   : String                                  // 4 chars, A–Z0–9 minus look-alikes
copied      : bool                                     // Copy-button feedback flip
players     : { x: String, o: String }                // display names, uppercased
board       : [Option<Mark>; 9]                        // flat, NOT 3×3 (see §6)
turn        : Mark { X, O }
winner      : Option<Winner> { X, O, Draw }
win_line    : Option<[usize; 3]>                       // the 3 winning indices
scores      : { x: u32, o: u32, draw: u32 }
history     : Vec<Snapshot>                             // for Undo
thinking    : bool                                     // CPU "is thinking" lock
```

`Snapshot = { board, turn, scores }` captured **before** each move.

**Config props** (the mockup's `data-props` — these are the tweakable knobs):

| Prop        | Type / options                           | Default   |
| ----------- | ---------------------------------------- | --------- |
| `xColor`    | color: `#7fa8d9` / `#6fb7a6` / `#9a8fd4` | `#7fa8d9` |
| `oColor`    | color: `#e08f8f` / `#e0a878` / `#d488b0` | `#e08f8f` |
| `firstMove` | enum: `Player` / `Computer`              | `Player`  |
| `showHints` | boolean (turn indicator on/off)          | `true`    |

**Derived values** (the mockup computes these each render in `renderVals`; in
Dioxus they're just expressions/memos in the component body, not stored state):
per-cell style, turn text/color, result text/color, score labels, `can_undo`,
share link string (`unbeatable3t.gg/#<code>`), mode tag.

---

## 6. Reconciling with the existing Rust (`packages/api/src/game.rs`)

Heads-up before you wire logic — the mockup and the current `Game` disagree in
ways worth deciding on up front:

| Concern       | `api::game::Game` today   | Mockup expects                                  |
| ------------- | ------------------------- | ----------------------------------------------- |
| Board shape   | `Vec<Vec<String>>` (3×3)  | flat length-9                                   |
| Modes         | single-player vs CPU only | single **and** two-human (`Local`)              |
| Win reporting | `bool` (did side win)     | winner **+ the winning line** for the highlight |
| Draw          | separate `is_full` check  | folded into one "check winner" result           |
| Scores        | none                      | X / O / draw tallies across rematches           |
| Undo          | none                      | full `history` stack (skips CPU move in single) |
| CPU delay     | none                      | ~520ms "thinking" pause                         |
| AI (hard)     | minimax present ✔         | same minimax — reuse it                         |

The minimax/`win` logic is solid and reusable. The gaps are **win-line tracking,
scores, history/undo, and a two-human path** (no CPU). Decide whether to extend
`Game` to a flat board that returns `{winner, line}`, or keep `Game` for
single-player and add a thin multiplayer state alongside. Either is fine — just
pick before you build the `Game` screen so the board component's props are stable.

---

## 7. Suggested file layout (matches current `packages/web/src`)

Only a suggestion — adjust to taste:

```
components/
  buttons.rs      // PrimaryButton, SecondaryButton, BackLink
  inputs.rs       // TextInput, RoomCodeInput
  labels.rs       // Kicker, SectionLabel, ScreenHeading, Logo
  board.rs        // Cell, Board, ResultOverlay
  scoreboard.rs   // Scoreboard, ScoreColumn, ModeTag, TurnIndicator, GameControls
  room.rs         // CodeBox(es), ShareLinkRow, DifficultyToggle
views/
  title.rs · sp_setup.rs · mp_choose.rs · mp_create.rs · mp_join.rs · game.rs
```

Put tokens in `assets/` CSS (or Tailwind theme) so colors/radii live in one spot.

---

### tl;dr build order

1. Tokens (colors, font, radii) → shared CSS/theme.
2. Atoms: buttons, inputs, labels, `Logo`.
3. Static screens: Title → MpChoose → SpSetup → MpJoin → MpCreate.
4. `Cell` + `Board` (static first, then click wiring).
5. Game state + `Scoreboard` / `TurnIndicator` / `ResultOverlay` / `GameControls`.
6. Wire logic (reconcile with `game.rs` per §6), then props (§5) last.

```

```
