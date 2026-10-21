use super::Dimensions;
use crate::bicycle_model::{Model, dynamic};
use crate::car::{Axle, Car};
use crate::ui::TEXT;
use crate::view::to_screen;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::FontSize;
use std::collections::VecDeque;

/// Maximum length of an arrow head [m].
const ARROW_HEAD: f32 = 0.3;
/// Half-size of the CG marker [m].
const CG_MARKER: f32 = 0.15;
const TEXT_OFFSET: f32 = 3.5;
/// Font size of the debug text [px].
const TEXT_SIZE: f32 = 13.0;

const CHASSIS: Color = Color::srgb(0.8, 0.8, 0.8);
const VELOCITY: Color = Color::srgb(0.1, 1.0, 0.3);
const FRAME: Color = Color::srgb(0.9, 0.9, 0.9);
const DRAG: Color = Color::srgb(1.0, 0.3, 0.8);
const TRAIL_ALPHA: f32 = 0.6;

const FRONT_WHEEL: Color = Color::srgb(1.0, 0.85, 0.1);
const REAR_WHEEL: Color = Color::srgb(0.1, 0.8, 1.0);

pub struct CarDebugPlugin {
    pub enabled: bool,
    pub gizmos: CarGizmos,
}

impl Default for CarDebugPlugin {
    fn default() -> Self {
        Self {
            enabled: true,
            gizmos: CarGizmos::default(),
        }
    }
}

impl Plugin for CarDebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_gizmo_config(
            self.gizmos.clone(),
            GizmoConfig {
                enabled: self.enabled,
                ..default()
            },
        )
        .add_systems(
            Update,
            (spawn, (record_trail, draw).run_if(enabled), update_text).chain(),
        );
    }
}

#[derive(Clone, Debug, Reflect, GizmoConfigGroup)]
pub struct CarGizmos {
    pub velocity: bool,
    pub chassis: bool,
    pub bicycle_model: bool,
    pub text: bool,
    pub trail: bool,
    pub velocity_arrow_time: f32,
    pub force_arrow_scale: f32,
    pub trail_duration: f32,
}

impl Default for CarGizmos {
    fn default() -> Self {
        Self {
            velocity: true,
            chassis: true,
            bicycle_model: true,
            text: true,
            trail: true,
            velocity_arrow_time: 0.25,
            force_arrow_scale: 1.0 / 2000.0,
            trail_duration: 3.0,
        }
    }
}

/// Axle state for drawing, in the world frame.
struct AxleState {
    position: Vec2,
    rotation: Rot2,
    longitudinal_force: f32,
    lateral_force: Option<f32>,
    color: Color,
}

#[derive(Component)]
pub struct DebugText(Entity);

#[derive(Component, Default)]
pub struct Trail(VecDeque<TrailPoint>);

