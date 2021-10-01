use std::io;
use structopt::StructOpt;
use yansi::Paint;

use iotics_grpc_client::{
    delete_twin, search, Filter, Property, Scope, Uri, Value, SEARCH_PAGE_SIZE,
};

use super::settings::{get_token, Settings};

#[derive(Debug, StructOpt)]
pub struct DeleteTwinsArgs {
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

pub struct DeleteTwins<'a, W>
where
    W: io::Write,
{
    stdout: &'a mut W,
    opts: DeleteTwinsArgs,
    settings: Settings,
    twins_found: usize,
}

impl<'a, W> DeleteTwins<'a, W>
where
    W: io::Write,
{
    pub fn new(stdout: &'a mut W, opts: DeleteTwinsArgs) -> Result<Self, anyhow::Error> {
        let settings = Settings::new(&opts.config, stdout)?;
        Ok(Self {
            stdout,
            opts,
            settings,
            twins_found: 0,
        })
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
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

                        for twin_did in twins_dids {
                            self.delete_twin(&token, &twin_did, false).await?;
                        }
                    }
                }
                Err(e) => {
                    writeln!(self.stdout, "{:?}", Paint::red(e))?;
                    self.stdout.flush()?;
                }
            }
        }

        writeln!(self.stdout)?;
        writeln!(
            self.stdout,
            "Found and deleted {} twins for model {}.",
            Paint::yellow(self.twins_found),
            Paint::blue(&self.opts.model_did),
        )?;
        self.stdout.flush()?;

        if self.opts.delete_model {
            let twin_did = self.opts.model_did.clone();
            self.delete_twin(&token, &twin_did, true).await?;
        }

        writeln!(self.stdout)?;
        writeln!(self.stdout, "Done.")?;

        Ok(())
    }

    pub async fn delete_twin(
        &mut self,
        token: &str,
        twin_did: &str,
        force_verbose: bool,
    ) -> Result<(), anyhow::Error> {
        if self.opts.verbose || force_verbose {
            write!(self.stdout, "Deleting twin {}... ", twin_did)?;
        }

        if !self.opts.verbose && !force_verbose && self.twins_found & 32 == 0 {
            writeln!(self.stdout)?;
        }

        let result = delete_twin(&self.settings.iotics.host_address, token, twin_did).await;

        match result {
            Ok(_) => {
                if self.opts.verbose || force_verbose {
                    writeln!(self.stdout, "{}", Paint::green("OK"))?;
                } else {
                    write!(self.stdout, "{}", Paint::green("."))?;
                }
            }
            Err(e) => {
                if self.opts.verbose || force_verbose {
                    writeln!(self.stdout, "{:?}", Paint::green(e))?;
                } else {
                    write!(self.stdout, "{}", Paint::red("E"))?;
                }
            }
        };

        self.stdout.flush()?;

        self.twins_found += 1;

        Ok(())
    }
}
