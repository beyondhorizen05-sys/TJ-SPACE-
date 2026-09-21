use tempfile::tempdir;
use tjspace_core::{patch_db::PatchDb, os_installer::{OsInstaller,InstallerConfig,PartitionLayout,PartitionSpec}};

fn mgr()->OsInstaller{let d=tempdir().unwrap();let db=PatchDb::open_database(d.path().join("state.db").to_str().unwrap()).unwrap();std::mem::forget(d);OsInstaller::new(db,InstallerConfig{dry_run:true,live_mode_required:true,command_timeout_seconds:5})}

#[test] fn layout_requires_efi_and_root(){let m=mgr();let bad=PartitionLayout{gpt:true,partitions:vec![PartitionSpec{name:"root".into(),size_bytes:1,filesystem:"ext4".into(),mountpoint:Some("/".into()),esp:false,swap:false}]};assert!(m.partition_disk("/dev/fake",&bad).is_err());}
#[test] fn confirmation_roundtrip(){let m=mgr();let id=m.request_confirmation("install","/dev/sda").unwrap();assert!(m.confirm(&id).is_ok());}
#[tokio::test] async fn missing_confirmation_is_rejected(){let m=mgr();let o=tjspace_core::os_installer::InstallOptions{layout:PartitionLayout{gpt:true,partitions:vec![PartitionSpec{name:"efi".into(),size_bytes:1,filesystem:"vfat".into(),mountpoint:Some("/boot/efi".into()),esp:true,swap:false},PartitionSpec{name:"root".into(),size_bytes:1,filesystem:"ext4".into(),mountpoint:Some("/".into()),esp:false,swap:false}]},base_source:"x".into(),hostname:"t".into(),admin_public_key:"k".into(),network:None,headless:true,restore_source:None,confirm_id:None};assert!(m.run_installer("/dev/sda",&o).await.is_err());}
#[test] fn invalid_partition_is_rejected(){let m=mgr();assert!(m.partition_disk("../dev/sda",&PartitionLayout{gpt:true,partitions:vec![]}).is_err());}
#[tokio::test] async fn verify_reports_missing_install(){let m=mgr();let r=m.verify_installation().await.unwrap();assert!(!r.daemon_present);}
