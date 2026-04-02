use crate::{gamepad::GamepadState, handler::GamepadOutput};
use std::fs::File;
use tracing::info;
use uhid_virt::{Bus, CreateParams, UHIDDevice};

const GAMEPAD_NAME: &str = "virtual-xbox-gamepad";
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

pub struct HidOutput {
    device: UHIDDevice<File>,
}

impl Drop for HidOutput {
    fn drop(&mut self) {
        let _ = self.device.destroy();
    }
}

impl GamepadOutput for HidOutput {
    fn send(&mut self, state: &GamepadState) {
        self.device.write(&state.hid_report()).unwrap();
    }
}

impl HidOutput {
    pub fn new() -> Result<Self, anyhow::Error> {
        let device = new_hid_gamepad()?;

        info!("Created device: {}", GAMEPAD_NAME);

        Ok(Self { device })
    }
}

fn new_hid_gamepad() -> Result<UHIDDevice<File>, anyhow::Error> {
    let create_params = CreateParams {
        name: GAMEPAD_NAME.to_string(),
        phys: "bluetooth/input0".to_string(),
        uniq: "00:11:22:33:44:55".to_string(),
        bus: Bus::BLUETOOTH,
        vendor: 0x045e,  // Microsoft
        product: 0x028e, // Xbox 360 controller
        version: 0,
        country: 0,
        rd_data: HID_GAMEPAD_RDESC.to_vec(),
    };

    Ok(UHIDDevice::create(create_params)?)
}

impl GamepadState {
    pub fn hid_report(&self) -> [u8; 9] {
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
