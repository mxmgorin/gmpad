pub mod gamepad;

pub fn fmt_err(err: &anyhow::Error) -> String {
    err.chain()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}
