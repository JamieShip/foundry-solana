use std::str::FromStr;

use clap::Parser;
use common::TransactionHandler;
use eyre::{Ok, Result};
use opts::CastSol;
use solana_sdk::signature::Signature;
pub mod cmd;
pub mod opts;

#[tokio::main]
async fn main() -> Result<()> {
    common::handler::install();
    common::log::subscriber();
    let castsol_cmd = CastSol::parse();

    match castsol_cmd.cmd {
        opts::CastSubCommand::Tx { tx_signature, rpc, with_balance_changes } => {
            TransactionHandler::new(rpc.url.unwrap()).handle_tx(
                &Signature::from_str(tx_signature.as_str()).unwrap(),
                with_balance_changes.unwrap_or(false),
            )?
        }
    }

    Ok(())
}
