use bevy::prelude::*;

use super::components::TurnIndicator;
use super::state::TurnState;

pub fn setup_turn_indicator(mut commands: Commands) {
    commands.spawn((
        TurnIndicator,
        Text::new("Current player: 1"),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));
}

pub fn update_turn_indicator(
    turn_state: Res<TurnState>,
    mut indicator_query: Query<(&mut Text, &mut TextColor), With<TurnIndicator>>,
) {
    if !turn_state.is_changed() {
        return;
    }

    let Ok((mut text, mut text_color)) = indicator_query.single_mut() else {
        return;
    };

    if let Some(winner) = turn_state.winner {
        *text = Text::new(format!("Winner: Player {}", winner + 1));
        *text_color = TextColor(turn_state.players[winner].pawn_color);
    } else {
        *text = Text::new(format!("Current player: {}", turn_state.current_player + 1));
        *text_color = TextColor(turn_state.players[turn_state.current_player].pawn_color);
    }
}
