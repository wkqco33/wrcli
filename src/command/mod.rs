pub mod args;
pub mod context;
pub mod help;

#[allow(clippy::module_inception)]
mod command;
mod dispatch;

pub use command::{Command, RunEFn, RunFn};
