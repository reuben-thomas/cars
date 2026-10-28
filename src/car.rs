mod params;
pub mod visual;

pub use params::{Brakes, Drivetrain, Params};

use crate::bicycle_model::{self, AckermannBicycle};
use bevy::math::Isometry2d;
use bevy::prelude::*;

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, step.in_set(CarSystems));
    }
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CarSystems;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Controls {
    /// Throttle pedal [0, 1].
    pub throttle: f32,
    /// Brake pedal [0, 1].
    pub brake: f32,
    /// Steering [-1, 1], positive = left, +/-1 = full lock.
    pub steering: f32,
    /// Drive backwards when throttle is applied.
    pub reverse: bool,
    /// Lock the rear wheels.
    pub handbrake: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axle {
    Front,
    Rear,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Style {
    #[default]
    Car,
    Van,
    Bus,
}

impl Style {
    pub const ALL: [Style; 3] = [Style::Car, Style::Van, Style::Bus];

    pub fn name(self) -> &'static str {
        match self {
            Style::Car => "car",
            Style::Van => "van",
            Style::Bus => "bus",
        }
    }
}

#[derive(Component, Clone, Debug)]
#[require(Transform, Visibility, Style)]
pub struct Car {
    params: Params,
    bicycle: AckermannBicycle,
    input: bicycle_model::Input,
}

impl Default for Car {
    fn default() -> Self {
        Self::new(Params::default())
    }
}

impl Car {
    pub fn new(params: Params) -> Self {
        Self {
            bicycle: AckermannBicycle::new(params.bicycle.clone()),
            params,
            input: bicycle_model::Input::default(),
        }
    }

    /// Advances the car by `dt` [s] under the given controls.
    pub fn step(&mut self, controls: Controls, dt: f32) {
        self.input = self.bicycle_input(controls, dt);
        self.bicycle.step(self.input, dt);
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn set_params(&mut self, params: Params) {
        self.bicycle.set_params(params.bicycle.clone());
        self.params = params;
    }

    pub fn state(&self) -> &bicycle_model::State {
        self.bicycle.state()
    }

    pub fn input(&self) -> &bicycle_model::Input {
        &self.input
    }

    pub fn model(&self) -> bicycle_model::Model {
        self.bicycle.model()
    }

    pub fn speed(&self) -> f32 {
        self.state().velocity.length()
    }

    pub fn pose(&self) -> Isometry2d {
        let state = self.state();
        Isometry2d::new(state.position, state.heading)
    }

    pub fn axle_position(&self, axle: Axle) -> Vec2 {
        let params = &self.params().bicycle;
        match axle {
            Axle::Front => Vec2::new(params.cg_to_front_axle, 0.0),
            Axle::Rear => Vec2::new(-params.cg_to_rear_axle, 0.0),
        }
    }

    pub fn wheel_rotation(&self, axle: Axle) -> Rot2 {
        match axle {
            Axle::Front => Rot2::radians(self.input().steering_angle),
            Axle::Rear => Rot2::IDENTITY,
        }
    }

    fn bicycle_input(&self, controls: Controls, dt: f32) -> bicycle_model::Input {
        let p = &self.params;
        let throttle = controls.throttle.clamp(0.0, 1.0);
        let brake = controls.brake.clamp(0.0, 1.0);
        let steering = controls.steering.clamp(-1.0, 1.0);

        let direction = if controls.reverse { -1.0 } else { 1.0 };
        let drive = direction * throttle * p.drivetrain.max_force;

        let vx = self.state().velocity.x;
        let stopping_force = p.bicycle.mass * vx.abs() / dt;
        let brake = -vx.signum() * (brake * p.brakes.max_force).min(stopping_force);

        bicycle_model::Input {
            steering_angle: steering * p.max_steering,
            front_longitudinal_force: p.drivetrain.front_bias() * drive
                + p.brakes.front_bias() * brake,
            rear_longitudinal_force: p.drivetrain.rear_bias * drive + p.brakes.rear_bias * brake,
            rear_locked: controls.handbrake,
        }
    }
}

fn step(time: Res<Time<Fixed>>, mut cars: Query<(&mut Car, &Controls)>) {
    for (mut car, controls) in &mut cars {
        car.step(*controls, time.delta_secs());
    }
}
