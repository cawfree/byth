use chrono::Local;

use derive_more::Display;

#[derive(Clone, Debug, Display)]
pub enum BythError {
    ArgsError(String),
    RpcError(String),
    FoundryProjectError(String),
    BlockError(String),
    HandlerError(String),
}

#[allow(dead_code)]
pub fn error(err: String) {
    println!("\x1b[31m{}\x1b[0m ({:?}): {}", "[ERROR]", Local::now(), err);
}

#[allow(dead_code)]
pub fn warn(warning: String) {
    println!("\x1b[33m{}\x1b[0m ({:?}): {}", "[WARNING]", Local::now(), warning);
}

#[allow(dead_code)]
pub fn info(info: String) {
    println!("\x1b[32m{}\x1b[0m ({:?}): {}", "[INFO]", Local::now(), info);
}

#[allow(dead_code)]
pub fn debug(debug: String) {
    println!("\x1b[34m{}\x1b[0m ({:?}): {}", "[DEBUG]", Local::now(), debug);
}

