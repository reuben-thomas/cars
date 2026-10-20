pub mod dynamic;
pub mod kinematic;

use crate::physics::constants::{AIR_DENSITY, GRAVITY};
use bevy_ecs::prelude::Resource;
use bevy_math::{Rot2, Vec2};

/// Maximum speed for which the kinematic model is used [m/s].
const KINEMATIC_SPEED_MAX: f32 = 1.5;
/// Minimum speed for which the dynamic model is used [m/s].
const DYNAMIC_SPEED_MIN: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct Params {
    /// Vehicle mass [kg].
    pub mass: f32,
    /// Yaw moment of inertia [kg*m^2].
    pub yaw_inertia: f32,
    /// Distance from CG to front axle [m].
    pub cg_to_front_axle: f32,
    /// Distance from CG to rear axle [m].
    pub cg_to_rear_axle: f32,
    /// Cornering stiffness per unit axle load [1/rad].
    pub cornering_coefficient: f32,
    /// Rolling resistance [kg/s].
    pub rolling_resistance: f32,
    /// Aerodynamic drag coefficient, Cd.
    pub drag_coefficient: f32,
    /// Frontal area [m^2].
    pub frontal_area: f32,
}

impl Params {
    pub const fn wheelbase(&self) -> f32 {
        self.cg_to_front_axle + self.cg_to_rear_axle
    }

    /// Aerodynamic drag force opposing a signed speed [m/s] [N].
    pub const fn drag_force(&self, speed: f32) -> f32 {
        -0.5 * AIR_DENSITY * self.drag_coefficient * self.frontal_area * speed * speed.abs()
    }

    /// Rolling resistance force opposing a signed speed [m/s] [N].
    pub const fn rolling_resistance_force(&self, speed: f32) -> f32 {
        -self.rolling_resistance * speed
    }

    /// Front and rear axle cornering stiffness [N/rad].
    pub const fn cornering_stiffness(&self) -> (f32, f32) {
        let weight = self.mass * GRAVITY;
        let front_load = weight * self.cg_to_rear_axle / self.wheelbase();
        let rear_load = weight * self.cg_to_front_axle / self.wheelbase();
        (
            front_load * self.cornering_coefficient,
            rear_load * self.cornering_coefficient,
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// Front road-wheel steering angle [rad], positive = left.
    pub steering_angle: f32,
    /// Force at the front axle along the front wheel's direction vector [N].
    pub front_longitudinal_force: f32,
    /// Force at the rear axle along the rear wheel's direction vector [N].
    pub rear_longitudinal_force: f32,
}

impl Input {
    /// Total longitudinal tire force [N].
    pub const fn longitudinal_force(&self) -> f32 {
        self.front_longitudinal_force + self.rear_longitudinal_force
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    /// CG position in the world frame [m].
    pub position: Vec2,
    /// CG velocity in the body frame [m/s].
    pub velocity: Vec2,
    /// Heading in the world frame.
    pub heading: Rot2,
    /// Yaw rate [rad/s].
    pub yaw_rate: f32,
}

impl State {
    pub fn slip_angle(&self) -> f32 {
        self.velocity.y.atan2(self.velocity.x.abs())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    Kinematic,
    Dynamic,
}

#[derive(Resource, Clone, Debug)]
pub struct AckermannBicycle {
    params: Params,
    state: State,
    model: Model,
}

impl AckermannBicycle {
    pub fn new(params: Params) -> Self {
        Self {
            params,
            state: State::default(),
            model: Model::Kinematic,
        }
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn model(&self) -> Model {
        self.model
    }

    /// Advances the model by `dt` [s].
    pub fn step(&mut self, input: Input, dt: f32) {
        match self.model {
            Model::Kinematic => kinematic::step(&self.params, &mut self.state, input, dt),
            Model::Dynamic => dynamic::step(&self.params, &mut self.state, input, dt),
        }
        self.update_model();
    }

    fn update_model(&mut self) {
        let speed = self.state.velocity.length();
        self.model = match self.model {
            Model::Kinematic if speed > KINEMATIC_SPEED_MAX => Model::Dynamic,
            Model::Dynamic if speed < DYNAMIC_SPEED_MIN => Model::Kinematic,
            model => model,
        };
    }
}
