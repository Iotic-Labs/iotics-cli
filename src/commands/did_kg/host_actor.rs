use std::{io, marker};

use actix::{Actor, Addr, AsyncContext, Context as ActixContext, WrapFuture};
use iotics_grpc_client::common::HostId;
use iotics_grpc_client::twin::list_all_twins;
use log::error;
use regex::Regex;

use crate::commands::did_kg::coordinator_actor::CoordinatorActor;
use crate::commands::did_kg::messages::{HostEmptyResultMessage, HostResultMessage};
use crate::commands::did_kg::resolver::{parse_did_document, TwinDidDocument};
use crate::commands::did_kg::DidKGArgs;
use crate::commands::settings::Settings;

pub struct HostActor<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    coordinator_addr: Addr<CoordinatorActor<W>>,
    settings: Settings,
    opts: DidKGArgs,
    token: String,
    remote_host_id: Option<HostId>,
}

impl<W> HostActor<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    pub fn new(
        coordinator_addr: Addr<CoordinatorActor<W>>,
        settings: Settings,
        opts: DidKGArgs,
        token: String,
        remote_host_id: Option<HostId>,
    ) -> Self {
        Self {
            coordinator_addr,
            settings,
            opts,
            token,
            remote_host_id,
        }
    }
}

impl<W> Actor for HostActor<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    type Context = ActixContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let coordinator_actor = self.coordinator_addr.clone();
        let remote_host_id = self.remote_host_id.clone();
        let opts = self.opts.clone();
        let settings = self.settings.clone();
        let token = self.token.clone();

        let fut = async move {
            let (host_did, url) = match remote_host_id {
                Some(remote_host_id) => {
                    let host_did = remote_host_id.value;
                    let url = get_host_url(&opts, &host_did).await;
                    (host_did, url)
                }
                _ => {
                    let url = settings.iotics.host_address.clone();
                    let url = url.replace("https://", "").replace(":10001", "");
                    ("?".to_string(), Some(url))
                }
            };

            if let Some(url) = url {
                let twins = get_twins(&opts, &settings, &url, &token).await;

                let url = url.replace(".iotics.space", "");

                coordinator_actor
                    .try_send(HostResultMessage {
                        host_did,
                        url,
                        twins,
                    })
                    .expect("failed to send HostResultMessage message");
            } else {
                coordinator_actor
                    .try_send(HostEmptyResultMessage)
                    .expect("failed to send HostEmptyResultMessage message");
            }
        }
        .into_actor(self);

        ctx.spawn(fut);
    }
}

async fn get_host_url(opts: &DidKGArgs, host_did: &str) -> Option<String> {
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
            if opts.verbose {
                error!("could not find url for host {}: {}", host_did, e);
            }
            None
        }
    }
}

async fn get_twins(
    opts: &DidKGArgs,
    settings: &Settings,
    url: &str,
    token: &str,
) -> Vec<TwinDidDocument> {
    let output: Result<Vec<TwinDidDocument>, anyhow::Error> = async {
        let host_url = format!("https://{}:10001", &url);
        let response = list_all_twins(&host_url, token).await?;

        if let Some(payload) = response.payload {
            let twin_dids: Vec<String> = payload
                .twins
                .into_iter()
                .map(|twin| twin.id.expect("").value)
                .collect();

            let mut twins = Vec::new();

            for twin_did in twin_dids.iter() {
                let result = parse_did_document(settings, twin_did).await;

                match result {
                    Ok(twin) => {
                        twins.push(twin);
                    }
                    Err(e) => {
                        if opts.verbose {
                            error!("failed to get did document {}: {}", twin_did, e);
                        }
                    }
                }
            }

            Ok(twins)
        } else {
            Ok(Vec::new())
        }
    }
    .await;

    match output {
        Ok(value) => value,
        Err(e) => {
            if opts.verbose {
                error!("could not find url for host {}: {}", url, e);
            }
            Vec::new()
        }
    }
}
