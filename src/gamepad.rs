use std::fs::File;

use evdev::{AbsoluteAxisCode, Device, EventSummary, KeyCode};
use tracing::debug;
use uhid_virt::{Bus, CreateParams, UHIDDevice};

const HID_GAMEPAD_RDESC: &[u8] = &[
    0x05, 0x01, //Usage Page (Generic Desktop Ctrls)
    0x09, 0x05, //Usage (Game Pad)
    0xA1, 0x01, //Collection (Application)
    0x05, 0x09, //  Usage Page (Button)
    0x19, 0x01, //  Usage Minimum (Button 1)
    0x29, 0x10, //  Usage Maximum (Button 16)
    0x15, 0x00, //  Logical Minimum (0)
    0x25, 0x01, //  Logical Maximum (1)
    0x75, 0x01, //  Report Size (1)
    0x95, 0x10, //  Report Count (16)
    0x81, 0x02, //  Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x05, 0x01, //  Usage Page (Generic Desktop Ctrls)
    0x15, 0x81, //  Logical Minimum (-127)
    0x25, 0x7F, //  Logical Maximum (127)
    0x09, 0x30, //  Usage (X)
    0x09, 0x31, //  Usage (Y)
    0x09, 0x32, //  Usage (Z)
    0x09, 0x35, //  Usage (Rz)
    0x75, 0x08, //  Report Size (8)
    0x95, 0x04, //  Report Count (4)
    0x81, 0x02, //  Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0, //End Col
];

// The gamepad report array is 6 bytes long and represents the report data for a hid gamePad.
// [0] and [1] represents the buttons' states of the gamePad. Assigned by using the formula: gamePadCode[0] |= 1; gamePadCode[0] &= ~1;
// [2] represents the X axis value of the gamePad. Assigned normally.
// [3] represents the Y axis value of the gamePad. Assigned normally.
// [4] represents the Z axis value of the gamePad. Assigned normally.
// [5] represents the RZ axis value of the gamePad. Assigned normally.
pub const HID_GAMEPAD_REPORT: [u8; 6] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

pub struct Gamepad {
    device: Device,
    state: GamepadState,
    prev_bitmask: u16,
    uhid_device: UHIDDevice<File>,
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
    lt: bool,
    rb: bool,
    rt: bool,
    start: bool,
    select: bool,
    mode: bool,
    thumbl: bool,
    thumbr: bool,
    // axes
    lx: i8,
    ly: i8,
    rx: i8,
    ry: i8,
}

impl GamepadState {
    pub fn bitmask(&self) -> u16 {
        let mut bits = 0u16;

        if self.a {
            bits |= 1 << 0;
        } // BTN_SOUTH
        if self.b {
            bits |= 1 << 1;
        } // BTN_EAST
        if self.x {
            bits |= 1 << 3;
        } // BTN_NORTH
        if self.y {
            bits |= 1 << 4;
        } // BTN_WEST

        if self.lb {
            bits |= 1 << 6;
        } // BTN_TL
        if self.rb {
            bits |= 1 << 7;
        } // BTN_TR

        if self.lt {
            bits |= 1 << 8;
        } // BTN_TL2
        if self.rt {
            bits |= 1 << 9;
        } // BTN_TR2

        if self.select {
            bits |= 1 << 10;
        } // BTN_SELECT
        if self.start {
            bits |= 1 << 11;
        } // BTN_START

        if self.mode {
            bits |= 1 << 12;
        } // BTN_MODE
        if self.thumbl {
            bits |= 1 << 13;
        } // BTN_THUMBL
        if self.thumbr {
            bits |= 1 << 14;
        } // BTN_THUMBR

        // D-pad should NOT be buttons

        bits
    }
}

impl Drop for Gamepad {
    fn drop(&mut self) {
        self.uhid_device.destroy().expect("TODO");
    }
}

impl Gamepad {
    pub fn new(device: Device) -> Self {
        let create_params = CreateParams {
            name: "virtual-gamepad".to_string(),
            phys: "".to_string(),
            uniq: "".to_string(),
            bus: Bus::USB,
            vendor: 0x045e, // Microsoft (optional but useful)
            product: 0x028e,
            version: 0,
            country: 0,
            rd_data: HID_GAMEPAD_RDESC.to_vec(),
        };

        let uhid_device = UHIDDevice::create(create_params).unwrap();

        Self {
            device,
            state: Default::default(),
            prev_bitmask: Default::default(),
            uhid_device,
        }
    }

    pub fn from_devices(mut devices: impl Iterator<Item = Device>) -> Option<Self> {
        devices.find(|dev| is_gamepad(dev)).map(|d| Gamepad::new(d))
    }

    pub fn name(&self) -> &str {
        self.device.name().unwrap_or_else(|| "Unknown")
    }

    pub fn update(&mut self) {
        let mut updated = false;
        for event in self.device.fetch_events().expect("TODO") {
            match event.destructure() {
                EventSummary::Key(_key, code, value) => {
                    self.prev_bitmask = self.state.bitmask();
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

                    updated = self.state.bitmask() != self.prev_bitmask;
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
            self.send_report();
        }
    }

    fn send_report(&mut self) {
        let buttons = self.state.bitmask();

        let report = [
            (buttons & 0xFF) as u8,
            (buttons >> 8) as u8,
            self.state.lx as u8,
            self.state.ly as u8,
            self.state.rx as u8,
            self.state.ry as u8,
        ];

        self.uhid_device.write(&report).unwrap();
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

fn normalize_axis(value: i32, min: i32, max: i32) -> i8 {
    let range = (max - min) as f32;
    let normalized = (value - min) as f32 / range; // 0..1
    let centered = normalized * 2.0 - 1.0; // -1..1
    (centered * 127.0) as i8
}
