use bonsaidb::local::config::{Builder, StorageConfiguration};
use clap::{Parser, Subcommand};

mod cli;
mod db;
mod ethereum;
mod foundry;
mod observe;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {

    #[command(subcommand)]
    command: Commands,

}

#[derive(Subcommand)]
enum Commands {

    Observe {

        #[clap(short = 'r', long = "rpc-url", env = "ETH_RPC_URL")]
        rpc_url: Option<String>,

        #[clap(short = 'p', long = "project")]
        project: Option<String>,

        #[clap(short = 'c', long = "concurrency")]
        parallelism: Option<u64>,

        #[clap(long = "function")]
        function: Option<String>,

        #[clap(short = 'b', long = "block-number")]
        block_number: Option<u64>, 

        #[clap(long, short, action)]
        debug: bool,

    },
    
}

#[tokio::main]
async fn main() {

    let db = db::BythDatabase::new(
        StorageConfiguration::new(".$.bonsaidb")
    )
        .unwrap();

    let err = match Cli::parse().command {

        Commands::Observe {
            rpc_url,
            project,
            parallelism,
            function,
            block_number,
            debug,
        } => observe::main(
            db,
            rpc_url,
            project,
            parallelism.unwrap_or(1),
            function,
            block_number,
            debug,
        ).await,

    };

    if let Some(err) = err {
        cli::error(format!("{}", err));
    }

}
