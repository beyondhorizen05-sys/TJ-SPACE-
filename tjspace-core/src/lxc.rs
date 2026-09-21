use anyhow::{anyhow,Result};
use crate::RuntimeConfig;
use tokio::process::Command;
use tokio::time::{timeout,Duration};

async fn run(cfg:&RuntimeConfig,args:&[&str])->Result<()>{
    if cfg.dry_run { return Ok(()); }
    let fut=Command::new("lxc").args(args).output();
    let out=timeout(Duration::from_secs(cfg.command_timeout_seconds),fut)
        .await.map_err(|_|anyhow!("lxc command timed out"))??;
    if !out.status.success(){return Err(anyhow!("lxc {:?} failed: {}",args,String::from_utf8_lossy(&out.stderr)))}
    Ok(())
}
pub async fn start(cfg:&RuntimeConfig,pkg:&str)->Result<()>{run(cfg,&["start",pkg]).await}
pub async fn stop(cfg:&RuntimeConfig,pkg:&str,graceful:bool)->Result<()>{
    if graceful { run(cfg,&["stop",pkg]).await } else { run(cfg,&["stop",pkg,"--force"]).await }
}
pub async fn remove(cfg:&RuntimeConfig,pkg:&str)->Result<()>{run(cfg,&["delete",pkg,"--force"]).await}
