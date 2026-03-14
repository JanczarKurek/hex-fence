
## Game rules
- There should not be a common limit for the number of fences but a separate limit for each kind
- There should be a timer for move
- Make number of fences dependend of the board size.

## Dev
- Make as much stuff loadable from files instead of compiled:
    - Configuration of the number of fences / other values in the game rules
- Extend network transport from single host+client connection to true multi-client sessions.

## Graphics
- Fences have empty spots on the joints

## UI/UX
- There should be controls help explaining what the shortcuts are
- There should be a sidepanel for selecting mode / type of fence together with fence counters.
- Game should alert player when other player makes move (wayland has some mechanism for that?)

## Performance
- Frame meter should be of use

## Done recently
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
