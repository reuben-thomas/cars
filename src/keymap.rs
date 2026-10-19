//! Keyboard bindings. Input systems and the help page both read the
//! [`Keymap`], so it is the single definition of what each key does.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::borrow::Cow;

pub struct KeymapPlugin;

impl Plugin for KeymapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Keymap>();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Accelerate,
    Brake,
    SteerLeft,
    SteerRight,
    Reset,
    ToggleBody,
    ToggleDebug,
    ToggleVelocity,
    ToggleChassis,
    ToggleBicycleModel,
    ToggleText,
    ToggleTrail,
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

/// Bindings in the order they are listed on the help page.
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
                binding(ToggleBody, &[F6], View, "Toggle car body"),
                binding(ToggleHelp, &[Slash], View, "Toggle this help"),
                binding(CloseHelp, &[Escape], View, "Close this help"),
                binding(ToggleDebug, &[Backquote], Debug, "Toggle debug overlay"),
                binding(ToggleVelocity, &[F1], Debug, "Toggle velocity"),
                binding(ToggleChassis, &[F2], Debug, "Toggle chassis"),
                binding(ToggleBicycleModel, &[F3], Debug, "Toggle bicycle model"),
                binding(ToggleText, &[F4], Debug, "Toggle text"),
                binding(ToggleTrail, &[F5], Debug, "Toggle trail"),
            ],
        }
    }
}

impl Keymap {
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    pub fn keys(&self, action: Action) -> &[KeyCode] {
        self.bindings
            .iter()
            .find(|binding| binding.action == action)
            .map_or(&[], |binding| &binding.keys)
    }
}

/// Keyboard state queried by [`Action`] rather than by key.
#[derive(SystemParam)]
pub struct Actions<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    keymap: Res<'w, Keymap>,
}

impl Actions<'_> {
    pub fn pressed(&self, action: Action) -> bool {
        self.keys
            .any_pressed(self.keymap.keys(action).iter().copied())
    }

    pub fn just_pressed(&self, action: Action) -> bool {
        self.keys
            .any_just_pressed(self.keymap.keys(action).iter().copied())
    }
}

/// Short label for a key, as printed on the keycap.
pub fn label(key: KeyCode) -> Cow<'static, str> {
    let name = match key {
        KeyCode::ArrowUp => "^",
        KeyCode::ArrowDown => "v",
        KeyCode::ArrowLeft => "<",
        KeyCode::ArrowRight => ">",
        KeyCode::Backquote => "`",
        // The `?` key on US layouts.
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
