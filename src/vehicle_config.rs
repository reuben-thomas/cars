use crate::car::visual::{PAINT, Paint};
use crate::car::{Car, Params};
use crate::player::Player;
use crate::ui::{self, KEYCAP, KEYCAP_HOVERED, KEYCAP_PRESSED, TEXT, TEXT_SIZE};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, SliderValue, ValueChange, observe};

const VALUE_WIDTH: f32 = 100.0;

const PAINTS: [Color; 6] = [
    PAINT,
    Color::srgb_u8(0xef, 0xef, 0xef),
    Color::srgb_u8(0x9f, 0xc5, 0xf8),
    Color::srgb_u8(0xf1, 0xc2, 0x32),
    Color::srgb_u8(0x6a, 0xc4, 0x7e),
    Color::srgb_u8(0x30, 0x34, 0x3a),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Mass,
    Friction,
    Aerodynamics,
    Chassis,
    Drivetrain,
}

impl Category {
    pub const ALL: [Category; 5] = [
        Category::Chassis,
        Category::Mass,
        Category::Friction,
        Category::Aerodynamics,
        Category::Drivetrain,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Category::Mass => "Mass",
            Category::Friction => "Friction",
            Category::Aerodynamics => "Aerodynamics",
            Category::Chassis => "Chassis",
            Category::Drivetrain => "Drivetrain",
        }
    }
}

struct Field {
    label: &'static str,
    category: Category,
    unit: &'static str,
    min: f32,
    max: f32,
    precision: i32,
    get: fn(&Params) -> f32,
    set: fn(&mut Params, f32),
}

const FIELDS: [Field; 15] = [
    Field {
        label: "wheelbase",
        category: Category::Chassis,
        unit: "m",
        min: 2.0,
        max: 3.2,
        precision: 2,
        get: |p| p.bicycle.wheelbase(),
        set: |p, v| {
            let front_weight = p.bicycle.front_weight();
            p.bicycle.set_axles(v, front_weight);
        },
    },
    Field {
        label: "front_weight",
        category: Category::Chassis,
        unit: "",
        min: 0.4,
        max: 0.6,
        precision: 2,
        get: |p| p.bicycle.front_weight(),
        set: |p, v| {
            let wheelbase = p.bicycle.wheelbase();
            p.bicycle.set_axles(wheelbase, v);
        },
    },
    Field {
        label: "mass",
        category: Category::Mass,
        unit: "kg",
        min: 500.0,
        max: 3000.0,
        precision: 0,
        get: |p| p.bicycle.mass,
        set: |p, v| p.bicycle.mass = v,
    },
    Field {
        label: "yaw_inertia",
        category: Category::Mass,
        unit: "kg m^2",
        min: 500.0,
        max: 5000.0,
        precision: 0,
        get: |p| p.bicycle.yaw_inertia,
        set: |p, v| p.bicycle.yaw_inertia = v,
    },
    Field {
        label: "front_cornering_coeff",
        category: Category::Friction,
        unit: "1/rad",
        min: 1.0,
        max: 20.0,
        precision: 1,
        get: |p| p.bicycle.front_cornering_coefficient,
        set: |p, v| p.bicycle.front_cornering_coefficient = v,
    },
    Field {
        label: "rear_cornering_coeff",
        category: Category::Friction,
        unit: "1/rad",
        min: 1.0,
        max: 20.0,
        precision: 1,
        get: |p| p.bicycle.rear_cornering_coefficient,
        set: |p, v| p.bicycle.rear_cornering_coefficient = v,
    },
    Field {
        label: "friction_coeff",
        category: Category::Friction,
        unit: "",
        min: 0.1,
        max: 1.5,
        precision: 2,
        get: |p| p.bicycle.friction_coefficient,
        set: |p, v| p.bicycle.friction_coefficient = v,
    },
    Field {
        label: "rolling_resist",
        category: Category::Friction,
        unit: "kg/s",
        min: 0.0,
        max: 50.0,
        precision: 1,
        get: |p| p.bicycle.rolling_resistance,
        set: |p, v| p.bicycle.rolling_resistance = v,
    },
    Field {
        label: "drag_coeff",
        category: Category::Aerodynamics,
        unit: "",
        min: 0.1,
        max: 1.0,
        precision: 2,
        get: |p| p.bicycle.drag_coefficient,
        set: |p, v| p.bicycle.drag_coefficient = v,
    },
    Field {
        label: "frontal_area",
        category: Category::Aerodynamics,
        unit: "m^2",
        min: 1.0,
        max: 4.0,
        precision: 1,
        get: |p| p.bicycle.frontal_area,
        set: |p, v| p.bicycle.frontal_area = v,
    },
    Field {
        label: "max_steering",
        category: Category::Chassis,
        unit: "deg",
        min: 10.0,
        max: 45.0,
        precision: 0,
        get: |p| p.max_steering.to_degrees(),
        set: |p, v| p.max_steering = v.to_radians(),
    },
    Field {
        label: "drive_force",
        category: Category::Drivetrain,
        unit: "kN",
        min: 1.0,
        max: 30.0,
        precision: 1,
        get: |p| p.drivetrain.max_force / 1000.0,
        set: |p, v| p.drivetrain.max_force = v * 1000.0,
    },
    Field {
        label: "drive_rear_bias",
        category: Category::Drivetrain,
        unit: "",
        min: 0.0,
        max: 1.0,
        precision: 2,
        get: |p| p.drivetrain.rear_bias,
        set: |p, v| p.drivetrain.rear_bias = v,
    },
    Field {
        label: "brake_force",
        category: Category::Drivetrain,
        unit: "kN",
        min: 1.0,
        max: 30.0,
        precision: 1,
        get: |p| p.brakes.max_force / 1000.0,
        set: |p, v| p.brakes.max_force = v * 1000.0,
    },
    Field {
        label: "brake_rear_bias",
        category: Category::Drivetrain,
        unit: "",
        min: 0.0,
        max: 1.0,
        precision: 2,
        get: |p| p.brakes.rear_bias,
        set: |p, v| p.brakes.rear_bias = v,
    },
];

