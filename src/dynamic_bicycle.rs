use bevy_ecs::prelude::Resource;
use bevy_math::{Rot2, Vec2};
use std::f32::consts::{PI, TAU};

/// Gravitational acceleration [m/s^2].
const GRAVITY: f32 = 9.81;
/// Minimum longitudinal speed used in slip angle calculations [m/s].
const SLIP_SPEED_FLOOR: f32 = 0.5;

#[derive(Clone, Debug)]
pub struct Params {
    /// Vehicle mass [kg].
    pub mass: f32,
    /// Yaw moment of inertia about the CG [kg*m^2].
    pub inertia: f32,
    /// Distance between axles [m].
    pub wheelbase: f32,
    /// Distance from CG to front axle [m].
    pub lf: f32,
    /// Cornering stiffness per unit vertical load [1/rad].
    pub cornering_coefficient: f32,
}

impl Params {
    /// Distance from CG to rear axle [m].
    pub fn lr(&self) -> f32 {
        self.wheelbase - self.lf
    }

    /// Front and rear axle cornering stiffness [N/rad].
    pub fn cornering_stiffness(&self) -> (f32, f32) {
        let weight = self.mass * GRAVITY;
        let front_load = weight * self.lr() / self.wheelbase;
        let rear_load = weight * self.lf / self.wheelbase;
        (
            front_load * self.cornering_coefficient,
            rear_load * self.cornering_coefficient,
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// Front road-wheel steering angle [rad], positive = left.
    pub steering: f32,
    /// Longitudinal front tire force along the wheel heading [N].
    pub fx_front: f32,
    /// Longitudinal rear tire force [N].
    pub fx_rear: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct Model {
    params: Params,
    position: Vec2,
    velocity: Vec2,
    heading: f32,
    yaw_rate: f32,
}

#[derive(Clone, Copy, Debug)]
struct Derivatives {
    /// Body-frame acceleration [m/s^2].
    velocity: Vec2,
    /// Yaw acceleration [rad/s^2].
    yaw_rate: f32,
}

impl Model {
    pub fn new(params: Params) -> Self {
        Self {
            params,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            heading: 0.0,
            yaw_rate: 0.0,
        }
    }

    /// Vehicle parameters.
    pub fn params(&self) -> &Params {
        &self.params
    }

    /// CG position in the world frame [m].
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// CG velocity in the body frame [m/s].
    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    /// Heading in the world frame [rad], wrapped to [-π, π].
    pub fn heading(&self) -> f32 {
        self.heading
    }

    /// Yaw rate [rad/s].
    pub fn yaw_rate(&self) -> f32 {
        self.yaw_rate
    }

    /// Advances the model by `dt` [s].
    pub fn step(&mut self, input: Input, dt: f32) {
        let derivatives = self.derivatives(input);
        self.integrate(derivatives, dt);
    }

    /// Front and rear tire slip angles [rad] for a steering angle [rad].
    pub fn slip_angles(&self, steering: f32) -> (f32, f32) {
        let Vec2 { x: vx, y: vy } = self.velocity;
        let (sin_delta, cos_delta) = steering.sin_cos();
        let front_lateral = vy + self.params.lf * self.yaw_rate;
        let front = (front_lateral * cos_delta - vx * sin_delta).atan2(
            (vx * cos_delta + front_lateral * sin_delta)
                .abs()
                .max(SLIP_SPEED_FLOOR),
        );
        let rear = (vy - self.params.lr() * self.yaw_rate).atan2(vx.abs().max(SLIP_SPEED_FLOOR));
        (front, rear)
    }

    /// Front and rear lateral tire forces in each tire's frame [N] for a steering angle [rad].
    pub fn lateral_forces(&self, steering: f32) -> (f32, f32) {
        let (cf, cr) = self.params.cornering_stiffness();
        let (af, ar) = self.slip_angles(steering);
        (-cf * af, -cr * ar)
    }

    fn derivatives(&self, input: Input) -> Derivatives {
        let p = &self.params;
        let Vec2 { x: vx, y: vy } = self.velocity;
        let (sin_delta, cos_delta) = input.steering.sin_cos();
        let (fy_front, fy_rear) = self.lateral_forces(input.steering);
        let front_x = input.fx_front * cos_delta - fy_front * sin_delta;
        let front_y = input.fx_front * sin_delta + fy_front * cos_delta;
        Derivatives {
            velocity: Vec2::new(
                (front_x + input.fx_rear) / p.mass + vy * self.yaw_rate,
                (front_y + fy_rear) / p.mass - vx * self.yaw_rate,
            ),
            yaw_rate: (p.lf * front_y - p.lr() * fy_rear) / p.inertia,
        }
    }

    fn integrate(&mut self, derivatives: Derivatives, dt: f32) {
        self.position += Rot2::radians(self.heading) * self.velocity * dt;
        self.heading = wrap_angle(self.heading + self.yaw_rate * dt);
        self.velocity += derivatives.velocity * dt;
        self.yaw_rate += derivatives.yaw_rate * dt;
    }
}

fn wrap_angle(angle: f32) -> f32 {
    let angle = angle % TAU;
    if angle.abs() > PI {
        angle - angle.signum() * TAU
    } else {
        angle
    }
}
