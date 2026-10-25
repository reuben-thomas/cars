use crate::bicycle_model;

#[derive(Clone, Debug)]
pub struct Params {
    pub bicycle: bicycle_model::Params,
    /// Road-wheel steering angle at full lock [rad].
    pub max_steering: f32,
    pub drivetrain: Drivetrain,
    pub brakes: Brakes,
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
                friction_coefficient: 0.8,
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
