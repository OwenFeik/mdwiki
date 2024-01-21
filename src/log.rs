#![allow(dead_code)]

use std::fmt::Display;

#[derive(PartialEq, PartialOrd)]
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

const MIN_LEVEL: Level = Level::Info;

fn log<D: Display>(level: Level, msg: D) {
    if level >= MIN_LEVEL {
        println!("{level} {}", msg);
    }
}

pub fn debug<D: Display>(msg: D) {
    log(Level::Debug, msg);
}

pub fn info<D: Display>(msg: D) {
    log(Level::Info, msg);
}

pub fn warning<D: Display>(msg: D) {
    log(Level::Warning, msg);
}

pub fn error<D: Display>(msg: D) {
    log(Level::Error, msg);
}
