use crate::car::Car;
use crate::keymap::{self, Action, ActionButton};
use crate::player::Player;
use crate::settings;
use crate::ui::{self, KEYCAP, KEYCAP_HOVERED, KEYCAP_PRESSED, TEXT, TEXT_SIZE};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

const MARGIN: f32 = 16.0;
const GAP: f32 = 4.0;
const ARROW_SIZE: f32 = 12.0;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (style_buttons, update_readout));
    }
}

#[derive(Component)]
struct Readout;

fn spawn(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(MARGIN),
            right: px(MARGIN),
            bottom: px(MARGIN),
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::flex(1.0),
                GridTrack::auto(),
                GridTrack::flex(1.0),
            ],
            align_items: AlignItems::End,
            ..default()
        },
        Pickable::IGNORE,
        GlobalZIndex(2),
        children![
            (
                Node {
                    justify_self: JustifySelf::Start,
                    column_gap: px(GAP),
                    ..default()
                },
                children![
                    arrow_button(Action::SteerLeft, KeyCode::ArrowLeft),
                    arrow_button(Action::SteerRight, KeyCode::ArrowRight),
                ],
            ),
            (Readout, ui::text("", TEXT_SIZE, TEXT)),
            (
                Node {
                    justify_self: JustifySelf::End,
                    column_gap: px(GAP),
                    ..default()
                },
                children![
                    text_button(Action::Handbrake, "(P)"),
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(GAP),
                            ..default()
                        },
                        children![
                            arrow_button(Action::Accelerate, KeyCode::ArrowUp),
                            arrow_button(Action::Brake, KeyCode::ArrowDown),
                        ],
                    ),
                ],
            ),
        ],
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(MARGIN),
            left: px(MARGIN),
            right: px(MARGIN),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexStart,
            column_gap: px(GAP),
            ..default()
        },
        Pickable::IGNORE,
        GlobalZIndex(2),
        children![
            settings::panel(),
            (
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
                children![text_button(
                    Action::ToggleHelp,
                    keymap::label(KeyCode::Slash)
                )],
            ),
        ],
    ));
}

fn arrow_button(action: Action, key: KeyCode) -> impl Bundle {
    let rotation = ui::arrow(key).expect("not an arrow key");
    (
        ActionButton(action),
        ui::button(ui::chevron(ARROW_SIZE, rotation)),
    )
}

fn text_button(action: Action, label: impl Into<String>) -> impl Bundle {
    (
        ActionButton(action),
        ui::button(ui::text(label, TEXT_SIZE, TEXT)),
    )
}

fn style_buttons(
    actions: Res<ButtonInput<Action>>,
    mut buttons: Query<(&ActionButton, &Hovered, &mut BackgroundColor)>,
) {
    for (&ActionButton(action), hovered, mut background) in &mut buttons {
        let color = if actions.pressed(action) {
            KEYCAP_PRESSED
        } else if hovered.get() {
            KEYCAP_HOVERED
        } else {
            KEYCAP
        };
        background.set_if_neq(BackgroundColor(color));
    }
}

fn update_readout(car: Single<&Car, With<Player>>, mut readout: Single<&mut Text, With<Readout>>) {
    let speed = car.speed() * 3.6;
    let heading = car.state().heading.as_degrees();
    readout.0 = format!("{speed:.1} kph\n{heading:.1} degrees");
}
