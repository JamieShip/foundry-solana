use clap::{Parser, Subcommand};

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("VERGEN_GIT_SHA"),
    " ",
    env!("VERGEN_BUILD_TIMESTAMP"),
    ")"
);

#[derive(Parser)]
#[command(
    name = "castsol",
    version = VERSION_MESSAGE,
    after_help = "Find more information on Github: https://github.com/JamieShip/foundry-solana",
    next_display_order = None,
)]
pub struct CastSol {
    #[command(subcommand)]
    pub cmd: CastSubCommand,
}

#[derive(Subcommand)]
pub enum CastSubCommand {
    #[command(visible_aliases = &["--transaction", "tx"])]
    Tx {
        #[arg(default_value = "string")]
        tx_signature: String,
        #[command(flatten)]
        rpc: RpcOpts,

        #[arg(short = 'w', long = "with-balance-changes")]
        with_balance_changes: Option<bool>,
    },
}

#[derive(Clone, Debug, Default, Parser)]
pub struct RpcOpts {
    /// The RPC endpoint.
    #[arg(short = 'r', long = "rpc-url", env = "SOL_RPC_URL")]
    pub url: Option<String>,
}
