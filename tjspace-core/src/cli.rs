use clap::Parser;
use tjspace_core::headless_api::{run_cli,Cli};
#[tokio::main]
async fn main(){std::process::exit(run_cli(Cli::parse()).await);}
