#[cfg(feature = "remote")]
pub mod bluetooth;
pub mod gamepad;
pub mod handler;
#[cfg(feature = "local")]
pub mod hid;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    #[cfg(feature = "local")]
    Local,
    #[cfg(feature = "remote")]
    Remote,
}

pub fn fmt_err(err: &anyhow::Error) -> String {
    err.chain()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}
