use crate::hid::VirtualGamepad;
use evdev::{AbsoluteAxisCode, Device, EventSummary, KeyCode};
use tracing::{debug, warn};

pub struct Gamepad {
    phys_device: Device,
    state: GamepadState,
    virt_gamepad: VirtualGamepad,
}

#[derive(Default, Clone, Copy)]
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

impl Gamepad {
    pub fn new(mut device: Device) -> Self {
        if let Err(e) = device.grab() {
            warn!("Unable to grab device, continuing but there might be conflicts: {e:?}");
        }

        Self {
            phys_device: device,
            state: Default::default(),
            virt_gamepad: VirtualGamepad::new().expect("TODO"),
        }
    }

    pub fn from_devices(mut devices: impl Iterator<Item = Device>) -> Option<Self> {
        devices.find(|dev| is_gamepad(dev)).map(|d| Gamepad::new(d))
    }

    pub fn name(&self) -> &str {
        self.phys_device.name().unwrap_or_else(|| "Unknown")
    }

    pub fn update(&mut self) {
        let mut updated = false;

        for event in self.phys_device.fetch_events().unwrap() {
            match event.destructure() {
                EventSummary::Key(_, code, value) => {
                    let pressed = value != 0;
                    debug!("{:?} pressed: {}", code, pressed);

                    match code {
                        KeyCode::BTN_SOUTH => self.state.a = pressed,
                        KeyCode::BTN_EAST => self.state.b = pressed,
                        KeyCode::BTN_NORTH => self.state.x = pressed,
                        KeyCode::BTN_WEST => self.state.y = pressed,
                        KeyCode::BTN_TL => self.state.lb = pressed,
                        KeyCode::BTN_TR => self.state.rb = pressed,
                        KeyCode::BTN_TL2 => self.state.lt = pressed,
                        KeyCode::BTN_TR2 => self.state.rt = pressed,
                        KeyCode::BTN_START => self.state.start = pressed,
                        KeyCode::BTN_SELECT => self.state.select = pressed,
                        KeyCode::BTN_MODE => self.state.mode = pressed,
                        KeyCode::BTN_THUMBL => self.state.thumbl = pressed,
                        KeyCode::BTN_THUMBR => self.state.thumbr = pressed,
                        KeyCode::BTN_DPAD_UP => self.state.dpad_up = pressed,
                        KeyCode::BTN_DPAD_DOWN => self.state.dpad_down = pressed,
                        KeyCode::BTN_DPAD_LEFT => self.state.dpad_left = pressed,
                        KeyCode::BTN_DPAD_RIGHT => self.state.dpad_right = pressed,
                        _ => {}
                    }

                    updated = true;
                }

                EventSummary::AbsoluteAxis(_, code, value) => {
                    let v = normalize_axis(value, -1800, 1800);

                    match code {
                        AbsoluteAxisCode::ABS_X => self.state.lx = v,
                        AbsoluteAxisCode::ABS_Y => self.state.ly = v,
                        AbsoluteAxisCode::ABS_RX => self.state.rx = v,
                        AbsoluteAxisCode::ABS_RY => self.state.ry = v,
                        _ => {}
                    }

                    updated = true;
                }

                _ => {}
            }
        }

        if updated {
            self.virt_gamepad.update(&self.state);
        }
    }
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

fn normalize_axis(value: i32, min: i32, max: i32) -> i8 {
    let range = (max - min) as f32;
    let normalized = (value - min) as f32 / range;
    let centered = normalized * 2.0 - 1.0;
    (centered * 127.0) as i8
}
