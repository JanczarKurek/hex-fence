use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PawnSelection {
    pub current_selected: bool,
}
