pub mod delete_twins;
mod settings;

use std::str;
use structopt::StructOpt;

use self::delete_twins::DeleteTwinsArgs;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Command {
    /// Deletes all twins that have been created from a given model
    DeleteTwins(DeleteTwinsArgs),
}
