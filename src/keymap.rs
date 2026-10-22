use bevy::input::InputSystems;
use bevy::picking::PickingSystems;
use bevy::picking::events::PointerPress;
use bevy::prelude::*;
use bevy::ui::Pressed;
use std::borrow::Cow;

pub struct KeymapPlugin;

impl Plugin for KeymapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Keymap>()
            .init_resource::<ButtonInput<Action>>()
            .add_observer(press_button)
            .add_systems(First, clear_actions)
            .add_systems(
                PreUpdate,
                update_actions
                    .after(InputSystems)
                    .after(PickingSystems::Last),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Accelerate,
    Brake,
    SteerLeft,
    SteerRight,
    Reset,
    ToggleDebug,
    ToggleHelp,
    CloseHelp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Driving,
    View,
    Debug,
}

impl Group {
    pub fn name(self) -> &'static str {
        match self {
            Group::Driving => "Driving",
            Group::View => "View",
            Group::Debug => "Debug",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub action: Action,
    pub keys: Vec<KeyCode>,
    pub group: Group,
    pub description: &'static str,
}

#[derive(Resource, Clone, Debug)]
pub struct Keymap {
    bindings: Vec<Binding>,
}

impl Default for Keymap {
    fn default() -> Self {
        use Action::*;
        use Group::*;
        use KeyCode::*;
        let binding = |action, keys: &[KeyCode], group, description| Binding {
            action,
            keys: keys.to_vec(),
            group,
            description,
        };
        Self {
            bindings: vec![
                binding(Accelerate, &[KeyW, ArrowUp], Driving, "Accelerate"),
                binding(Brake, &[KeyS, ArrowDown], Driving, "Brake / reverse"),
                binding(SteerLeft, &[KeyA, ArrowLeft], Driving, "Steer left"),
                binding(SteerRight, &[KeyD, ArrowRight], Driving, "Steer right"),
                binding(Reset, &[KeyR], Driving, "Reset car"),
                binding(ToggleHelp, &[Slash], View, "Toggle this help"),
                binding(CloseHelp, &[Escape], View, "Close this help"),
                binding(ToggleDebug, &[Backquote], Debug, "Toggle debug overlay"),
            ],
        }
    }
}

impl Keymap {
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ActionButton(pub Action);

fn clear_actions(mut actions: ResMut<ButtonInput<Action>>) {
    actions.clear();
}

fn press_button(
    press: On<PointerPress>,
    buttons: Query<&ActionButton>,
    mut actions: ResMut<ButtonInput<Action>>,
) {
    if let Ok(&ActionButton(action)) = buttons.get(press.entity) {
        actions.press(action);
    }
}

fn update_actions(
    keys: Res<ButtonInput<KeyCode>>,
    keymap: Res<Keymap>,
    buttons: Query<(&ActionButton, Has<Pressed>)>,
    mut actions: ResMut<ButtonInput<Action>>,
) {
    for binding in keymap.bindings() {
        let action = binding.action;
        let bound = || binding.keys.iter().copied();
        if keys.any_just_pressed(bound()) {
            actions.press(action);
        }
        let held = keys.any_pressed(bound())
            || buttons
                .iter()
                .any(|(button, pressed)| button.0 == action && pressed);
        if held {
            actions.press(action);
        } else {
            actions.release(action);
        }
    }
}

pub fn label(key: KeyCode) -> Cow<'static, str> {
    let name = match key {
        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",
        KeyCode::Backquote => "`",
        KeyCode::Slash => "?",
        KeyCode::Escape => "Esc",
        _ => {
            let name = format!("{key:?}");
            let name = name
                .strip_prefix("Key")
                .or_else(|| name.strip_prefix("Digit"))
                .unwrap_or(&name);
            return Cow::Owned(name.to_owned());
        }
    };
    Cow::Borrowed(name)
}
