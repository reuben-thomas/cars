use super::{Input, Params, State};
use bevy_math::{Rot2, Vec2};

pub fn step(params: &Params, state: &mut State, input: Input, dt: f32) {
    let speed = signed_speed(state);
    let resistance = params.drag_force(speed) + params.rolling_resistance_force(speed);
    let acceleration = (input.longitudinal_force() + resistance) / params.mass;

    let direction = Vec2::from_angle(slip_angle(params, input.steering_angle));
    let yaw_rate_per_speed = direction.x * input.steering_angle.tan() / params.wheelbase();
    state.position += state.heading * direction * speed * dt;
    state.heading =
        (state.heading * Rot2::radians(speed * yaw_rate_per_speed * dt)).fast_renormalize();

    let speed = speed + acceleration * dt;
    state.velocity = direction * speed;
    state.yaw_rate = speed * yaw_rate_per_speed;
}

/// Angle between velocity and heading for a steering angle [rad].
pub fn slip_angle(params: &Params, steering: f32) -> f32 {
    (params.cg_to_rear_axle / params.wheelbase() * steering.tan()).atan()
}

fn signed_speed(state: &State) -> f32 {
    state.velocity.x.signum() * state.velocity.length()
}
