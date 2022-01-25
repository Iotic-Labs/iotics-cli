use actix::{Actor, Arbiter, System};
use async_trait::async_trait;
use std::{io, marker, thread};
use structopt::StructOpt;

use crate::commands::list_hosts::coordinator_actor::CoordinatorActor;
use crate::commands::list_hosts::NetworkType;
use crate::commands::settings::{AuthBuilder, Settings};
use crate::RunnableCommand;

#[derive(Debug, StructOpt, Clone)]
pub struct ListHostsArgs {
    /// Configuration file stored in the `configuration` folder. Don't include the extension.
    #[structopt(short, long)]
    pub config: String,
    /// Fetch host version
    #[structopt(long)]
    pub with_version: bool,
    /// Fetch the twins count oh the host
    #[structopt(long)]
    pub with_twins: bool,
    /// Logging level
    #[structopt(short, long)]
    pub verbose: bool,
}

pub struct ListHosts<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    stdout: W,
    opts: ListHostsArgs,
    settings: Settings,
}

impl<W> ListHosts<W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    pub fn new(stdout: W, opts: ListHostsArgs) -> Result<Self, anyhow::Error> {
        let mut stdout = stdout;
        let settings = Settings::new(&opts.config, &mut stdout)?;
        Ok(Self {
            stdout,
            opts,
            settings,
        })
    }

    fn run_sync(self) -> Result<(), anyhow::Error> {
        thread::spawn(move || {
            let system = System::new();
            let stdout = Box::new(self.stdout);
            let opts = self.opts.clone();
            let settings = self.settings;

            let network_type = if settings.iotics.host_address.contains(".dev.") {
                NetworkType::Dev
            } else {
                NetworkType::Prod
            };

            let auth_builder = AuthBuilder::new(settings);

            let execution = async {
                let actor = CoordinatorActor::new(stdout, opts, auth_builder, network_type);

                actor.start();
            };

            let arbiter = Arbiter::new();
            arbiter.spawn(execution);

            Ok(system.run()?)
        })
        .join()
        .expect("Actix thread panicked")
    }
}

#[async_trait]
impl<W> RunnableCommand for ListHosts<W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    async fn run(self) -> Result<(), anyhow::Error> {
        self.run_sync()?;

        Ok(())
    }
}
