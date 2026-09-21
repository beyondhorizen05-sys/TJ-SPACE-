use clap::{Parser,Subcommand};
use tjspace_core::Core;

#[derive(Parser)]
#[command(name="tjs-box",version,about="TJ SPACE headless core")]
struct Args{
    #[command(subcommand)] command:Option<Command>,
    #[arg(long,default_value="tjspace.yaml")] config:String,
}
#[derive(Subcommand)]
enum Command{
    Daemon,
    State,
    Start{package_id:String},
    Stop{package_id:String,#[arg(long)]force:bool},
    Restart{package_id:String},
    Install{s9pk_path:String},
    Uninstall{package_id:String,#[arg(long)]purge_data:bool},
    Backup{package_id:String,target_id:String},
    Restore{monolith_id:String},
}
#[tokio::main]
async fn main()->anyhow::Result<()>{
    let args=Args::parse();
    let invoked=std::env::args().next().unwrap_or_default();
    let command=match args.command{Some(c)=>c,None if invoked.ends_with("tjs-cli")=>Command::State,None=>Command::Daemon};
    let core=Core::init_core(&args.config).await?;
    match command{
        Command::Daemon=>core.serve().await?,
        Command::State=>println!("{}",serde_json::to_string_pretty(&core.get_system_state().await?)?),
        Command::Start{package_id}=>core.start_service(&package_id).await?,
        Command::Stop{package_id,force}=>core.stop_service(&package_id,!force).await?,
        Command::Restart{package_id}=>core.restart_service(&package_id).await?,
        Command::Install{s9pk_path}=>println!("{}",core.install_package(&s9pk_path).await?),
        Command::Uninstall{package_id,purge_data}=>core.uninstall_package(&package_id,purge_data).await?,
        Command::Backup{package_id,target_id}=>println!("{}",core.orchestrate_backup(&package_id,&target_id).await?),
        Command::Restore{monolith_id}=>core.orchestrate_restore(&monolith_id).await?,
    }
    Ok(())
}
