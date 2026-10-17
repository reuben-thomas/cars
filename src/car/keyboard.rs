use super::{Car, Controls};
use bevy::prelude::*;

/// Rate at which the steering moves towards its target [full locks/s].
const STEERING_RATE: f32 = 2.0;
/// Longitudinal speed above which the opposite pedal brakes instead of
/// changing direction [m/s].
const DIRECTION_CHANGE_SPEED: f32 = 0.02;

pub const HELP: &str = "W/Up: forward | S/Down: brake/reverse | A/D/Left/Right: steer \
                        | R: reset | `: debug | F1: velocity | F2: chassis \
                        | F3: bicycle model | F4: text | F5: trail | F6: body";

#[derive(Component)]
pub struct KeyboardControlled;

pub fn drive(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    mut cars: Query<(&Car, &mut Controls), With<KeyboardControlled>>,
) {
    let steering_target = axis(
        keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]),
        keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]),
    );
    let request = axis(
        keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]),
        keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]),
    );
    let max_steering_change = STEERING_RATE * time.delta_secs();

    for (car, mut controls) in &mut cars {
        controls.steering +=
            (steering_target - controls.steering).clamp(-max_steering_change, max_steering_change);

        // Pressing against the direction of travel brakes to a stop first.
        let braking = request * car.state().velocity.x < -DIRECTION_CHANGE_SPEED;
        controls.brake = if braking { 1.0 } else { 0.0 };
        controls.throttle = if braking { 0.0 } else { request.abs() };
        controls.reverse = request < 0.0;
    }
}

pub fn reset(keys: Res<ButtonInput<KeyCode>>, mut cars: Query<&mut Car, With<KeyboardControlled>>) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }
    for mut car in &mut cars {
        let reset = Car::new(car.params().clone());
        *car = reset;
    }
}

fn axis(positive: bool, negative: bool) -> f32 {
    f32::from(u8::from(positive)) - f32::from(u8::from(negative))
}
