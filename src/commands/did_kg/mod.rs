mod coordinator_actor;
#[allow(clippy::module_inception)]
pub mod did_kg;
mod diddy;
mod host_actor;
mod messages;
mod resolver;

pub use did_kg::*;
