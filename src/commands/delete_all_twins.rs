use async_trait::async_trait;
use std::{io, marker};
use structopt::StructOpt;
use yansi::Paint;

use iotics_grpc_client::twin::list_all_twins;

use super::{
    settings::{get_token, Settings},
    RunnableCommand,
};
use crate::commands::helpers::delete_and_log_twin;

#[derive(Debug, StructOpt)]
pub struct DeleteAllTwinsArgs {
    /// Configuration file stored in the `configuration` folder. Don't include the extension.
    #[structopt(short, long)]
    pub config: String,
    /// Logging level
    #[structopt(short, long)]
    pub verbose: bool,
}

pub struct DeleteAllTwins<'a, W>
where
    W: io::Write + marker::Send,
{
    stdout: &'a mut W,
    opts: DeleteAllTwinsArgs,
    settings: Settings,
    twins_found: usize,
    twins_deleted: usize,
}

impl<'a, W> DeleteAllTwins<'a, W>
where
    W: io::Write + marker::Send,
{
    pub fn new(stdout: &'a mut W, opts: DeleteAllTwinsArgs) -> Result<Self, anyhow::Error> {
        let settings = Settings::new(&opts.config, stdout)?;
        Ok(Self {
            stdout,
            opts,
            settings,
            twins_found: 0,
            twins_deleted: 0,
        })
    }
}

#[async_trait]
impl<'a, W> RunnableCommand for DeleteAllTwins<'a, W>
where
    W: io::Write + marker::Send,
{
    async fn run(&mut self) -> Result<(), anyhow::Error> {
        let token = get_token(&self.settings)?;

        let response = list_all_twins(&self.settings.iotics.host_address, &token).await;

        match response {
            Ok(response) => {
                if let Some(payload) = response.payload {
                    let twins_dids = payload
                        .twins
                        .into_iter()
                        .map(|twin| twin.id.expect("this should not happen").value)
                        .collect::<Vec<String>>();

                    writeln!(
                        self.stdout,
                        "Found {} twins. Deleting...",
                        Paint::yellow(twins_dids.len()),
                    )?;
                    self.stdout.flush()?;

                    for twin_did in twins_dids {
                        let result = delete_and_log_twin(
                            self.stdout,
                            &self.settings.iotics.host_address,
                            &token,
                            &twin_did,
                            self.twins_found,
                            self.opts.verbose,
                        )
                        .await;

                        self.twins_found += 1;

                        if result.is_ok() {
                            self.twins_deleted += 1;
                        }
                    }

                    writeln!(self.stdout)?;
                    writeln!(
                        self.stdout,
                        "Deleted {} twins.",
                        Paint::red(self.twins_deleted),
                    )?;
                    self.stdout.flush()?;
                }
            }
            Err(e) => {
                writeln!(self.stdout, "{:?}", Paint::red(e))?;
                self.stdout.flush()?;
            }
        }

        writeln!(self.stdout)?;
        writeln!(self.stdout, "Done.")?;

        Ok(())
    }
}
