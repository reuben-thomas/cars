use bevy_math::Vec2;

pub const PIXELS_PER_METRE: f32 = 20.0;

pub fn to_screen(point: Vec2) -> Vec2 {
    point * PIXELS_PER_METRE
}
