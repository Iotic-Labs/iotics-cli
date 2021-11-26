use std::time::Duration;
use std::{io, marker};

use actix::{Actor, ActorContext, AsyncContext, Context, Handler, System, WrapFuture};

use iotics_grpc_client::common::{Property, Scope, Uri, Value};
use iotics_grpc_client::search::{search, Filter};
use log::error;
use yansi::Paint;

use crate::commands::list_hosts::messages::{
    HostEmptyResultMessage, HostResultMessage, ProcessHostMessage,
};
use crate::commands::list_hosts::ListHostsArgs;
use crate::commands::settings::{get_token, Settings};

use super::host_actor::HostActor;

pub struct CoordinatorActor<W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    stdout: Box<W>,
    opts: ListHostsArgs,
    settings: Settings,
    hosts_found: u64,
    hosts_handled: u64,
}

impl<W> CoordinatorActor<W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    pub fn new(stdout: Box<W>, opts: ListHostsArgs, settings: Settings) -> Self {
        Self {
            stdout,
            opts,
            settings,
            hosts_found: 0,
            hosts_handled: 0,
        }
    }
}

impl<W> Actor for CoordinatorActor<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(1024);
        let addr = ctx.address();

        writeln!(
            self.stdout,
            "{:4} {:28} {:58} {:12} {:6}",
            "#", "Host", "DID", "Version", "Twins"
        )
        .expect("this should not happen");
        self.stdout.flush().expect("this should not happen");

        let settings = self.settings.clone();

        let fut = async move {
            let result = async move {
                let token = get_token(&settings)?;

                let mut search_stream = search(
                    &settings.iotics.host_address,
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

                while let Some(result) = search_stream.recv().await {
                    match result {
                        Ok(page) => {
                            if let Some(payload) = page.payload {
                                addr.try_send(ProcessHostMessage {
                                    payload,
                                    token: token.clone(),
                                })
                                .expect("failed to send ProcessHostMessage message");
                            }
                        }
                        Err(e) => {
                            error!("search error: {}", e);
                        }
                    }
                }

                Ok::<_, anyhow::Error>(())
            }
            .await;

            match result {
                Ok(_) => {}
                Err(_e) => {
                    // TODO: log error
                }
            }
        }
        .into_actor(self);

        ctx.spawn(fut);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        System::current().stop_with_code(0);
    }
}

impl<W> Handler<ProcessHostMessage> for CoordinatorActor<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    type Result = ();

    fn handle(&mut self, message: ProcessHostMessage, ctx: &mut Context<Self>) -> Self::Result {
        let addr = ctx.address();
        let settings = self.settings.clone();
        let opts = self.opts.clone();

        let remote_host_id = message.payload.remote_host_id;

        self.hosts_found += 1;
        let host_actor = HostActor::new(addr, settings, opts, message.token, remote_host_id);
        host_actor.start();
    }
}

impl<W> Handler<HostResultMessage> for CoordinatorActor<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    type Result = ();

    fn handle(&mut self, message: HostResultMessage, ctx: &mut Context<Self>) -> Self::Result {
        self.hosts_handled += 1;

        let twins_count = match message.twins_count {
            Some(twins_count) => format!("{:6}", twins_count),
            None => "".to_string(),
        };

        writeln!(
            self.stdout,
            "{:4} {:28} {:58} {:12} {:6}",
            Paint::yellow(self.hosts_handled),
            Paint::green(message.url),
            message.host_did,
            Paint::blue(message.version),
            twins_count
        )
        .expect("this should not happen");
        self.stdout.flush().expect("this should not happen");

        if self.hosts_found == self.hosts_handled {
            ctx.stop();
        }
    }
}

impl<W> Handler<HostEmptyResultMessage> for CoordinatorActor<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    type Result = ();

    fn handle(&mut self, _: HostEmptyResultMessage, ctx: &mut Context<Self>) -> Self::Result {
        self.hosts_handled += 1;

        if self.hosts_found == self.hosts_handled {
            ctx.stop();
        }
    }
}