impl Trail {
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

struct TrailPoint {
    time: f32,
    front: Vec2,
    rear: Vec2,
}

fn enabled(store: Res<GizmoConfigStore>) -> bool {
    store.config::<CarGizmos>().0.enabled
}

fn spawn(mut commands: Commands, cars: Query<Entity, Added<Car>>) {
    let font = TextFont {
        font_size: FontSize::Px(TEXT_SIZE),
        ..default()
    };
    for car in &cars {
        commands.entity(car).insert(Trail::default());
        let key = [
            ("\n\nkey: ", TEXT),
            ("velocity", VELOCITY),
            (", ", TEXT),
            ("front axle", FRONT_WHEEL),
            (", ", TEXT),
            ("rear axle", REAR_WHEEL),
            (", ", TEXT),
            ("aerodynamic drag", DRAG),
            (", ", TEXT),
            ("frame", CHASSIS),
        ]
        .map(|(label, color)| (TextSpan::new(label), font.clone(), TextColor(color)));
        commands.spawn((
            DebugText(car),
            Text2d::default(),
            Anchor::CENTER_LEFT,
            font.clone(),
            TextColor(TEXT),
            Transform::default(),
            Children::spawn(SpawnIter(key.into_iter())),
        ));
    }
}

fn record_trail(
    time: Res<Time>,
    store: Res<GizmoConfigStore>,
    mut cars: Query<(&Car, &mut Trail)>,
) {
    let duration = store.config::<CarGizmos>().1.trail_duration;
    let now = time.elapsed_secs();
    for (car, mut trail) in &mut cars {
        let pose = car.pose();
        trail.0.push_back(TrailPoint {
            time: now,
            front: pose.transform_point(car.axle_position(Axle::Front)),
            rear: pose.transform_point(car.axle_position(Axle::Rear)),
        });
        while trail
            .0
            .front()
            .is_some_and(|point| now - point.time > duration)
        {
            trail.0.pop_front();
        }
    }
}

fn update_text(
    store: Res<GizmoConfigStore>,
    cars: Query<&Car>,
    mut texts: Query<(&DebugText, &mut Text2d, &mut Transform, &mut Visibility)>,
) {
    let (config, debug) = store.config::<CarGizmos>();
    let shown = config.enabled && debug.text;
    for (DebugText(entity), mut text, mut transform, mut visibility) in &mut texts {
        let Ok(car) = cars.get(*entity) else {
            continue;
        };
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if !shown {
            continue;
        }

        let state = car.state();
        transform.translation = to_screen(state.position + Vec2::X * TEXT_OFFSET).extend(1.0);

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

fn draw(mut gizmos: Gizmos<CarGizmos>, cars: Query<(&Car, &Dimensions, &Trail)>) {
    let debug = gizmos.config_ext.clone();
    for (car, dimensions, trail) in &cars {
        if debug.trail {
            draw_trail(&mut gizmos, trail);
        }
        if debug.chassis {
            draw_chassis(&mut gizmos, car, dimensions);
        }
        if debug.bicycle_model {
            draw_bicycle_model(&mut gizmos, car, dimensions, debug.force_arrow_scale);
        }
        if debug.velocity {
            draw_velocity(&mut gizmos, car, debug.velocity_arrow_time);
        }
    }
}

fn draw_trail(gizmos: &mut Gizmos<CarGizmos>, trail: &Trail) {
    let points = &trail.0;
    gizmos.linestrip_2d(
        points.iter().map(|point| to_screen(point.front)),
        FRONT_WHEEL.with_alpha(TRAIL_ALPHA),
    );
    gizmos.linestrip_2d(
        points.iter().map(|point| to_screen(point.rear)),
        REAR_WHEEL.with_alpha(TRAIL_ALPHA),
    );
}

fn draw_velocity(gizmos: &mut Gizmos<CarGizmos>, car: &Car, arrow_time: f32) {
    let state = car.state();
    let velocity = state.heading * state.velocity;
    arrow(
        gizmos,
        state.position,
        state.position + velocity * arrow_time,
        VELOCITY,
    );
}

fn draw_chassis(gizmos: &mut Gizmos<CarGizmos>, car: &Car, dimensions: &Dimensions) {
    let pose = car.pose();
    let chassis = dimensions.chassis();
    rectangle(
        gizmos,
        pose.transform_point(chassis.center()),
        pose.rotation,
        chassis.size(),
        CHASSIS,
    );
    for wheel in dimensions.wheels(car) {
        rectangle(
            gizmos,
            pose.transform_point(wheel.centre),
            pose.rotation * car.wheel_rotation(wheel.axle),
            dimensions.wheel_size(),
            CHASSIS,
        );
    }
}

fn draw_bicycle_model(
    gizmos: &mut Gizmos<CarGizmos>,
    car: &Car,
    dimensions: &Dimensions,
    force_scale: f32,
) {
    let params = &car.params().bicycle;
    let state = car.state();
    let input = car.input();
    let pose = car.pose();

    let front = pose.transform_point(car.axle_position(Axle::Front));
    let rear = pose.transform_point(car.axle_position(Axle::Rear));
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
        AxleState {
            position: front,
            rotation: pose.rotation * car.wheel_rotation(Axle::Front),
            longitudinal_force: input.front_longitudinal_force,
            lateral_force: front_lateral_force,
            color: FRONT_WHEEL,
        },
        AxleState {
            position: rear,
            rotation: pose.rotation * car.wheel_rotation(Axle::Rear),
            longitudinal_force: input.rear_longitudinal_force,
            lateral_force: rear_lateral_force,
            color: REAR_WHEEL,
        },
    ];

    for axle in axles {
        let tip = |vector: Vec2| axle.position + vector;
        rectangle(
            gizmos,
            axle.position,
            axle.rotation,
            dimensions.wheel_size(),
            axle.color,
        );
        arrow(
            gizmos,
            axle.position,
            tip(axle.rotation * Vec2::X * axle.longitudinal_force * force_scale),
            axle.color,
        );
        if let Some(force) = axle.lateral_force {
            arrow(
                gizmos,
                axle.position,
                tip(axle.rotation * Vec2::Y * force * force_scale),
                axle.color,
            );
        }
    }

    let velocity = state.heading * state.velocity;
    let drag = params.drag_force(velocity.length());
    arrow(
        gizmos,
        state.position,
        state.position + velocity.normalize_or_zero() * drag * force_scale,
        DRAG,
    );
}

fn line(gizmos: &mut Gizmos<CarGizmos>, start: Vec2, end: Vec2, color: Color) {
    gizmos.line_2d(to_screen(start), to_screen(end), color);
}

fn arrow(gizmos: &mut Gizmos<CarGizmos>, start: Vec2, end: Vec2, color: Color) {
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

fn rectangle(
    gizmos: &mut Gizmos<CarGizmos>,
    centre: Vec2,
    rotation: Rot2,
    size: Vec2,
    color: Color,
) {
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
