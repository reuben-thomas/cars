use crate::car::visual::ShowBody;
use crate::car::visual::debug::{CarGizmos, Trail};
use crate::car::{Car, CarSystems, Controls};
use crate::keymap::Action;
use bevy::prelude::*;

/// Rate at which the steering moves towards its target [full locks/s].
const STEERING_RATE: f32 = 2.0;
/// Longitudinal speed above which the opposite pedal brakes instead of
/// changing direction [m/s].
const DIRECTION_CHANGE_SPEED: f32 = 0.02;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, drive.before(CarSystems))
            .add_systems(Update, (reset, toggle_debug));
    }
}

#[derive(Component)]
pub struct Player;

fn drive(
    actions: Res<ButtonInput<Action>>,
    time: Res<Time<Fixed>>,
    mut cars: Query<(&Car, &mut Controls), With<Player>>,
) {
    let steering_target = axis(
        actions.pressed(Action::SteerLeft),
        actions.pressed(Action::SteerRight),
    );
    let request = axis(
        actions.pressed(Action::Accelerate),
        actions.pressed(Action::Brake),
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

fn reset(
    actions: Res<ButtonInput<Action>>,
    mut cars: Query<(&mut Car, Option<&mut Trail>), With<Player>>,
) {
    if !actions.just_pressed(Action::Reset) {
        return;
    }
    for (mut car, trail) in &mut cars {
        let reset = Car::new(car.params().clone());
        *car = reset;
        if let Some(mut trail) = trail {
            trail.clear();
        }
    }
}

fn toggle_debug(
    actions: Res<ButtonInput<Action>>,
    mut store: ResMut<GizmoConfigStore>,
    mut show: ResMut<ShowBody>,
) {
    if actions.just_pressed(Action::ToggleDebug) {
        let config = store.config_mut::<CarGizmos>().0;
        config.enabled = !config.enabled;
        show.0 = !config.enabled;
    }
}

fn axis(positive: bool, negative: bool) -> f32 {
    f32::from(u8::from(positive)) - f32::from(u8::from(negative))
}
