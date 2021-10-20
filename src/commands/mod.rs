use async_trait::async_trait;
use std::str;
use structopt::StructOpt;

pub mod delete_all_twins;
pub mod delete_twins_by_model;
mod helpers;
mod settings;

use self::{delete_all_twins::DeleteAllTwinsArgs, delete_twins_by_model::DeleteTwinsByModelArgs};

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Command {
    /// Deletes all twins that have been created from a given model
    DeleteTwinsByModel(DeleteTwinsByModelArgs),
    /// Deletes all twins from a host that have been created by the given identity
    DeleteAllTwins(DeleteAllTwinsArgs),
}

#[async_trait]
pub trait RunnableCommand: Sized {
    async fn run(&mut self) -> Result<(), anyhow::Error>;
}
