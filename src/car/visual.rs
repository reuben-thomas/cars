mod body;
pub mod debug;

use super::{Axle, Car};
use crate::keymap::{Action, Actions};
use crate::view::to_screen;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ShowBody>().add_systems(
        Update,
        (
            sync_transform,
            toggle_body,
            body::spawn,
            body::steer_wheels,
            body::light_lamps,
        ),
    );
}

#[derive(Component, Clone, Debug)]
pub struct Dimensions {
    pub chassis_width: f32,
    pub cg_to_chassis_front: f32,
    pub cg_to_chassis_rear: f32,
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
            chassis_width: 1.85,
            cg_to_chassis_front: 2.3,
            cg_to_chassis_rear: 2.4,
            front_track: 1.75,
            rear_track: 1.75,
            wheel_radius: 0.33,
            wheel_width: 0.2,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Wheel {
    pub axle: Axle,
    pub centre: Vec2,
}

impl Dimensions {
    pub fn chassis(&self) -> Rect {
        let half_width = self.chassis_width / 2.0;
        Rect::new(
            -self.cg_to_chassis_rear,
            -half_width,
            self.cg_to_chassis_front,
            half_width,
        )
    }

    pub fn wheel_size(&self) -> Vec2 {
        Vec2::new(2.0 * self.wheel_radius, self.wheel_width)
    }

    pub fn track(&self, axle: Axle) -> f32 {
        match axle {
            Axle::Front => self.front_track,
            Axle::Rear => self.rear_track,
        }
    }

    pub fn wheels(&self, car: &Car) -> [Wheel; 4] {
        let wheel = |axle, side: f32| Wheel {
            axle,
            centre: car.axle_position(axle) + Vec2::Y * side * self.track(axle) / 2.0,
        };
        [
            wheel(Axle::Front, 1.0),
            wheel(Axle::Front, -1.0),
            wheel(Axle::Rear, 1.0),
            wheel(Axle::Rear, -1.0),
        ]
    }
}

#[derive(Resource)]
pub struct ShowBody(pub bool);

impl Default for ShowBody {
    fn default() -> Self {
        Self(true)
    }
}

fn sync_transform(mut cars: Query<(&Car, &mut Transform)>) {
    for (car, mut transform) in &mut cars {
        let state = car.state();
        transform.translation = to_screen(state.position).extend(transform.translation.z);
        transform.rotation = Quat::from_rotation_z(state.heading.as_radians());
    }
}

fn toggle_body(
    actions: Actions,
    mut show: ResMut<ShowBody>,
    mut cars: Query<&mut Visibility, With<Car>>,
) {
    if actions.just_pressed(Action::ToggleBody) {
        show.0 = !show.0;
    }
    let visibility = if show.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut car in &mut cars {
        car.set_if_neq(visibility);
    }
}
