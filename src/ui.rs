use bevy::prelude::*;
use bevy::text::FontSize;

pub const TITLE_SIZE: f32 = 22.0;
pub const HEADING_SIZE: f32 = 16.0;
pub const TEXT_SIZE: f32 = 14.0;

pub const PANEL: Color = Color::srgba(0.06, 0.08, 0.11, 0.9);
pub const KEYCAP: Color = Color::srgb(0.1, 0.13, 0.17);
pub const KEYCAP_HOVERED: Color = Color::srgb(0.16, 0.2, 0.26);
pub const KEYCAP_PRESSED: Color = Color::srgb(0.25, 0.32, 0.4);
pub const BORDER: Color = Color::srgb(0.35, 0.42, 0.5);
pub const HEADING: Color = Color::srgb(0.55, 0.7, 0.85);
pub const TEXT: Color = Color::srgb(0.85, 0.9, 0.95);

pub fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

pub fn keycap(content: impl Bundle, node: Node) -> impl Bundle {
    (
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..node
        },
        BackgroundColor(KEYCAP),
        BorderColor::all(BORDER),
        children![content],
    )
}

pub fn panel(node: Node) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(12)),
            ..node
        },
        BackgroundColor(PANEL),
        BorderColor::all(BORDER),
    )
}

pub fn arrow(key: KeyCode) -> Option<Rot2> {
    let degrees = match key {
        KeyCode::ArrowUp => 0.0,
        KeyCode::ArrowRight => 90.0,
        KeyCode::ArrowDown => 180.0,
        KeyCode::ArrowLeft => 270.0,
        _ => return None,
    };
    Some(Rot2::degrees(degrees))
}

pub fn chevron(size: f32, rotation: Rot2) -> impl Bundle {
    let offset = rotation * Vec2::new(0.0, size * 0.2);
    (
        Node {
            width: px(size),
            height: px(size),
            border: UiRect {
                left: px(2),
                top: px(2),
                ..default()
            },
            ..default()
        },
        BorderColor::all(TEXT),
        UiTransform {
            translation: Val2::px(offset.x, offset.y),
            rotation: rotation * Rot2::degrees(45.0),
            ..default()
        },
    )
}
