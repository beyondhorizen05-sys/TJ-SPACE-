use anyhow::Result;
use serde::{Deserialize,Serialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RegistryConfig{pub url:String,#[serde(default)]pub enabled:bool}
pub fn validate(cfg:&RegistryConfig)->Result<()>{
    let (scheme,rest)=cfg.url.split_once("://").ok_or_else(||anyhow::anyhow!("invalid registry URL"))?;
    if rest.is_empty(){anyhow::bail!("invalid registry URL")}
    if scheme!="https" && scheme!="http"{anyhow::bail!("registry URL must be HTTP(S)")}
    Ok(())
}
