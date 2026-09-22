    if value.is_empty() || value.len() > 128 || !value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.') {
        bail!("invalid identifier");
    }
    Ok(())
}
fn validate_container_id(value: &str) -> Result<()> {
    validate_id(value)?;
    if !value.starts_with("tjs-") { bail!("container id must start with tjs-"); }
    Ok(())
}
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;