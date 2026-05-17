use crate::{
    gamepad::{GamepadState, HID_GAMEPAD_RDESC, HID_REPORT_LEN},
    handler::GamepadOutput,
};
use anyhow::{Context, Result};
use bluer::{
    Adapter, Address, AddressType, Session, Uuid,
    l2cap::{SocketAddr, Stream, StreamListener},
    rfcomm::{Profile, ProfileHandle, Role},
};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};

/// Bluetooth HID profile UUID (0x1124).
const HID_UUID: Uuid = Uuid::from_u128(0x0000_1124_0000_1000_8000_00805f9b34fb);

/// HID Control L2CAP PSM.
const HID_CONTROL_PSM: u16 = 0x11;
/// HID Interrupt L2CAP PSM.
const HID_INTERRUPT_PSM: u16 = 0x13;

/// Class of Device: peripheral, gamepad.
/// Bytes: 0x00 (service) 0x25 (major: peripheral) 0x08 (minor: gamepad)
const COD_GAMEPAD: u32 = 0x00_25_08;

/// HID data input transaction type: 0xA0 (DATA) | 0x01 (Input).
const HIDP_TRANS_DATA_INPUT: u8 = 0xA1;

const ADAPTER_ALIAS: &str = "gmpad";

pub struct BluetoothOutput {
    tx: mpsc::UnboundedSender<[u8; HID_REPORT_LEN + 1]>,
    _adapter: Adapter,
    _profile: ProfileHandle,
    _acceptor: JoinHandle<()>,
}

impl GamepadOutput for BluetoothOutput {
    fn send(&mut self, state: &GamepadState) {
        let report = state.hid_report();
        let mut packet = [0u8; HID_REPORT_LEN + 1];
        packet[0] = HIDP_TRANS_DATA_INPUT;
        packet[1..].copy_from_slice(&report);

        if self.tx.send(packet).is_err() {
            warn!("HID acceptor task gone; dropping report");
        }
    }
}

impl BluetoothOutput {
    pub async fn new() -> Result<Self> {
        let session = Session::new().await.context("open bluez session")?;
        let adapter = setup_adapter(&session).await.context("setup adapter")?;

        if let Err(err) = set_device_class() {
            warn!("Failed to set device class (non-fatal): {err}");
        }

        let control = bind_l2cap(HID_CONTROL_PSM)
            .await
            .with_context(|| format!("bind L2CAP PSM 0x{HID_CONTROL_PSM:02x} (HID control)"))?;
        let interrupt = bind_l2cap(HID_INTERRUPT_PSM)
            .await
            .with_context(|| format!("bind L2CAP PSM 0x{HID_INTERRUPT_PSM:02x} (HID interrupt)"))?;
        info!(
            "Listening on L2CAP PSM 0x{HID_CONTROL_PSM:02x} (control) and 0x{HID_INTERRUPT_PSM:02x} (interrupt)"
        );

        let profile = register_hid_profile(&session)
            .await
            .context("register HID profile with bluez")?;

        let (tx, rx) = mpsc::unbounded_channel();
        let acceptor = tokio::spawn(acceptor_loop(control, interrupt, rx));

        info!("Bluetooth HID device ready; waiting for host to connect");

        Ok(Self {
            tx,
            _adapter: adapter,
            _profile: profile,
            _acceptor: acceptor,
        })
    }
}

async fn setup_adapter(session: &Session) -> Result<Adapter> {
    let adapter = session.default_adapter().await?;

    adapter.set_powered(true).await?;
    adapter.set_alias(ADAPTER_ALIAS.to_string()).await.ok();
    adapter.set_discoverable_timeout(0).await.ok();
    adapter.set_pairable_timeout(0).await.ok();
    adapter.set_discoverable(true).await?;
    adapter.set_pairable(true).await?;

    info!(
        "Adapter {} ({}) ready: powered, discoverable, pairable",
        adapter.name(),
        adapter.address().await?
    );

    Ok(adapter)
}

fn set_device_class() -> Result<()> {
    use std::process::Command;

    let class_hex = format!("0x{COD_GAMEPAD:06x}");
    let status = Command::new("hciconfig")
        .args(["hci0", "class", &class_hex])
        .status()
        .context("spawn hciconfig")?;

    if !status.success() {
        anyhow::bail!("hciconfig hci0 class {class_hex} exited with {status}");
    }

    info!("Device class set to {class_hex} (peripheral / gamepad)");
    Ok(())
}

async fn bind_l2cap(psm: u16) -> Result<StreamListener> {
    let sa = SocketAddr::new(Address::any(), AddressType::BrEdr, psm);
    let listener = StreamListener::bind(sa).await?;
    Ok(listener)
}

