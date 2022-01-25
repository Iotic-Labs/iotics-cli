mod coordinator_actor;
mod host_actor;
#[allow(clippy::module_inception)]
pub mod list_hosts;
mod messages;

pub use list_hosts::*;

#[derive(Clone)]
pub enum NetworkType {
    Prod,
    Dev,
}
