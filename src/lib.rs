pub mod bluetooth;
pub mod gamepad;
pub mod handler;
pub mod hid;

#[derive(Debug)]
pub enum Mode {
    Local,
    Remote,
}

pub fn fmt_err(err: &anyhow::Error) -> String {
    err.chain()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}
