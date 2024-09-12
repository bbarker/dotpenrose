#![warn(clippy::all)]
#![warn(future_incompatible, rust_2024_compatibility)]

pub mod log;
pub mod menus;
pub mod workspaces;
use std::process::Command;

use once_cell::sync::Lazy;
use penrose::{core::bindings::KeyEventHandler, x11rb::RustConn};
use sysinfo::System;
// #![deny(unused_crate_dependencies)]

pub static SYSTEM: Lazy<System> = Lazy::new(System::new_all);

pub type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

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
