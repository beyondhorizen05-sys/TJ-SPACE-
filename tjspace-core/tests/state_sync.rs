use tjspace_core::{
    patch_db::{Patch, PatchOp, TypedDiff},
    state_sync::{ApplyInterpolationHint, SyncConfig, SyncFilter, SyncBridge},
};
use serde_json::json;

fn diff(revision:u64,path:&str)->TypedDiff{
    TypedDiff{
        version:1,
        revision,
        patch:Patch{version:1,path:path.into(),op:PatchOp::Set,value:Some(json!(revision)),actor:"test".into(),authorization:"allow".into()},
        previous:None,
        current:Some(json!(revision)),
    }
}

#[test]
fn sequence_gap_detection_is_strict() {
    let bridge=SyncBridge::new(
        tjspace_core::patch_db::PatchDb::open_database(":memory:").unwrap(),
        SyncConfig::default(),
        "token".into(),
    );
    assert!(!bridge.DetectGap(0,1));
    assert!(bridge.DetectGap(0,2));
    assert!(!bridge.DetectGap(10,9));
}

#[test]
fn interpolation_hint_is_deterministic() {
    let smooth=ApplyInterpolationHint(&diff(1,"scene.camera"));
    assert_eq!(smooth.mode,"smooth");
    assert_eq!(smooth.duration_ms,100);
    assert!(smooth.authoritative);
    let delete=TypedDiff{patch:Patch{op:PatchOp::Delete,..diff(2,"scene.camera").patch},..diff(2,"scene.camera")};
    assert_eq!(ApplyInterpolationHint(&delete).mode,"snap");
}

#[test]
fn batch_coalesces_repeated_paths() {
    let bridge=SyncBridge::new(
        tjspace_core::patch_db::PatchDb::open_database(":memory:").unwrap(),
        SyncConfig{max_batch_diffs:8,..Default::default()},
        "".into(),
    );
    let a=bridge.OpenSyncSession("client","").unwrap().0;
    let d1=bridge.AssignSequence(&a,diff(1,"x")).unwrap();
    let d2=bridge.AssignSequence(&a,diff(2,"x")).unwrap();
    let d3=bridge.AssignSequence(&a,diff(3,"y")).unwrap();
    let batches=bridge.BatchDiffs(&[d1,d2,d3],50);
    assert_eq!(batches.len(),1);
    assert_eq!(batches[0].diffs.len(),2);
    assert_eq!(batches[0].diffs[0].diff.revision,2);
}

#[test]
fn sync_auth_is_enforced() {
    let bridge=SyncBridge::new(
        tjspace_core::patch_db::PatchDb::open_database(":memory:").unwrap(),
        SyncConfig::default(),
        "secret".into(),
    );
    assert!(bridge.OpenSyncSession("client","bad").is_err());
    assert!(bridge.OpenSyncSession("client","secret").is_ok());
}

#[tokio::test]
async fn full_snapshot_uses_patch_db_revision() {
    let db=tjspace_core::patch_db::PatchDb::open_database(":memory:").unwrap();
    let p=Patch{version:1,path:"world.value".into(),op:PatchOp::Set,value:Some(json!(42)),actor:"test".into(),authorization:"allow".into()};
    db.apply_patch(p).unwrap();
    let bridge=SyncBridge::new(db,SyncConfig::default(),"".into());
    let session=bridge.OpenSyncSession("client","").unwrap().0;
    let env=bridge.RequestFullSnapshot(&session).unwrap();
    assert_eq!(env.server_revision,1);
    assert!(env.snapshot.is_some());
}

#[test]
fn config_has_bounded_defaults() {
    let c=SyncConfig::default();
    assert!(c.batch_latency_ms <= 1000);
    assert!(c.max_batch_diffs > 0);
    assert!(c.session_queue >= 4);
    assert!(!SyncFilter{prefix:Some("x".into())}.prefix.unwrap().is_empty());
}
