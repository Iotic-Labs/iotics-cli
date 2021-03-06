use log::{error, LevelFilter};
use std::io::stdout;
use structopt::StructOpt;

mod commands;

use commands::delete_all_twins::DeleteAllTwins;
use commands::delete_twins_by_model::DeleteTwinsByModel;
use commands::describe_twin::DescribeTwin;
use commands::follow_by_model::FollowByModel;
use commands::list_hosts::ListHosts;
use commands::{Command, RunnableCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);

    pretty_env_logger::formatted_timed_builder()
        .filter(Some("iotics_"), log_level)
        .init();

    let mut stdout = stdout();
    let command = Command::from_args();

    match command {
        Command::DeleteTwinsByModel(args) => {
            let command = DeleteTwinsByModel::new(&mut stdout, args);

            match command {
                Ok(command) => {
                    let result = command.run().await;

                    if let Err(e) = result {
                        error!("{:?}", e);
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
        Command::DeleteAllTwins(args) => {
            let command = DeleteAllTwins::new(&mut stdout, args);

            match command {
                Ok(command) => {
                    let result = command.run().await;

                    if let Err(e) = result {
                        error!("{:?}", e);
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
        Command::FollowByModel(args) => {
            let command = FollowByModel::new(&mut stdout, args);

            match command {
                Ok(command) => {
                    let result = command.run().await;

                    if let Err(e) = result {
                        error!("{:?}", e);
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
        Command::ListHosts(args) => {
            let command = ListHosts::new(stdout, args);

            match command {
                Ok(command) => {
                    let result = command.run().await;

                    if let Err(e) = result {
                        error!("{:?}", e);
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
        Command::DescribeTwin(args) => {
            let command = DescribeTwin::new(&mut stdout, args);

            match command {
                Ok(command) => {
                    let result = command.run().await;

                    if let Err(e) = result {
                        error!("{:?}", e);
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }

    Ok(())
}