async fn register_hid_profile(session: &Session) -> Result<ProfileHandle> {
    let profile = Profile {
        uuid: HID_UUID,
        name: Some(ADAPTER_ALIAS.to_string()),
        role: Some(Role::Server),
        require_authentication: Some(false),
        require_authorization: Some(false),
        auto_connect: Some(false),
        service_record: Some(sdp_record()),
        ..Default::default()
    };

    session
        .register_profile(profile)
        .await
        .map_err(anyhow::Error::from)
}

fn sdp_record() -> String {
    let mut hex = String::with_capacity(HID_GAMEPAD_RDESC.len() * 2);
    for b in HID_GAMEPAD_RDESC {
        hex.push_str(&format!("{b:02X}"));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" ?>
<record>
  <attribute id="0x0001">
    <sequence><uuid value="0x1124"/></sequence>
  </attribute>
  <attribute id="0x0004">
    <sequence>
      <sequence>
        <uuid value="0x0100"/>
        <uint16 value="0x0011"/>
      </sequence>
      <sequence>
        <uuid value="0x0011"/>
      </sequence>
    </sequence>
  </attribute>
  <attribute id="0x0005">
    <sequence><uuid value="0x1002"/></sequence>
  </attribute>
  <attribute id="0x0006">
    <sequence>
      <uint16 value="0x656e"/>
      <uint16 value="0x006a"/>
      <uint16 value="0x0100"/>
    </sequence>
  </attribute>
  <attribute id="0x0009">
    <sequence>
      <sequence>
        <uuid value="0x1124"/>
        <uint16 value="0x0101"/>
      </sequence>
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
  <attribute id="0x0101">
    <text value="gmpad gamepad"/>
  </attribute>
  <attribute id="0x0102">
    <text value="gmpad"/>
  </attribute>
  <attribute id="0x0200">
    <uint16 value="0x0100"/>
  </attribute>
  <attribute id="0x0201">
    <uint16 value="0x0111"/>
  </attribute>
  <attribute id="0x0202">
    <uint8 value="0x08"/>
  </attribute>
  <attribute id="0x0203">
    <uint8 value="0x00"/>
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
        <text encoding="hex" value="{hex}"/>
      </sequence>
    </sequence>
  </attribute>
  <attribute id="0x0207">
    <sequence>
      <sequence>
        <uint16 value="0x0409"/>
        <uint16 value="0x0100"/>
      </sequence>
    </sequence>
  </attribute>
  <attribute id="0x020b">
    <uint16 value="0x0100"/>
  </attribute>
  <attribute id="0x020d">
    <boolean value="true"/>
  </attribute>
</record>"#,
    )
}

async fn acceptor_loop(
    control: StreamListener,
    interrupt: StreamListener,
    mut rx: mpsc::UnboundedReceiver<[u8; HID_REPORT_LEN + 1]>,
) {
    loop {
        match accept_session(&control, &interrupt).await {
            Ok((ctrl, intr, peer)) => {
                info!("HID host connected from {peer}");
                if let Err(err) = serve_session(ctrl, intr, &mut rx).await {
                    warn!("HID session with {peer} ended: {err:#}");
                } else {
                    info!("HID session with {peer} closed cleanly");
                }
            }
            Err(err) => {
                error!("Failed to accept HID session: {err:#}");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn accept_session(
    control: &StreamListener,
    interrupt: &StreamListener,
) -> Result<(Stream, Stream, Address)> {
    // BlueZ HID profile: host opens control channel first, then interrupt.
    let (ctrl_stream, ctrl_addr) = control.accept().await.context("accept control")?;
    debug!("Control channel from {}", ctrl_addr.addr);

    let (intr_stream, intr_addr) = interrupt.accept().await.context("accept interrupt")?;
    debug!("Interrupt channel from {}", intr_addr.addr);

    if ctrl_addr.addr != intr_addr.addr {
        warn!(
            "Control ({}) and interrupt ({}) channels from different hosts",
            ctrl_addr.addr, intr_addr.addr
        );
    }

    Ok((ctrl_stream, intr_stream, ctrl_addr.addr))
}

async fn serve_session(
    mut ctrl: Stream,
    mut intr: Stream,
    rx: &mut mpsc::UnboundedReceiver<[u8; HID_REPORT_LEN + 1]>,
) -> Result<()> {
    let mut ctrl_buf = [0u8; 64];

    loop {
        tokio::select! {
            packet = rx.recv() => {
                let Some(packet) = packet else { return Ok(()); };
                intr.write_all(&packet).await.context("write interrupt")?;
            }
            res = ctrl.read(&mut ctrl_buf) => {
                let n = res.context("read control")?;
                if n == 0 {
                    anyhow::bail!("control channel EOF");
                }
                debug!("Control RX {n} bytes: {:02x?}", &ctrl_buf[..n]);
                // Minimal handshake: ignore SET_PROTOCOL, GET_REPORT etc.
                // Many hosts work without explicit responses on this channel.
            }
        }
    }
}
