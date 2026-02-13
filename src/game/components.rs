use bevy::prelude::*;

#[derive(Component)]
pub struct Pawn {
    pub player_index: usize,
}

#[derive(Component)]
pub struct TurnIndicator;

#[derive(Component)]
pub struct MoveHighlight;
