use crate::{
    gamepad::{GamepadState, HID_GAMEPAD_RDESC},
    handler::GamepadOutput,
};
use std::fs::File;
use tracing::info;
use uhid_virt::{Bus, CreateParams, UHIDDevice};

const GAMEPAD_NAME: &str = "virtual-xbox-gamepad";

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
