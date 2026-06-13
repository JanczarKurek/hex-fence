# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`giereczka` is a Bevy 0.16 hex-board strategy game — a Quoridor variant on a hexagonal grid. Each player races a pawn from their starting board side to the opposite side, and may instead spend a turn placing a 3-segment "fence" wall to block opponents. Supports 2/3/6 players, local hotseat, AI opponents, and host-relayed TCP network multiplayer.

The authoritative game rules are in `GAME_RULES.md` (in Polish). Hex coordinate conventions are in `HEX_GRID.md`. `ISSUES.md` tracks the feature backlog and known problems — keep it updated when fixing something or noting a new idea.

## Commands

- `cargo check` — fast type-check. **Always run this after making changes before reporting success** (per `AGENTS.md`).
- `cargo run` — build and launch the game.
- `cargo test` — run tests (rules engine tests live in `src/game/rules_tests.rs`).
- `cargo test <name>` — run a single test, e.g. `cargo test sequence_places_fence_and_updates_state`.
- `cargo fmt` / `cargo clippy` — format and lint; fix clippy warnings before merging.

Toolchain is pinned to stable via `rust-toolchain.toml`. On Nix, `shell.nix` provides system deps (alsa, wayland, vulkan, libclang). The build uses Bevy `dynamic_linking` + `wayland` features.

Network mode can be driven by env vars at launch: `GIERECZKA_NET_MODE` (`local`/`host`/`client`), `GIERECZKA_NET_ADDR` (default `127.0.0.1:4000`), `GIERECZKA_NET_LOCAL_PLAYER` (slot index). These are also configurable through the lobby UI.

## Architecture

Bevy ECS app. `main.rs` composes five plugins: `BoardPlugin`, `CameraPlugin`, `NetworkPlugin`, `UiPlugin`, `GamePlugin`. A two-state `AppPhase` machine (`Menu` ↔ `InGame`, in `app_state.rs`) gates systems with `run_if(in_state(...))` and drives setup/teardown via `OnEnter`/`OnExit`.

### The rules engine is decoupled from Bevy

`src/game/state.rs` holds `TurnState` — the single source of truth for game state (pawn positions, blocked edges, fences left, current player, winner). It is a pure, deterministic, fully unit-tested rules engine with **no rendering dependencies**. The key entry point is `try_apply_action(GameAction) -> Result<AppliedAction, ActionError>`, where `GameAction` is `Move { target }` or `PlaceFence { edges: [EdgeKey; 3] }`.

When changing game rules, change `TurnState` and cover it in `rules_tests.rs` — do not scatter rules logic into Bevy systems.

### Action flow (how input, AI, and network converge)

All three input sources emit the same event, tagged with `ActionSource::Local` or `Remote`:

1. **Input** (`game/input.rs`), **AI** (`game/ai.rs`), and **incoming network messages** (`network.rs`) each write a `GameActionRequest`.
2. `apply_game_action_requests` (`game/actions.rs`) is the *only* system that mutates `TurnState`. It applies the action, updates visuals (pawn transforms, fence meshes, sounds), and emits `GameActionApplied`.
3. `send_local_actions_over_network` forwards `Local` applied actions to peers; remote ones are not re-sent.

`NetRuntime::can_control_player(config, player_index)` gates whether the local client may act for the current player — input and AI both check it.

### Hex grid & fences

`hex_grid.rs` uses axial coords `(q, r)` with implicit `s = -q - r` (see `HEX_GRID.md` for direction indices and on-board test). An `EdgeKey` (in `state.rs`) is a sorted pair of adjacent cells representing one fence segment between them; `blocked_edges: HashSet<EdgeKey>` defines all walls.

A fence is three edges. `game/fence.rs::fence_edges(anchor, shape, orientation)` builds the `[EdgeKey; 3]` for a shape (`S`, `SMirrored`, `C`, `Y`) rotated by `orientation` (0–5). Placement validity (`TurnState::can_place_fence`) requires: fences remaining, all 3 edges distinct/on-board/adjacent/unoccupied, **and** every player still has a BFS path to their goal side (`has_path_to_goal`) — a fence can never fully wall off any player.

Pawn movement (`legal_moves_for_current`) implements hex jump-over rules: landing on an occupied neighbor pushes the move to the cell beyond, with BFS fanout to side cells when the straight cell is blocked (see `GAME_RULES.md` rule 4).

### Networking

`network.rs` is a host-relayed TCP star topology. The host runs a `TcpListener` and relays `NetMessage`s between clients (`Broadcast` / `BroadcastExcept` / `SendToPeer`). A background `thread` bridges sockets to the Bevy world over `mpsc` channels (`NetCommand` out, `NetEvent` in). The wire protocol is the `NetMessage` enum, serialized as line-delimited JSON (serde). `NetLobbyState` + `NetRuntime` track lobby slot assignments before a match starts.

### Player setup

`game/player.rs` maps `player_count` → starting/goal board sides (`goal_side = (start + 3) % 6`) and `fences_per_player`. `GameConfig` (`app_state.rs`) carries per-player `PlayerControl` (Human/RandomAi), `AiStrategy` (Heuristic/AlphaBeta), and `PlayerColor`. AI (`game/ai.rs`) runs on a cooldown timer; Heuristic uses path-distance heuristics with defensive fence placement, AlphaBeta does a depth-limited search.

### UI & settings

`ui/start_menu.rs` is the menu + local/network lobby (large). `ui/in_game_menu.rs` is the in-game HUD, settings popup, exit/rematch buttons. `game/ui.rs` renders the turn indicator and player panel. `settings.rs` persists `AppSettings` (audio / network / rebindable controls) to the XDG config dir as `giereczka/settings.toml`. In-game keyboard controls (rebindable) toggle fence mode, cycle fence shape, and rotate fence orientation; mouse selects/moves the pawn or places a fence.

## Conventions

- Keep files small; put each Bevy system in a focused file, and split a module into a directory when it grows (per `AGENTS.md`).
- Game state cleanup is explicit: `OnExit(AppPhase::InGame)` despawns board/pawn/fence/HUD entities, and rematch reuses the same cleanup helpers — when adding new in-game entities, add them to the cleanup paths in `game/mod.rs` and `board.rs`.
- `settings.toml` and `target/` are gitignored.
