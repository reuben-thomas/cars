use super::{Dimensions, Paint, Wheel};
use crate::car::{Car, Controls};
use crate::view::to_screen;
use bevy::prelude::*;

const TRIM: Color = Color::srgb_u8(0x43, 0x43, 0x43);
const ROOF: Color = Color::srgb_u8(0xd9, 0xd9, 0xd9);
const HEADLIGHT: Color = Color::WHITE;
const INDICATOR: Color = Color::srgb_u8(0xf1, 0xc2, 0x32);
const TAILLIGHT: Color = Color::srgb_u8(0xe0, 0x66, 0x66);
const GLOW: f32 = 5.0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lamp {
    Head,
    Indicator,
    Tail,
}

impl Lamp {
    fn lit(self, controls: &Controls) -> bool {
        match self {
            Lamp::Head | Lamp::Indicator => true,
            Lamp::Tail => controls.reverse,
        }
    }

    const fn color(self) -> Color {
        match self {
            Lamp::Head => HEADLIGHT,
            Lamp::Indicator => INDICATOR,
            Lamp::Tail => TAILLIGHT,
        }
    }
}

mod drawing {
    use super::Lamp;
    use bevy::prelude::*;

    pub const CHASSIS: Rect = rect(-180.0, 0.0, 180.0, 680.0);

    pub struct Part {
        pub rect: Rect,
        pub color: Color,
        pub z: f32,
        pub lamp: Option<Lamp>,
        pub mirrored: bool,
        pub painted: bool,
    }

    impl Part {
        pub fn rects(&self) -> impl Iterator<Item = Rect> {
            let Rect { min, max } = self.rect;
            let mirror = rect(-max.x, min.y, -min.x, max.y);
            std::iter::once(self.rect).chain(self.mirrored.then_some(mirror))
        }
    }

    pub const PARTS: [Part; 6] = [
        painted(part(-180.0, 0.0, 180.0, 680.0, Color::NONE, 0.1)),
        part(-150.0, 180.0, 150.0, 640.0, super::TRIM, 0.2),
        part(-150.0, 320.0, 150.0, 580.0, super::ROOF, 0.3),
        mirrored(lamp(100.0, 0.0, 165.0, 35.0, Lamp::Head)),
        mirrored(lamp(100.0, 35.0, 165.0, 60.0, Lamp::Indicator)),
        lamp(-180.0, 665.0, 180.0, 680.0, Lamp::Tail),
    ];

    const fn rect(left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        Rect {
            min: Vec2::new(left, top),
            max: Vec2::new(right, bottom),
        }
    }

    const fn part(left: f32, top: f32, right: f32, bottom: f32, color: Color, z: f32) -> Part {
        Part {
            rect: rect(left, top, right, bottom),
            color,
            z,
            lamp: None,
            mirrored: false,
            painted: false,
        }
    }

    const fn lamp(left: f32, top: f32, right: f32, bottom: f32, lamp: Lamp) -> Part {
        Part {
            lamp: Some(lamp),
            ..part(left, top, right, bottom, lamp.color(), 0.2)
        }
    }

    const fn painted(part: Part) -> Part {
        Part {
            painted: true,
            ..part
        }
    }

    const fn mirrored(part: Part) -> Part {
        Part {
            mirrored: true,
            ..part
        }
    }
}

fn from_drawing(chassis: Rect, point: Vec2) -> Vec2 {
    let fraction = (point - drawing::CHASSIS.min) / drawing::CHASSIS.size();
    Vec2::new(
        chassis.max.x - fraction.y * chassis.width(),
        chassis.max.y - fraction.x * chassis.height(),
    )
}

fn sprite(rect: Rect, color: Color, z: f32) -> (Sprite, Transform) {
    (
        Sprite::from_color(color, to_screen(rect.size())),
        Transform::from_translation(to_screen(rect.center()).extend(z)),
    )
}

type BodyChanged = Or<(Changed<Dimensions>, Changed<Paint>)>;

pub(super) fn spawn(
    mut commands: Commands,
    cars: Query<(Entity, &Car, &Dimensions, &Paint), BodyChanged>,
) {
    for (entity, car, dimensions, paint) in &cars {
        let chassis = dimensions.chassis();
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|parent| {
                for wheel in dimensions.wheels(car) {
                    let rect = Rect::from_center_size(wheel.centre, dimensions.wheel_size());
                    parent.spawn((wheel, sprite(rect, TRIM, 0.0)));
                }
                for part in drawing::PARTS {
                    for rect in part.rects() {
                        let rect = Rect::from_corners(
                            from_drawing(chassis, rect.min),
                            from_drawing(chassis, rect.max),
                        );
                        let color = if part.painted { paint.0 } else { part.color };
                        let mut part_entity = parent.spawn(sprite(rect, color, part.z));
                        if let Some(lamp) = part.lamp {
                            part_entity.insert(lamp);
                        }
                    }
                }
            });
    }
}

pub(super) fn steer_wheels(
    cars: Query<&Car>,
    mut wheels: Query<(&Wheel, &ChildOf, &mut Transform)>,
) {
    for (wheel, child_of, mut transform) in &mut wheels {
        if let Ok(car) = cars.get(child_of.parent()) {
            let centre = to_screen(car.axle_position(wheel.axle) + Vec2::Y * wheel.centre.y);
            transform.translation = centre.extend(transform.translation.z);
            transform.rotation = Quat::from_rotation_z(car.wheel_rotation(wheel.axle).as_radians());
        }
    }
}

pub(super) fn light_lamps(
    cars: Query<&Controls>,
    mut lamps: Query<(&Lamp, &ChildOf, &mut Sprite)>,
) {
    for (lamp, child_of, mut sprite) in &mut lamps {
        let Ok(controls) = cars.get(child_of.parent()) else {
            continue;
        };
        let color = if lamp.lit(controls) {
            let unlit = LinearRgba::from(lamp.color());
            LinearRgba::rgb(unlit.red * GLOW, unlit.green * GLOW, unlit.blue * GLOW).into()
        } else {
            lamp.color()
        };
        if sprite.color != color {
            sprite.color = color;
        }
    }
}
