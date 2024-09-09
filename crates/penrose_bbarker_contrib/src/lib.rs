#![warn(clippy::all)]
#![warn(future_incompatible, rust_2024_compatibility)]

use std::process::Command;
// #![deny(unused_crate_dependencies)]

pub fn is_running(program: &str) -> bool {
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("ps -ef | grep {} | grep -v grep", program))
        .output()
        .unwrap();

    !output.stdout.is_empty()
}

pub fn is_in_path(program: &str) -> bool {
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("type {}", program))
        .output()
        .unwrap();

    output.status.success()
}
