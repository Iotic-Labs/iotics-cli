use async_trait::async_trait;
use std::{io, marker};
use structopt::StructOpt;
use yansi::Paint;

use iotics_grpc_client::{search, Filter, Property, Scope, Uri, Value, SEARCH_PAGE_SIZE};

use super::{
    settings::{get_token, Settings},
    RunnableCommand,
};
use crate::commands::helpers::delete_and_log_twin;

#[derive(Debug, StructOpt)]
pub struct DeleteTwinsByModelArgs {
    /// Configuration file stored in the `configuration` folder. Don't include the extension.
    #[structopt(short, long)]
    pub config: String,
    /// The model DID
    #[structopt(short, long)]
    pub model_did: String,
    /// If this flag is present, the model will be deleted as well
    #[structopt(short, long)]
    pub delete_model: bool,
    /// Logging level
    #[structopt(short, long)]
    pub verbose: bool,
}

pub struct DeleteTwinsByModel<'a, W>
where
    W: io::Write + marker::Send,
{
    stdout: &'a mut W,
    opts: DeleteTwinsByModelArgs,
    settings: Settings,
    twins_found: usize,
    twins_deleted: usize,
}

impl<'a, W> DeleteTwinsByModel<'a, W>
where
    W: io::Write + marker::Send,
{
    pub fn new(stdout: &'a mut W, opts: DeleteTwinsByModelArgs) -> Result<Self, anyhow::Error> {
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
impl<'a, W> RunnableCommand for DeleteTwinsByModel<'a, W>
where
    W: io::Write + marker::Send,
{
    async fn run(&mut self) -> Result<(), anyhow::Error> {
        let token = get_token(&self.settings)?;

        let mut stream = search(
            &self.settings.iotics.host_address,
            &token,
            Filter {
                properties: vec![Property {
                    key: "https://data.iotics.com/app#model".to_string(),
                    value: Some(Value::UriValue(Uri {
                        value: self.opts.model_did.clone(),
                    })),
                }],
                location: None,
                text: None,
            },
            Scope::Local,
            None,
        )
        .await?;

        while let Some(response) = stream.recv().await {
            match response {
                Ok(page) => {
                    if let Some(payload) = page.payload {
                        if payload.twins.len() < SEARCH_PAGE_SIZE as usize {
                            // this must be the last page, close the stream
                            stream.close();
                        }

                        let twins_dids: Vec<String> = payload
                            .twins
                            .into_iter()
                            .map(|twin| twin.id.expect("this should not happen").value)
                            .collect();

                        writeln!(
                            self.stdout,
                            "Found {} twins for model {}. Deleting...",
                            Paint::yellow(twins_dids.len()),
                            Paint::blue(&self.opts.model_did),
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
                            "Deleted {} twins for model {}.",
                            Paint::red(self.twins_deleted),
                            Paint::blue(&self.opts.model_did),
                        )?;
                        self.stdout.flush()?;
                    }
                }
                Err(e) => {
                    writeln!(self.stdout, "{:?}", Paint::red(e))?;
                    self.stdout.flush()?;
                }
            }
        }

        if self.opts.delete_model {
            let twin_did = self.opts.model_did.clone();
            delete_and_log_twin(
                self.stdout,
                &self.settings.iotics.host_address,
                &token,
                &twin_did,
                0,
                true,
            )
            .await?;
        }

        writeln!(self.stdout)?;
        writeln!(self.stdout, "Done.")?;

        Ok(())
    }
}
