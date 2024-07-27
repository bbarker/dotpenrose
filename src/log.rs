use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

// TODO: might be nice to open an alert window if there is an error;
//     : also look into anyhow

pub fn log_penrose(message: &str) -> std::io::Result<()> {
    let log_path = env::var("HOME")
        .map(PathBuf::from)
        .map(|mut path| {
            path.push(".penrose.log");
            path
        })
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME environment variable not set",
            )
        })?;

    log_path.parent().map(create_dir_all).transpose()?;

    OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path)
        .and_then(|mut file| writeln!(file, "{}", message))
}
