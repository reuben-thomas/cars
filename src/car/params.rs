use super::Style;
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
        Self::preset(Style::Car)
    }
}

impl Params {
    pub fn preset(style: Style) -> Self {
        let car = Self::car();
        match style {
            Style::Car => car,
            Style::Van => Self {
                bicycle: bicycle_model::Params {
                    mass: 2500.0,
                    yaw_inertia: 6000.0,
                    cg_to_front_axle: 1.5,
                    cg_to_rear_axle: 1.8,
                    rolling_resistance: 10.0,
                    drag_coefficient: 0.4,
                    frontal_area: 3.8,
                    ..car.bicycle
                },
                drivetrain: Drivetrain::rwd(15_000.0),
                brakes: Brakes::new(20_000.0, 0.4),
                ..car
            },
            Style::Bus => Self {
                bicycle: bicycle_model::Params {
                    mass: 12_000.0,
                    yaw_inertia: 150_000.0,
                    cg_to_front_axle: 3.6,
                    cg_to_rear_axle: 2.4,
                    front_cornering_coefficient: 7.0,
                    rear_cornering_coefficient: 7.0,
                    friction_coefficient: 0.7,
                    rolling_resistance: 40.0,
                    drag_coefficient: 0.6,
                    frontal_area: 7.5,
                },
                max_steering: 0.65,
                drivetrain: Drivetrain::rwd(24_000.0),
                brakes: Brakes::new(60_000.0, 0.5),
            },
        }
    }

    fn car() -> Self {
        Self {
            bicycle: bicycle_model::Params {
                mass: 1200.0,
                yaw_inertia: 2000.0,
                cg_to_front_axle: 1.3,
                cg_to_rear_axle: 1.3,
                front_cornering_coefficient: 8.0,
                rear_cornering_coefficient: 8.0,
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
