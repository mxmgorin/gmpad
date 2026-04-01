use anyhow::Context;
use evdev::Device;
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

    let input_devices = detect_input_devices().unwrap_or_else(|err| {
        error!("Failed to get input devices: {}", fmt_err(&err));
        std::process::exit(1);
    });

    for device in input_devices {
        match device {
            Ok(device) => info!(
                "Found device {}",
                device.name().unwrap_or_else(|| "Unknown")
            ),
            Err(err) => warn!("Can't open device {}", fmt_err(&err)),
        }
    }
}

fn detect_input_devices() -> anyhow::Result<impl Iterator<Item = anyhow::Result<Device>>> {
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
        .map(|path| Device::open(&path).with_context(|| format!("{}", path.display())));

    Ok(iter)
}

fn fmt_err(err: &anyhow::Error) -> String {
    err.chain()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}
