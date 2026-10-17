mod dynamics;
pub mod keyboard;
pub mod visual;

pub use dynamics::{Brakes, Drivetrain, Params};

use crate::bicycle_model;
use bevy::math::Isometry2d;
use bevy::prelude::*;
use dynamics::Dynamics;

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(visual::plugin)
            .add_systems(FixedUpdate, (keyboard::drive, step).chain())
            .add_systems(Update, keyboard::reset);
    }
}

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
}

impl Default for Params {
    fn default() -> Self {
        Self {
            bicycle: bicycle_model::Params {
                mass: 1200.0,
                yaw_inertia: 2000.0,
                cg_to_front_axle: 1.3,
                cg_to_rear_axle: 1.3,
                cornering_coefficient: 8.0,
                rolling_resistance: 6.0,
                drag_coefficient: 0.35,
                frontal_area: 2.0,
            },
            max_steering: 0.5,
            drivetrain: Drivetrain::rwd(10_000.0),
            brakes: Brakes::new(10_000.0, 0.5),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axle {
    Front,
    Rear,
}

#[derive(Component, Clone, Debug)]
#[require(Transform, Visibility)]
pub struct Car {
    dynamics: Dynamics,
}

impl Default for Car {
    fn default() -> Self {
        Self::new(Params::default())
    }
}

impl Car {
    pub fn new(params: Params) -> Self {
        Self {
            dynamics: Dynamics::new(params),
        }
    }

    /// Advances the car by `dt` [s] under the given controls.
    pub fn step(&mut self, controls: Controls, dt: f32) {
        self.dynamics.step(controls, dt);
    }

    pub fn params(&self) -> &Params {
        self.dynamics.params()
    }

    pub fn state(&self) -> &bicycle_model::State {
        self.dynamics.state()
    }

    pub fn input(&self) -> &bicycle_model::Input {
        self.dynamics.input()
    }

    pub fn model(&self) -> bicycle_model::Model {
        self.dynamics.model()
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
}

fn step(time: Res<Time<Fixed>>, mut cars: Query<(&mut Car, &Controls)>) {
    for (mut car, controls) in &mut cars {
        car.step(*controls, time.delta_secs());
    }
}
