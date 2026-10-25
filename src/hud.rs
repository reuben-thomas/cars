use crate::car::Car;
use crate::keymap::{self, Action, ActionButton};
use crate::player::Player;
use crate::settings;
use crate::ui::{
    self, BORDER_WIDTH, BUTTON_SIZE, KEYCAP, KEYCAP_HOVERED, KEYCAP_PRESSED, RADIUS, TEXT,
    TEXT_SIZE,
};
use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Button, observe};

const MARGIN: f32 = 16.0;
const GAP: f32 = 4.0;
const ARROW_SIZE: f32 = 12.0;
const ICON_SIZE: f32 = 24.0;
const REPOSITORY: &str = "https://github.com/reuben-thomas/cars";

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (style_buttons, update_readout));
    }
}

#[derive(Component)]
struct Readout;

#[derive(Component)]
struct RepositoryLink;

fn spawn(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let github = images.add(
        Image::from_buffer(
            include_bytes!("../assets/github.png"),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true,
            ImageSampler::Default,
            RenderAssetUsages::RENDER_WORLD,
        )
        .expect("invalid github icon"),
    );
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
        GlobalZIndex(2),
        children![
            settings::panel(),
            (
                Node {
                    flex_shrink: 0.0,
                    column_gap: px(GAP),
                    ..default()
                },
                children![
                    text_button(Action::ToggleHelp, keymap::label(KeyCode::Slash)),
                    repository_button(github),
                ],
            ),
        ],
    ));
}

fn arrow_button(action: Action, key: KeyCode) -> impl Bundle {
    let rotation = ui::arrow(key).expect("not an arrow key");
    button(action, ui::chevron(ARROW_SIZE, rotation))
}

fn text_button(action: Action, label: impl Into<String>) -> impl Bundle {
    button(action, ui::text(label, TEXT_SIZE, TEXT))
}

fn repository_button(icon: Handle<Image>) -> impl Bundle {
    (
        RepositoryLink,
        ui::Link(REPOSITORY),
        observe(ui::open_link),
        plain_button((
            ImageNode::new(icon).with_color(TEXT),
            Node {
                width: px(ICON_SIZE),
                height: px(ICON_SIZE),
                ..default()
            },
        )),
    )
}

fn button(action: Action, content: impl Bundle) -> impl Bundle {
    (ActionButton(action), plain_button(content))
}

fn plain_button(content: impl Bundle) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        ui::keycap(
            content,
            Node {
                min_width: px(BUTTON_SIZE),
                min_height: px(BUTTON_SIZE),
                padding: UiRect::horizontal(px(10)),
                border: UiRect::all(px(BORDER_WIDTH)),
                border_radius: BorderRadius::all(px(RADIUS)),
                ..default()
            },
        ),
    )
}

#[allow(clippy::type_complexity)]
fn style_buttons(
    actions: Res<ButtonInput<Action>>,
    mut buttons: Query<
        (
            Option<&ActionButton>,
            Has<Pressed>,
            &Hovered,
            &mut BackgroundColor,
        ),
        Or<(With<ActionButton>, With<RepositoryLink>)>,
    >,
) {
    for (action, pressed, hovered, mut background) in &mut buttons {
        let pressed = match action {
            Some(&ActionButton(action)) => actions.pressed(action),
            None => pressed,
        };
        let color = if pressed {
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
