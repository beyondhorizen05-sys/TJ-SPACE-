use tempfile::tempdir;
use tjspace_core::hardware::{HardwareConfig, HardwareManager};

#[test]
fn confirmation_is_issued_and_consumed() {
    let dir=tempdir().unwrap();
    let db=tjspace_core::patch_db::PatchDb::open_database(dir.path().join("state.db").to_str().unwrap()).unwrap();
    let mgr=HardwareManager::new(db, HardwareConfig { dry_run:true, ..Default::default() });
    let c=mgr.request_confirmation("delete-volume","vol-test").unwrap();
    mgr.confirm_destructive_operation(&c.operation_id).unwrap();
}

#[test]
fn filesystem_validation_rejects_unknown_type() {
    let dir=tempdir().unwrap();
    let db=tjspace_core::patch_db::PatchDb::open_database(dir.path().join("state.db").to_str().unwrap()).unwrap();
    let mgr=HardwareManager::new(db, HardwareConfig { dry_run:true, ..Default::default() });
    let result=futures_lite::future::block_on(mgr.CreateVolume("x",&["/dev/nope".into()],"none","ntfs"));
    assert!(result.is_err());
}
