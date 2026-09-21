use crate::version::VERSION;
pub const DAEMON_ALIAS:&str="tjsd";
pub const CLI_ALIAS:&str="tjs-cli";
pub fn binary_identity()->&'static str{VERSION}
