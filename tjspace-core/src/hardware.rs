use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::patch_db::{Patch, PatchDb, PatchOp};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disk {
    pub id: String,
    pub path: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub size_bytes: u64,
    pub rotational: bool,
    pub filesystem: Option<String>,
    pub mountpoint: Option<String>,
    pub health: DiskHealthSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskHealthSummary {
    pub status: String,
    pub temperature_c: Option<f64>,
    pub power_on_hours: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
    pub uncorrectable_sectors: Option<u64>,
    pub smart_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmartData {
    pub disk_id: String,
    pub status: String,
    pub temperature_c: Option<f64>,
    pub power_on_hours: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
    pub uncorrectable_sectors: Option<u64>,
    pub wear_percent: Option<f64>,
    pub attributes: HashMap<String, Value>,
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: String,
    pub name: String,
    pub device: String,
    pub disks: Vec<String>,
    pub raid_level: String,
    pub filesystem: String,
    pub size_bytes: u64,
    pub mountpoint: Option<String>,
    pub vg_name: String,
    pub lv_name: String,
    pub md_device: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_percent: u8,
    pub security: String,
    pub channel: Option<String>,
    pub frequency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiCredentials {
    pub password: Option<String>,
    pub identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuInfo {
    pub model: String,
    pub logical_cores: usize,
    pub physical_cores: Option<usize>,
    pub frequency_mhz: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThermalReading {
    pub name: String,
    pub temperature_c: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriverInfo {
    pub module: String,
    pub loaded: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuInfo {
    pub name: String,
    pub driver: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemHardware {
    pub cpu: CpuInfo,
    pub ram_total_bytes: u64,
    pub ram_available_bytes: u64,
    pub gpus: Vec<GpuInfo>,
    pub thermals: Vec<ThermalReading>,
    pub drivers: Vec<DriverInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeRequest {
    pub name: String,
    pub disks: Vec<String>,
    pub raid_level: String,
    pub fs_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confirmation {
    pub operation_id: String,
    pub operation: String,
    pub issued_at: String,
    pub expires_at: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: String,
    pub operation: String,
    pub resource: String,
    pub destructive: bool,
    pub confirmed: bool,
    pub success: bool,
    pub actor: String,
    pub timestamp: String,
    pub detail: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct HardwareConfig {
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default = "default_command_timeout")]
    pub command_timeout_seconds: u64,
    #[serde(default = "default_confirmation_ttl")]
    pub confirmation_ttl_seconds: u64,
    #[serde(default = "default_wifi_timeout")]
    pub wifi_timeout_seconds: u64,
    #[serde(default = "default_mount_root")]
    pub mount_root: String,
}
fn default_command_timeout() -> u64 { 60 }
fn default_confirmation_ttl() -> u64 { 300 }
fn default_wifi_timeout() -> u64 { 30 }
fn default_mount_root() -> String { "/mnt/tjspace".into() }

#[derive(Clone)]
pub struct HardwareManager {
    patch_db: PatchDb,
    config: HardwareConfig,
    confirmations: Arc<Mutex<HashMap<String, Confirmation>>>,
}

impl HardwareManager {
    pub fn new(patch_db: PatchDb, config: HardwareConfig) -> Self {
        Self { patch_db, config, confirmations: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn request_confirmation(&self, operation: &str, resource: &str) -> Result<Confirmation> {
        let operation_id = format!("hw-{}", Uuid::new_v4());
        let now = Utc::now();
        let expires = now + chrono::Duration::seconds(self.config.confirmation_ttl_seconds as i64);
        let c = Confirmation {
            operation_id: operation_id.clone(),
            operation: format!("{operation}:{resource}"),
            issued_at: now.to_rfc3339(),
            expires_at: expires.to_rfc3339(),
            confirmed: false,
        };
        self.confirmations.lock().map_err(|_| anyhow!("confirmation lock poisoned"))?
            .insert(operation_id.clone(), c.clone());
        self.audit("confirmation-issued", resource, true, true, true, &operation_id)?;
        Ok(c)
    }

    pub fn confirm_destructive_operation(&self, operation_id: &str) -> Result<()> {
        let mut guard = self.confirmations.lock().map_err(|_| anyhow!("confirmation lock poisoned"))?;
        let c = guard.get_mut(operation_id).ok_or_else(|| anyhow!("confirmation not found or already confirmed"))?;
        if c.expires_at < Utc::now().to_rfc3339() {
            guard.remove(operation_id);
            return Err(anyhow!("confirmation expired"));
        }
        if c.confirmed { return Err(anyhow!("confirmation already confirmed")); }
        c.confirmed = true;
        let resource = c.operation.clone();
        self.audit("confirmation-consumed", &resource, true, true, true, operation_id)?;
        Ok(())
    }

    fn require_confirmation(&self, operation_id: Option<&str>, operation: &str) -> Result<()> {
        let id = operation_id.ok_or_else(|| anyhow!("explicit confirmation required for {operation}"))?;
        let mut guard = self.confirmations.lock().map_err(|_| anyhow!("confirmation lock poisoned"))?;
        let c = guard.get(id).ok_or_else(|| anyhow!("invalid or already consumed confirmation"))?;
        if c.expires_at < Utc::now().to_rfc3339() || !c.confirmed || !c.operation.starts_with(operation) {
            return Err(anyhow!("confirmation invalid, expired, unconfirmed, or bound to another operation"));
        }
        guard.remove(id);
        Ok(())
    }

    fn audit(&self, operation: &str, resource: &str, destructive: bool, confirmed: bool, success: bool, detail: &str) -> Result<()> {
        let record = AuditRecord {
            id: Uuid::new_v4().to_string(),
            operation: operation.into(),
            resource: resource.into(),
            destructive,
            confirmed,
            success,
            actor: "tjsd".into(),
            timestamp: Utc::now().to_rfc3339(),
            detail: detail.into(),
        };
        self.patch_db.apply_patch(Patch {
            version: crate::patch_db::PATCH_VERSION,
            path: format!("hardware.audit.{}", record.id),
            op: PatchOp::Set,
            value: Some(serde_json::to_value(&record)?),
            actor: "tjsd".into(),
            authorization: "allow".into(),
        })?;
        Ok(())
    }

    pub async fn EnumerateDisks(&self) -> Result<Vec<Disk>> {
        let output = run_command("lsblk", &["-J", "-b", "-o", "NAME,TYPE,SIZE,MODEL,SERIAL,ROTA,FSTYPE,MOUNTPOINT"], self.config.command_timeout_seconds).await?;
        if !output.status.success() {
            return Err(anyhow!("lsblk failed: {}", output.stderr));
        }
        let root: Value = serde_json::from_str(&output.stdout)?;
        let mut disks = Vec::new();
        for d in root["blockdevices"].as_array().cloned().unwrap_or_default() {
            if d["type"].as_str() != Some("disk") { continue; }
            let name = d["name"].as_str().unwrap_or_default();
            let path = format!("/dev/{name}");
            let id = disk_id(&path);
            let health = self.GetDiskHealth(&id).await.unwrap_or_else(|_| DiskHealthSummary {
                status: "unknown".into(), ..Default::default()
            });
            disks.push(Disk {
                id, path,
                model: opt_string(&d["model"]),
                serial: opt_string(&d["serial"]),
                size_bytes: d["size"].as_u64().unwrap_or(0),
                rotational: d["rota"].as_bool().unwrap_or(false),
                filesystem: opt_string(&d["fstype"]),
                mountpoint: opt_string(&d["mountpoint"]),
                health,
            });
        }
        Ok(disks)
    }

    pub async fn GetDiskHealth(&self, disk_id: &str) -> Result<DiskHealthSummary> {
        let path = resolve_disk_id(disk_id)?;
        let output = run_command("smartctl", &["-Aj", &path], self.config.command_timeout_seconds).await?;
        let raw: Value = serde_json::from_str(&output.stdout).unwrap_or_else(|_| json!({"stderr": output.stderr}));
        let smart_available = raw["smart_support"]["available"].as_bool().unwrap_or(false);
        let status = if raw["smart_status"]["passed"].as_bool() == Some(true) { "healthy" }
            else if raw["smart_status"].get("passed").is_some() { "failed" } else { "unknown" };
        let temp = raw["temperature"]["current"].as_f64();
        let mut summary = DiskHealthSummary {
            status: status.into(),
            temperature_c: temp,
            power_on_hours: None,
            reallocated_sectors: None,
            pending_sectors: None,
            uncorrectable_sectors: None,
            smart_available,
        };
        if let Some(attrs) = raw["ata_smart_attributes"]["table"].as_array() {
            for a in attrs {
                let name = a["name"].as_str().unwrap_or_default();
                let val = a["raw"]["value"].as_u64();
                match name {
                    "Power_On_Hours" => summary.power_on_hours = val,
                    "Reallocated_Sector_Ct" => summary.reallocated_sectors = val,
                    "Current_Pending_Sector" => summary.pending_sectors = val,
                    "Offline_Uncorrectable" => summary.uncorrectable_sectors = val,
                    _ => {}
                }
            }
        }
        Ok(summary)
    }

    pub async fn GetSmartData(&self, disk_id: &str) -> Result<SmartData> {
        let path = resolve_disk_id(disk_id)?;
        let output = run_command("smartctl", &["-Aj", &path], self.config.command_timeout_seconds).await?;
        let raw: Value = serde_json::from_str(&output.stdout).unwrap_or_else(|_| json!({"stderr": output.stderr}));
        let summary = self.GetDiskHealth(disk_id).await?;
        let mut attributes = HashMap::new();
        if let Some(attrs) = raw["ata_smart_attributes"]["table"].as_array() {
            for a in attrs {
                if let Some(name) = a["name"].as_str() {
                    attributes.insert(name.into(), a.clone());
                }
            }
        }
        Ok(SmartData {
            disk_id: disk_id.into(),
            status: summary.status,
            temperature_c: summary.temperature_c,
            power_on_hours: summary.power_on_hours,
            reallocated_sectors: summary.reallocated_sectors,
            pending_sectors: summary.pending_sectors,
            uncorrectable_sectors: summary.uncorrectable_sectors,
            wear_percent: raw["nvme_smart_health_information_log"]["percentage_used"].as_f64(),
            attributes,
            raw,
        })
    }

    pub async fn CreateVolume(&self, name: &str, disks: &[String], raid_level: &str, fs_type: &str) -> Result<Volume> {
        self.create_volume_confirmed(name, disks, raid_level, fs_type, None).await
    }

    pub async fn CreateVolumeConfirmed(&self, name: &str, disks: &[String], raid_level: &str, fs_type: &str, confirmation_id: &str) -> Result<Volume> {
        self.create_volume_confirmed(name, disks, raid_level, fs_type, Some(confirmation_id)).await
    }

    async fn create_volume_confirmed(&self, name: &str, disks: &[String], raid_level: &str, fs_type: &str, confirmation_id: Option<&str>) -> Result<Volume> {
        validate_name(name)?;
        if disks.is_empty() { return Err(anyhow!("at least one disk is required")); }
        validate_fs(fs_type)?;
        let normalized = normalize_raid(raid_level)?;
        let resolved: Vec<String> = disks.iter().map(|d| resolve_disk_id(d)).collect::<Result<_>>()?;
        let op = format!("create-volume:{name}");
        if !self.config.dry_run {
            self.require_confirmation(confirmation_id, &op)?;
            for d in &resolved { ensure_unmounted(d)?; }
        }
        let id = format!("vol-{}", Uuid::new_v4());
        let vg = format!("tjsvg-{}", &id[4..12]);
        let lv = format!("tjlv-{}", &id[4..12]);
        let md = if normalized != "none" && normalized != "lvm" { Some(format!("/dev/md/{lv}")) } else { None };
        if !self.config.dry_run {
            if let Some(level) = md.as_ref() {
                let level_name = normalized.as_str();
                let md_name = level.trim_start_matches("/dev/md/");
                let md_path = format!("/dev/md/{md_name}"); let raid_devices = disks.len().to_string(); let mut args = vec!["--create", md_path.as_str(), "--level", level_name, "--raid-devices", raid_devices.as_str()];
                let owned: Vec<String> = resolved.clone();
                let refs: Vec<&str> = owned.iter().map(String::as_str).collect();
                args.extend(refs);
                run_checked("mdadm", &args, self.config.command_timeout_seconds).await?;
                run_checked("pvcreate", &["-ff", "-y", level], self.config.command_timeout_seconds).await?;
                run_checked("vgcreate", &[&vg, level], self.config.command_timeout_seconds).await?;
            } else {
                for d in &resolved { run_checked("pvcreate", &["-ff", "-y", d], self.config.command_timeout_seconds).await?; }
                let refs: Vec<&str> = resolved.iter().map(String::as_str).collect();
                let mut args = vec![vg.as_str()];
                args.extend(refs);
                run_checked("vgcreate", &args, self.config.command_timeout_seconds).await?;
            }
            run_checked("lvcreate", &["-n", &lv, "-l", "90%FREE", &vg], self.config.command_timeout_seconds).await?;
            let device = format!("/dev/{vg}/{lv}");
            run_checked("mkfs", &["-t", fs_type, &device], self.config.command_timeout_seconds).await?;
        }
        let device = format!("/dev/{vg}/{lv}");
        let size = if self.config.dry_run { 0 } else { block_size(&device).await.unwrap_or(0) };
        let volume = Volume {
            id: id.clone(), name: name.into(), device, disks: disks.to_vec(), raid_level: normalized,
            filesystem: fs_type.into(), size_bytes: size, mountpoint: None, vg_name: vg, lv_name: lv,
            md_device: md, created_at: Utc::now().to_rfc3339(),
        };
        self.patch_volume(&volume)?;
        self.audit("create-volume", &volume.id, true, confirmation_id.is_some() || self.config.dry_run, true, &volume.device)?;
        Ok(volume)
    }

    pub async fn ResizeVolume(&self, volume_id: &str, new_size: u64) -> Result<Volume> {
        self.resize_volume_confirmed(volume_id, new_size, None).await
    }

    pub async fn ResizeVolumeConfirmed(&self, volume_id: &str, new_size: u64, confirmation_id: &str) -> Result<Volume> {
        self.resize_volume_confirmed(volume_id, new_size, Some(confirmation_id)).await
    }

    async fn resize_volume_confirmed(&self, volume_id: &str, new_size: u64, confirmation_id: Option<&str>) -> Result<Volume> {
        let mut volume = self.get_volume(volume_id)?;
        if new_size == 0 { return Err(anyhow!("new size must be non-zero")); }
        if !self.config.dry_run { self.require_confirmation(confirmation_id, &format!("resize-volume:{volume_id}"))?; }
        let current = block_size(&volume.device).await.unwrap_or(volume.size_bytes);
        if new_size < current && volume.mountpoint.is_some() { return Err(anyhow!("shrinking a mounted filesystem is refused")); }
        let size_arg = format!("{new_size}B");
        if !self.config.dry_run {
            if new_size >= current {
                run_checked("lvextend", &["-L", &size_arg, &volume.device], self.config.command_timeout_seconds).await?;
                grow_fs(&volume.filesystem, &volume.device).await?;
            } else {
                shrink_fs(&volume.filesystem, &volume.device, new_size).await?;
                run_checked("lvreduce", &["-L", &size_arg, "--yes", &volume.device], self.config.command_timeout_seconds).await?;
            }
        }
        volume.size_bytes = new_size;
        self.patch_volume(&volume)?;
        self.audit("resize-volume", volume_id, true, confirmation_id.is_some() || self.config.dry_run, true, &size_arg)?;
        Ok(volume)
    }

    pub async fn DeleteVolume(&self, volume_id: &str) -> Result<()> {
        self.delete_volume_confirmed(volume_id, None).await
    }

    pub async fn DeleteVolumeConfirmed(&self, volume_id: &str, confirmation_id: &str) -> Result<()> {
        self.delete_volume_confirmed(volume_id, Some(confirmation_id)).await
    }

    async fn delete_volume_confirmed(&self, volume_id: &str, confirmation_id: Option<&str>) -> Result<()> {
        let volume = self.get_volume(volume_id)?;
        if !self.config.dry_run { self.require_confirmation(confirmation_id, &format!("delete-volume:{volume_id}"))?; }
        if !self.config.dry_run {
            if volume.mountpoint.is_some() { return Err(anyhow!("unmount volume before deletion")); }
            run_checked("lvremove", &["-y", &volume.device], self.config.command_timeout_seconds).await?;
            run_checked("vgremove", &["-y", &volume.vg_name], self.config.command_timeout_seconds).await?;
            if let Some(md) = volume.md_device.as_deref() { run_checked("mdadm", &["--stop", md], self.config.command_timeout_seconds).await?; }
            for d in &volume.disks {
                let p = resolve_disk_id(d)?;
                let _ = run_command("pvremove", &["-ff", "-y", &p], self.config.command_timeout_seconds).await?;
            }
        }
        self.patch_db.apply_patch(Patch { version: crate::patch_db::PATCH_VERSION, path: format!("hardware.volumes.{volume_id}"), op: PatchOp::Delete, value: None, actor: "tjsd".into(), authorization: "allow".into() })?;
        self.audit("delete-volume", volume_id, true, confirmation_id.is_some() || self.config.dry_run, true, "deleted")?;
        Ok(())
    }

    pub async fn MountVolume(&self, volume_id: &str, mount_point: &str) -> Result<()> {
        let mut volume = self.get_volume(volume_id)?;
        validate_mountpoint(mount_point)?;
        if !self.config.dry_run {
            fs::create_dir_all(mount_point)?;
            run_checked("mount", &[&volume.device, mount_point], self.config.command_timeout_seconds).await?;
        }
        volume.mountpoint = Some(mount_point.into());
        self.patch_volume(&volume)?;
        Ok(())
    }

    pub async fn UnmountVolume(&self, volume_id: &str) -> Result<()> {
        let mut volume = self.get_volume(volume_id)?;
        let mount = volume.mountpoint.clone().ok_or_else(|| anyhow!("volume is not mounted"))?;
        if !self.config.dry_run { run_checked("umount", &[&mount], self.config.command_timeout_seconds).await?; }
        volume.mountpoint = None;
        self.patch_volume(&volume)?;
        Ok(())
    }

    pub async fn ScanWifiNetworks(&self) -> Result<Vec<WifiNetwork>> {
        let output = run_command("nmcli", &["-t", "-f", "SSID,SIGNAL,SECURITY,CHAN,FREQ", "device", "wifi", "list", "--rescan", "yes"], self.config.wifi_timeout_seconds).await?;
        if !output.status.success() { return Err(anyhow!("nmcli scan failed: {}", output.stderr)); }
        let mut out = Vec::new();
        for line in output.stdout.lines() {
            let fields = split_nmcli(line);
            if fields.len() < 5 || fields[0].is_empty() { continue; }
            out.push(WifiNetwork {
                ssid: fields[0].clone(),
                signal_percent: fields[1].parse().unwrap_or(0).min(100),
                security: fields[2].clone(),
                channel: Some(fields[3].clone()).filter(|s| !s.is_empty()),
                frequency: Some(fields[4].clone()).filter(|s| !s.is_empty()),
            });
        }
        Ok(out)
    }

    pub async fn ConfigureWifi(&self, ssid: &str, credentials: &WifiCredentials) -> Result<()> {
        if ssid.is_empty() || ssid.len() > 255 { return Err(anyhow!("invalid SSID")); }
        let mut args = vec!["device", "wifi", "connect", ssid];
        let password = credentials.password.as_deref();
        if let Some(p) = password { args.extend(["password", p]); }
        if let Some(identity) = credentials.identity.as_deref() { args.extend(["ifname", identity]); }
        let output = run_command("nmcli", &args, self.config.wifi_timeout_seconds).await?;
        if !output.status.success() { return Err(anyhow!("WiFi configuration failed: {}", output.stderr)); }
        self.audit("configure-wifi", ssid, false, true, true, "NetworkManager connection activated")?;
        Ok(())
    }

    pub async fn GetDrivers(&self) -> Result<Vec<DriverInfo>> {
        let mut drivers = Vec::new();
        if let Ok(text) = fs::read_to_string("/proc/modules") {
            for line in text.lines() {
                if let Some(module) = line.split_whitespace().next() {
                    drivers.push(DriverInfo { module: module.into(), loaded: true, description: None });
                }
            }
        }
        drivers.sort_by(|a,b| a.module.cmp(&b.module));
        Ok(drivers)
    }

    pub async fn LoadDriver(&self, module: &str) -> Result<()> {
        validate_module_name(module)?;
        if !self.config.dry_run {
            run_checked("modprobe", &[module], self.config.command_timeout_seconds).await?;
        }
        self.audit("load-driver", module, false, true, true, "modprobe")?;
        Ok(())
    }

    pub async fn UnloadDriver(&self, module: &str) -> Result<()> {
        self.unload_driver_confirmed(module, None).await
    }

    pub async fn UnloadDriverConfirmed(&self, module: &str, confirmation_id: &str) -> Result<()> {
        self.unload_driver_confirmed(module, Some(confirmation_id)).await
    }

    async fn unload_driver_confirmed(&self, module: &str, confirmation_id: Option<&str>) -> Result<()> {
        validate_module_name(module)?;
        if !self.config.dry_run {
            self.require_confirmation(confirmation_id, &format!("unload-driver:{module}"))?;
            run_checked("modprobe", &["-r", module], self.config.command_timeout_seconds).await?;
        }
        self.audit("unload-driver", module, true, confirmation_id.is_some() || self.config.dry_run, true, "modprobe -r")?;
        Ok(())
    }

    pub async fn GetSystemHardware(&self) -> Result<SystemHardware> {
        let cpu_text = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let model = cpu_text.lines().find_map(|l| l.strip_prefix("model name\t: ")).unwrap_or("unknown").into();
        let logical = cpu_text.lines().filter(|l| l.starts_with("processor\t:")).count();
        let frequency = cpu_text.lines().find_map(|l| l.strip_prefix("cpu MHz\t: ")).and_then(|v| v.parse::<f64>().ok()).map(|v| v as u64);
        let mut ram_total=0; let mut ram_avail=0;
        for line in fs::read_to_string("/proc/meminfo").unwrap_or_default().lines() {
            if let Some(v)=line.strip_prefix("MemTotal:").and_then(parse_kib) { ram_total=v; }
            if let Some(v)=line.strip_prefix("MemAvailable:").and_then(parse_kib) { ram_avail=v; }
        }
        let mut thermals=Vec::new();
        if let Ok(entries)=fs::read_dir("/sys/class/thermal") {
            for e in entries.flatten() {
                let p=e.path();
                if !p.file_name().unwrap_or_default().to_string_lossy().starts_with("thermal_zone") { continue; }
                let name=fs::read_to_string(p.join("type")).unwrap_or_else(|_| "unknown".into()).trim().into();
                let t=fs::read_to_string(p.join("temp")).ok().and_then(|v| v.trim().parse::<f64>().ok()).map(|v| v/1000.0);
                thermals.push(ThermalReading{name,temperature_c:t});
            }
        }
        let mut gpus=Vec::new();
        if let Ok(out)=Command::new("lspci").args(["-mm","-nnk"]).output() {
            let text=String::from_utf8_lossy(&out.stdout);
            let mut current: Option<GpuInfo>=None;
            for line in text.lines() {
                if line.contains("VGA compatible controller") || line.contains("3D controller") {
                    if let Some(g)=current.take(){gpus.push(g);}
                    let name=line.split("VGA compatible controller:").nth(1).or_else(||line.split("3D controller:").nth(1)).unwrap_or(line).trim().to_string();
                    current=Some(GpuInfo{name,driver:None});
                } else if line.trim_start().starts_with("Kernel driver in use:") {
                    if let Some(g)=current.as_mut(){g.driver=Some(line.split(':').nth(1).unwrap_or("").trim().into());}
                }
            }
            if let Some(g)=current{gpus.push(g);}
        }
        Ok(SystemHardware {
            cpu: CpuInfo { model, logical_cores: logical, physical_cores: None, frequency_mhz: frequency },
            ram_total_bytes: ram_total, ram_available_bytes: ram_avail, gpus, thermals,
            drivers: self.GetDrivers().await?,
        })
    }

    pub fn get_state_snapshot(&self) -> Result<crate::patch_db::Snapshot> { self.patch_db.get_snapshot() }

    fn patch_volume(&self, volume: &Volume) -> Result<()> {
        self.patch_db.apply_patch(Patch { version: crate::patch_db::PATCH_VERSION, path: format!("hardware.volumes.{}", volume.id), op: PatchOp::Set, value: Some(serde_json::to_value(volume)?), actor: "tjsd".into(), authorization: "allow".into() })?;
        Ok(())
    }

    fn get_volume(&self, id: &str) -> Result<Volume> {
        let snap=self.patch_db.get_snapshot()?;
        snap.state.get("hardware").and_then(|v|v.get("volumes")).and_then(|v|v.get(id))
            .cloned().ok_or_else(||anyhow!("volume not found: {id}")).and_then(|v|serde_json::from_value(v).context("invalid persisted volume metadata"))
    }
}

async fn run_checked(program: &str, args: &[&str], timeout_secs: u64) -> Result<String> {
    let out=run_command(program,args,timeout_secs).await?;
    if !out.status.success(){return Err(anyhow!("{program} failed: {}",out.stderr.trim()));}
    Ok(out.stdout)
}
struct CmdOut { status: std::process::ExitStatus, stdout:String, stderr:String }
async fn run_command(program:&str,args:&[&str],timeout_secs:u64)->Result<CmdOut>{
    let p=program.to_string(); let a=args.iter().map(|s|s.to_string()).collect::<Vec<_>>();
    let fut=tokio::task::spawn_blocking(move||Command::new(p).args(a).output());
    let out=timeout(Duration::from_secs(timeout_secs),fut).await.context("command timeout")??;
    Ok(CmdOut{status:out.status,stdout:String::from_utf8_lossy(&out.stdout).into(),stderr:String::from_utf8_lossy(&out.stderr).into()})
}
fn opt_string(v:&Value)->Option<String>{v.as_str().map(str::to_owned).filter(|s|!s.is_empty())}
fn disk_id(path:&str)->String{format!("disk-{}",blake3::hash(path.as_bytes()).to_hex())}
fn resolve_disk_id(id:&str)->Result<String>{
    if id.starts_with("/dev/"){return safe_dev(id);}
    if let Some(name)=id.strip_prefix("disk-"){
        if name.len()==64 && name.chars().all(|c|c.is_ascii_hexdigit()){
            let entries=fs::read_dir("/sys/block").context("read /sys/block")?;
            for e in entries.flatten(){let p=format!("/dev/{}",e.file_name().to_string_lossy());if disk_id(&p)==id{return safe_dev(&p)?;}}
        }
    }
    Err(anyhow!("unknown disk id: {id}"))
}
fn safe_dev(p:&str)->Result<String>{
    let pb=Path::new(p);
    if !pb.starts_with("/dev") || p.contains('\0') || p.contains("..") {return Err(anyhow!("unsafe device path"));}
    Ok(p.into())
}
fn ensure_unmounted(d:&str)->Result<()>{
    let out=std::process::Command::new("lsblk").args(["-n","-o","MOUNTPOINT",d]).output()?;
    if String::from_utf8_lossy(&out.stdout).lines().any(|l|!l.trim().is_empty()){return Err(anyhow!("disk has mounted filesystem: {d}"))}
    Ok(())
}
fn validate_module_name(v:&str)->Result<()> { if v.is_empty() || v.len()>128 || !v.chars().all(|c| c.is_ascii_alphanumeric() || c=='_' || c=='-'){ return Err(anyhow!("invalid module name")); } Ok(()) }
fn validate_name(v:&str)->Result<()>{
    if v.is_empty()||v.len()>64||!v.chars().all(|c|c.is_ascii_alphanumeric()||c=='-'||c=='_'){return Err(anyhow!("invalid volume name"))} Ok(())
}
fn validate_fs(v:&str)->Result<()>{
    match v {"ext4"|"xfs"|"btrfs"|"vfat"=>Ok(()),_=>Err(anyhow!("unsupported filesystem"))}
}
fn normalize_raid(v:&str)->Result<String>{
    let s=v.to_ascii_lowercase();
    match s.as_str() {"none"|"lvm"|"raid0"|"raid1"|"raid5"|"raid6"|"raid10"=>Ok(s),_=>Err(anyhow!("unsupported RAID level"))}
}
fn validate_mountpoint(v:&str)->Result<()>{
    let p=Path::new(v);
    if !p.is_absolute() || v.contains('\0') || v.contains(".."){return Err(anyhow!("invalid mount point"))} Ok(())
}
async fn block_size(device:&str)->Result<u64>{
    let out=run_command("blockdev",&["--getsize64",device],30).await?;
    Ok(out.stdout.trim().parse()?)
}
async fn grow_fs(fs_type:&str,device:&str)->Result<()>{
    match fs_type {
        "ext4"|"xfs"|"btrfs"=>{let cmd=match fs_type{"xfs"=>"xfs_growfs", "btrfs"=>"btrfs", _=>"resize2fs"}; let args=if fs_type=="btrfs"{vec![device,"resize","max"]}else{vec![device]}; run_checked(cmd,&args,60).await.map(|_|())}
        _=>Ok(())
    }
}
async fn shrink_fs(fs_type:&str,device:&str,new_size:u64)->Result<()>{
    match fs_type {
        "ext4" => { run_checked("e2fsck",&["-f",device],120).await?; let size_arg=format!("{new_size}B"); run_checked("resize2fs",&[device,&size_arg],120).await.map(|_|()) },
        "xfs"|"btrfs" => Err(anyhow!("filesystem shrink is not supported for {fs_type}")),
        _ => Err(anyhow!("filesystem shrink unsupported")),
    }
}
fn parse_kib(s:&str)->Option<u64>{s.split_whitespace().next()?.parse::<u64>().ok().map(|v|v*1024)}
fn split_nmcli(s:&str)->Vec<String>{
    let mut out=Vec::new(); let mut cur=String::new(); let mut esc=false;
    for c in s.chars(){if esc{cur.push(c);esc=false}else if c=='\\'{esc=true}else if c==':'{out.push(cur);cur=String::new()}else{cur.push(c)}} out.push(cur);out
}
