mod body;
pub mod debug;

use super::{Axle, Car, Style};
use crate::view::to_screen;
use bevy::prelude::*;

pub const PAINT: Color = Color::srgb_u8(0xe0, 0x5a, 0x5a);

pub struct CarBodyPlugin;

impl Plugin for CarBodyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShowBody>().add_systems(
            Update,
            (
                sync_transform,
                show_body,
                body::spawn,
                body::steer_wheels,
                body::light_lamps,
            ),
        );
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
#[require(Paint)]
pub struct Dimensions {
    pub chassis_width: f32,
    pub front_overhang: f32,
    pub rear_overhang: f32,
    /// Wheel radius [m].
    pub wheel_radius: f32,
    /// Wheel width [m].
    pub wheel_width: f32,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self::for_style(Style::Car)
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Paint(pub Color);

impl Default for Paint {
    fn default() -> Self {
        Self(PAINT)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Wheel {
    pub axle: Axle,
    pub centre: Vec2,
}

impl Dimensions {
    pub const fn for_style(style: Style) -> Self {
        match style {
            Style::Car => Self {
                chassis_width: 1.85,
                front_overhang: 1.0,
                rear_overhang: 1.1,
                wheel_radius: 0.33,
                wheel_width: 0.2,
            },
            Style::Van => Self {
                chassis_width: 2.0,
                front_overhang: 0.9,
                rear_overhang: 1.0,
                wheel_radius: 0.35,
                wheel_width: 0.22,
            },
            Style::Bus => Self {
                chassis_width: 2.55,
                front_overhang: 2.6,
                rear_overhang: 3.4,
                wheel_radius: 0.5,
                wheel_width: 0.3,
            },
        }
    }

    pub fn chassis(&self, car: &Car) -> Rect {
        let half_width = self.chassis_width / 2.0;
        Rect::new(
            car.axle_position(Axle::Rear).x - self.rear_overhang,
            -half_width,
            car.axle_position(Axle::Front).x + self.front_overhang,
            half_width,
        )
    }

    pub fn wheel_size(&self) -> Vec2 {
        Vec2::new(2.0 * self.wheel_radius, self.wheel_width)
    }

    pub fn track(&self) -> f32 {
        self.chassis_width - self.wheel_width
    }

    pub fn wheels(&self, car: &Car) -> [Wheel; 4] {
        let wheel = |axle, side: f32| Wheel {
            axle,
            centre: car.axle_position(axle) + Vec2::Y * side * self.track() / 2.0,
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

fn show_body(show: Res<ShowBody>, mut cars: Query<&mut Visibility, With<Car>>) {
    let visibility = if show.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut car in &mut cars {
        car.set_if_neq(visibility);
    }
}
