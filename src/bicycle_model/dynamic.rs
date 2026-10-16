use super::{Input, Params, State};
use bevy_math::{Rot2, Vec2};

const SLIP_SPEED_FLOOR: f32 = 0.5;

#[derive(Clone, Copy, Debug)]
struct Derivatives {
    velocity: Vec2,
    yaw_rate: f32,
}

pub fn step(params: &Params, state: &mut State, input: Input, dt: f32) {
    let derivatives = derivatives(params, state, input);
    integrate(state, derivatives, dt);
}

/// Front and rear tire slip angles [rad] for a steering angle [rad].
pub fn slip_angles(params: &Params, state: &State, steering: f32) -> (f32, f32) {
    let (lf, lr) = (params.cg_to_front_axle, params.cg_to_rear_axle);
    let Vec2 { x: vx, y: vy } = state.velocity;
    let (sin_delta, cos_delta) = steering.sin_cos();
    let front_lateral = vy + lf * state.yaw_rate;
    let front = (front_lateral * cos_delta - vx * sin_delta).atan2(
        (vx * cos_delta + front_lateral * sin_delta)
            .abs()
            .max(SLIP_SPEED_FLOOR),
    );
    let rear = (vy - lr * state.yaw_rate).atan2(vx.abs().max(SLIP_SPEED_FLOOR));
    (front, rear)
}

/// Front and rear lateral tire forces (in tire frame) [N] for a steering angle [rad].
pub fn lateral_forces(params: &Params, state: &State, steering: f32) -> (f32, f32) {
    let (cf, cr) = params.cornering_stiffness();
    let (af, ar) = slip_angles(params, state, steering);
    (-cf * af, -cr * ar)
}

fn derivatives(p: &Params, state: &State, input: Input) -> Derivatives {
    let (lf, lr) = (p.cg_to_front_axle, p.cg_to_rear_axle);
    let (fx_front, fx_rear) = (
        input.front_longitudinal_force,
        input.rear_longitudinal_force,
    );
    let Vec2 { x: vx, y: vy } = state.velocity;
    let (sin_delta, cos_delta) = input.steering_angle.sin_cos();
    let (fy_front, fy_rear) = lateral_forces(p, state, input.steering_angle);
    let front_x = fx_front * cos_delta - fy_front * sin_delta;
    let front_y = fx_front * sin_delta + fy_front * cos_delta;
    Derivatives {
        velocity: Vec2::new(
            (front_x + fx_rear) / p.mass + vy * state.yaw_rate,
            (front_y + fy_rear) / p.mass - vx * state.yaw_rate,
        ),
        yaw_rate: (lf * front_y - lr * fy_rear) / p.yaw_inertia,
    }
}

fn integrate(state: &mut State, derivatives: Derivatives, dt: f32) {
    state.position += state.heading * state.velocity * dt;
    state.heading = (state.heading * Rot2::radians(state.yaw_rate * dt)).fast_renormalize();
    state.velocity += derivatives.velocity * dt;
    state.yaw_rate += derivatives.yaw_rate * dt;
}
