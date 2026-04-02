use evdev::Device;
use gmpad::{Mode, bluetooth::BluetoothOutput, fmt_err, handler::EventHandler, hid::HidOutput};
use std::os::unix::fs::FileTypeExt;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    let tracing_filter =
        EnvFilter::try_from_env("GMPAD_LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("info"));

    let tracing_sub = fmt().with_env_filter(tracing_filter);
    tracing_sub.init();
    let mode = parse_mode();

    info!(
        "Runnning {} v{} (built {}) in mode {:?}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("BUILD_DATE"),
        mode
    );

    let devices = detect_input_devices().unwrap_or_else(|err| {
        error!("Failed to get input devices: {}", fmt_err(&err));
        std::process::exit(1);
    });
    let mut handler = match EventHandler::from_devices(devices) {
        Ok(x) => x,
        Err(err) => {
            error!("{}", fmt_err(&err));
            std::process::exit(1);
        }
    };

    match mode {
        Mode::Local => match HidOutput::new() {
            Ok(x) => handler.run(x),
            Err(err) => {
                error!("{}", fmt_err(&err));
                std::process::exit(1);
            }
        },
        Mode::Remote => match BluetoothOutput::new().await {
            Ok(x) => handler.run(x),
            Err(err) => {
                error!("{}", fmt_err(&err));
                std::process::exit(1);
            }
        },
    };
}

fn parse_mode() -> Mode {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: gmpad [local|remote]");
        std::process::exit(1);
    });

    match arg.as_str() {
        "local" => Mode::Local,
        "remote" => Mode::Remote,
        _ => {
            eprintln!("Invalid mode: {arg}");
            eprintln!("Usage: gmpad [local|remote]");
            std::process::exit(1);
        }
    }
}

fn detect_input_devices() -> anyhow::Result<impl Iterator<Item = Device>> {
    info!("Detecting inputs devices...");

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
                    "Found input device: {}",
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
