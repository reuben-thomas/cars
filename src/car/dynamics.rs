use super::Controls;
use crate::bicycle_model::{self, AckermannBicycle};

#[derive(Clone, Debug)]
pub struct Params {
    pub bicycle: bicycle_model::Params,
    /// Road-wheel steering angle at full lock [rad].
    pub max_steering: f32,
    pub drivetrain: Drivetrain,
    pub brakes: Brakes,
}

#[derive(Clone, Copy, Debug)]
pub struct Drivetrain {
    /// Total drive force at full throttle [N].
    pub max_force: f32,
    /// Fraction of drive force on the rear axle [0, 1].
    pub rear_bias: f32,
}

impl Drivetrain {
    pub const fn fwd(max_force: f32) -> Self {
        Self::awd(max_force, 0.0)
    }

    pub const fn rwd(max_force: f32) -> Self {
        Self::awd(max_force, 1.0)
    }

    pub const fn awd(max_force: f32, rear_bias: f32) -> Self {
        Self {
            max_force,
            rear_bias,
        }
    }

    pub const fn front_bias(&self) -> f32 {
        1.0 - self.rear_bias
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Brakes {
    /// Total brake force at full brake [N].
    pub max_force: f32,
    /// Fraction of brake force on the rear axle [0, 1].
    pub rear_bias: f32,
}

impl Brakes {
    pub const fn new(max_force: f32, rear_bias: f32) -> Self {
        Self {
            max_force,
            rear_bias,
        }
    }

    pub const fn front_bias(&self) -> f32 {
        1.0 - self.rear_bias
    }
}

#[derive(Clone, Debug)]
pub(super) struct Dynamics {
    params: Params,
    bicycle: AckermannBicycle,
    input: bicycle_model::Input,
}

impl Dynamics {
    pub fn new(params: Params) -> Self {
        Self {
            bicycle: AckermannBicycle::new(params.bicycle.clone()),
            params,
            input: bicycle_model::Input::default(),
        }
    }

    pub fn params(&self) -> &Params {
        &self.params
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

    pub fn step(&mut self, controls: Controls, dt: f32) {
        self.input = self.bicycle_input(controls, dt);
        self.bicycle.step(self.input, dt);
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
        }
    }
}
