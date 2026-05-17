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

/// HID report descriptor for the gamepad. Shared between the local uhid
/// device and the SDP record advertised over Bluetooth, so both sides agree
/// on the byte layout produced by `GamepadState::hid_report`.
pub const HID_GAMEPAD_RDESC: &[u8] = &[
    0x05, 0x01, // USAGE_PAGE (Generic Desktop)
    0x09, 0x05, // USAGE (Game Pad)
    0xA1, 0x01, // COLLECTION (Application)
    // Buttons (16 total)
    0x05, 0x09, // USAGE_PAGE (Button)
    0x19, 0x01, // USAGE_MINIMUM (1)
    0x29, 0x10, // USAGE_MAXIMUM (16)
    0x15, 0x00, // LOGICAL_MINIMUM (0)
    0x25, 0x01, // LOGICAL_MAXIMUM (1)
    0x75, 0x01, // REPORT_SIZE (1)
    0x95, 0x10, // REPORT_COUNT (16)
    0x81, 0x02, // INPUT (Data,Var,Abs)
    // D-pad (Hat switch)
    0x05, 0x01, // USAGE_PAGE (Generic Desktop)
    0x09, 0x39, // USAGE (Hat switch)
    0x15, 0x00, 0x25, 0x07, 0x35, 0x00, 0x46, 0x3B, 0x01, 0x65, 0x14, 0x75, 0x04, 0x95, 0x01, 0x81,
    0x02, // Padding (align to byte)
    0x75, 0x04, 0x95, 0x01, 0x81, 0x01,
    // Analog sticks (LX, LY, RX, RY)
    0x09, 0x30, 0x09, 0x31, 0x09, 0x33, 0x09, 0x34, 0x15, 0x81, 0x25, 0x7F, 0x75, 0x08, 0x95, 0x04,
    0x81, 0x02, // Triggers (LT, RT)
    0x09, 0x32, 0x09, 0x35, 0x15, 0x00, 0x25, 0xFF, 0x75, 0x08, 0x95, 0x02, 0x81, 0x02,
    0xC0, // END_COLLECTION
];

pub const HID_REPORT_LEN: usize = 9;

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

impl GamepadState {
    pub fn hid_report(&self) -> [u8; HID_REPORT_LEN] {
        let buttons = self.buttons_bytes();
        let hat = self.hat_byte();

        [
            (buttons & 0xFF) as u8,
            (buttons >> 8) as u8,
            hat,
            self.lx as u8,
            self.ly as u8,
            self.rx as u8,
            self.ry as u8,
            if self.lt { 255 } else { 0 },
            if self.rt { 255 } else { 0 },
        ]
    }

    fn buttons_bytes(&self) -> u16 {
        let mut bits = 0;

        if self.a {
            bits |= 1 << 0;
        }
        if self.b {
            bits |= 1 << 1;
        }
        // bit 2 reserved (BTN_C)
        if self.x {
            bits |= 1 << 3;
        }
        if self.y {
            bits |= 1 << 4;
        }
        // bit 5 reserved (BTN_Z)
        if self.lb {
            bits |= 1 << 6;
        }
        if self.rb {
            bits |= 1 << 7;
        }
        if self.lt {
            bits |= 1 << 8;
        }
        if self.rt {
            bits |= 1 << 9;
        }
        if self.select {
            bits |= 1 << 10;
        }
        if self.start {
            bits |= 1 << 11;
        }
        if self.mode {
            bits |= 1 << 12;
        }
        if self.thumbl {
            bits |= 1 << 13;
        }
        if self.thumbr {
            bits |= 1 << 14;
        }

        bits
    }

    fn hat_byte(&self) -> u8 {
        match (
            self.dpad_up,
            self.dpad_down,
            self.dpad_left,
            self.dpad_right,
        ) {
            (true, false, false, false) => 0,
            (true, false, false, true) => 1,
            (false, false, false, true) => 2,
            (false, true, false, true) => 3,
            (false, true, false, false) => 4,
            (false, true, true, false) => 5,
            (false, false, true, false) => 6,
            (true, false, true, false) => 7,
            _ => 8,
        }
    }
}
