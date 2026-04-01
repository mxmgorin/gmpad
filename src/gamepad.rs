use evdev::{AbsoluteAxisCode, Device, KeyCode};

#[derive(Default, Clone, Copy, Debug)]
pub struct GamepadState {
    // buttons
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub lb: bool,
    pub rb: bool,
    pub lt: bool,
    pub rt: bool,
    pub start: bool,
    pub select: bool,
    pub mode: bool,
    pub thumbl: bool,
    pub thumbr: bool,

    // dpad
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,

    // axes
    pub lx: i8,
    pub ly: i8,
    pub rx: i8,
    pub ry: i8,
}

pub fn is_gamepad(dev: &Device) -> bool {
    let keys = match dev.supported_keys() {
        Some(k) => k,
        None => return false,
    };

    let axes = match dev.supported_absolute_axes() {
        Some(a) => a,
        None => return false,
    };

    let has_buttons = keys.contains(KeyCode::BTN_SOUTH)
        || keys.contains(KeyCode::BTN_EAST)
        || keys.contains(KeyCode::BTN_NORTH)
        || keys.contains(KeyCode::BTN_WEST);

    let has_stick =
        axes.contains(AbsoluteAxisCode::ABS_X) && axes.contains(AbsoluteAxisCode::ABS_Y);

    has_buttons && has_stick
}

pub fn normalize_axis(value: i32, min: i32, max: i32) -> i8 {
    let range = (max - min) as f32;
    let normalized = (value - min) as f32 / range;
    let centered = normalized * 2.0 - 1.0;
    (centered * 127.0) as i8
}
