use crate::car::Car;
use crate::bicycle_model::{Model, dynamic};
use crate::view::to_screen;
use super::Dimensions;
use bevy::prelude::*;
use bevy::text::FontSize;

/// Travel time represented by velocity arrows [s].
const VELOCITY_ARROW_TIME: f32 = 0.25;
/// Length of force arrows per unit force [m/N].
const FORCE_ARROW_SCALE: f32 = 1.0 / 2000.0;
/// Maximum length of an arrow head [m].
const ARROW_HEAD: f32 = 0.3;
/// Half-size of the CG marker [m].
const CG_MARKER: f32 = 0.15;
/// Offset of the debug text from the CG on screen [px].
const TEXT_OFFSET: Vec2 = Vec2::new(150.0, 60.0);
/// Font size of the debug text [px].
const TEXT_SIZE: f32 = 13.0;

const CHASSIS: Color = Color::srgb(0.8, 0.8, 0.8);
const VELOCITY: Color = Color::srgb(0.1, 1.0, 0.3);
const FRAME: Color = Color::srgb(0.9, 0.9, 0.9);
const DRAG: Color = Color::srgb(1.0, 0.3, 0.8);
const TEXT: Color = Color::srgb(0.85, 0.9, 0.95);

const FRONT_WHEEL: Color = Color::srgb(1.0, 0.85, 0.1);
const REAR_WHEEL: Color = Color::srgb(0.1, 0.8, 1.0);

/// Axle state for drawing, in the world frame.
struct Axle {
    position: Vec2,
    rotation: Rot2,
    longitudinal_force: f32,
    lateral_force: Option<f32>,
    color: Color,
}

#[derive(Component)]
pub struct DebugText(Entity);

#[derive(Resource)]
pub struct DebugGizmos {
    pub velocity: bool,
    pub chassis: bool,
    pub bicycle_model: bool,
    pub text: bool,
}

impl Default for DebugGizmos {
    fn default() -> Self {
        Self {
            velocity: true,
            chassis: true,
            bicycle_model: true,
            text: true,
        }
    }
}

pub fn toggle(keys: Res<ButtonInput<KeyCode>>, mut debug: ResMut<DebugGizmos>) {
    if keys.just_pressed(KeyCode::F1) {
        debug.velocity = !debug.velocity;
    }
    if keys.just_pressed(KeyCode::F2) {
        debug.chassis = !debug.chassis;
    }
    if keys.just_pressed(KeyCode::F3) {
        debug.bicycle_model = !debug.bicycle_model;
    }
    if keys.just_pressed(KeyCode::F4) {
        debug.text = !debug.text;
    }
}

pub fn spawn_text(mut commands: Commands, cars: Query<Entity, Added<Car>>) {
    for car in &cars {
        commands.spawn((
            DebugText(car),
            Text2d::default(),
            TextFont {
                font_size: FontSize::Px(TEXT_SIZE),
                ..default()
            },
            TextColor(TEXT),
            Transform::default(),
        ));
    }
}

