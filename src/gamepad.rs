use evdev::{AbsoluteAxisCode, Device, EventSummary, KeyCode};
use std::fs::File;
use tracing::debug;
use uhid_virt::{Bus, CreateParams, UHIDDevice};

const HID_GAMEPAD_RDESC: &[u8] = &[
    0x05, 0x01, // USAGE_PAGE (Generic Desktop)
    0x09, 0x05, // USAGE (Game Pad)
    0xA1, 0x01, // COLLECTION (Application)
    // --- Buttons (16 total):
    0x05, 0x09, // USAGE_PAGE (Button)
    0x19, 0x01, // USAGE_MINIMUM (1)
    0x29, 0x10, // USAGE_MAXIMUM (16)
    0x15, 0x00, // LOGICAL_MINIMUM (0)
    0x25, 0x01, // LOGICAL_MAXIMUM (1)
    0x75, 0x01, // REPORT_SIZE (1)
    0x95, 0x10, // REPORT_COUNT (16)
    0x81, 0x02, // INPUT (Data,Var,Abs)
    // ---

    // --- D-pad (Hat switch):
    0x05, 0x01, // USAGE_PAGE (Generic Desktop)
    0x09, 0x39, // USAGE (Hat switch)
    0x15, 0x00, 0x25, 0x07, 0x35, 0x00, 0x46, 0x3B, 0x01, 0x65, 0x14, 0x75, 0x04, 0x95, 0x01, 0x81,
    0x02, // ---

    // --- Padding (align to byte):
    0x75, 0x04, 0x95, 0x01, 0x81, 0x01,
    // ---

    // --- Analog sticks (LX, LY, RX, RY):
    0x09, 0x30, 0x09, 0x31, 0x09, 0x33, 0x09, 0x34, 0x15, 0x81, 0x25, 0x7F, 0x75, 0x08, 0x95, 0x04,
    0x81, 0x02, // ---

    // --- Triggers (LT, RT):
    0x09, 0x32, 0x09, 0x35, 0x15, 0x00, 0x25, 0xFF, 0x75, 0x08, 0x95, 0x02, 0x81, 0x02,
    // ---
    0xC0, // END_COLLECTION
];

pub struct Gamepad {
    device: Device,
    state: GamepadState,
    uhid_device: UHIDDevice<File>,
}

#[derive(Default, Clone, Copy)]
struct GamepadState {
    // buttons
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    lb: bool,
    rb: bool,
    lt: bool,
    rt: bool,
    start: bool,
    select: bool,
    mode: bool,
    thumbl: bool,
    thumbr: bool,

    // dpad
    dpad_up: bool,
    dpad_down: bool,
    dpad_left: bool,
    dpad_right: bool,

    // axes
    lx: i8,
    ly: i8,
    rx: i8,
    ry: i8,
}

impl GamepadState {
    pub fn hid_report(&self) -> [u8; 9] {
        let buttons = self.buttons();
        let hat = self.hat();

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

    fn buttons(&self) -> u16 {
        let mut bits = 0;

        if self.a {
            bits |= 1 << 0;
        } // BTN_SOUTH
        if self.b {
            bits |= 1 << 1;
        } // BTN_EAST

        // skip bit 2 (BTN_C)

        if self.x {
            bits |= 1 << 3;
        } // BTN_NORTH
        if self.y {
            bits |= 1 << 4;
        } // BTN_WEST

        // skip bit 5 (BTN_Z)

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
        } // BTN_SELECT (Back)
        if self.start {
            bits |= 1 << 11;
        } // BTN_START

        if self.mode {
            bits |= 1 << 12;
        } // BTN_MODE (Xbox button)

        if self.thumbl {
            bits |= 1 << 13;
        } // BTN_THUMBL
        if self.thumbr {
            bits |= 1 << 14;
        } // BTN_THUMBR

        bits
    }

    fn hat(&self) -> u8 {
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

impl Drop for Gamepad {
    fn drop(&mut self) {
        let _ = self.uhid_device.destroy();
    }
}

impl Gamepad {
    pub fn new(device: Device) -> Self {
        let create_params = CreateParams {
            name: "virtual-xbox-gamepad".to_string(),
            phys: "".to_string(),
            uniq: "".to_string(),
            bus: Bus::USB,
            vendor: 0x045e,
            product: 0x028e,
            version: 0,
            country: 0,
            rd_data: HID_GAMEPAD_RDESC.to_vec(),
        };

        let uhid_device = UHIDDevice::create(create_params).unwrap();

        Self {
            device,
            state: Default::default(),
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

        if updated {
            self.send_report();
        }
    }

    fn send_report(&mut self) {
        self.uhid_device.write(&self.state.hid_report()).unwrap();
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
