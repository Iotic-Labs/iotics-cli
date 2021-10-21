use async_trait::async_trait;
use futures::future::join_all;
use std::{io, marker};
use structopt::StructOpt;
use yansi::Paint;

use iotics_grpc_client::{
    create_interest_api_client, follow_with_client, search, Filter, Property, Scope, TwinId, Uri,
    Value,
};

use super::{
    settings::{get_token, Settings},
    RunnableCommand,
};

#[derive(Debug, StructOpt)]
pub struct FollowByModelArgs {
    /// Configuration file stored in the `configuration` folder. Don't include the extension.
    #[structopt(short, long)]
    pub config: String,
    /// The model DID
    #[structopt(short, long)]
    pub model_did: String,
    /// The feed ID
    #[structopt(long)]
    pub feed_id: String,
    /// The follower twin DID. It should be local to the given host in the configuration file
    #[structopt(long)]
    pub follower_twin_did: String,
    /// The maximum number of twins to follow
    #[structopt(long)]
    pub maximum_twins: usize,
    /// Logging level
    #[structopt(short, long)]
    pub verbose: bool,
}

pub struct FollowByModel<'a, W>
where
    W: io::Write + marker::Send,
{
    stdout: &'a mut W,
    opts: FollowByModelArgs,
    settings: Settings,
}

impl<'a, W> FollowByModel<'a, W>
where
    W: io::Write + marker::Send,
{
    pub fn new(stdout: &'a mut W, opts: FollowByModelArgs) -> Result<Self, anyhow::Error> {
        let settings = Settings::new(&opts.config, stdout)?;
        Ok(Self {
            stdout,
            opts,
            settings,
        })
    }
}

#[async_trait]
impl<'a, W> RunnableCommand for FollowByModel<'a, W>
where
    W: io::Write + marker::Send,
{
    async fn run(&mut self) -> Result<(), anyhow::Error> {
        let token = get_token(&self.settings)?;

        let mut search_stream = search(
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
            Scope::Global,
            None,
        )
        .await?;

        let mut count = 0;
        let mut follow_handles = Vec::new();
        let client = create_interest_api_client(&self.settings.iotics.host_address).await?;
        let follower_twin_id = TwinId {
            value: self.opts.follower_twin_did.clone(),
        };

        while let Some(response) = search_stream.recv().await {
            match response {
                Ok(page) => {
                    if let Some(payload) = page.payload {
                        let twins = payload.twins;

                        if !twins.is_empty() {
                            writeln!(
                                self.stdout,
                                "Found {} twins for model {}. Following...",
                                Paint::yellow(twins.len()),
                                Paint::blue(&self.opts.model_did),
                            )?;
                            self.stdout.flush()?;

                            for twin in twins {
                                let token = token.clone();
                                let mut interest_channel = client.clone();
                                let followed_twin_id =
                                    twin.id.expect("this should not happen").clone();
                                let followed_host_id = payload.remote_host_id.clone();
                                let followed_feed = self.opts.feed_id.clone();
                                let follower_twin_id = follower_twin_id.clone();
                                let verbose = self.opts.verbose;

                                let fut = async move {
                                    let twin_did = followed_twin_id.value.clone();

                                    if verbose {
                                        println!(
                                            "Follower #{} started for twin {} and feed {}",
                                            Paint::yellow(count),
                                            Paint::blue(&twin_did),
                                            Paint::blue(&followed_feed)
                                        );
                                    }

                                    let mut follow_stream = follow_with_client(
                                        &mut interest_channel,
                                        &token,
                                        followed_host_id,
                                        followed_twin_id,
                                        followed_feed,
                                        follower_twin_id,
                                    )
                                    .await;

                                    match follow_stream.as_mut() {
                                        Ok(follow_stream) => {
                                            let mut active = true;

                                            while active {
                                                let message = follow_stream.message().await;

                                                match message {
                                                    Ok(Some(result)) => {
                                                        if let Some(payload) = result.payload {
                                                            if let Some(feed_data) =
                                                                payload.feed_data
                                                            {
                                                                if feed_data
                                                                    .mime
                                                                    .starts_with("application/json")
                                                                {
                                                                    let json_data: Result<
                                                                        serde_json::Value,
                                                                        serde_json::Error,
                                                                    > = serde_json::from_slice(
                                                                        &feed_data.data,
                                                                    );

                                                                    match json_data {
                                                                        Ok(json_data) => {
                                                                            println!(
                                                                                "[{}] {:?}",
                                                                                twin_did,
                                                                                Paint::green(
                                                                                    json_data
                                                                                )
                                                                            );
                                                                        }
                                                                        Err(e) => {
                                                                            println!(
                                                                                "[{}] failed to deserialize: {:?}",
                                                                                twin_did,
                                                                                Paint::red(e)
                                                                            );
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Ok(None) => {
                                                        println!("[{}] got message None", twin_did);
                                                    }
                                                    Err(e) => {
                                                        active = false;
                                                        println!(
                                                            "[{}] crashed: {:?}",
                                                            twin_did,
                                                            Paint::red(e)
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!(
                                                "[{}] failed to follow: {:?}",
                                                twin_did,
                                                Paint::red(e)
                                            );
                                        }
                                    }
                                };

                                let handle = tokio::spawn(fut);

                                count += 1;
                                follow_handles.push(handle);

                                if count >= self.opts.maximum_twins {
                                    search_stream.close();
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    writeln!(self.stdout, "{:?}", Paint::red(e))?;
                    self.stdout.flush()?;
                }
            }
        }

        join_all(follow_handles).await;

        Ok(())
    }
}
