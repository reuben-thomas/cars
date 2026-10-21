use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::ui_widgets::{
    Activate, Button, Slider, SliderPrecision, SliderRange, SliderThumb, SliderValue, TrackClick,
    observe,
};

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

pub const SLIDER_HEIGHT: f32 = 16.0;
pub const TRACK_HEIGHT: f32 = 4.0;
pub const THUMB_SIZE: f32 = 12.0;
pub const SWATCH_SIZE: f32 = 22.0;
pub const DISCLOSURE_SIZE: f32 = 6.0;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, position_thumbs);
    }
}

#[derive(Component)]
struct CollapsibleBody;

#[derive(Component)]
struct Disclosure;

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

pub fn label(value: impl Into<String>, width: f32) -> impl Bundle {
    (
        Node {
            width: px(width),
            flex_shrink: 0.0,
            ..default()
        },
        children![text(value, TEXT_SIZE, TEXT)],
    )
}

pub fn collapsible(title: impl Into<String>, content: impl Bundle) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        children![
            (
                Button,
                Hovered::default(),
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: px(12),
                    ..default()
                },
                observe(toggle_collapsible),
                children![
                    text(title, TEXT_SIZE, HEADING),
                    (
                        Disclosure,
                        UiTransform {
                            rotation: Rot2::degrees(-90.0),
                            ..default()
                        },
                        Node {
                            width: px(DISCLOSURE_SIZE * 2.0),
                            height: px(DISCLOSURE_SIZE * 2.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![chevron(DISCLOSURE_SIZE, Rot2::degrees(180.0))],
                    ),
                ],
            ),
            (CollapsibleBody, content),
        ],
    )
}

fn toggle_collapsible(
    activate: On<Activate>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    mut bodies: Query<&mut Node, With<CollapsibleBody>>,
    mut disclosures: Query<&mut UiTransform, With<Disclosure>>,
) {
    let header = activate.entity;
    let Ok(section) = parents.get(header).map(ChildOf::parent) else {
        return;
    };
    let Ok(siblings) = children.get(section) else {
        return;
    };
    let mut collapsed = false;
    for &child in siblings {
        if let Ok(mut body) = bodies.get_mut(child) {
            collapsed = body.display != Display::None;
            body.display = if collapsed {
                Display::None
            } else {
                Display::Flex
            };
        }
    }
    for child in children.iter_descendants(header) {
        if let Ok(mut disclosure) = disclosures.get_mut(child) {
            disclosure.rotation = if collapsed {
                Rot2::degrees(-90.0)
            } else {
                Rot2::IDENTITY
            };
        }
    }
}

pub fn slider(min: f32, max: f32, precision: i32) -> impl Bundle {
    (
        Slider {
            track_click: TrackClick::Snap,
            ..default()
        },
        SliderRange::new(min, max),
        SliderValue(min),
        SliderPrecision(precision),
        Hovered::default(),
        Node {
            flex_grow: 1.0,
            height: px(SLIDER_HEIGHT),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![
            (
                Node {
                    height: px(TRACK_HEIGHT),
                    border_radius: BorderRadius::all(px(TRACK_HEIGHT / 2.0)),
                    ..default()
                },
                BackgroundColor(BORDER),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(THUMB_SIZE),
                    top: px(0),
                    bottom: px(0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    SliderThumb,
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(THUMB_SIZE),
                        height: px(THUMB_SIZE),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(KEYCAP),
                    BorderColor::all(TEXT),
                )],
            ),
        ],
    )
}

fn position_thumbs(
    sliders: Query<(Entity, &SliderValue, &SliderRange), Changed<SliderValue>>,
    children: Query<&Children>,
    mut thumbs: Query<&mut Node, With<SliderThumb>>,
) {
    for (slider, value, range) in &sliders {
        for child in children.iter_descendants(slider) {
            if let Ok(mut thumb) = thumbs.get_mut(child) {
                thumb.left = percent(range.thumb_position(value.0) * 100.0);
            }
        }
    }
}

pub fn swatch(color: Color) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        Node {
            width: px(SWATCH_SIZE),
            height: px(SWATCH_SIZE),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        BackgroundColor(color),
        BorderColor::all(BORDER),
    )
}

pub fn swatch_border(selected: bool, hovered: bool) -> BorderColor {
    let color = if selected {
        TEXT
    } else if hovered {
        KEYCAP_HOVERED
    } else {
        BORDER
    };
    BorderColor::all(color)
}
