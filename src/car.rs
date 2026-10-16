mod dynamics;

pub use dynamics::Params;

use crate::bicycle_model;
use bevy_ecs::prelude::Component;
use dynamics::Dynamics;

/// Driver controls.
#[derive(Clone, Copy, Debug, Default)]
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

impl Params {
    pub const BMW_E30: Self = Self {
        bicycle: bicycle_model::Params {
            mass: 1093.3,
            yaw_inertia: 1791.6,
            cg_to_front_axle: 1.1562,
            cg_to_rear_axle: 1.4227,
            cornering_coefficient: 21.92,
            rolling_resistance: 6.5,
            drag_coefficient: 0.38,
            frontal_area: 1.9,
        },
        max_steering: 0.46,
        max_drive_force: 2800.0,
        max_brake_force: 10_000.0,
        drive_bias: 0.0,
        brake_bias: 0.66,
    };
}

#[derive(Component, Clone, Debug)]
pub struct Car {
    dynamics: Dynamics,
}

impl Car {
    pub fn new(params: Params) -> Self {
        Self {
            dynamics: Dynamics::new(params),
        }
    }

    pub fn bmw_e30() -> Self {
        Self::new(Params::BMW_E30)
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

    /// Speed of the CG [m/s].
    pub fn speed(&self) -> f32 {
        self.state().velocity.length()
    }
}
