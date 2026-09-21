use anyhow::Result;
pub fn validate_endpoint(endpoint:&str)->Result<()>{
    if endpoint.trim().is_empty(){anyhow::bail!("endpoint is empty")}
    if endpoint.contains(' '){anyhow::bail!("endpoint contains whitespace")}
    Ok(())
}
