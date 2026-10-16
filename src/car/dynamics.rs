use super::Controls;
use crate::bicycle_model::{self, AckermannBicycle};

#[derive(Clone, Debug)]
pub struct Params {
    pub bicycle: bicycle_model::Params,
    /// Road-wheel steering angle at full lock [rad].
    pub max_steering: f32,
    /// Total drive force at full throttle [N].
    pub max_drive_force: f32,
    /// Total brake force at full brake [N].
    pub max_brake_force: f32,
    /// Fraction of drive force on the front axle [0, 1].
    pub drive_bias: f32,
    /// Fraction of brake force on the front axle [0, 1].
    pub brake_bias: f32,
}

#[derive(Clone, Debug)]
pub(super) struct Dynamics {
    params: Params,
    bicycle: AckermannBicycle,
}

impl Dynamics {
    pub fn new(params: Params) -> Self {
        Self {
            bicycle: AckermannBicycle::new(params.bicycle.clone()),
            params,
        }
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn state(&self) -> &bicycle_model::State {
        self.bicycle.state()
    }

    pub fn step(&mut self, controls: Controls, dt: f32) {
        let input = self.input(controls, dt);
        self.bicycle.step(input, dt);
    }

    fn input(&self, controls: Controls, dt: f32) -> bicycle_model::Input {
        let p = &self.params;
        let throttle = controls.throttle.clamp(0.0, 1.0);
        let brake = controls.brake.clamp(0.0, 1.0);
        let steering = controls.steering.clamp(-1.0, 1.0);

        let direction = if controls.reverse { -1.0 } else { 1.0 };
        let drive = direction * throttle * p.max_drive_force;

        let vx = self.state().velocity.x;
        let stopping_force = p.bicycle.mass * vx.abs() / dt;
        let brake = -vx.signum() * (brake * p.max_brake_force).min(stopping_force);

        bicycle_model::Input {
            steering_angle: steering * p.max_steering,
            front_longitudinal_force: p.drive_bias * drive + p.brake_bias * brake,
            rear_longitudinal_force: (1.0 - p.drive_bias) * drive + (1.0 - p.brake_bias) * brake,
        }
    }
}
