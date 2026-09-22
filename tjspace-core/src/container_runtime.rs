use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{process::Command, time::timeout};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_HEALTH_INTERVAL_MS: u64 = 5000;
const DEFAULT_HEALTH_COOLDOWN_MS: u64 = 1000;
const DEFAULT_BACKOFF_MAX_MS: u64 = 60000;
const MAX_SCRIPT_OUTPUT: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
    pub name: String,
    #[serde(default = "default_distribution")]
    pub distribution: String,
    #[serde(default = "default_release")]
    pub release: String,
    #[serde(default)]
    pub architecture: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}
fn default_distribution() -> String { "ubuntu".into() }
fn default_release() -> String { "24.04".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_max_bytes: Option<u64>,
    pub cpu_max_micros: Option<u64>,
    pub cpu_period_micros: Option<u64>,
    pub pids_max: Option<u64>,
    pub io_max: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HealthCheckSpec {
    Tcp { host: String, port: u16, #[serde(default)] policy: HealthPolicy },
    Http { url: String, #[serde(default = "default_http_success")] expected_status: u16, #[serde(default)] policy: HealthPolicy },
    Script { command: String, #[serde(default)] args: Vec<String>, #[serde(default)] policy: HealthPolicy },
}
fn default_http_success() -> u16 { 200 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthPolicy {
    #[serde(default = "default_interval")]
    pub interval_ms: u64,
    #[serde(default = "default_cooldown")]
    pub cooldown_ms: u64,
    #[serde(default = "default_backoff")]
    pub adaptive_backoff: bool,
    #[serde(default = "default_backoff_max")]
    pub max_backoff_ms: u64,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}
fn default_interval() -> u64 { DEFAULT_HEALTH_INTERVAL_MS }
fn default_cooldown() -> u64 { DEFAULT_HEALTH_COOLDOWN_MS }
fn default_backoff() -> bool { true }
fn default_backoff_max() -> u64 { DEFAULT_BACKOFF_MAX_MS }
fn default_timeout() -> u64 { 5000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    pub healthy: bool,
    pub latency_ms: u64,
    pub detail: String,
    pub next_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub host_path: PathBuf,
    pub container_path: PathBuf,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcontainerSpec {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Clone)]
pub struct ContainerRuntime {
    config: crate::RuntimeConfig,
    root: PathBuf,
    health_backoff: Arc<Mutex<HashMap<String, u64>>>,
}

impl ContainerRuntime {
    pub fn new(config: crate::RuntimeConfig) -> Self {
        Self {
            config,
            root: PathBuf::from("/var/lib/tjspace/containers"),
            health_backoff: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_root(config: crate::RuntimeConfig, root: impl Into<PathBuf>) -> Self {
        Self {
            config,
            root: root.into(),
            health_backoff: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_lxc_container(&self, package_id: &str, spec: &ContainerSpec) -> Result<String> {
        validate_id(package_id)?;
        validate_id(&spec.name)?;
        let id = format!("tjs-{}", package_id);
        let mut args = vec!["create", "-n", id.as_str()];
        let template = spec.template.as_deref().unwrap_or("download");
        let mut owned = Vec::<String>::new();
        if template == "download" {
            owned.extend([
                "--",
                "-d",
                &spec.distribution,
                "-r",
                &spec.release,
            ].into_iter().map(str::to_owned));
            if let Some(arch) = &spec.architecture {
                owned.push("-a".into());
                owned.push(arch.clone());
            }
        } else {
            owned.push("--".into());
            owned.push("-t".into());
            owned.push(template.into());
        }
        let owned_refs: Vec<&str> = owned.iter().map(String::as_str).collect();
        args.extend(owned_refs);
        self.run("lxc", &args).await?;
        let dir = self.root.join(package_id);
        if self.config.dry_run {
            tokio::fs::create_dir_all(&dir).await?;
        } else {
            tokio::fs::create_dir_all(&dir).await?;
        }
        self.write_runtime_config(&dir, spec).await?;
        for (key, value) in &spec.config {
            self.run("lxc", &["config", "set", id.as_str(), key.as_str(), value.as_str()]).await?;
        }
        Ok(id)
    }

    pub async fn start_lxc_container(&self, container_id: &str) -> Result<()> {
        validate_container_id(container_id)?;
        self.run("lxc", &["start", container_id]).await
    }

    pub async fn stop_lxc_container(&self, container_id: &str, graceful: bool) -> Result<()> {
        validate_container_id(container_id)?;
        if graceful {
            self.run("lxc", &["stop", container_id]).await
        } else {
            self.run("lxc", &["stop", container_id, "--force"]).await
        }
    }

    pub async fn init_container_runtime(&self, container_id: &str) -> Result<()> {
        validate_container_id(container_id)?;
        self.run("lxc", &[
            "exec", container_id, "--", "/usr/local/bin/tjs-container",
            "--root", "/opt/tjspace", "--package-entry", "/opt/tjspace/package/index.js",
        ]).await.map(|_| ())
    }

    pub async fn load_package_js(&self, container_id: &str, js_path: &str) -> Result<()> {
        validate_container_id(container_id)?;
        let source = Path::new(js_path);
        if !source.file_name().map(|x| x == "javascript.squashfs").unwrap_or(false) {
            bail!("jsPath must name javascript.squashfs");
        }
        if !self.config.dry_run && !source.exists() {
            bail!("javascript.squashfs does not exist: {js_path}");
        }
        let mountpoint = "/opt/tjspace/package";
        self.run("lxc", &["config", "device", "add", container_id, "tjs-javascript", "disk",
            &format!("source={js_path}"), "path=/opt/tjspace/package.squashfs", "readonly=true"]).await?;
        self.run("lxc", &["exec", container_id, "--", "mkdir", "-p", mountpoint]).await?;
        self.run("lxc", &[
            "exec", container_id, "--", "mount", "-o", "loop,ro", "/opt/tjspace/package.squashfs", mountpoint,
        ]).await?;
        if !self.config.dry_run {
            self.run("lxc", &[
                "exec", container_id, "--", "test", "-f", "/opt/tjspace/package/index.js",
            ]).await?;
        }
        Ok(())
    }

    pub async fn start_subcontainer(&self, container_id: &str, sub_name: &str, spec: &SubcontainerSpec) -> Result<()> {
        validate_container_id(container_id)?;
        validate_id(sub_name)?;
        let pidfile = format!("/run/tjspace-sub-{sub_name}.pid");
        let mut env_prefix = String::new();
        for (key, value) in &spec.env {
            validate_env_key(key)?;
            env_prefix.push_str(" ");
            env_prefix.push_str(key);
            env_prefix.push('=');
            env_prefix.push_str(&shell_quote(value));
        }
        let mut command = format!("env{env_prefix} {} ", shell_quote(&spec.command));
        command.push_str(&spec.args.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" "));
        let shell = format!("({command}) >/dev/null 2>&1 & echo $! > {}", shell_quote(&pidfile));
        self.run("lxc", &["exec", container_id, "--", "sh", "-c", &shell]).await?;
        Ok(())
    }

    pub async fn attach_to_subcontainer(&self, container_id: &str, sub_name: &str) -> Result<()> {
        validate_container_id(container_id)?;
        validate_id(sub_name)?;
        let pidfile = format!("/run/tjspace-sub-{sub_name}.pid");
        self.run_interactive("lxc", &[
            "exec", container_id, "--", "sh", "-c",
            &format!("test -s {0} && exec nsenter -t $(cat {0}) -m -u -i -n -p -w /bin/sh", shell_quote(&pidfile)),
        ]).await
    }

    pub async fn poll_health_check(&self, container_id: &str, check_spec: &HealthCheckSpec) -> Result<HealthResult> {
        validate_container_id(container_id)?;
        let key = format!("{}:{:?}", container_id, check_spec);
        let policy = match check_spec {
            HealthCheckSpec::Tcp { policy, .. } | HealthCheckSpec::Http { policy, .. } | HealthCheckSpec::Script { policy, .. } => policy,
        };
        let delay = self.health_backoff.lock().ok().and_then(|m| m.get(&key).copied()).unwrap_or(policy.interval_ms);
        let start = std::time::Instant::now();
        let result = match check_spec {
            HealthCheckSpec::Tcp { host, port, .. } => self.health_tcp(host, *port, policy.timeout_ms).await,
            HealthCheckSpec::Http { url, expected_status, .. } => self.health_http(url, *expected_status, policy.timeout_ms).await,
            HealthCheckSpec::Script { command, args, .. } => self.health_script(command, args, policy.timeout_ms).await,
        };
        let latency_ms = start.elapsed().as_millis() as u64;
        let healthy = result.is_ok();
        let detail = result.err().map(|e| e.to_string()).unwrap_or_else(|| "ok".into());
        let next_delay_ms = if healthy {
            policy.cooldown_ms.max(policy.interval_ms)
        } else if policy.adaptive_backoff {
            let next = delay.saturating_mul(2).min(policy.max_backoff_ms.max(policy.interval_ms));
            if let Ok(mut m) = self.health_backoff.lock() { m.insert(key, next); }
            next
        } else {
            policy.interval_ms
        };
        if healthy {
            if let Ok(mut m) = self.health_backoff.lock() { m.remove(&key); }
        }
        Ok(HealthResult { healthy, latency_ms, detail, next_delay_ms })
    }

    pub async fn run_effect(&self, container_id: &str, effect_name: &str, args: Value) -> Result<()> {
        validate_container_id(container_id)?;
        validate_id(effect_name)?;
        let encoded = serde_json::to_string(&args)?;
        self.run("lxc", &[
            "exec", container_id, "--", "/usr/local/bin/tjs-container",
            "--root", "/opt/tjspace", "--effect", effect_name, "--args", &encoded,
        ]).await
    }

    pub async fn apply_resource_limits(&self, container_id: &str, limits: &ResourceLimits) -> Result<()> {
        validate_container_id(container_id)?;
        let cg = PathBuf::from("/sys/fs/cgroup").join(container_id);
        self.write_cgroup(&cg, "memory.max", limits.memory_max_bytes.map(|v| v.to_string())).await?;
        match (limits.cpu_max_micros, limits.cpu_period_micros) {
            (Some(quota), Some(period)) => self.write_cgroup(&cg, "cpu.max", Some(format!("{quota} {period}"))).await?,
            (Some(quota), None) => self.write_cgroup(&cg, "cpu.max", Some(format!("{quota} 100000"))).await?,
            _ => {}
        }
        self.write_cgroup(&cg, "pids.max", limits.pids_max.map(|v| v.to_string())).await?;
        self.write_cgroup(&cg, "io.max", limits.io_max.clone()).await?;
        Ok(())
    }

    pub async fn mount_volume(&self, container_id: &str, volume_id: &str) -> Result<()> {
        validate_container_id(container_id)?;
        validate_id(volume_id)?;
        let volume = self.root.join("volumes").join(volume_id);
        let target = format!("/opt/tjspace/volumes/{volume_id}");
        self.run("lxc", &["exec", container_id, "--", "mkdir", "-p", &target]).await?;
        self.run("lxc", &[
            "config", "device", "add", container_id, volume_id, "disk",
            &format!("source={}", volume.display()), &format!("path={target}"),
        ]).await
    }

    pub async fn unmount_volume(&self, container_id: &str, volume_id: &str) -> Result<()> {
        validate_container_id(container_id)?;
        validate_id(volume_id)?;
        self.run("lxc", &["config", "device", "remove", container_id, volume_id]).await
    }

    #[allow(non_snake_case)] pub async fn CreateLxcContainer(&self, package_id: &str, spec: &ContainerSpec) -> Result<String> { self.create_lxc_container(package_id, spec).await }
    #[allow(non_snake_case)] pub async fn StartLxcContainer(&self, id: &str) -> Result<()> { self.start_lxc_container(id).await }
    #[allow(non_snake_case)] pub async fn StopLxcContainer(&self, id: &str, graceful: bool) -> Result<()> { self.stop_lxc_container(id, graceful).await }
    #[allow(non_snake_case)] pub async fn InitContainerRuntime(&self, id: &str) -> Result<()> { self.init_container_runtime(id).await }
    #[allow(non_snake_case)] pub async fn LoadPackageJs(&self, id: &str, path: &str) -> Result<()> { self.load_package_js(id, path).await }
    #[allow(non_snake_case)] pub async fn StartSubcontainer(&self, id: &str, name: &str, spec: &SubcontainerSpec) -> Result<()> { self.start_subcontainer(id, name, spec).await }
    #[allow(non_snake_case)] pub async fn AttachToSubcontainer(&self, id: &str, name: &str) -> Result<()> { self.attach_to_subcontainer(id, name).await }
    #[allow(non_snake_case)] pub async fn PollHealthCheck(&self, id: &str, check: &HealthCheckSpec) -> Result<HealthResult> { self.poll_health_check(id, check).await }
    #[allow(non_snake_case)] pub async fn RunEffect(&self, id: &str, name: &str, args: Value) -> Result<()> { self.run_effect(id, name, args).await }
    #[allow(non_snake_case)] pub async fn ApplyResourceLimits(&self, id: &str, limits: &ResourceLimits) -> Result<()> { self.apply_resource_limits(id, limits).await }
    #[allow(non_snake_case)] pub async fn MountVolume(&self, id: &str, volume: &str) -> Result<()> { self.mount_volume(id, volume).await }
    #[allow(non_snake_case)] pub async fn UnmountVolume(&self, id: &str, volume: &str) -> Result<()> { self.unmount_volume(id, volume).await }

    async fn write_runtime_config(&self, dir: &Path, spec: &ContainerSpec) -> Result<()> {
        tokio::fs::write(dir.join("runtime.json"), serde_json::to_vec_pretty(spec)?).await?;
        Ok(())
    }

    async fn run(&self, program: &str, args: &[&str]) -> Result<()> {
        if self.config.dry_run { return Ok(()); }
        let output = timeout(
            Duration::from_secs(self.config.command_timeout_seconds.max(1)),
            Command::new(program).args(args).output()
        ).await.context("runtime command timeout")??;
        if !output.status.success() {
            bail!("{program} {:?} failed: {}", args, String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    async fn run_interactive(&self, program: &str, args: &[&str]) -> Result<()> {
        if self.config.dry_run { return Ok(()); }
        let status = Command::new(program).args(args).status().await?;
        if !status.success() { bail!("interactive command failed with {status}"); }
        Ok(())
    }

    async fn write_cgroup(&self, root: &Path, file: &str, value: Option<String>) -> Result<()> {
        if let Some(value) = value {
            if self.config.dry_run { return Ok(()); }
            let path = root.join(file);
            if !path.exists() { bail!("cgroup v2 control missing: {}", path.display()); }
            tokio::fs::write(path, value).await?;
        }
        Ok(())
    }

    async fn health_tcp(&self, host: &str, port: u16, timeout_ms: u64) -> Result<()> {
        let fut = tokio::net::TcpStream::connect((host, port));
        timeout(Duration::from_millis(timeout_ms.max(1)), fut).await??;
        Ok(())
    }

    async fn health_http(&self, url: &str, expected: u16, timeout_ms: u64) -> Result<()> {
        let client = reqwest::Client::builder().timeout(Duration::from_millis(timeout_ms.max(1))).build()?;
        let response = client.get(url).send().await?;
        if response.status().as_u16() != expected {
            bail!("HTTP status {} != {}", response.status(), expected);
        }
        Ok(())
    }

    async fn health_script(&self, command: &str, args: &[String], timeout_ms: u64) -> Result<()> {
        let output = timeout(
            Duration::from_millis(timeout_ms.max(1)),
            Command::new(command).args(args).output()
        ).await??;
        if output.stdout.len() > MAX_SCRIPT_OUTPUT || output.stderr.len() > MAX_SCRIPT_OUTPUT {
            bail!("health script output exceeded limit");
        }
        if !output.status.success() {
            bail!("health script exited with {}", output.status);
        }
        Ok(())
    }
}

impl Default for HealthPolicy {
    fn default() -> Self {
        Self {
            interval_ms: DEFAULT_HEALTH_INTERVAL_MS,
            cooldown_ms: DEFAULT_HEALTH_COOLDOWN_MS,
            adaptive_backoff: true,
            max_backoff_ms: DEFAULT_BACKOFF_MAX_MS,
            timeout_ms: DEFAULT_TIMEOUT_SECS * 1000 / 6,
        }
    }
}

fn validate_env_key(value: &str) -> Result<()> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        bail!("invalid environment variable name");
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > 128 || !value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.') {
        bail!("invalid identifier");
    }
    Ok(())
}
fn validate_container_id(value: &str) -> Result<()> {
    validate_id(value)?;
    if !value.starts_with("tjs-") { bail!("container id must start with tjs-"); }
    Ok(())
}
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn dry_run_container_lifecycle() {
        let dir = tempdir().unwrap();
        let mut cfg = crate::RuntimeConfig::default();
        cfg.dry_run = true;
        let rt = ContainerRuntime::with_root(cfg, dir.path());
        let spec = ContainerSpec { name: "demo".into(), distribution: "ubuntu".into(), release: "24.04".into(), architecture: None, template: None, config: HashMap::new(), env: HashMap::new() };
        let id = rt.create_lxc_container("demo", &spec).await.unwrap();
        assert_eq!(id, "tjs-demo");
        rt.start_lxc_container(&id).await.unwrap();
        rt.init_container_runtime(&id).await.unwrap();
    }

    #[tokio::test]
    async fn invalid_ids_are_rejected() {
        let mut cfg = crate::RuntimeConfig::default();
        cfg.dry_run = true;
        let rt = ContainerRuntime::with_root(cfg, "/tmp/tjspace-test");
        assert!(rt.start_lxc_container("../escape").await.is_err());
        assert!(rt.start_lxc_container("bad").await.is_err());
    }

    #[tokio::test]
    async fn adaptive_health_backoff_is_bounded() {
        let mut cfg = crate::RuntimeConfig::default();
        cfg.dry_run = true;
        let rt = ContainerRuntime::with_root(cfg, "/tmp/tjspace-test");
        let check = HealthCheckSpec::Tcp { host: "127.0.0.1".into(), port: 1, policy: HealthPolicy::default() };
        let result = rt.poll_health_check("tjs-demo", &check).await.unwrap();
        assert!(!result.healthy);
        assert!(result.next_delay_ms <= DEFAULT_BACKOFF_MAX_MS);
    }

    #[tokio::test]
    async fn resource_limit_paths_are_safe() {
        let dir = tempdir().unwrap();
        let mut cfg = crate::RuntimeConfig::default();
        cfg.dry_run = true;
        let rt = ContainerRuntime::with_root(cfg, dir.path());
        rt.apply_resource_limits("tjs-demo", &ResourceLimits { memory_max_bytes: Some(1024), cpu_max_micros: Some(10000), cpu_period_micros: Some(100000), pids_max: Some(64), io_max: None }).await.unwrap();
    }

    #[test]
    fn identifier_validation_blocks_shell_metacharacters() {
        assert!(validate_id("a;b").is_err());
        assert!(validate_id("a b").is_err());
        assert!(validate_id("a/../b").is_err());
    }
}
