use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PawnSelection {
    pub current_selected: bool,
}

pub fn reset_selection(mut selection: ResMut<PawnSelection>) {
    selection.current_selected = false;
}
