use evdev::{AbsoluteAxisCode, Device, KeyCode};
use std::os::unix::fs::FileTypeExt;
use tracing::{error, info, level_filters::LevelFilter, warn};
use tracing_subscriber::EnvFilter;

fn main() {
    let tracing_sub = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );
    tracing_sub.init();

    info!("Detecting inputs...");

    let mut input_devices = detect_input_devices().unwrap_or_else(|err| {
        error!("Failed to get input devices: {}", fmt_err(&err));
        std::process::exit(1);
    });

    let gamepad = input_devices.find(|dev| is_gamepad(dev));

    if let Some(gp) = gamepad {
        info!("Found gamepad: {}", gp.name().unwrap_or_else(|| "Unknown"));
    } else {
        info!("Gamepad not found");
    }
}

fn detect_input_devices() -> anyhow::Result<impl Iterator<Item = Device>> {
    let iter = std::fs::read_dir("/dev/input")?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_char_device() {
                return None;
            }

            let filename = entry.file_name().into_string().ok()?;

            if !filename.starts_with("event") {
                return None;
            }

            Some(entry.path())
        })
        .filter_map(|path| match Device::open(&path) {
            Ok(dev) => {
                info!(
                    "Found input device {}",
                    dev.name().unwrap_or_else(|| "Unknown")
                );
                Some(dev)
            }

            Err(err) => {
                warn!("Can't open input device {}: {}", path.display(), err);
                None
            }
        });

    Ok(iter)
}

fn fmt_err(err: &anyhow::Error) -> String {
    err.chain()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}

fn is_gamepad(dev: &Device) -> bool {
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
