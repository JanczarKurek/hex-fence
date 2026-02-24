use bevy::prelude::*;

#[derive(Component)]
pub struct Pawn {
    pub player_index: usize,
}

#[derive(Component)]
pub struct InGameHudUi;

#[derive(Component)]
pub struct TurnStatusText;

#[derive(Component)]
pub struct PlayerListEntry {
    pub player_index: usize,
}

#[derive(Component)]
pub struct PlayerListLabel {
    pub player_index: usize,
}

#[derive(Component)]
pub struct PlayerPanelBody;

#[derive(Component)]
pub struct PlayerPanelToggleButton;

#[derive(Component)]
pub struct PlayerPanelToggleText;

#[derive(Resource, Default)]
pub struct HoveredGoalPreview {
    pub player_index: Option<usize>,
}

#[derive(Resource, Default)]
pub struct PlayerPanelUiState {
    pub collapsed: bool,
}

#[derive(Component)]
pub struct MoveHighlight;
