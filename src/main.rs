use bevy::camera::Hdr;
use bevy::camera::visibility::RenderLayers;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use cars::car::visual::Dimensions;
use cars::car::visual::debug::CarDebugPlugin;
use cars::car::{Car, CarPlugin, Controls, keyboard};
use cars::help::HelpPlugin;
use cars::keymap::KeymapPlugin;
use cars::view::{PIXELS_PER_METRE, to_screen};

const PHYSICS_HZ: f64 = 240.0;
const GRID_SPACING: f32 = 5.0;
const GRID: Color = Color::srgb(0.12, 0.16, 0.2);
const GRID_LAYER: usize = 1;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct GridGizmos;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "cars".into(),
                    resolution: (1100, 750).into(),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            KeymapPlugin,
            CarPlugin,
            HelpPlugin,
            CarDebugPlugin {
                enabled: false,
                ..default()
            },
        ))
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.05)))
        .insert_resource(Time::<Fixed>::from_hz(PHYSICS_HZ))
        .init_gizmo_group::<GridGizmos>()
        .add_systems(Startup, setup)
        .add_systems(Update, (follow_car, draw_grid))
        .run();
}

fn setup(mut commands: Commands, mut gizmos: ResMut<GizmoConfigStore>) {
    gizmos.config_mut::<GridGizmos>().0.render_layers = RenderLayers::layer(GRID_LAYER);
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            ..default()
        },
        Hdr,
        RenderLayers::layer(GRID_LAYER),
    ));
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Hdr,
        Bloom {
            intensity: 0.4,
            prefilter: BloomPrefilter {
                threshold: 1.0,
                threshold_softness: 0.2,
            },
            composite_mode: BloomCompositeMode::Additive,
            ..Bloom::NATURAL
        },
    ));
    commands.spawn((
        Car::default(),
        Dimensions::default(),
        Controls::default(),
        keyboard::KeyboardControlled,
    ));
}

fn follow_car(car: Single<&Car>, mut cameras: Query<&mut Transform, With<Camera2d>>) {
    let centre = to_screen(car.state().position);
    for mut camera in &mut cameras {
        camera.translation.x = centre.x;
        camera.translation.y = centre.y;
    }
}

fn draw_grid(
    mut gizmos: Gizmos<GridGizmos>,
    car: Single<&Car>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let centre = car.state().position;
    let extent = window.width().max(window.height()) / PIXELS_PER_METRE / 2.0 + GRID_SPACING;
    let first = ((centre - extent) / GRID_SPACING).floor().as_ivec2();
    let last = ((centre + extent) / GRID_SPACING).ceil().as_ivec2();

    for x in first.x..=last.x {
        let x = x as f32 * GRID_SPACING;
        gizmos.line_2d(
            to_screen(Vec2::new(x, centre.y - extent)),
            to_screen(Vec2::new(x, centre.y + extent)),
            GRID,
        );
    }
    for y in first.y..=last.y {
        let y = y as f32 * GRID_SPACING;
        gizmos.line_2d(
            to_screen(Vec2::new(centre.x - extent, y)),
            to_screen(Vec2::new(centre.x + extent, y)),
            GRID,
        );
    }
}
