use tjspace_core::headless_api::{issue_jwt, Cli, rpc_for_path};
use clap::Parser;

#[test]
fn jwt_is_compact_and_three_part() {
    let token=issue_jwt("test-secret","ci","admin",60).unwrap();
    assert_eq!(token.split('.').count(),3);
}

#[test]
fn cli_parses_json_and_quiet_after_command() {
    let cli=Cli::try_parse_from(["tjspace-cli","status","--json","--quiet"]).unwrap();
    assert!(cli.json && cli.quiet);
}

#[test]
fn cli_parses_service_commands() {
    let cli=Cli::try_parse_from(["tjspace-cli","service","stop","svc.demo","--force"]).unwrap();
    match cli.command { tjspace_core::headless_api::CliCommand::Service(tjspace_core::headless_api::ServiceCommand::Stop{package_id,force})=>{assert_eq!(package_id,"svc.demo");assert!(force)}, _=>panic!("wrong command") }
}

#[test]
fn rest_paths_map_to_rpc() {
    assert_eq!(rpc_for_path("service/start").as_deref(),Some("StartService"));
    assert_eq!(rpc_for_path("package/verify").as_deref(),Some("VerifyPackage"));
    assert_eq!(rpc_for_path("update/rollback").as_deref(),Some("RollbackOs"));
    assert!(rpc_for_path("unknown/path").is_none());
}

#[test]
fn volume_create_is_scriptable() {
    let cli=Cli::try_parse_from(["tjspace-cli","volume","create","data","--disks","/dev/sda","/dev/sdb","--raid-level","raid1","--fs-type","ext4","--json"]).unwrap();
    match cli.command { tjspace_core::headless_api::CliCommand::Volume(tjspace_core::headless_api::VolumeCommand::Create{name,disks,raid_level,fs_type,..})=>{assert_eq!(name,"data");assert_eq!(disks.len(),2);assert_eq!(raid_level,"raid1");assert_eq!(fs_type,"ext4")}, _=>panic!("wrong command") }
}
