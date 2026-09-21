use clap::{Parser,Subcommand};
use tjspace_core::{Core, s9pk_toolchain};

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
    PackageBuild{project_dir:String,output_path:String},
    PackageManifest{s9pk_path:String},
    PackageMerkle{s9pk_path:String},
    PackageSign{s9pk_path:String,private_key_hex:String},
    PackageVerify{s9pk_path:String,public_key_hex:String},
    PackagePartial{s9pk_path:String,start:u64,end:u64},
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
        Command::PackageBuild{project_dir,output_path}=>s9pk_toolchain::BuildS9pk(&project_dir,&output_path)?,
        Command::PackageManifest{s9pk_path}=>println!("{}",serde_json::to_string_pretty(&s9pk_toolchain::ParseManifest(&s9pk_path)?)?),
        Command::PackageMerkle{s9pk_path}=>println!("{}",hex::encode(s9pk_toolchain::ComputeMerkleRoot(&s9pk_path)?)),
        Command::PackageSign{s9pk_path,private_key_hex}=>s9pk_toolchain::SignS9pk(&s9pk_path,&hex::decode(private_key_hex)?)?,
        Command::PackageVerify{s9pk_path,public_key_hex}=>s9pk_toolchain::VerifyS9pk(&s9pk_path,&hex::decode(public_key_hex)?)?,
        Command::PackagePartial{s9pk_path,start,end}=>{let b=s9pk_toolchain::ExtractPartial(&s9pk_path,start..end)?;std::io::Write::write_all(&mut std::io::stdout(),&b)?;},
    }
    Ok(())
}
