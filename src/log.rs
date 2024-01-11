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

fn log(level: Level, msg: &str) {
    println!("{level} {msg}");
}

pub fn debug(msg: &str) {
    log(Level::Debug, msg);
}

pub fn info(msg: &str) {
    log(Level::Info, msg);
}

pub fn warning(msg: &str) {
    log(Level::Warning, msg);
}

pub fn error(msg: &str) {
    log(Level::Error, msg);
}
