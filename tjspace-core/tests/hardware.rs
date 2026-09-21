use tempfile::tempdir;
use tjspace_core::hardware::{HardwareConfig, HardwareManager};

fn manager() -> HardwareManager {
    let dir=tempdir().unwrap();
    let db=tjspace_core::patch_db::PatchDb::open_database(dir.path().join("state.db").to_str().unwrap()).unwrap();
    std::mem::forget(dir);
    HardwareManager::new(db, HardwareConfig { dry_run:true, ..Default::default() })
}

#[test]
fn confirmation_is_issued_and_consumed() {
    let mgr=manager();
    let confirmation=mgr.request_confirmation("delete-volume","vol-test").unwrap();
    mgr.confirm_destructive_operation(&confirmation.operation_id).unwrap();
}

#[test]
fn confirmation_cannot_be_confirmed_twice() {
    let mgr=manager();
    let confirmation=mgr.request_confirmation("delete-volume","vol-test").unwrap();
    mgr.confirm_destructive_operation(&confirmation.operation_id).unwrap();
    assert!(mgr.confirm_destructive_operation(&confirmation.operation_id).is_err());
}

#[test]
fn expired_confirmation_is_rejected() {
    let dir=tempdir().unwrap();
    let db=tjspace_core::patch_db::PatchDb::open_database(dir.path().join("state.db").to_str().unwrap()).unwrap();
    let mgr=HardwareManager::new(db, HardwareConfig { dry_run:true, confirmation_ttl_seconds:0, ..Default::default() });
    let confirmation=mgr.request_confirmation("delete-volume","vol-test").unwrap();
    assert!(mgr.confirm_destructive_operation(&confirmation.operation_id).is_err());
}

#[tokio::test]
async fn unknown_filesystem_is_rejected() {
    let mgr=manager();
    let result=mgr.CreateVolume("x",&["/dev/nope".into()],"none","ntfs").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn invalid_driver_name_is_rejected() {
    let mgr=manager();
    assert!(mgr.LoadDriver("../bad").await.is_err());
}

#[tokio::test]
async fn invalid_wifi_ssid_is_rejected() {
    let mgr=manager();
    let long_ssid="x".repeat(256);
    let creds=tjspace_core::hardware::WifiCredentials { password:None, identity:None };
    assert!(mgr.ConfigureWifi(&long_ssid,&creds).await.is_err());
}

#[test]
fn audit_is_persisted_in_patch_db() {
    let mgr=manager();
    let _=mgr.request_confirmation("create-volume","vol-test").unwrap();
    let snapshot=mgr.get_state_snapshot().unwrap();
    assert!(snapshot.state["hardware"]["audit"].as_object().map(|m|!m.is_empty()).unwrap_or(false));
}
