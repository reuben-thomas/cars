//! Help page listing the keybindings, generated from the [`Keymap`].

use crate::keymap::{self, Action, Actions, Binding, Keymap};
use bevy::picking::events::{PointerEnter, PointerLeave};
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::ui_widgets::{Activate, Button, observe};

/// Margin between the help button and the window corner [px].
const MARGIN: f32 = 16.0;
/// Width of the column holding the keycaps [px].
const KEYS_WIDTH: f32 = 150.0;
const TITLE_SIZE: f32 = 22.0;
const HEADING_SIZE: f32 = 16.0;
const TEXT_SIZE: f32 = 14.0;

const OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
const PANEL: Color = Color::srgb(0.06, 0.08, 0.11);
const KEYCAP: Color = Color::srgb(0.1, 0.13, 0.17);
const KEYCAP_HOVERED: Color = Color::srgb(0.16, 0.2, 0.26);
const BORDER: Color = Color::srgb(0.35, 0.42, 0.5);
const HEADING: Color = Color::srgb(0.55, 0.7, 0.85);
const TEXT: Color = Color::srgb(0.85, 0.9, 0.95);

pub struct HelpPlugin;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_button, spawn_page))
            .add_systems(Update, (toggle, close));
    }
}

#[derive(Component)]
struct HelpPage;

#[derive(Component)]
struct HelpButton;

fn spawn_button(mut commands: Commands) {
    commands.spawn((
        HelpButton,
        Button,
        Node {
            position_type: PositionType::Absolute,
            right: px(MARGIN),
            bottom: px(MARGIN),
            width: px(40),
            height: px(40),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(8)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(KEYCAP),
        BorderColor::all(BORDER),
        children![text("?", HEADING_SIZE, TEXT)],
        observe(toggle_on_click),
        observe(highlight::<PointerEnter>(KEYCAP_HOVERED)),
        observe(highlight::<PointerLeave>(KEYCAP)),
    ));
}

fn spawn_page(mut commands: Commands, keymap: Res<Keymap>) {
    commands
        .spawn((
            HelpPage,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(OVERLAY),
            GlobalZIndex(1),
            Visibility::Hidden,
        ))
        .with_children(|page| {
            page.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(16),
                    padding: UiRect::all(px(24)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(12)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(BORDER),
            ))
            .with_children(|panel| {
                panel.spawn(text("Keybindings", TITLE_SIZE, TEXT));
                for group in keymap.bindings().chunk_by(|a, b| a.group == b.group) {
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            ..default()
                        })
                        .with_children(|section| {
                            section.spawn(text(group[0].group.name(), HEADING_SIZE, HEADING));
                            for binding in group {
                                section.spawn(row(binding));
                            }
                        });
                }
            });
        });
}

fn row(binding: &Binding) -> impl Bundle {
    let keycaps: Vec<_> = binding.keys.iter().copied().map(keycap).collect();
    (
        Node {
            column_gap: px(12),
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Node {
                    width: px(KEYS_WIDTH),
                    column_gap: px(4),
                    ..default()
                },
                Children::spawn(SpawnIter(keycaps.into_iter())),
            ),
            text(binding.description, TEXT_SIZE, TEXT),
        ],
    )
}

fn keycap(key: KeyCode) -> impl Bundle {
    (
        Node {
            min_width: px(26),
            padding: UiRect::axes(px(6), px(2)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(KEYCAP),
        BorderColor::all(BORDER),
        children![text(keymap::label(key), TEXT_SIZE, TEXT)],
    )
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn toggle(actions: Actions, mut page: Single<&mut Visibility, With<HelpPage>>) {
    if actions.just_pressed(Action::ToggleHelp) {
        page.toggle_inherited_hidden();
    }
}

fn close(actions: Actions, mut page: Single<&mut Visibility, With<HelpPage>>) {
    if actions.just_pressed(Action::CloseHelp) {
        page.set_if_neq(Visibility::Hidden);
    }
}

fn toggle_on_click(_: On<Activate>, mut page: Single<&mut Visibility, With<HelpPage>>) {
    page.toggle_inherited_hidden();
}

fn highlight<E: EntityEvent>(color: Color) -> impl Fn(On<E>, Query<&mut BackgroundColor>) {
    move |event, mut backgrounds| {
        if let Ok(mut background) = backgrounds.get_mut(event.event_target()) {
            background.0 = color;
        }
    }
}
