use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::{Path, PathBuf}, process::Command, time::Duration};
use tokio::time::timeout;
use uuid::Uuid;

use crate::patch_db::{Patch, PatchDb, PatchOp};

const STATE_PATH: &str = "os-update.state.json";
const HISTORY_PATH: &str = "os-update.history.json";
const SAFE_MODE_PATH: &str = "os-safe-mode";
const UPDATE_LOCK: &str = "os-update.lock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsUpdateConfig {
    #[serde(default = "default_channel")]
    pub release_channel: String,
    #[serde(default)]
    pub release_index_url: String,
    #[serde(default = "default_state_root")]
    pub state_root: PathBuf,
    #[serde(default)]
    pub public_key_hex: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default = "default_timeout")]
    pub command_timeout_seconds: u64,
    #[serde(default = "default_boot_attempts")]
    pub max_boot_attempts: u32,
}
fn default_channel() -> String { "stable".into() }
fn default_state_root() -> PathBuf { PathBuf::from("tjspace-os-state") }
fn default_timeout() -> u64 { 120 }
fn default_boot_attempts() -> u32 { 2 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRelease {
    pub release_id: String,
    pub version: String,
    pub channel: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub signature_hex: Option<String>,
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsReleaseIndex {
    pub channel: String,
    pub current_version: String,
    pub releases: Vec<OsRelease>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecord {
    pub id: String,
    pub release_id: String,
    pub from_version: String,
    pub to_version: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
    pub slot: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OsState {
    active_slot: String,
    boot_slot: String,
    active_version: String,
    previous_version: Option<String>,
    pending_release: Option<String>,
    pending_slot: Option<String>,
    boot_attempts: u32,
    safe_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryResetResult {
    pub reset_id: String,
    pub preserve_data: bool,
    pub os_wiped: bool,
    pub preserved_volume_root: PathBuf,
}

#[derive(Clone)]
pub struct OsUpdateManager {
    patch_db: PatchDb,
    config: OsUpdateConfig,
    http: reqwest::Client,
}

impl Default for OsUpdateConfig { fn default()->Self { Self { release_channel:"stable".into(), release_index_url:String::new(), state_root:PathBuf::from("tjspace-os-state"), public_key_hex:None, dry_run:false, command_timeout_seconds:120, max_boot_attempts:2 } } }

impl OsUpdateManager {
    pub fn new(patch_db: PatchDb, config: OsUpdateConfig) -> Self {
        Self { patch_db, config, http: reqwest::Client::new() }
    }

    pub async fn CheckForOsUpdate(&self) -> Result<Option<OsRelease>> {
        if self.config.release_index_url.is_empty() {
            return Err(anyhow!("release_index_url is not configured"));
        }
        let url = format!("{}?channel={}", self.config.release_index_url, urlencoding(&self.config.release_channel));
        let index: OsReleaseIndex = self.http.get(url).send().await?.error_for_status()?.json().await?;
        if index.channel != self.config.release_channel {
            return Err(anyhow!("release channel mismatch"));
        }
        let state = self.load_state()?;
        let current = state.active_version;
        Ok(index.releases.into_iter().filter(|r| r.channel == self.config.release_channel && r.version > current).max_by(|a,b| a.version.cmp(&b.version)))
    }

    pub async fn DownloadOsUpdate(&self, release_id: &str) -> Result<PathBuf> {
        let release = self.find_release(release_id).await?;
        let dir = self.config.state_root.join("downloads");
        fs::create_dir_all(&dir)?;
        let tmp = dir.join(format!("{}.part", safe_id(release_id)?));
        let final_path = dir.join(format!("{}.osupd", safe_id(release_id)?));
        let bytes = self.http.get(&release.url).send().await?.error_for_status()?.bytes().await?;
        fs::write(&tmp, &bytes)?;
        let got = sha256_file(&tmp)?;
        if !eq_hex(&got, &release.sha256) {
            let _ = fs::remove_file(&tmp);
            return Err(anyhow!("OS update SHA-256 mismatch"));
        }
        fs::rename(tmp, &final_path)?;
        Ok(final_path)
    }

    pub async fn VerifyOsUpdate(&self, release_id: &str) -> Result<bool> {
        let release = self.find_release(release_id).await?;
        let path = self.config.state_root.join("downloads").join(format!("{}.osupd", safe_id(release_id)?));
        if !path.exists() { return Err(anyhow!("download not found")); }
        let digest = sha256_file(&path)?;
        if !eq_hex(&digest, &release.sha256) { return Ok(false); }
        if let (Some(sig_hex), Some(key_hex)) = (release.signature_hex.as_deref(), self.config.public_key_hex.as_deref()) {
            let key = VerifyingKey::from_bytes(&hex::decode(key_hex)?.try_into().map_err(|_| anyhow!("public key must be 32 bytes"))?)?;
            let sig = Signature::from_slice(&hex::decode(sig_hex)?)?;
            key.verify(digest.as_bytes(), &sig).map_err(|e| anyhow!("OS update signature invalid: {e}"))?;
        }
        validate_bundle(&path)?;
        Ok(true)
    }

    pub async fn ApplyOsUpdate(&self, release_id: &str) -> Result<UpdateRecord> {
        if self.config.dry_run { return self.apply_dry_run(release_id); }
        let _lock = acquire_lock(&self.config.state_root)?;
        if !self.VerifyOsUpdate(release_id).await? { return Err(anyhow!("OS update verification failed")); }
        let mut state = self.load_state()?;
        if state.safe_mode { return Err(anyhow!("cannot apply OS update while in safe mode")); }
        let release = self.find_release(release_id).await?;
        let inactive = if state.active_slot == "A" { "B" } else { "A" };
        let slot_dir = self.config.state_root.join("slots").join(inactive);
        fs::create_dir_all(&slot_dir)?;
        wipe_dir(&slot_dir)?;
        extract_bundle(&self.config.state_root.join("downloads").join(format!("{}.osupd", safe_id(release_id)?)), &slot_dir)?;
        validate_slot(&slot_dir)?;
        let record_id = Uuid::new_v4().to_string();
        let mut record = UpdateRecord { id:record_id.clone(), release_id:release_id.into(), from_version:state.active_version.clone(), to_version:release.version.clone(), started_at:Utc::now(), finished_at:None, status:"staged".into(), slot:inactive.into(), error:None };
        state.previous_version = Some(state.active_version.clone());
        state.pending_release = Some(release_id.into());
        state.pending_slot = Some(inactive.into());
        state.boot_slot = inactive.into();
        state.boot_attempts = 0;
        self.save_state(&state)?;
        self.persist_history(&record)?;
        if let Err(e) = self.commit_boot_selection(&state) {
            state.boot_slot = state.active_slot.clone();
            state.pending_release = None;
            state.pending_slot = None;
            self.save_state(&state)?;
            record.status = "rolled_back".into();
            record.error = Some(e.to_string());
            record.finished_at = Some(Utc::now());
            self.persist_history(&record)?;
            return Err(e);
        }
        // The new slot becomes active only after a successful boot-health acknowledgement.
        record.status = "pending_reboot".into();
        record.finished_at = Some(Utc::now());
        self.persist_history(&record)?;
        Ok(record)
    }

    pub fn EnterSafeMode(&self) -> Result<()> {
        let mut state = self.load_state()?;
        state.safe_mode = true;
        self.save_state(&state)?;
        fs::create_dir_all(&self.config.state_root)?;
        fs::write(self.config.state_root.join(SAFE_MODE_PATH), b"1")?;
        self.patch_db.apply_patch(Patch{version:1,path:"os.safe_mode".into(),op:PatchOp::Set,value:serde_json::json!(true),actor:"tjsd".into(),authorization:"allow".into()})?;
        Ok(())
    }

    pub fn ExitSafeMode(&self) -> Result<()> {
        let mut state = self.load_state()?;
        state.safe_mode = false;
        self.save_state(&state)?;
        let _ = fs::remove_file(self.config.state_root.join(SAFE_MODE_PATH));
        self.patch_db.apply_patch(Patch{version:1,path:"os.safe_mode".into(),op:PatchOp::Set,value:serde_json::json!(false),actor:"tjsd".into(),authorization:"allow".into()})?;
        Ok(())
    }

    pub fn FactoryReset(&self, preserve_data: bool) -> Result<FactoryResetResult> {
        let _lock = acquire_lock(&self.config.state_root)?;
        let root = self.config.state_root.clone();
        let volumes = root.join("volumes");
        let backup = root.join("factory-reset-preserve");
        fs::create_dir_all(&backup)?;
        if preserve_data && volumes.exists() { copy_tree(&volumes, &backup.join("volumes"))?; }
        let reset_id = Uuid::new_v4().to_string();
        let slots = root.join("slots");
        if slots.exists() { wipe_dir(&slots)?; }
        fs::create_dir_all(slots.join("A"))?;
        fs::create_dir_all(slots.join("B"))?;
        let mut state = default_state();
        state.safe_mode = true;
        self.save_state(&state)?;
        self.patch_db.apply_patch(Patch{version:1,path:"os.factory_reset".into(),op:PatchOp::Set,value:serde_json::json!({"id":reset_id,"preserve_data":preserve_data,"at":Utc::now()}),actor:"tjsd".into(),authorization:"allow".into()})?;
        Ok(FactoryResetResult{reset_id,preserve_data,os_wiped:true,preserved_volume_root:backup})
    }

    pub fn RollbackOs(&self, target_version: &str) -> Result<()> {
        let mut state = self.load_state()?;
        if state.previous_version.as_deref() != Some(target_version) && state.active_version != target_version {
            return Err(anyhow!("target version is not an available rollback target"));
        }
        let target_slot = if state.active_version == target_version { state.active_slot.clone() } else if state.active_slot == "A" { "B".into() } else { "A".into() };
        validate_slot(&self.config.state_root.join("slots").join(&target_slot))?;
        state.boot_slot = target_slot;
        state.pending_release = None;
        state.pending_slot = None;
        state.boot_attempts = 0;
        self.save_state(&state)?;
        self.commit_boot_selection(&state)?;
        Ok(())
    }

    pub fn IsSafeMode(&self) -> Result<bool> { Ok(self.load_state()?.safe_mode) }

    pub fn GetUpdateHistory(&self) -> Result<Vec<UpdateRecord>> {
        let path = self.config.state_root.join(HISTORY_PATH);
        if !path.exists() { return Ok(Vec::new()); }
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }

    pub fn BootHealthAck(&self, version: &str) -> Result<()> {
        let mut state = self.load_state()?;
        if state.pending_slot.is_none() { return Ok(()); }
        if state.active_slot != state.boot_slot { return Err(anyhow!("boot slot mismatch")); }
        if state.active_version != version { return Err(anyhow!("boot version mismatch")); }
        state.pending_release = None;
        state.pending_slot = None;
        state.boot_attempts = 0;
        state.previous_version = None;
        self.save_state(&state)?;
        self.patch_db.apply_patch(Patch{version:1,path:"os.active_version".into(),op:PatchOp::Set,value:serde_json::json!(version),actor:"tjsd".into(),authorization:"allow".into()})?;
        Ok(())
    }

    pub fn RecoverFailedBoot(&self) -> Result<bool> {
        let mut state = self.load_state()?;
        if state.pending_slot.is_none() { return Ok(false); }
        state.boot_attempts += 1;
        if state.boot_attempts <= self.config.max_boot_attempts { self.save_state(&state)?; return Ok(false); }
        let old = if state.boot_slot == "A" { "B" } else { "A" };
        state.boot_slot = old.into();
        state.active_slot = old.into();
        state.pending_slot = None;
        state.pending_release = None;
        state.boot_attempts = 0;
        state.active_version = state.previous_version.clone().unwrap_or(state.active_version);
        self.save_state(&state)?;
        self.commit_boot_selection(&state)?;
        let mut history = self.get_update_history()?;
        if let Some(r) = history.last_mut() { r.status = "automatic_rollback".into(); r.finished_at = Some(Utc::now()); }
        fs::write(self.config.state_root.join(HISTORY_PATH), serde_json::to_vec_pretty(&history)?)?;
        Ok(true)
    }

    fn apply_dry_run(&self, release_id: &str) -> Result<UpdateRecord> {
        let state = self.load_state()?;
        let record = UpdateRecord{id:Uuid::new_v4().to_string(),release_id:release_id.into(),from_version:state.active_version.clone(),to_version:"dry-run".into(),started_at:Utc::now(),finished_at:Some(Utc::now()),status:"dry_run".into(),slot:"B".into(),error:None};
        self.persist_history(&record)?;
        Ok(record)
    }

    async fn find_release(&self, release_id: &str) -> Result<OsRelease> {
        if self.config.release_index_url.is_empty() { return Err(anyhow!("release_index_url is not configured")); }
        let url = format!("{}?channel={}", self.config.release_index_url, urlencoding(&self.config.release_channel));
        let index: OsReleaseIndex = self.http.get(url).send().await?.error_for_status()?.json().await?;
        index.releases.into_iter().find(|r| r.release_id == release_id && r.channel == self.config.release_channel).ok_or_else(|| anyhow!("release not found"))
    }

    fn load_state(&self) -> Result<OsState> {
        fs::create_dir_all(&self.config.state_root)?;
        let p = self.config.state_root.join(STATE_PATH);
        if !p.exists() { let s=default_state(); self.save_state(&s)?; return Ok(s); }
        Ok(serde_json::from_slice(&fs::read(p)?)?)
    }
    fn save_state(&self, state:&OsState)->Result<()> {
        fs::create_dir_all(&self.config.state_root)?;
        atomic_write(&self.config.state_root.join(STATE_PATH), &serde_json::to_vec_pretty(state)?)
    }
    fn persist_history(&self, record:&UpdateRecord)->Result<()> {
        let mut h=self.get_update_history()?;
        h.push(record.clone());
        atomic_write(&self.config.state_root.join(HISTORY_PATH), &serde_json::to_vec_pretty(&h)?)
    }
    fn require_confirmation(&self)->Result<()> { Ok(()) }
    fn commit_boot_selection(&self,state:&OsState)->Result<()> {
        atomic_write(&self.config.state_root.join("boot-slot"), state.boot_slot.as_bytes())?;
        Ok(())
    }
}

fn default_state()->OsState{OsState{active_slot:"A".into(),boot_slot:"A".into(),active_version:env!("CARGO_PKG_VERSION").into(),previous_version:None,pending_release:None,pending_slot:None,boot_attempts:0,safe_mode:false}}

fn validate_bundle(path:&Path)->Result<()> { if !path.exists(){return Err(anyhow!("bundle missing"))}; let f=fs::File::open(path)?; let mut ar=tar::Archive::new(f); let mut ok=false; for e in ar.entries()? { let e=e?; let p=e.path()?; if p=="usr/local/bin/tjs-box" {ok=true;break;} } if !ok{return Err(anyhow!("OS bundle missing usr/local/bin/tjs-box"))}; Ok(()) }
fn extract_bundle(path:&Path,dst:&Path)->Result<()> { let f=fs::File::open(path)?; let mut ar=tar::Archive::new(f); ar.unpack(dst)?; Ok(()) }
fn validate_slot(p:&Path)->Result<()> { if !p.join("usr/local/bin/tjs-box").exists(){return Err(anyhow!("slot validation failed: tjs-box missing"))}; Ok(()) }
fn wipe_dir(p:&Path)->Result<()> { if p.exists(){ for e in fs::read_dir(p)? { let q=e?.path(); if q.is_dir(){fs::remove_dir_all(q)?}else{fs::remove_file(q)?} } } else {fs::create_dir_all(p)?}; Ok(()) }
fn copy_tree(src:&Path,dst:&Path)->Result<()> { fs::create_dir_all(dst)?; for e in fs::read_dir(src)? { let e=e?; let d=dst.join(e.file_name()); if e.path().is_dir(){copy_tree(&e.path(),&d)?}else{fs::copy(e.path(),d)?;} } Ok(()) }
fn atomic_write(path:&Path,data:&[u8])->Result<()> { let tmp=path.with_extension("tmp"); fs::write(&tmp,data)?; fs::rename(tmp,path)?; Ok(()) }
fn sha256_file(path:&Path)->Result<String>{ let b=fs::read(path)?; let mut h=Sha256::new(); h.update(&b); Ok(hex::encode(h.finalize())) }
fn eq_hex(a:&str,b:&str)->bool{a.eq_ignore_ascii_case(b)}
fn safe_id(v:&str)->Result<String>{if v.is_empty()||v.len()>128||!v.chars().all(|c|c.is_ascii_alphanumeric()||c=='-'||c=='_'||c=='.'){return Err(anyhow!("invalid release id"))}Ok(v.into())}
fn urlencoding(v:&str)->String{v.replace('%',"%25").replace(' ',"%20").replace('&',"%26").replace('?',"%3F").replace('=',"%3D")}
fn validate_path_arg(p:&str)->Result<()> { if p.is_empty()||p.contains(' '){Err(anyhow!("invalid path"))}else{Ok(())} }
fn acquire_lock(root:&Path)->Result<fs::File>{ fs::create_dir_all(root)?; let p=root.join(UPDATE_LOCK); match fs::OpenOptions::new().write(true).create_new(true).open(&p){Ok(f)=>Ok(f),Err(e)=>Err(anyhow!("another OS update/reset is active: {e}"))} }
fn run(cfg:&OsUpdateConfig,cmd:&str,args:&[&str])->Result<()> { if cfg.dry_run{return Ok(())}; let status=Command::new(cmd).args(args).status().with_context(||format!("failed to execute {cmd}"))?; if !status.success(){return Err(anyhow!("{cmd} failed with {status}"))} Ok(())}
fn require_live()->Result<()> { if std::env::var("TJS_LIVE_IMAGE").ok().as_deref()!=Some("1"){Err(anyhow!("operation requires TJ SPACE live/recovery environment"))}else{Ok(())} }

impl Drop for OsUpdateManager {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.config.state_root.join(UPDATE_LOCK));
    }
}
