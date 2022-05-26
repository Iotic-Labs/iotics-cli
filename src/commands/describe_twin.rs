use async_trait::async_trait;
use iotics_grpc_client::common::{HostId, TwinId};
use std::{io, marker};
use structopt::StructOpt;
use yansi::Paint;

use iotics_grpc_client::twin::describe::describe_twin;

use crate::commands::settings::{AuthBuilder, Settings};
use crate::commands::RunnableCommand;

#[derive(Debug, StructOpt)]
pub struct DescribeTwinArgs {
    /// Configuration file stored in the `configuration` folder. Don't include the extension.
    #[structopt(short, long)]
    pub config: String,
    /// The DID of the twin to be described
    #[structopt(long)]
    pub twin_did: String,
    /// Optional. The ID of the remote host, if the twin to be described is not stored on the host that's making the request
    #[structopt(long)]
    pub host_id: Option<String>,
    /// Logging level
    #[structopt(short, long)]
    pub verbose: bool,
}

pub struct DescribeTwin<'a, W>
where
    W: io::Write + marker::Send,
{
    stdout: &'a mut W,
    opts: DescribeTwinArgs,
    settings: Settings,
}

impl<'a, W> DescribeTwin<'a, W>
where
    W: io::Write + marker::Send,
{
    pub fn new(stdout: &'a mut W, opts: DescribeTwinArgs) -> Result<Self, anyhow::Error> {
        let settings = Settings::new(&opts.config, stdout)?;
        Ok(Self {
            stdout,
            opts,
            settings,
        })
    }
}

#[async_trait]
impl<'a, W> RunnableCommand for DescribeTwin<'a, W>
where
    W: io::Write + marker::Send,
{
    async fn run(mut self) -> Result<(), anyhow::Error> {
        let auth_builder = AuthBuilder::new(self.settings.clone());

        let response = describe_twin(
            auth_builder.clone(),
            TwinId {
                value: self.opts.twin_did,
            },
            self.opts.host_id.map(|host_id| HostId { value: host_id }),
        )
        .await;

        match response {
            Ok(result) => {
                writeln!(self.stdout, "{:#?}", Paint::green(result))?;
                self.stdout.flush()?;
            }
            Err(e) => {
                writeln!(self.stdout, "{:?}", Paint::red(e))?;
                self.stdout.flush()?;
            }
        }

        Ok(())
    }
}
