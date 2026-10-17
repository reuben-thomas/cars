mod dynamics;
pub mod keyboard;
pub mod visual;

pub use dynamics::{Brakes, Drivetrain, Params};

use crate::bicycle_model;
use bevy::prelude::*;
use dynamics::Dynamics;
use visual::debug;

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<debug::DebugGizmos>()
            .add_systems(FixedUpdate, (keyboard::drive, step).chain())
            .add_systems(
                Update,
                (
                    keyboard::reset,
                    debug::toggle,
                    debug::draw,
                    (debug::spawn_text, debug::update_text).chain(),
                ),
            );
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

#[derive(Component, Clone, Debug)]
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

    /// Speed of the CG [m/s].
    pub fn speed(&self) -> f32 {
        self.state().velocity.length()
    }
}

fn step(time: Res<Time<Fixed>>, mut cars: Query<(&mut Car, &Controls)>) {
    for (mut car, controls) in &mut cars {
        car.step(*controls, time.delta_secs());
    }
}
