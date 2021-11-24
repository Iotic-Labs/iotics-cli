use anyhow::Context;
use async_trait::async_trait;
use log::error;
use regex::Regex;
use std::time::Duration;
use std::{io, marker};
use structopt::StructOpt;
use yansi::Paint;

use iotics_grpc_client::common::{Property, Scope, Uri, Value};
use iotics_grpc_client::search::{search, Filter};

use super::{
    settings::{get_token, Settings},
    RunnableCommand,
};

#[derive(Debug, StructOpt)]
pub struct ListHostsArgs {
    /// Configuration file stored in the `configuration` folder. Don't include the extension.
    #[structopt(short, long)]
    pub config: String,
    /// Fetch host version
    #[structopt(long)]
    pub version: bool,
    /// Logging level
    #[structopt(short, long)]
    pub verbose: bool,
}

pub struct ListHosts<'a, W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    stdout: &'a mut W,
    opts: ListHostsArgs,
    settings: Settings,
}

impl<'a, W> ListHosts<'a, W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    pub fn new(stdout: &'a mut W, opts: ListHostsArgs) -> Result<Self, anyhow::Error> {
        let settings = Settings::new(&opts.config, stdout)?;
        Ok(Self {
            stdout,
            opts,
            settings,
        })
    }

    async fn get_host_url(&self, host_did: &str) -> Option<String> {
        let output = async {
            let result = run_script::run_script!(format!("dig TXT {}.iotics.space", &host_did))?;

            let (_, output, _) = result;
            let re = Regex::new(r#"IN TXT "(.*)""#).expect("this should not happen");

            if let Some(captures) = re.captures(&output) {
                let url = captures
                    .get(1)
                    .ok_or(anyhow::anyhow!("could not find host url"))?
                    .as_str()
                    .to_string();
                Ok(url)
            } else {
                Err(anyhow::anyhow!("could not find host url"))
            }
        }
        .await;

        match output {
            Ok(value) => Some(value),
            Err(e) => {
                if self.opts.verbose {
                    error!("could not find url for host {}: {}", host_did, e);
                }
                None
            }
        }
    }

    async fn get_host_version(&self, url: &str) -> String {
        let output: Result<String, anyhow::Error> = async {
            match self.opts.version {
                true => {
                    let version_url = format!("https://{}/index.json", url);

                    let response = reqwest::get(version_url)
                        .await
                        .context("get version url failed")?;
                    response.error_for_status_ref()?;

                    let json_response: VersionPayload = response.json().await?;
                    Ok(json_response.version)
                }
                false => Ok("".to_string()),
            }
        }
        .await;

        match output {
            Ok(value) => value,
            Err(e) => {
                if self.opts.verbose {
                    error!("could not find url for host {}: {}", url, e);
                }
                "".to_string()
            }
        }
    }
}

#[async_trait]
impl<'a, W> RunnableCommand for ListHosts<'a, W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    async fn run(&mut self) -> Result<(), anyhow::Error> {
        let token = get_token(&self.settings)?;

        writeln!(self.stdout, "{:4} {:28} {:58} Version", "#", "Host", "DID")?;
        self.stdout.flush()?;

        let mut search_stream = search(
            &self.settings.iotics.host_address,
            &token,
            Filter {
                properties: vec![Property {
                    key: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                    value: Some(Value::UriValue(Uri {
                        value: "http://data.iotics.com/public#HostTwin".to_string(),
                    })),
                }],
                location: None,
                text: None,
            },
            Scope::Global,
            Some(Duration::from_secs(10)),
        )
        .await?;

        let mut count = 0;

        while let Some(response) = search_stream.recv().await {
            match response {
                Ok(page) => {
                    if let Some(payload) = page.payload {
                        let remote_host_id = payload.remote_host_id;

                        let (host_did, url) = match remote_host_id {
                            Some(remote_host_id) => {
                                let host_did = remote_host_id.value;
                                let url = self.get_host_url(&host_did).await;
                                (host_did, url)
                            }
                            _ => {
                                let url = self.settings.iotics.host_address.clone();
                                let url = url.replace("https://", "").replace(":10001", "");
                                ("?".to_string(), Some(url))
                            }
                        };

                        if let Some(url) = url {
                            let version = self.get_host_version(&url).await;

                            let url = url.replace(".iotics.space", "");

                            count += 1;
                            writeln!(
                                self.stdout,
                                "{:4} {:28} {:58} {}",
                                Paint::yellow(count),
                                Paint::green(url),
                                host_did,
                                Paint::blue(version)
                            )?;
                            self.stdout.flush()?;
                        }
                    }
                }
                Err(e) => {
                    error!("search error: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct VersionPayload {
    pub version: String,
}
