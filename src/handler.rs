use crate::gamepad::{GamepadState, is_gamepad, normalize_axis};
use evdev::{AbsoluteAxisCode, Device, EventSummary, KeyCode};
use tracing::{debug, info, warn};

pub trait GamepadOutput {
    fn send(&mut self, state: &GamepadState);
}

pub struct EventHandler {
    device: Device,
    state: GamepadState,
}

impl EventHandler {
    pub fn from_device(mut device: Device) -> Result<Self, anyhow::Error> {
        if let Err(e) = device.grab() {
            warn!("Unable to grab device, continuing but there might be conflicts: {e:?}");
        }

        Ok(Self {
            device,
            state: Default::default(),
        })
    }

    pub fn from_devices(mut devices: impl Iterator<Item = Device>) -> Result<Self, anyhow::Error> {
        let Some(gamepad) = devices.find(|dev| is_gamepad(dev)) else {
            return Err(anyhow::Error::msg("Gamepad not found"));
        };

        Self::from_device(gamepad)
    }

    pub fn run(&mut self, mut output: impl GamepadOutput) {
        info!("Listening events from {}..", self.device_name());

        loop {
            if self.update() {
                output.send(&self.state);
            }
        }
    }

    pub fn device_name(&self) -> &str {
        self.device.name().unwrap_or_else(|| "Unknown")
    }

    pub fn update(&mut self) -> bool {
        let mut updated = false;

        for event in self.device.fetch_events().unwrap() {
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

        updated
    }
}
