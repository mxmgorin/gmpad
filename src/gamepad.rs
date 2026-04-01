use evdev::{AbsoluteAxisCode, Device, EventSummary, KeyCode};
use tracing::debug;

pub struct Gamepad {
    device: Device,
    state: GamepadState,
}

#[derive(Default)]
struct GamepadState {
    dpad_up: bool,
    dpad_down: bool,
    dpad_left: bool,
    dpad_right: bool,
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    lb: bool,
    rb: bool,
    start: bool,
    select: bool,
}

impl Gamepad {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            state: Default::default(),
        }
    }

    pub fn from_devices(mut devices: impl Iterator<Item = Device>) -> Option<Self> {
        devices.find(|dev| is_gamepad(dev)).map(|d| Gamepad::new(d))
    }

    pub fn name(&self) -> &str {
        self.device.name().unwrap_or_else(|| "Unknown")
    }

    pub fn update(&mut self) {
        for event in self.device.fetch_events().expect("TODO") {
            if let EventSummary::Key(_key, code, value) = event.destructure() {
                let pressed = value != 0;
                debug!("{:?} pressed: {}", code, pressed);

                match code {
                    KeyCode::BTN_SOUTH => self.state.a = pressed,
                    KeyCode::BTN_EAST => self.state.b = pressed,
                    KeyCode::BTN_NORTH => self.state.x = pressed,
                    KeyCode::BTN_WEST => self.state.y = pressed,
                    KeyCode::BTN_TL => self.state.lb = pressed,
                    KeyCode::BTN_TR => self.state.rb = pressed,
                    KeyCode::BTN_START => self.state.start = pressed,
                    KeyCode::BTN_SELECT => self.state.select = pressed,
                    KeyCode::BTN_DPAD_DOWN => self.state.dpad_down = pressed,
                    KeyCode::BTN_DPAD_UP => self.state.dpad_up = pressed,
                    KeyCode::BTN_DPAD_LEFT => self.state.dpad_left = pressed,
                    KeyCode::BTN_DPAD_RIGHT => self.state.dpad_right = pressed,
                    _ => {}
                }
            }
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

    // Check for common gamepad buttons
    let has_buttons = keys.contains(KeyCode::BTN_SOUTH) || // A
        keys.contains(KeyCode::BTN_EAST)  || // B
        keys.contains(KeyCode::BTN_NORTH) || // X
        keys.contains(KeyCode::BTN_WEST); // Y

    // Check for analog sticks
    let has_left_stick =
        axes.contains(AbsoluteAxisCode::ABS_X) && axes.contains(AbsoluteAxisCode::ABS_Y);

    has_buttons && has_left_stick
}
