use anyhow::Result;
use std::net::SocketAddr;
pub fn parse_bind(value:&str)->Result<SocketAddr>{Ok(value.parse()?)}
