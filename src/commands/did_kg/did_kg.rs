use actix::{Actor, Arbiter, System};
use async_trait::async_trait;
use std::{io, marker, thread};
use structopt::StructOpt;

use crate::commands::did_kg::coordinator_actor::CoordinatorActor;
use crate::commands::settings::Settings;
use crate::RunnableCommand;

#[derive(Debug, StructOpt, Clone)]
pub struct DidKGArgs {
    /// Configuration file stored in the `configuration` folder. Don't include the extension.
    #[structopt(short, long)]
    pub config: String,
    /// Logging level
    #[structopt(short, long)]
    pub verbose: bool,
}

pub struct DidKG<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    stdout: W,
    opts: DidKGArgs,
    settings: Settings,
}

impl<W> DidKG<W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    pub fn new(stdout: W, opts: DidKGArgs) -> Result<Self, anyhow::Error> {
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

            let execution = async {
                let actor = CoordinatorActor::new(stdout, opts, settings);

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
impl<W> RunnableCommand for DidKG<W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    async fn run(self) -> Result<(), anyhow::Error> {
        self.run_sync()?;

        Ok(())
    }
}
