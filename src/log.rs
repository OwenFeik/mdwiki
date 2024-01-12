#![allow(dead_code)]

use std::fmt::Display;

enum Level {
    Debug,
    Info,
    Warning,
    Error,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warning => "WARN",
            Self::Error => "ERROR",
        };

        f.write_str(&format!("[{: <5}]", string))
    }
}

fn log<S: AsRef<str>>(level: Level, msg: S) {
    println!("{level} {}", msg.as_ref());
}

pub fn debug<S: AsRef<str>>(msg: S) {
    log(Level::Debug, msg);
}

pub fn info<S: AsRef<str>>(msg: S) {
    log(Level::Info, msg);
}

pub fn warning<S: AsRef<str>>(msg: S) {
    log(Level::Warning, msg);
}

pub fn error<S: AsRef<str>>(msg: S) {
    log(Level::Error, msg);
}
