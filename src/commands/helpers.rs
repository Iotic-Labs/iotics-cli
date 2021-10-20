use std::{io, marker};
use yansi::Paint;

use iotics_grpc_client::delete_twin;

pub async fn delete_and_log_twin<W>(
    stdout: &'_ mut W,
    host_address: &str,
    token: &str,
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

    let result = delete_twin(host_address, token, twin_did).await;

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
