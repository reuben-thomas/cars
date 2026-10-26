use crate::car::visual::debug::CarGizmos;
use crate::keymap::{Action, ActionButton};
use crate::ui::{self, BORDER, BORDER_WIDTH, BUTTON_SIZE, HEADING, TEXT_SIZE};
use crate::vehicle_config::{self, Category};
use bevy::prelude::*;
use bevy::ui::Checked;

const WIDTH: f32 = 430.0;
const PADDING: f32 = 12.0;
const SECTION_HEIGHT: f32 = 36.0;
const ROW_GAP: f32 = 8.0;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_debug_switch);
    }
}

#[derive(Component)]
struct DebugSwitch;

pub fn panel() -> impl Bundle {
    (
        ui::panel(Node {
            width: px(WIDTH),
            max_width: percent(100),
            min_width: px(0),
            padding: UiRect::horizontal(px(PADDING)),
            ..default()
        }),
        children![ui::collapsible(
            ui::text("Settings", TEXT_SIZE, HEADING),
            BUTTON_SIZE - 2.0 * BORDER_WIDTH,
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::bottom(px(PADDING)),
                    ..default()
                },
                Children::spawn(SpawnWith(|body: &mut ChildSpawner| {
                    for category in Category::ALL {
                        body.spawn(section(category.name(), vehicle_config::section(category)));
                    }
                    body.spawn(section("Simulation", simulation()));
                    body.spawn(vehicle_config::reset_button());
                })),
            ),
        )],
    )
}

fn section(title: &'static str, content: impl Bundle) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::bottom(px(ROW_GAP)),
            border: UiRect::top(px(1)),
            ..default()
        },
        BorderColor::all(BORDER),
        children![
            (
                Node {
                    min_height: px(SECTION_HEIGHT),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![ui::text(title, TEXT_SIZE, HEADING)],
            ),
            content,
        ],
    )
}

fn simulation() -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            ui::label("debug_mode"),
            (DebugSwitch, ActionButton(Action::ToggleDebug), ui::switch()),
        ],
    )
}

fn sync_debug_switch(
    mut commands: Commands,
    store: Res<GizmoConfigStore>,
    switch: Single<(Entity, Has<Checked>), With<DebugSwitch>>,
) {
    let (switch, checked) = *switch;
    let enabled = store.config::<CarGizmos>().0.enabled;
    if enabled && !checked {
        commands.entity(switch).insert(Checked);
    } else if !enabled && checked {
        commands.entity(switch).remove::<Checked>();
    }
}
