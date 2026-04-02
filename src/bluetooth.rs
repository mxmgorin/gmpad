use crate::{gamepad::GamepadState, handler::GamepadOutput};

const SDP_XML: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<record>
  <attribute id="0x0001">
    <sequence><uuid value="0x1124"/></sequence>
  </attribute>

  <attribute id="0x0004">
    <sequence>
      <sequence><uuid value="0x0100"/></sequence>
      <sequence><uuid value="0x0011"/></sequence>
    </sequence>
  </attribute>

  <attribute id="0x000d">
    <sequence>
      <sequence>
        <sequence>
          <uuid value="0x0100"/>
          <uint16 value="0x0013"/>
        </sequence>
        <sequence>
          <uuid value="0x0011"/>
        </sequence>
      </sequence>
    </sequence>
  </attribute>

  <attribute id="0x0100">
    <text value="gmpad"/>
  </attribute>

  <attribute id="0x0202">
    <uint8 value="0x08"/>
  </attribute>

  <attribute id="0x0204">
    <boolean value="true"/>
  </attribute>

  <attribute id="0x0205">
    <boolean value="true"/>
  </attribute>

  <attribute id="0x0206">
    <sequence>
      <sequence>
        <uint8 value="0x22"/>
        <text encoding="hex" value="
          05010905A1011500250135004501750195100509190129108102
          75089501050109301581257F8102C0
        "/>
      </sequence>
    </sequence>
  </attribute>
</record>
"#;

pub struct BluetoothOutput {
    //interrupt: std::fs::File,
    _adapter: Adapter,
}

impl GamepadOutput for BluetoothOutput {
    fn send(&mut self, state: &GamepadState) {
        let report = state.hid_report();

        let mut packet = [0u8; 9];
        packet[0] = 0xA1;
        packet[1..].copy_from_slice(&report);

        //use std::io::Write;
        //let _ = self.interrupt.write_all(&packet);
    }
}

impl BluetoothOutput {
    pub async fn new() -> anyhow::Result<Self> {
        let session = Session::new().await?;
        let adapter = setup_adapter(&session).await?;
        set_device_class()?;
        register_sdp().await?;

        info!("Bluetooth HID device ready");

        Ok(Self { _adapter: adapter })
    }
}

use bluer::{Adapter, Session};
use tracing::info;

async fn setup_adapter(session: &Session) -> anyhow::Result<Adapter> {
    let adapter = session.default_adapter().await?;

    adapter.set_powered(true).await?;
    adapter.set_discoverable(true).await?;
    adapter.set_pairable(true).await?;

    info!("Adapter ready: {}", adapter.name());

    Ok(adapter)
}

use std::process::Command;

fn set_device_class() -> anyhow::Result<()> {
    let status = Command::new("hciconfig")
        .args(["hci0", "class", "0x002508"]) // Gamepad
        .status()?;

    info!("Set device class: {}", status);

    Ok(())
}

use zbus::Connection;

async fn register_sdp() -> anyhow::Result<()> {
    let conn = Connection::system().await?;
    let proxy =
        zbus::Proxy::new(&conn, "org.bluez", "/org/bluez/hci0", "org.bluez.Adapter1").await?;
    let handle: u32 = proxy.call("AddRecord", &(SDP_XML)).await?;

    info!("SDP record registered: handle={}", handle);

    Ok(())
}
