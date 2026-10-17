pub mod debug;

use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Dimensions {
    /// Overall body length [m].
    pub length: f32,
    /// Overall body width [m].
    pub width: f32,
    /// Distance from the front axle to the front of the body [m].
    pub front_overhang: f32,
    /// Distance between the front wheel centres [m].
    pub front_track: f32,
    /// Distance between the rear wheel centres [m].
    pub rear_track: f32,
    /// Wheel radius [m].
    pub wheel_radius: f32,
    /// Wheel width [m].
    pub wheel_width: f32,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            length: 4.4,
            width: 1.8,
            front_overhang: 0.9,
            front_track: 1.5,
            rear_track: 1.5,
            wheel_radius: 0.3,
            wheel_width: 0.2,
        }
    }
}
