use std::{fs, sync::Arc};
use tempfile::tempdir;
use tjspace_core::{os_updates::{OsUpdateConfig,OsUpdateManager}, patch_db::PatchDb};

fn manager(root: &std::path::Path)->OsUpdateManager {
    let db=PatchDb::open_database(root.join("state.db")).unwrap();
    OsUpdateManager::new(db,OsUpdateConfig{state_root:root.join("os"),dry_run:true,..Default::default()})
}
#[tokio::test]
async fn default_state_and_history_are_empty() {
    let d=tempdir().unwrap(); let m=manager(d.path());
    assert!(m.get_update_history().unwrap().is_empty());
}
#[test]
fn safe_mode_persists() {
    let d=tempdir().unwrap(); let m=manager(d.path());
    m.EnterSafeMode().unwrap();
    assert!(d.path().join("os/os-safe-mode").exists());
    m.ExitSafeMode().unwrap();
    assert!(!d.path().join("os/os-safe-mode").exists());
}
#[test]
fn factory_reset_preserves_volume_copy() {
    let d=tempdir().unwrap(); let m=manager(d.path());
    fs::create_dir_all(d.path().join("os/volumes/data")).unwrap();
    fs::write(d.path().join("os/volumes/data/file"),b"x").unwrap();
    let r=m.FactoryReset(true).unwrap();
    assert!(r.os_wiped);
    assert!(r.preserved_volume_root.join("volumes/data/file").exists());
}
#[test]
fn rollback_rejects_unknown_version() {
    let d=tempdir().unwrap(); let m=manager(d.path());
    assert!(m.RollbackOs("9.9.9").is_err());
}
#[test]
fn bundle_validation_requires_daemon() {
    let d=tempdir().unwrap(); let p=d.path().join("x.tar");
    let f=fs::File::create(&p).unwrap(); let mut ar=tar::Builder::new(f); ar.append_data(&mut tar::Header::new_gnu(),"README",b"x".as_slice()).unwrap(); ar.into_inner().unwrap();
    assert!(super_validate(&p).is_err());
}
fn super_validate(p:&std::path::Path)->anyhow::Result<()> {
    let f=fs::File::open(p)?; let mut ar=tar::Archive::new(f); for e in ar.entries()? { let e=e?; if e.path()?=="usr/local/bin/tjs-box"{return Ok(())} } Err(anyhow::anyhow!("missing"))
}
