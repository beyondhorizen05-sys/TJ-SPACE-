use anyhow::Result;
pub async fn validate_version(current:&str,target:&str)->Result<()>{
    if target.trim().is_empty(){anyhow::bail!("target version empty")}
    if current==target{anyhow::bail!("target version equals current")}
    Ok(())
}
