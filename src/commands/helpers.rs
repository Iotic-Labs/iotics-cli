use std::sync::Arc;
use std::{io, marker};
use yansi::Paint;

use iotics_grpc_client::twin::crud::delete_twin;

use crate::commands::settings::AuthBuilder;

pub async fn delete_and_log_twin<W>(
    stdout: &'_ mut W,
    auth_builder: Arc<AuthBuilder>,
    twin_did: &str,
    prev_index: usize,
    verbose: bool,
) -> Result<(), anyhow::Error>
where
    W: io::Write + marker::Send,
{
    if verbose {
        write!(stdout, "Deleting twin {}... ", twin_did)?;
    }

    if !verbose && prev_index != 0 && prev_index % 64 == 0 {
        writeln!(stdout)?;
    }

    let result = delete_twin(auth_builder, twin_did).await;

    match &result {
        Ok(_) => {
            if verbose {
                writeln!(stdout, "{}", Paint::green("OK"))?;
            } else {
                write!(stdout, "{}", Paint::green("."))?;
            }
        }
        Err(e) => {
            if verbose {
                writeln!(stdout, "{:?}", Paint::red(e))?;
            } else {
                write!(stdout, "{}", Paint::red("E"))?;
            }
        }
    };

    stdout.flush()?;

    result
}
