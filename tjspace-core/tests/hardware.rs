use tempfile::tempdir;
use tjspace_core::hardware::{HardwareConfig, HardwareManager};

#[test]
fn confirmation_is_issued_and_consumed() {
    let dir=tempdir().unwrap();
    let db=tjspace_core::patch_db::PatchDb::open_database(dir.path().join("state.db").to_str().unwrap()).unwrap();
    let mgr=HardwareManager::new(db, HardwareConfig { dry_run:true, ..Default::default() });
    let confirmation=mgr.request_confirmation("delete-volume","vol-test").unwrap();
    mgr.confirm_destructive_operation(&confirmation.operation_id).unwrap();
}

#[tokio::test]
async fn filesystem_validation_rejects_unknown_type() {
    let dir=tempdir().unwrap();
    let db=tjspace_core::patch_db::PatchDb::open_database(dir.path().join("state.db").to_str().unwrap()).unwrap();
    let mgr=HardwareManager::new(db, HardwareConfig { dry_run:true, ..Default::default() });
    let result=mgr.CreateVolume("x",&["/dev/nope".into()],"none","ntfs").await;
    assert!(result.is_err());
}

#[test]
fn confirmation_is_bound_to_operation() {
    let dir=tempdir().unwrap();
    let db=tjspace_core::patch_db::PatchDb::open_database(dir.path().join("state.db").to_str().unwrap()).unwrap();
    let mgr=HardwareManager::new(db, HardwareConfig { dry_run:true, ..Default::default() });
    let confirmation=mgr.request_confirmation("delete-volume","vol-a").unwrap();
    assert!(mgr.confirm_destructive_operation(&confirmation.operation_id).is_ok());
    assert!(mgr.confirm_destructive_operation(&confirmation.operation_id).is_err());
}
