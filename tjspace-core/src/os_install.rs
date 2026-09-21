use anyhow::Result;
use std::path::Path;
pub fn validate_boot_target(path:&str)->Result<()>{
    if !Path::new(path).exists(){anyhow::bail!("boot target does not exist")}
    Ok(())
}
