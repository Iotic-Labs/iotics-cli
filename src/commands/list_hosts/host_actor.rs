use std::{io, marker};

use actix::{Actor, Addr, AsyncContext, Context as ActixContext, WrapFuture};
use anyhow::Context;
use iotics_grpc_client::common::HostId;
use iotics_grpc_client::twin::list_all_twins;
use log::error;
use regex::Regex;

use crate::commands::list_hosts::coordinator_actor::CoordinatorActor;
use crate::commands::list_hosts::messages::{HostEmptyResultMessage, HostResultMessage};
use crate::commands::list_hosts::ListHostsArgs;
use crate::commands::settings::Settings;

pub struct HostActor<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    coordinator_addr: Addr<CoordinatorActor<W>>,
    settings: Settings,
    opts: ListHostsArgs,
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
        opts: ListHostsArgs,
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
                let version = get_host_version(&opts, &url).await;
                let twins_count = get_twin_count(&opts, &url, &token).await;

                let url = url.replace(".iotics.space", "");

                coordinator_actor
                    .try_send(HostResultMessage {
                        host_did,
                        version,
                        url,
                        twins_count,
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

#[derive(serde::Deserialize)]
struct VersionPayload {
    pub version: String,
}

async fn get_host_url(opts: &ListHostsArgs, host_did: &str) -> Option<String> {
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

async fn get_host_version(opts: &ListHostsArgs, url: &str) -> String {
    let output: Result<String, anyhow::Error> = async {
        match opts.with_version {
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
            if opts.verbose {
                error!("could not find url for host {}: {}", url, e);
            }
            "".to_string()
        }
    }
}

async fn get_twin_count(opts: &ListHostsArgs, url: &str, token: &str) -> Option<usize> {
    let output: Result<Option<usize>, anyhow::Error> = async {
        match opts.with_twins {
            true => {
                let host_url = format!("https://{}:10001", &url);
                let response = list_all_twins(&host_url, token).await?;

                if let Some(payload) = response.payload {
                    Ok(Some(payload.twins.len() as usize))
                } else {
                    Ok(Some(0))
                }
            }
            false => Ok(None),
        }
    }
    .await;

    match output {
        Ok(value) => value,
        Err(e) => {
            if opts.verbose {
                error!("could not find url for host {}: {}", url, e);
            }
            None
        }
    }
}
