use evdev::Device;
use gmpad::gamepad::Gamepad;
use std::os::unix::fs::FileTypeExt;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

fn main() {
    let tracing_filter =
        EnvFilter::try_from_env("GMPAD_LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("info"));

    let tracing_sub = fmt().with_env_filter(tracing_filter);
    tracing_sub.init();

    info!("Detecting inputs...");

    let devices = detect_input_devices().unwrap_or_else(|err| {
        error!("Failed to get input devices: {}", gmpad::fmt_err(&err));
        std::process::exit(1);
    });

    let Some(mut gamepad) = Gamepad::from_devices(devices) else {
        error!("Gamepad not found");
        std::process::exit(1);
    };

    info!("Listening events from {}..", gamepad.name());

    loop {
        gamepad.update();
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
