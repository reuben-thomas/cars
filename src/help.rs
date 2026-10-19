use crate::keymap::{self, Action, Binding, Keymap};
use crate::ui::{self, HEADING, HEADING_SIZE, TEXT, TEXT_SIZE, TITLE_SIZE};
use bevy::prelude::*;

const KEYS_WIDTH: f32 = 150.0;
const KEYCAP_SIZE: f32 = 24.0;
const ARROW_SIZE: f32 = 7.0;

const OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);

pub struct HelpPlugin;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_page)
            .add_systems(Update, (toggle, close));
    }
}

#[derive(Component)]
struct HelpPage;

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
            page.spawn(ui::panel(Node {
                row_gap: px(16),
                padding: UiRect::all(px(24)),
                ..default()
            }))
            .with_children(|panel| {
                panel.spawn(ui::text("Keybindings", TITLE_SIZE, TEXT));
                for group in keymap.bindings().chunk_by(|a, b| a.group == b.group) {
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            ..default()
                        })
                        .with_children(|section| {
                            section.spawn(ui::text(group[0].group.name(), HEADING_SIZE, HEADING));
                            for binding in group {
                                section.spawn(row(binding));
                            }
                        });
                }
            });
        });
}

fn row(binding: &Binding) -> impl Bundle {
    let keys = binding.keys.clone();
    let keycaps = SpawnWith(move |caps: &mut ChildSpawner| {
        for key in keys {
            let node = Node {
                min_width: px(KEYCAP_SIZE),
                min_height: px(KEYCAP_SIZE),
                padding: UiRect::axes(px(6), px(2)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            };
            match ui::arrow(key) {
                Some(rotation) => caps.spawn(ui::keycap(ui::chevron(ARROW_SIZE, rotation), node)),
                None => caps.spawn(ui::keycap(
                    ui::text(keymap::label(key), TEXT_SIZE, TEXT),
                    node,
                )),
            };
        }
    });
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
                Children::spawn(keycaps),
            ),
            ui::text(binding.description, TEXT_SIZE, TEXT),
        ],
    )
}

fn toggle(actions: Res<ButtonInput<Action>>, mut page: Single<&mut Visibility, With<HelpPage>>) {
    if actions.just_pressed(Action::ToggleHelp) {
        page.toggle_inherited_hidden();
    }
}

fn close(actions: Res<ButtonInput<Action>>, mut page: Single<&mut Visibility, With<HelpPage>>) {
    if actions.just_pressed(Action::CloseHelp) {
        page.set_if_neq(Visibility::Hidden);
    }
}
