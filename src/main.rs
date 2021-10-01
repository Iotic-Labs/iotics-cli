use log::{error, LevelFilter};
use std::io::stdout;
use structopt::StructOpt;

mod commands;

use commands::{delete_twins::DeleteTwins, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);

    pretty_env_logger::formatted_timed_builder()
        .filter(Some("iotics_cli"), log_level)
        .init();

    let mut stdout = stdout();
    let command = Command::from_args();

    match command {
        Command::DeleteTwins(args) => {
            let command = DeleteTwins::new(&mut stdout, args);

            match command {
                Ok(mut command) => {
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
