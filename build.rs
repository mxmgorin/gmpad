fn main() {
    use std::process::Command;

    let date = Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
        .expect("failed to execute date");

    let date_str = String::from_utf8(date.stdout).unwrap();

    println!("cargo:rustc-env=BUILD_DATE={}", date_str.trim());
}
