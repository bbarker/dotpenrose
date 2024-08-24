use std::env;
use std::fmt::Debug;
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

pub trait LogPenroseError<T, E>
where
    Self: Sized,
{
    fn log_err(self, fstr: &str) -> Option<T>;
}

impl<T, E: Debug> LogPenroseError<T, E> for Result<T, E> {
    fn log_err(self, fstr: &str) -> Option<T> {
        match self {
            Ok(val) => Some(val),
            Err(err) => {
                let msg = &format!("{}: {:?}", fstr, err);
                log_penrose(msg).unwrap_or_else(|er| {
                    eprintln!("Couldn't log error {}\nDue to error {:?}", msg, er)
                });
                None
            }
        }
    }
}

impl<T> LogPenroseError<T, ()> for Option<T> {
    fn log_err(self, fstr: &str) -> Self {
        match self {
            Some(_) => self,
            None => {
                let msg = &format!("{}: None when Some expected", fstr);
                log_penrose(msg).unwrap_or_else(|er| {
                    eprintln!("Couldn't log error {}\nDue to error {:?}", msg, er)
                });
                self
            }
        }
    }
}
