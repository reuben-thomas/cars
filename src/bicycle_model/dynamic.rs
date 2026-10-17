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

/// Front and rear axle velocities in the body frame [m/s].
pub fn axle_velocities(params: &Params, state: &State) -> (Vec2, Vec2) {
    let (lf, lr) = (params.cg_to_front_axle, params.cg_to_rear_axle);
    let Vec2 { x: vx, y: vy } = state.velocity;
    (
        Vec2::new(vx, vy + lf * state.yaw_rate),
        Vec2::new(vx, vy - lr * state.yaw_rate),
    )
}

/// Front and rear tire slip angles [rad] for a steering angle [rad].
pub fn slip_angles(params: &Params, state: &State, steering: f32) -> (f32, f32) {
    let (front, rear) = axle_velocities(params, state);
    let front = Rot2::radians(-steering) * front;
    (
        front.y.atan2(front.x.abs().max(SLIP_SPEED_FLOOR)),
        rear.y.atan2(rear.x.abs().max(SLIP_SPEED_FLOOR)),
    )
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
    let drag = state.velocity.normalize_or_zero() * p.drag_force(state.velocity.length());
    let rolling_resistance = p.rolling_resistance_force(vx);
    Derivatives {
        velocity: Vec2::new(
            (front_x + fx_rear + rolling_resistance + drag.x) / p.mass + vy * state.yaw_rate,
            (front_y + fy_rear + drag.y) / p.mass - vx * state.yaw_rate,
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
