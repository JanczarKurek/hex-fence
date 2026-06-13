
## Game rules
- There should not be a common limit for the number of fences but a separate limit for each kind
- There should be a timer for move
- Make number of fences dependend of the board size.

## Dev
- Make as much stuff loadable from files instead of compiled:
    - Configuration of the number of fences / other values in the game rules

## Self-play training pipeline (AlphaZero)
- The pure rules engine lives in `crates/core` (`giereczka-core`, Bevy-free). The game, the
  `selfplay`/`eval` binaries, and Python training all share it.
- `pipeline/run.py` drives the generation loop: self-play (`selfplay --model`, MCTS+ONNX) ->
  train (`train.py`) -> export (`export_onnx.py`) -> parity gate -> eval (`eval`) -> promote.
  Needs `ORT_DYLIB_PATH` (set by `shell.nix`). See `justfile` for one-shot commands.
- Currently 2-player, radius 3 (then 4). The encoding contract (planes + action index map) is in
  `crates/core/src/encoding.rs`, mirrored in `pipeline/contract.py` and gated by `test_contract.py`.
- TODO: train a strong model (the smoke runs only prove the machinery); larger games/sims/gens.
- TODO: in-game `Neural` AI uses single-pass policy argmax; could add an MCTS "strong" mode.
- TODO: parallel leaf-batched inference; v2 bitset board rep for faster `can_place_fence`.
- TODO: extend self-play/encoding to 3/6 players (currently 2-player zero-sum only).

## Graphics
- Fences have empty spots on the joints

## UI/UX
- There should be controls help explaining what the shortcuts are
- There should be a sidepanel for selecting mode / type of fence together with fence counters.
- Game should alert player when other player makes move (wayland has some mechanism for that?)

## Performance
- Frame meter should be of use

## Done recently
- Extended network transport from a single host/client pair to host-relayed multi-client sessions.
- Added local AI players support with a basic random-legal-move bot.
- Upgraded AI with path-based heuristics and defensive fence placement.
- Added selectable AI type (Heuristic / Alpha-Beta) in game setup.
- Added mirrored S fence variant so both S configurations can be placed.
- Added frame limiting (focused ~60 FPS, lower idle when unfocused).
- Added in-game quit `X` button and post-win rematch button (local and network).
- Changed in-game `X` to return to main menu and added cleanup on leaving match.
- Migrated settings to XDG config path and persisted last used network config.
- Reworked main menu to a vertical Hex Fence layout with Local/Network/Settings/Authors/Quit.
- Added Authors popup (Codex first, Janczar Knurek second) and menu Settings sound popup.
- Reworked local game setup into a lobby layout: per-player Human/AI toggle, per-player AI type, and per-player color selection.
- Added a network lobby flow: host enters lobby immediately, client enters on connect, both can pick player slots, and host can start matches with >2 players including server-side AIs.
