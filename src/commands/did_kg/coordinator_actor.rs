use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use std::{io, marker};

use actix::{Actor, ActorContext, AsyncContext, Context, Handler, System, WrapFuture};

use iotics_grpc_client::common::{Property, Scope, Uri, Value};
use iotics_grpc_client::search::{search, Filter};
use log::error;
use oxigraph::io::GraphFormat;
use oxigraph::model::vocab::{rdf, rdfs};
use oxigraph::model::{GraphName, LiteralRef, NamedNodeRef, TripleRef};
use oxigraph::MemoryStore;
use yansi::Paint;

use crate::commands::did_kg::diddy;
use crate::commands::did_kg::messages::{
    GenerateKbMessage, HostEmptyResultMessage, HostResultMessage, ProcessHostMessage,
};
use crate::commands::did_kg::resolver::TwinDidDocument;
use crate::commands::did_kg::DidKGArgs;
use crate::commands::settings::{get_token, Settings};

use super::host_actor::HostActor;

pub struct CoordinatorActor<W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    stdout: Box<W>,
    opts: DidKGArgs,
    settings: Settings,
    hosts_found: usize,
    hosts_handled: usize,
    host_twins: HashMap<String, Vec<TwinDidDocument>>,
}

impl<W> CoordinatorActor<W>
where
    W: io::Write + marker::Send + marker::Sync,
{
    pub fn new(stdout: Box<W>, opts: DidKGArgs, settings: Settings) -> Self {
        Self {
            stdout,
            opts,
            settings,
            hosts_found: 0,
            hosts_handled: 0,
            host_twins: HashMap::new(),
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
            "{:4} {:28} {:58} {:6}",
            "#", "Host", "DID", "Twins"
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
        let twins = format!("{:6}", message.twins.len());

        writeln!(
            self.stdout,
            "{:4} {:28} {:58} {:6}",
            Paint::yellow(self.hosts_handled),
            Paint::green(&message.url),
            message.host_did,
            twins
        )
        .expect("this should not happen");
        self.stdout.flush().expect("this should not happen");

        self.host_twins.insert(message.url, message.twins);

        if self.hosts_found == self.hosts_handled {
            let addr = ctx.address();
            addr.try_send(GenerateKbMessage)
                .expect("failed to send GenerateKbMessage message");
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
            let addr = ctx.address();
            addr.try_send(GenerateKbMessage)
                .expect("failed to send GenerateKbMessage message");
        }
    }
}

impl<W> Handler<GenerateKbMessage> for CoordinatorActor<W>
where
    W: io::Write + marker::Send + marker::Sync + 'static,
{
    type Result = ();

    fn handle(&mut self, _: GenerateKbMessage, ctx: &mut Context<Self>) -> Self::Result {
        let host_count = self.host_twins.len();
        let twin_count = self
            .host_twins
            .values()
            .fold(0, |acc, twins| acc + twins.len());

        writeln!(
            self.stdout,
            "Generating the Knowledge Graph from {} hosts and {} twins...",
            host_count, twin_count
        )
        .expect("this should not happen");
        self.stdout.flush().expect("this should not happen");

        // generate KG
        let store = MemoryStore::new();
        store
            .load_graph(
                diddy::ONTOLOGY.as_ref(),
                GraphFormat::Turtle,
                &GraphName::DefaultGraph,
                None,
            )
            .expect("failed to load the ontology");

        // TODO: add data
        for (host, _) in &self.host_twins {
            store.insert(
                TripleRef::new(NamedNodeRef::new_unchecked(host), rdf::TYPE, diddy::HOST)
                    .in_graph(&GraphName::DefaultGraph),
            );
            store.insert(
                TripleRef::new(
                    NamedNodeRef::new_unchecked(host),
                    rdfs::LABEL,
                    LiteralRef::new_simple_literal(host),
                )
                .in_graph(&GraphName::DefaultGraph),
            );
            store.insert(
                TripleRef::new(
                    NamedNodeRef::new_unchecked(host),
                    diddy::HOST_ADDRESS,
                    LiteralRef::new_simple_literal(&format!("https://{}.iotics.space", host)),
                )
                .in_graph(&GraphName::DefaultGraph),
            );
        }

        // dump to file
        let mut buffer = Vec::new();
        store
            .dump_graph(&mut buffer, GraphFormat::Turtle, &GraphName::DefaultGraph)
            .expect("failed to dump the graph");

        let file_path = "export/iotics_twins.ttl".to_string();
        let mut file = File::create(&file_path).expect("failed to create the export file");
        file.write_all(diddy::ONTOLOGY.as_bytes())
            .expect("failed to write to the export file");
        file.write_all(&buffer)
            .expect("failed to write to the export file");

        ctx.stop();
    }
}
