use bevy::prelude::*;

use crate::app_state::{AiStrategy, GameConfig};

use super::components::{InGameHudUi, PlayerListEntry, TurnStatusText};
use super::fence::{FencePlacementState, FenceShape};
use super::state::TurnState;

pub fn setup_turn_indicator(mut commands: Commands, turn_state: Res<TurnState>) {
    commands.spawn((
        InGameHudUi,
        TurnStatusText,
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

    commands
        .spawn((
            InGameHudUi,
            Node {
                width: Val::Px(300.0),
                max_width: Val::Percent(32.0),
                position_type: PositionType::Absolute,
                top: Val::Px(70.0),
                right: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor(Color::srgb(0.24, 0.24, 0.28)),
            BackgroundColor(Color::srgba(0.06, 0.07, 0.11, 0.86)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Players"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
            ));

            for player in &turn_state.players {
                panel.spawn((
                    PlayerListEntry {
                        player_index: player.index,
                    },
                    Text::new(format!("  Player {}", player.index + 1)),
                    TextFont::from_font_size(18.0),
                    TextColor(player.pawn_color),
                ));
            }
        });
}

pub fn update_turn_indicator(
    game_config: Res<GameConfig>,
    turn_state: Res<TurnState>,
    fence_placement: Res<FencePlacementState>,
    mut ui_queries: ParamSet<(
        Query<(&mut Text, &mut TextColor), With<TurnStatusText>>,
        Query<(&PlayerListEntry, &mut Text, &mut TextColor)>,
    )>,
) {
    if !turn_state.is_changed() && !fence_placement.is_changed() {
        return;
    }

    let mut status_rows = ui_queries.p0();
    let Ok((mut text, mut text_color)) = status_rows.single_mut() else {
        return;
    };

    if let Some(winner) = turn_state.winner {
        *text = Text::new(format!("Winner: Player {}", winner + 1));
        *text_color = TextColor(turn_state.players[winner].pawn_color);
    } else {
        let fences_left = turn_state.fences_left[turn_state.current_player];
        let current_control = game_config.player_control(turn_state.current_player);
        let control_suffix = if current_control.is_ai() {
            match game_config.ai_strategy {
                AiStrategy::Heuristic => " [AI:H]",
                AiStrategy::AlphaBeta => " [AI:AB]",
            }
        } else {
            ""
        };
        let current_player_label = format!("{}{}", turn_state.current_player + 1, control_suffix);
        let mode = if fence_placement.enabled {
            format!(
                "Fence ({}, rot {})",
                fence_shape_name(fence_placement.shape),
                fence_placement.orientation
            )
        } else {
            "Pawn".to_string()
        };
        *text = Text::new(format!(
            "Current player: {} | Mode: {} | Fences left: {}",
            current_player_label, mode, fences_left
        ));
        *text_color = TextColor(turn_state.players[turn_state.current_player].pawn_color);
    }

    let mut player_rows = ui_queries.p1();
    for (entry, mut row_text, mut row_color) in &mut player_rows {
        let player_index = entry.player_index;
        let active_marker = if turn_state.current_player == player_index {
            ">"
        } else {
            " "
        };
        let control_suffix = if game_config.player_control(player_index).is_ai() {
            match game_config.ai_strategy {
                AiStrategy::Heuristic => " [AI:H]",
                AiStrategy::AlphaBeta => " [AI:AB]",
            }
        } else {
            ""
        };
        let winner_marker = if turn_state.winner == Some(player_index) {
            " [WINNER]"
        } else {
            ""
        };

        let fences_left = turn_state
            .fences_left
            .get(player_index)
            .copied()
            .unwrap_or_default();

        *row_text = Text::new(format!(
            "{} Player {}{} | fences: {}{}",
            active_marker,
            player_index + 1,
            control_suffix,
            fences_left,
            winner_marker
        ));
        *row_color = TextColor(turn_state.players[player_index].pawn_color);
    }
}

fn fence_shape_name(shape: FenceShape) -> &'static str {
    match shape {
        FenceShape::S => "S",
        FenceShape::SMirrored => "S-mirror",
        FenceShape::C => "C",
        FenceShape::Y => "Y",
    }
}
