use bevy::prelude::*;

use crate::app_state::{AiStrategy, GameConfig};

use super::components::{
    HoveredGoalPreview, InGameHudUi, PlayerListEntry, PlayerListLabel, PlayerPanelBody,
    PlayerPanelToggleButton, PlayerPanelToggleText, PlayerPanelUiState, TurnStatusText,
};
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
            panel
                .spawn((Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(28.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|header| {
                    header.spawn((
                        Text::new("Players"),
                        TextFont::from_font_size(20.0),
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));

                    header
                        .spawn((
                            Button,
                            PlayerPanelToggleButton,
                            Node {
                                width: Val::Px(28.0),
                                height: Val::Px(28.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor(Color::srgba(0.95, 0.95, 1.0, 0.35)),
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                PlayerPanelToggleText,
                                Text::new("-"),
                                TextFont::from_font_size(20.0),
                                TextColor(Color::srgb(0.9, 0.9, 0.95)),
                            ));
                        });
                });

            panel
                .spawn((
                    PlayerPanelBody,
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                ))
                .with_children(|body| {
                    for player in &turn_state.players {
                        body.spawn((
                            Button,
                            PlayerListEntry {
                                player_index: player.index,
                            },
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(32.0),
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|row| {
                            row.spawn((
                                PlayerListLabel {
                                    player_index: player.index,
                                },
                                Text::new(format!("  Player {}", player.index + 1)),
                                TextFont::from_font_size(18.0),
                                TextColor(player.pawn_color),
                            ));
                        });
                    }
                });
        });
}

pub fn handle_player_panel_toggle_button(
    mut state: ResMut<PlayerPanelUiState>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayerPanelToggleButton>),
    >,
) {
    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                state.collapsed = !state.collapsed;
                *color = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.18));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06));
            }
        }
    }
}

pub fn sync_player_panel_collapsed_state(
    panel_state: Res<PlayerPanelUiState>,
    mut hovered_preview: ResMut<HoveredGoalPreview>,
    mut panel_bodies: Query<&mut Node, With<PlayerPanelBody>>,
    mut toggle_texts: Query<&mut Text, With<PlayerPanelToggleText>>,
) {
    if !panel_state.is_changed() {
        return;
    }

    let display = if panel_state.collapsed {
        hovered_preview.player_index = None;
        Display::None
    } else {
        Display::Flex
    };

    for mut body in &mut panel_bodies {
        body.display = display;
    }
    for mut text in &mut toggle_texts {
        *text = Text::new(if panel_state.collapsed { "+" } else { "-" });
    }
}

pub fn update_hovered_goal_preview(
    panel_state: Res<PlayerPanelUiState>,
    mut hovered_preview: ResMut<HoveredGoalPreview>,
    row_interactions: Query<(&Interaction, &PlayerListEntry), With<Button>>,
) {
    if panel_state.collapsed {
        if hovered_preview.player_index.is_some() {
            hovered_preview.player_index = None;
        }
        return;
    }

    let hovered_player =
        row_interactions
            .iter()
            .find_map(|(interaction, entry)| match *interaction {
                Interaction::Hovered | Interaction::Pressed => Some(entry.player_index),
                Interaction::None => None,
            });

    if hovered_preview.player_index != hovered_player {
        hovered_preview.player_index = hovered_player;
    }
}

pub fn update_turn_indicator(
    game_config: Res<GameConfig>,
    turn_state: Res<TurnState>,
    fence_placement: Res<FencePlacementState>,
    mut ui_queries: ParamSet<(
        Query<(&mut Text, &mut TextColor), With<TurnStatusText>>,
        Query<(&PlayerListLabel, &mut Text, &mut TextColor)>,
        Query<(&PlayerListEntry, &Interaction, &mut BackgroundColor), With<Button>>,
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
            match game_config.player_ai_strategy(turn_state.current_player) {
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

    let mut player_labels = ui_queries.p1();
    for (entry, mut row_text, mut row_color) in &mut player_labels {
        let player_index = entry.player_index;
        let active_marker = if turn_state.current_player == player_index {
            ">"
        } else {
            " "
        };
        let control_suffix = if game_config.player_control(player_index).is_ai() {
            match game_config.player_ai_strategy(player_index) {
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

    let mut player_rows = ui_queries.p2();
    for (entry, interaction, mut row_bg) in &mut player_rows {
        let player_color = turn_state.players[entry.player_index].pawn_color.to_srgba();
        let highlight = Color::srgba(
            player_color.red,
            player_color.green,
            player_color.blue,
            0.22,
        );
        let active = Color::srgba(1.0, 1.0, 1.0, 0.08);
        *row_bg = match *interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(highlight),
            Interaction::None if turn_state.current_player == entry.player_index => {
                BackgroundColor(active)
            }
            Interaction::None => BackgroundColor(Color::NONE),
        };
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