pub fn update_text(
    debug: Res<DebugGizmos>,
    cars: Query<&Car>,
    mut texts: Query<(&DebugText, &mut Text2d, &mut Transform, &mut Visibility)>,
) {
    for (DebugText(entity), mut text, mut transform, mut visibility) in &mut texts {
        let Ok(car) = cars.get(*entity) else {
            continue;
        };
        *visibility = if debug.text {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        let state = car.state();
        transform.translation = (to_screen(state.position) + TEXT_OFFSET).extend(1.0);

        let model = car.model();
        let speed = car.speed() * 3.6;
        let Vec2 { x: vx, y: vy } = state.velocity;
        let heading = state.heading.as_degrees();
        let yaw_rate = state.yaw_rate.to_degrees();
        let slip_angle = state.slip_angle().to_degrees();
        let steering = car.input().steering_angle.to_degrees();
        text.0 = format!(
            "model: {model:?}\n\
             speed: {speed:.1} km/h\n\
             velocity: ({vx:.2}, {vy:.2}) m/s\n\
             heading: {heading:.1} deg\n\
             yaw rate: {yaw_rate:.1} deg/s\n\
             slip angle: {slip_angle:.1} deg\n\
             steering: {steering:.1} deg"
        );
    }
}

pub fn draw(mut gizmos: Gizmos, debug: Res<DebugGizmos>, cars: Query<(&Car, &Dimensions)>) {
    for (car, dimensions) in &cars {
        if debug.chassis {
            draw_chassis(&mut gizmos, car, dimensions);
        }
        if debug.bicycle_model {
            draw_bicycle_model(&mut gizmos, car, dimensions);
        }
        if debug.velocity {
            draw_velocity(&mut gizmos, car);
        }
    }
}

fn draw_velocity(gizmos: &mut Gizmos, car: &Car) {
    let state = car.state();
    let velocity = state.heading * state.velocity;
    arrow(
        gizmos,
        state.position,
        state.position + velocity * VELOCITY_ARROW_TIME,
        VELOCITY,
    );
}

fn draw_chassis(gizmos: &mut Gizmos, car: &Car, dimensions: &Dimensions) {
    let params = &car.params().bicycle;
    let (lf, lr) = (params.cg_to_front_axle, params.cg_to_rear_axle);
    let heading = car.state().heading;
    let front_wheel = heading * Rot2::radians(car.input().steering_angle);
    let wheel = Vec2::new(2.0 * dimensions.wheel_radius, dimensions.wheel_width);

    let body_centre = lf + dimensions.front_overhang - dimensions.length / 2.0;
    rectangle(
        gizmos,
        body_to_world(car, Vec2::new(body_centre, 0.0)),
        heading,
        Vec2::new(dimensions.length, dimensions.width),
        CHASSIS,
    );
    // Both front wheels turn by the bicycle model's steering angle.
    for side in [-0.5, 0.5] {
        let front = body_to_world(car, Vec2::new(lf, side * dimensions.front_track));
        let rear = body_to_world(car, Vec2::new(-lr, side * dimensions.rear_track));
        rectangle(gizmos, front, front_wheel, wheel, CHASSIS);
        rectangle(gizmos, rear, heading, wheel, CHASSIS);
    }
}

fn draw_bicycle_model(gizmos: &mut Gizmos, car: &Car, dimensions: &Dimensions) {
    let params = &car.params().bicycle;
    let state = car.state();
    let input = car.input();
    let wheel = Vec2::new(2.0 * dimensions.wheel_radius, dimensions.wheel_width);

    let front = body_to_world(car, Vec2::new(params.cg_to_front_axle, 0.0));
    let rear = body_to_world(car, Vec2::new(-params.cg_to_rear_axle, 0.0));
    line(gizmos, rear, front, FRAME);
    line(
        gizmos,
        state.position - Vec2::X * CG_MARKER,
        state.position + Vec2::X * CG_MARKER,
        FRAME,
    );
    line(
        gizmos,
        state.position - Vec2::Y * CG_MARKER,
        state.position + Vec2::Y * CG_MARKER,
        FRAME,
    );

    let (front_lateral_force, rear_lateral_force) = match car.model() {
        Model::Dynamic => {
            let (front, rear) = dynamic::lateral_forces(params, state, input.steering_angle);
            (Some(front), Some(rear))
        }
        Model::Kinematic => (None, None),
    };
    let axles = [
        Axle {
            position: front,
            rotation: state.heading * Rot2::radians(input.steering_angle),
            longitudinal_force: input.front_longitudinal_force,
            lateral_force: front_lateral_force,
            color: FRONT_WHEEL,
        },
        Axle {
            position: rear,
            rotation: state.heading,
            longitudinal_force: input.rear_longitudinal_force,
            lateral_force: rear_lateral_force,
            color: REAR_WHEEL,
        },
    ];

    for axle in axles {
        let tip = |vector: Vec2| axle.position + vector;
        rectangle(gizmos, axle.position, axle.rotation, wheel, axle.color);
        arrow(
            gizmos,
            axle.position,
            tip(axle.rotation * Vec2::X * axle.longitudinal_force * FORCE_ARROW_SCALE),
            axle.color,
        );
        if let Some(force) = axle.lateral_force {
            arrow(
                gizmos,
                axle.position,
                tip(axle.rotation * Vec2::Y * force * FORCE_ARROW_SCALE),
                axle.color,
            );
        }
    }

    let velocity = state.heading * state.velocity;
    let drag = params.drag_force(velocity.length());
    arrow(
        gizmos,
        state.position,
        state.position + velocity.normalize_or_zero() * drag * FORCE_ARROW_SCALE,
        DRAG,
    );
}

fn body_to_world(car: &Car, point: Vec2) -> Vec2 {
    let state = car.state();
    state.position + state.heading * point
}

fn line(gizmos: &mut Gizmos, start: Vec2, end: Vec2, color: Color) {
    gizmos.line_2d(to_screen(start), to_screen(end), color);
}

fn arrow(gizmos: &mut Gizmos, start: Vec2, end: Vec2, color: Color) {
    let shaft = end - start;
    let Some(direction) = shaft.try_normalize() else {
        return;
    };
    line(gizmos, start, end, color);
    let head = ARROW_HEAD.min(shaft.length() / 2.0);
    for side in [-1.0, 1.0] {
        let barb = Rot2::degrees(150.0 * side) * direction * head;
        line(gizmos, end, end + barb, color);
    }
}

fn rectangle(gizmos: &mut Gizmos, centre: Vec2, rotation: Rot2, size: Vec2, color: Color) {
    let half = size / 2.0;
    let corners = [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
    ]
    .map(|corner| centre + rotation * corner);
    for (start, end) in corners.iter().zip(corners.iter().cycle().skip(1)) {
        line(gizmos, *start, *end, color);
    }
}