pub struct VehicleConfigPlugin;

impl Plugin for VehicleConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_sliders,
                (update_values, style_swatches, style_reset_button),
            )
                .chain(),
        );
    }
}

#[derive(Component, Clone, Copy)]
struct FieldSlider(usize);

#[derive(Component, Clone, Copy)]
struct FieldValue(usize);

#[derive(Component, Clone, Copy)]
struct Swatch(Color);

#[derive(Component)]
struct ResetButton;

pub fn section(category: Category) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        Children::spawn(SpawnWith(move |body: &mut ChildSpawner| {
            if category == Category::Chassis {
                body.spawn(paint_row());
            }
            for (index, field) in FIELDS.iter().enumerate() {
                if field.category == category {
                    body.spawn(field_row(index));
                }
            }
        })),
    )
}

pub fn reset_button() -> impl Bundle {
    (
        ResetButton,
        ui::button(ui::text("Reset", TEXT_SIZE, TEXT)),
        observe(reset),
    )
}

fn reset(_: On<Activate>, mut car: Single<(&mut Car, &mut Paint), With<Player>>) {
    let (car, paint) = &mut *car;
    car.set_params(Params::default());
    paint.set_if_neq(Paint::default());
}

fn style_reset_button(
    mut button: Single<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<ResetButton>>,
) {
    let (hovered, pressed, background) = &mut *button;
    let color = if *pressed {
        KEYCAP_PRESSED
    } else if hovered.get() {
        KEYCAP_HOVERED
    } else {
        KEYCAP
    };
    background.set_if_neq(BackgroundColor(color));
}

fn paint_row() -> impl Bundle {
    let swatches = SpawnWith(|row: &mut ChildSpawner| {
        for color in PAINTS {
            row.spawn((Swatch(color), ui::swatch(color), observe(paint)));
        }
    });
    (
        Node {
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            ui::label("colour"),
            (
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(6),
                    row_gap: px(6),
                    ..default()
                },
                Children::spawn(swatches),
            ),
        ],
    )
}

fn paint(
    activate: On<Activate>,
    swatches: Query<&Swatch>,
    mut car: Single<&mut Paint, With<Player>>,
) {
    if let Ok(&Swatch(color)) = swatches.get(activate.entity) {
        car.set_if_neq(Paint(color));
    }
}

fn style_swatches(
    car: Single<&Paint, With<Player>>,
    mut swatches: Query<(&Swatch, &Hovered, &mut BorderColor)>,
) {
    for (&Swatch(color), hovered, mut border) in &mut swatches {
        border.set_if_neq(ui::swatch_border(car.0 == color, hovered.get()));
    }
}

fn field_row(index: usize) -> impl Bundle {
    let field = &FIELDS[index];
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        },
        children![
            ui::label(field.label),
            (
                FieldSlider(index),
                ui::slider(field.min, field.max, field.precision),
                observe(set_field),
            ),
            (
                Node {
                    width: px(VALUE_WIDTH),
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                children![(FieldValue(index), ui::text("", TEXT_SIZE, TEXT))],
            ),
        ],
    )
}

fn set_field(
    change: On<ValueChange<f32>>,
    sliders: Query<&FieldSlider>,
    mut car: Single<&mut Car, With<Player>>,
) {
    let Ok(&FieldSlider(index)) = sliders.get(change.source) else {
        return;
    };
    let mut params = car.params().clone();
    (FIELDS[index].set)(&mut params, change.value);
    car.set_params(params);
}

fn sync_sliders(
    mut commands: Commands,
    car: Single<&Car, (With<Player>, Changed<Car>)>,
    sliders: Query<(Entity, &FieldSlider, &SliderValue)>,
) {
    for (entity, &FieldSlider(index), value) in &sliders {
        let current = (FIELDS[index].get)(car.params());
        if value.0 != current {
            commands.entity(entity).insert(SliderValue(current));
        }
    }
}

fn update_values(
    sliders: Query<(&FieldSlider, &SliderValue), Changed<SliderValue>>,
    mut texts: Query<(&FieldValue, &mut Text)>,
) {
    for (&FieldSlider(index), value) in &sliders {
        let field = &FIELDS[index];
        for (_, mut text) in texts.iter_mut().filter(|(text, _)| text.0 == index) {
            text.0 = format!(
                "{:.*} {}",
                field.precision.max(0) as usize,
                value.0,
                field.unit
            );
        }
    }
}
