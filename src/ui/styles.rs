use bevy::prelude::*;

pub(super) const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub(super) const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub(super) const PRESSED_BUTTON: Color = Color::srgb(0.8, 0.2, 0.2);
pub(super) const MENU_PANEL_BG: Color = Color::srgba(0.06, 0.07, 0.08, 0.95);
pub(super) const MENU_SELECTED: Color = Color::srgb(0.20, 0.58, 0.36);
pub(super) const MENU_START: Color = Color::srgb(0.23, 0.62, 0.40);
pub(super) const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.1, 0.95);
pub(super) const PANEL_BORDER: Color = Color::srgb(0.22, 0.22, 0.25);
pub(super) const POPUP_OVERLAY: Color = Color::srgba(0.02, 0.03, 0.04, 0.75);
pub(super) const SURFACE_DIM: Color = Color::srgba(0.10, 0.11, 0.13, 0.8);
pub(super) const SURFACE_CARD: Color = Color::srgba(0.06, 0.07, 0.09, 0.92);
pub(super) const SURFACE_CARD_BORDER: Color = Color::srgba(0.95, 0.95, 1.0, 0.1);
pub(super) const DROPDOWN_BG: Color = Color::srgba(0.06, 0.07, 0.1, 0.98);
pub(super) const DROPDOWN_BORDER: Color = Color::srgba(0.9, 0.9, 0.95, 0.35);
pub(super) const TAB_ACTIVE: Color = Color::srgb(0.22, 0.33, 0.44);
pub(super) const TAB_INACTIVE: Color = Color::srgb(0.13, 0.13, 0.15);
pub(super) const SLIDER_TRACK: Color = Color::srgb(0.18, 0.18, 0.2);
pub(super) const SLIDER_FILL: Color = Color::srgb(0.25, 0.68, 0.44);
pub(super) const MENU_TEXT: Color = Color::srgb(0.9, 0.9, 0.9);
pub(super) const VALUE_TEXT: Color = Color::srgb(0.86, 0.86, 0.9);

pub(super) fn button_node(width: f32, height: f32, border_px: f32) -> Node {
    Node {
        width: Val::Px(width),
        height: Val::Px(height),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(border_px)),
        ..default()
    }
}

pub(super) fn row_node(gap_px: f32) -> Node {
    Node {
        width: Val::Percent(100.0),
        column_gap: Val::Px(gap_px),
        flex_direction: FlexDirection::Row,
        ..default()
    }
}

pub(super) fn column_node(gap_px: f32) -> Node {
    Node {
        width: Val::Percent(100.0),
        row_gap: Val::Px(gap_px),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

pub(super) fn wrap_row_node(gap_px: f32) -> Node {
    Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        row_gap: Val::Px(gap_px),
        column_gap: Val::Px(gap_px),
        ..default()
    }
}

pub(super) fn overlay_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        display: Display::None,
        ..default()
    }
}

pub(super) fn popup_panel_node(max_width_px: f32, row_gap_px: f32) -> Node {
    Node {
        width: Val::Percent(100.0),
        max_width: Val::Px(max_width_px),
        padding: UiRect::all(Val::Px(16.0)),
        border: UiRect::all(Val::Px(2.0)),
        row_gap: Val::Px(row_gap_px),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::FlexStart,
        ..default()
    }
}

pub(super) fn popup_panel_bundle(node: Node) -> (Node, BorderColor, BackgroundColor) {
    (
        node,
        BorderColor(PANEL_BORDER),
        BackgroundColor(MENU_PANEL_BG),
    )
}

pub(super) fn tab_button_node(width_px: f32) -> Node {
    Node {
        width: Val::Px(width_px),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

pub(super) fn button_bundle<C: Component>(
    marker: C,
    node: Node,
    background: Color,
) -> (Button, C, Node, BorderColor, BackgroundColor) {
    (
        Button,
        marker,
        node,
        BorderColor(Color::BLACK),
        BackgroundColor(background),
    )
}

pub(super) fn neutral_button_color(interaction: Interaction) -> Color {
    match interaction {
        Interaction::Pressed => PRESSED_BUTTON,
        Interaction::Hovered => HOVERED_BUTTON,
        Interaction::None => NORMAL_BUTTON,
    }
}

pub(super) fn selected_button_color(selected: bool, interaction: Interaction) -> Color {
    if selected {
        MENU_SELECTED
    } else {
        match interaction {
            Interaction::Hovered => HOVERED_BUTTON,
            _ => NORMAL_BUTTON,
        }
    }
}

pub(super) fn text_bundle(
    content: impl Into<String>,
    size: f32,
    color: Color,
) -> (Text, TextFont, TextColor) {
    (
        Text::new(content),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

pub(super) fn white_text(content: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    text_bundle(content, size, Color::WHITE)
}

pub(super) fn menu_text(content: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    text_bundle(content, size, MENU_TEXT)
}
