use crate::{gamepad::GamepadState, handler::GamepadOutput};

pub struct BluetoothOutput {
    interrupt: std::fs::File,
}

impl GamepadOutput for BluetoothOutput {
    fn send(&mut self, state: &GamepadState) {
        let report = state.hid_report();

        let mut packet = [0u8; 9];
        packet[0] = 0xA1;
        packet[1..].copy_from_slice(&report);

        use std::io::Write;
        let _ = self.interrupt.write_all(&packet);
    }
}
