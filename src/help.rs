use crate::keymap::{self, Action, ActionButton, Binding, Keymap};
use crate::ui::{self, HEADING, HEADING_SIZE, TEXT, TEXT_SIZE, TITLE_SIZE};
use bevy::prelude::*;

const KEYS_WIDTH: f32 = 150.0;
const KEYCAP_SIZE: f32 = 24.0;
const ARROW_SIZE: f32 = 7.0;
const ABOUT_WIDTH: f32 = 420.0;
const MARGIN: f32 = 16.0;
const WORD_GAP: f32 = 4.0;

const OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);

#[derive(Clone, Copy)]
enum Span {
    Text(&'static str),
    Link(&'static str, &'static str),
}

const ABOUT: [&[Span]; 2] = [
    &[
        Span::Text("This project was inspired by Pat Kerr's "),
        Span::Link("2D Vehicles", "https://patkerr.co.uk/2d-vehicles/"),
        Span::Text(
            " as a throwback to his initial vehicle physics implementation that was included in GTA.",
        ),
    ],
    &[
        Span::Text(
            "As my throwback to projects that are certainly equally successful, I've chosen to use the dynamic bicycle model that we use in ",
        ),
        Span::Link(
            "winstxnhdw/AutoCarROS2",
            "https://github.com/winstxnhdw/AutoCarROS2",
        ),
        Span::Text(", and the late "),
        Span::Link(
            "NP-DevCar",
            "https://github.com/reuben-thomas/NP-DevCar/tree/master",
        ),
        Span::Text("."),
    ],
];
const REPOSITORY: (&str, &str) = (
    "reuben-thomas/cars",
    "https://github.com/reuben-thomas/cars",
);

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
                padding: UiRect::all(px(MARGIN)),
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
                max_width: percent(100),
                row_gap: px(16),
                padding: UiRect::all(px(24)),
                ..default()
            }))
            .with_children(|panel| {
                panel.spawn((
                    Node {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: px(16),
                        ..default()
                    },
                    children![
                        ui::text("Help", TITLE_SIZE, TEXT),
                        (
                            ActionButton(Action::CloseHelp),
                            ui::button(ui::text("X", TEXT_SIZE, TEXT)),
                        ),
                    ],
                ));
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(10),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn(ui::text("Keybindings", HEADING_SIZE, HEADING));
                        for group in keymap.bindings().chunk_by(|a, b| a.group == b.group) {
                            section
                                .spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(6),
                                    ..default()
                                })
                                .with_children(|group_section| {
                                    group_section.spawn(ui::text(
                                        group[0].group.name(),
                                        TEXT_SIZE,
                                        HEADING,
                                    ));
                                    for binding in group {
                                        group_section.spawn(row(binding));
                                    }
                                });
                        }
                    });
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(10),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn(ui::text("About", HEADING_SIZE, HEADING));
                        for spans in ABOUT {
                            section.spawn(paragraph(spans));
                        }
                    });
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(10),
                        ..default()
                    },
                    children![
                        ui::text("Source", HEADING_SIZE, HEADING),
                        ui::link(REPOSITORY.0, REPOSITORY.1),
                    ],
                ));
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

fn paragraph(spans: &[Span]) -> impl Bundle {
    let mut words: Vec<Vec<Span>> = vec![Vec::new()];
    for &span in spans {
        match span {
            Span::Text(text) => {
                for (index, word) in text.split(' ').enumerate() {
                    if index > 0 {
                        words.push(Vec::new());
                    }
                    if !word.is_empty() {
                        words.last_mut().unwrap().push(Span::Text(word));
                    }
                }
            }
            Span::Link(..) => words.last_mut().unwrap().push(span),
        }
    }
    words.retain(|word| !word.is_empty());
    (
        Node {
            max_width: px(ABOUT_WIDTH),
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(WORD_GAP),
            row_gap: px(WORD_GAP),
            ..default()
        },
        Children::spawn(SpawnWith(move |line: &mut ChildSpawner| {
            for word in words {
                line.spawn(Node::default()).with_children(|pieces| {
                    for span in word {
                        match span {
                            Span::Text(text) => pieces.spawn(ui::text(text, TEXT_SIZE, TEXT)),
                            Span::Link(label, url) => pieces.spawn(ui::link(label, url)),
                        };
                    }
                });
            }
        })),
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
