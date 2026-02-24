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
pub struct MoveHighlight;
