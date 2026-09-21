use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Utc;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
    sync::{Mutex as AsyncMutex, Semaphore},
};
use uuid::Uuid;

const MAX_FRAME: usize = 4 * 1024 * 1024;
const DEFAULT_IN_FLIGHT: usize = 32;
const COOKIE_VERSION: u32 = 1;
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(default = "default_auth_state_path")]
    pub auth_state_path: String,
    #[serde(default = "default_max_in_flight")]
    pub max_in_flight: usize,
    #[serde(default)]
    pub whitelist: HashSet<String>,
    #[serde(default)]
    pub method_scopes: HashMap<String, Vec<String>>,
    #[serde(default = "default_nonce_ttl")]
    pub nonce_ttl_seconds: i64,
    #[serde(default = "default_clock_skew")]
    pub clock_skew_seconds: i64,
}
fn default_auth_state_path() -> String { "tjspace-rpc-auth.db".into() }
fn default_max_in_flight() -> usize { DEFAULT_IN_FLIGHT }
fn default_nonce_ttl() -> i64 { 300 }
fn default_clock_skew() -> i64 { 60 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEnvelope {
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub auth_cookie: Option<String>,
    #[serde(default, alias = "X-TJS-Auth-Sig")]
    pub auth_sig: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub trace_id: Option<String>,
}
fn default_jsonrpc() -> String { "2.0".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCookieClaims {
    pub version: u32,
    pub device_id: String,
    pub scopes: Vec<String>,
    pub issued_at: i64,
    pub expires_at: i64,
    pub jti: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthDecision {
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub device_id: String,
    pub method: String,
    pub decision: String,
    pub reason: String,
    pub nonce: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Clone)]
pub struct RpcTransport {
    core: Arc<crate::Core>,
    db: Arc<Mutex<Connection>>,
    cookie_key: Arc<Vec<u8>>,
    config: TransportConfig,
    in_flight: Arc<Semaphore>,
    write_lock: Arc<AsyncMutex<()>>,
}

impl RpcTransport {
    pub fn new(core: Arc<crate::Core>, config: TransportConfig) -> Result<Self> {
        let db_path = Path::new(&config.auth_state_path);
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS auth_meta (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL
            );
            CREATE TABLE IF NOT EXISTS rpc_nonces (
                nonce TEXT PRIMARY KEY,
                seen_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS rpc_audit (
                seq INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL UNIQUE,
                timestamp INTEGER NOT NULL,
                device_id TEXT NOT NULL,
                method TEXT NOT NULL,
                decision TEXT NOT NULL,
                reason TEXT NOT NULL,
                nonce TEXT,
                trace_id TEXT
            );",
        )?;

        let key: Vec<u8> = match conn
            .query_row("SELECT value FROM auth_meta WHERE key='cookie_hmac_key'", [], |r| r.get(0))
            .optional()?
        {
            Some(v) if v.len() == 32 => v,
            _ => {
                let mut generated = [0u8; 32];
                OsRng.fill_bytes(&mut generated);
                conn.execute(
                    "INSERT INTO auth_meta(key,value) VALUES('cookie_hmac_key',?1)
                     ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    params![generated.to_vec()],
                )?;
                generated.to_vec()
            }
        };

        Ok(Self {
            core,
            db: Arc::new(Mutex::new(conn)),
            cookie_key: Arc::new(key),
            in_flight: Arc::new(Semaphore::new(config.max_in_flight.max(1))),
            config,
            write_lock: Arc::new(AsyncMutex::new(())),
        })
    }

    #[cfg(unix)]
    pub async fn listen_on_unix_socket(&self, path: &str) -> Result<()> {
        if Path::new(path).exists() {
            std::fs::remove_file(path).with_context(|| format!("remove stale socket {path}"))?;
        }
        let listener = tokio::net::UnixListener::bind(path)?;
        let this = self.clone();
        loop {
            let (stream, _) = listener.accept().await?;
            let transport = this.clone();
            tokio::spawn(async move {
                if let Err(err) = transport.handle_connection(stream).await {
                    tracing::warn!(trace_id=%Uuid::new_v4(), service_id="tjsd", error=%err, "rpc_unix_connection_failed");
                }
            });
        }
    }

    #[cfg(not(unix))]
    pub async fn listen_on_unix_socket(&self, _path: &str) -> Result<()> {
        bail!("Unix sockets are not supported on this platform");
    }

    pub async fn listen_on_tcp(&self, addr: &str, port: u16) -> Result<()> {
        let listener = TcpListener::bind((addr, port)).await?;
        let this = self.clone();
        loop {
            let (stream, peer) = listener.accept().await?;
            let transport = this.clone();
            tokio::spawn(async move {
                if let Err(err) = transport.handle_connection(stream).await {
                    tracing::warn!(
                        trace_id=%Uuid::new_v4(),
                        service_id="tjsd",
                        peer=%peer,
                        error=%err,
                        "rpc_tcp_connection_failed"
                    );
                }
            });
        }
    }

    #[allow(non_snake_case)]
    pub async fn ListenOnUnixSocket(&self, path: &str) -> Result<()> {
        self.listen_on_unix_socket(path).await
    }

    #[allow(non_snake_case)]
    pub async fn ListenOnTcp(&self, addr: &str, port: u16) -> Result<()> {
        self.listen_on_tcp(addr, port).await
    }

    pub async fn handle_connection<S>(&self, mut stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send,
    {
        loop {
            let mut header = [0u8; 4];
            match stream.read_exact(&mut header).await {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
                Err(err) => return Err(err.into()),
            }
            let len = u32::from_be_bytes(header) as usize;
            if len == 0 || len > MAX_FRAME {
                bail!("invalid RPC frame length {len}");
            }

            let mut body = vec![0u8; len];
            stream.read_exact(&mut body).await?;
            let envelope: RpcEnvelope = serde_json::from_slice(&body)
                .map_err(|e| anyhow!("invalid JSON-RPC frame: {e}"))?;

            let permit = self.in_flight.clone().acquire_owned().await?;
            let response = self.dispatch(envelope).await;
            drop(permit);

            let bytes = serde_json::to_vec(&response)?;
            if bytes.len() > MAX_FRAME {
                bail!("RPC response exceeds maximum frame size");
            }
            let _guard = self.write_lock.lock().await;
            stream.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
            stream.write_all(&bytes).await?;
            stream.flush().await?;
        }
    }

    #[allow(non_snake_case)]
    pub async fn HandleConnection<S>(&self, stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        self.handle_connection(stream).await
    }

    async fn dispatch(&self, req: RpcEnvelope) -> JsonRpcResponse {
        let trace_id = req.trace_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        let device_id = req.device_id.clone().unwrap_or_default();

        let auth_result = self.authenticate(&req).await;
        let (scopes, authenticated) = match auth_result {
            Ok(scopes) => (scopes, true),
            Err(reason) => {
                let _ = self.EmitAuditEvent(AuditEvent {
                    event_id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now().timestamp(),
                    device_id: device_id.clone(),
                    method: req.method.clone(),
                    decision: "deny".into(),
                    reason: reason.clone(),
                    nonce: req.nonce.clone(),
                    trace_id: Some(trace_id.clone()),
                });
                if !self.config.whitelist.contains(&req.method) {
                    return Self::error(req.id, -32001, reason);
                }
                (Vec::new(), false)
            }
        };

        if authenticated {
            let decision = self.AuthorizeMethod(&device_id, &req.method, &scopes);
            let _ = self.EmitAuditEvent(AuditEvent {
                event_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().timestamp(),
                device_id: device_id.clone(),
                method: req.method.clone(),
                decision: if decision.allowed { "allow" } else { "deny" }.into(),
                reason: decision.reason.clone(),
                nonce: req.nonce.clone(),
                trace_id: Some(trace_id.clone()),
            });
            if !decision.allowed {
                return Self::error(req.id, -32003, decision.reason);
            }
        }

        let mut core_req = crate::RpcRequest {
            method: req.method.clone(),
            params: req.params.clone(),
            trace_id: Some(trace_id.clone()),
            auth_token: None,
        };
        if !self.core.config.auth_token.is_empty() {
            core_req.auth_token = Some(self.core.config.auth_token.clone());
        }

        let out = self.core.handle_rpc_request(core_req).await;
        if out.ok {
            JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: out.result,
                error: None,
            }
        } else {
            JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32010,
                    message: out.error.map(|e| e.message).unwrap_or_else(|| "RPC failure".into()),
                }),
            }
        }
    }

    async fn authenticate(&self, req: &RpcEnvelope) -> Result<Vec<String>> {
        let device_id = req.device_id.as_deref().ok_or_else(|| anyhow!("device_id required"))?;
        let nonce = req.nonce.as_deref().ok_or_else(|| anyhow!("nonce required"))?;
        let timestamp = req.timestamp.ok_or_else(|| anyhow!("timestamp required"))?;
        let now = Utc::now().timestamp();
        if (now - timestamp).abs() > self.config.clock_skew_seconds {
            bail!("request timestamp outside allowed clock skew");
        }

        self.claim_nonce(nonce, now)?;
        if let Some(cookie) = req.auth_cookie.as_deref() {
            let claims = self.VerifyAuthCookie(cookie)?;
            if claims.device_id != device_id {
                bail!("auth cookie device mismatch");
            }
            if claims.expires_at < now {
                bail!("auth cookie expired");
            }
            return Ok(claims.scopes);
        }

        let _signature = req
            .auth_sig
            .as_deref()
            .or_else(|| req.headers.get("X-TJS-Auth-Sig").map(String::as_str))
            .or_else(|| req.headers.get("x-tjs-auth-sig").map(String::as_str))
            .ok_or_else(|| anyhow!("X-TJS-Auth-Sig required"))?;
        let public_key = self.load_device_public_key(device_id)?;
        self.ValidateAuthSignature(req, &public_key)?;
        Ok(req.scopes.clone())
    }

    fn claim_nonce(&self, nonce: &str, now: i64) -> Result<()> {
        if nonce.is_empty() || nonce.len() > 256 {
            bail!("invalid nonce");
        }
        let db = self.db.lock().map_err(|_| anyhow!("auth DB lock poisoned"))?;
        let cutoff = now - self.config.nonce_ttl_seconds.max(1);
        db.execute("DELETE FROM rpc_nonces WHERE seen_at < ?1", params![cutoff])?;
        let changed = db.execute(
            "INSERT INTO rpc_nonces(nonce,seen_at) VALUES(?1,?2) ON CONFLICT(nonce) DO NOTHING",
            params![nonce, now],
        )?;
        if changed != 1 {
            bail!("replayed nonce");
        }
        Ok(())
    }

    fn load_device_public_key(&self, device_id: &str) -> Result<Vec<u8>> {
        let key = format!("device_public_key:{device_id}");
        let db = self.db.lock().map_err(|_| anyhow!("auth DB lock poisoned"))?;
        db.query_row("SELECT value FROM auth_meta WHERE key=?1", params![key], |r| r.get(0))
            .optional()?
            .ok_or_else(|| anyhow!("no public key registered for device"))
    }

    pub fn register_device_public_key(&self, device_id: &str, public_key: &[u8]) -> Result<()> {
        if device_id.is_empty() || public_key.len() != 32 {
            bail!("device id or Ed25519 public key invalid");
        }
        let key = format!("device_public_key:{device_id}");
        let db = self.db.lock().map_err(|_| anyhow!("auth DB lock poisoned"))?;
        db.execute(
            "INSERT INTO auth_meta(key,value) VALUES(?1,?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, public_key.to_vec()],
        )?;
        Ok(())
    }

    pub fn validate_auth_signature(&self, req: &RpcEnvelope, public_key: &[u8]) -> Result<()> {
        if req.jsonrpc != "2.0" {
            bail!("unsupported JSON-RPC version");
        }
        let sig_text = req
            .auth_sig
            .as_deref()
            .or_else(|| req.headers.get("X-TJS-Auth-Sig").map(String::as_str))
            .or_else(|| req.headers.get("x-tjs-auth-sig").map(String::as_str))
            .ok_or_else(|| anyhow!("X-TJS-Auth-Sig missing"))?;
        let sig_bytes = decode_binary(sig_text)?;
        let signature = Signature::from_slice(&sig_bytes).map_err(|e| anyhow!("invalid signature: {e}"))?;
        let key_bytes: [u8; 32] = public_key.try_into().map_err(|_| anyhow!("Ed25519 public key must be 32 bytes"))?;
        let key = VerifyingKey::from_bytes(&key_bytes).map_err(|e| anyhow!("invalid Ed25519 public key: {e}"))?;
        let message = signing_bytes(req)?;
        key.verify(&message, &signature).map_err(|_| anyhow!("invalid X-TJS-Auth-Sig"))?;
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn ValidateAuthSignature(&self, req: &RpcEnvelope, public_key: &[u8]) -> Result<()> {
        self.validate_auth_signature(req, public_key)
    }

    pub fn issue_auth_cookie(&self, device_id: &str, scopes: Vec<String>, ttl: Duration) -> Result<String> {
        if device_id.is_empty() {
            bail!("device id required");
        }
        let now = Utc::now().timestamp();
        let mut random = [0u8; 16];
        OsRng.fill_bytes(&mut random);
        let claims = AuthCookieClaims {
            version: COOKIE_VERSION,
            device_id: device_id.into(),
            scopes,
            issued_at: now,
            expires_at: now + ttl.as_secs().min(i64::MAX as u64) as i64,
            jti: URL_SAFE_NO_PAD.encode(random),
        };
        let payload = serde_json::to_vec(&claims)?;
        let encoded = URL_SAFE_NO_PAD.encode(&payload);
        let mut mac = HmacSha256::new_from_slice(&self.cookie_key)
            .map_err(|_| anyhow!("invalid cookie key"))?;
        mac.update(encoded.as_bytes());
        let tag = mac.finalize().into_bytes();
        Ok(format!("{}.{}", encoded, URL_SAFE_NO_PAD.encode(tag)))
    }

    #[allow(non_snake_case)]
    pub fn IssueAuthCookie(&self, device_id: &str, scopes: Vec<String>, ttl: Duration) -> Result<String> {
        self.issue_auth_cookie(device_id, scopes, ttl)
    }

    pub fn verify_auth_cookie(&self, cookie: &str) -> Result<AuthCookieClaims> {
        let (payload, tag) = cookie.split_once('.').ok_or_else(|| anyhow!("malformed auth cookie"))?;
        let tag = URL_SAFE_NO_PAD.decode(tag).map_err(|_| anyhow!("invalid cookie MAC encoding"))?;
        let mut mac = HmacSha256::new_from_slice(&self.cookie_key)
            .map_err(|_| anyhow!("invalid cookie key"))?;
        mac.update(payload.as_bytes());
        mac.verify_slice(&tag).map_err(|_| anyhow!("invalid auth cookie"))?;
        let payload = URL_SAFE_NO_PAD.decode(payload).map_err(|_| anyhow!("invalid cookie payload"))?;
        let claims: AuthCookieClaims = serde_json::from_slice(&payload)?;
        let now = Utc::now().timestamp();
        if claims.version != COOKIE_VERSION || claims.expires_at <= now || claims.issued_at > now + self.config.clock_skew_seconds {
            bail!("expired or invalid auth cookie");
        }
        Ok(claims)
    }

    #[allow(non_snake_case)]
    pub fn VerifyAuthCookie(&self, cookie: &str) -> Result<AuthCookieClaims> {
        self.verify_auth_cookie(cookie)
    }

    pub fn authorize_method(&self, device_id: &str, method: &str, scopes: &[String]) -> AuthDecision {
        if self.config.whitelist.contains(method) {
            return AuthDecision { allowed: true, reason: "method explicitly whitelisted".into() };
        }
        let required = match self.config.method_scopes.get(method) {
            Some(v) => v,
            None => return AuthDecision { allowed: true, reason: "authenticated device; method has no additional scope requirement".into() },
        };
        let have: HashSet<&str> = scopes.iter().map(String::as_str).collect();
        if required.iter().all(|scope| have.contains(scope.as_str())) {
            AuthDecision { allowed: true, reason: format!("required scopes satisfied for {device_id}") }
        } else {
            AuthDecision { allowed: false, reason: format!("missing required scope for {method}") }
        }
    }

    #[allow(non_snake_case)]
    pub fn AuthorizeMethod(&self, device_id: &str, method: &str, scopes: &[String]) -> AuthDecision {
        self.authorize_method(device_id, method, scopes)
    }

    pub fn emit_audit_event(&self, event: AuditEvent) -> Result<()> {
        let db = self.db.lock().map_err(|_| anyhow!("auth DB lock poisoned"))?;
        db.execute(
            "INSERT INTO rpc_audit(event_id,timestamp,device_id,method,decision,reason,nonce,trace_id)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                event.event_id,
                event.timestamp,
                event.device_id,
                event.method,
                event.decision,
                event.reason,
                event.nonce,
                event.trace_id
            ],
        )?;
        Ok(())
    }

    #[allow(non_snake_case)]
    pub fn EmitAuditEvent(&self, event: AuditEvent) -> Result<()> {
        self.emit_audit_event(event)
    }

    fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message: message.into() }),
        }
    }
}

fn signing_bytes(req: &RpcEnvelope) -> Result<Vec<u8>> {
    let params = serde_json::to_string(&req.params)?;
    Ok(format!(
        "TJS-RPC-SIG-V1\n{}\n{}\n{}\n{}\n{}\n{}",
        req.jsonrpc,
        req.method,
        req.device_id.as_deref().unwrap_or(""),
        req.nonce.as_deref().unwrap_or(""),
        req.timestamp.unwrap_or_default(),
        params
    )
    .into_bytes())
}

fn decode_binary(value: &str) -> Result<Vec<u8>> {
    if let Ok(bytes) = URL_SAFE_NO_PAD.decode(value) {
        return Ok(bytes);
    }
    hex::decode(value).map_err(|_| anyhow!("signature must be base64url or hex"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use tempfile::tempdir;
    use sqlx::postgres::PgPoolOptions;
    use tokio::io::duplex;

    fn test_transport() -> RpcTransport {
        let dir = tempdir().unwrap();
        let path = dir.path().join("auth.db");
        let patch_path = dir.path().join("state.db");
        let core = Arc::new(crate::Core {
            config: crate::Config {
                database_url: "postgres://invalid/test".into(),
                bind: "127.0.0.1:0".into(),
                auth_token: String::new(),
                runtime: crate::RuntimeConfig::default(),
                node_id: "test-node".into(),
                patch_db_path: patch_path.to_string_lossy().into_owned(),
            },
            db: PgPoolOptions::new().connect_lazy("postgres://invalid/test").unwrap(),
            patch_db: crate::patch_db::PatchDb::open_database(patch_path.to_str().unwrap()).unwrap(),
            handlers: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        });
        let mut config = TransportConfig::default();
        config.auth_state_path = path.to_string_lossy().into_owned();
        RpcTransport::new(core, config).unwrap()
    }

    #[tokio::test]
    async fn signature_and_replay_are_enforced() {
        let transport = test_transport();
        let signing = SigningKey::generate(&mut OsRng);
        transport.register_device_public_key("dev-1", &signing.verifying_key().to_bytes()).unwrap();
        let mut req = RpcEnvelope {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(1)),
            method: "GetSystemState".into(),
            params: serde_json::json!({}),
            device_id: Some("dev-1".into()),
            nonce: Some(Uuid::new_v4().to_string()),
            timestamp: Some(Utc::now().timestamp()),
            scopes: vec!["read".into()],
            auth_cookie: None,
            auth_sig: None,
            headers: HashMap::new(),
            trace_id: None,
        };
        let bytes = signing.sign(&signing_bytes(&req).unwrap()).to_bytes();
        req.auth_sig = Some(URL_SAFE_NO_PAD.encode(bytes));
        assert!(transport.validate_auth_signature(&req, &signing.verifying_key().to_bytes()).is_ok());
        transport.claim_nonce(req.nonce.as_deref().unwrap(), Utc::now().timestamp()).unwrap();
        assert!(transport.claim_nonce(req.nonce.as_deref().unwrap(), Utc::now().timestamp()).is_err());
    }

    #[test]
    fn cookie_issue_and_verify() {
        let transport = test_transport();
        let cookie = transport.issue_auth_cookie("dev-1", vec!["read".into()], Duration::from_secs(60)).unwrap();
        let claims = transport.verify_auth_cookie(&cookie).unwrap();
        assert_eq!(claims.device_id, "dev-1");
        assert_eq!(claims.scopes, vec!["read"]);
    }

    #[test]
    fn authorization_is_scope_aware_and_auditable() {
        let transport = test_transport();
        let mut config = transport.config.clone();
        config.method_scopes.insert("Admin".into(), vec!["admin".into()]);
        let core = transport.core.clone();
        let t = RpcTransport::new(core, config).unwrap();
        let deny = t.authorize_method("dev-1", "Admin", &["read".into()]);
        assert!(!deny.allowed);
        let allow = t.authorize_method("dev-1", "Admin", &["admin".into()]);
        assert!(allow.allowed);
        t.emit_audit_event(AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp(),
            device_id: "dev-1".into(),
            method: "Admin".into(),
            decision: "deny".into(),
            reason: "test".into(),
            nonce: None,
            trace_id: None,
        }).unwrap();
    }

    #[tokio::test]
    async fn length_prefixed_transport_round_trip() {
        let transport = test_transport();
        let request = RpcEnvelope {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(1)),
            method: "GetSystemState".into(),
            params: serde_json::json!({}),
            device_id: None,
            nonce: None,
            timestamp: None,
            auth_cookie: None,
            auth_sig: None,
            headers: HashMap::new(),
            trace_id: None,
        };
        let bytes = serde_json::to_vec(&request).unwrap();
        let mut input = Vec::new();
        input.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        input.extend_from_slice(&bytes);
        let (mut a, b) = duplex(16 * 1024);
        tokio::spawn(async move {
            let mut writer = b;
            writer.write_all(&input).await.unwrap();
        });
        let _ = transport.handle_connection(&mut a).await;
    }

    #[test]
    fn audit_is_append_only_by_api() {
        let transport = test_transport();
        let event = AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp(),
            device_id: "dev".into(),
            method: "M".into(),
            decision: "allow".into(),
            reason: "r".into(),
            nonce: Some("n".into()),
            trace_id: None,
        };
        transport.emit_audit_event(event.clone()).unwrap();
        let db = transport.db.lock().unwrap();
        let count: i64 = db.query_row("SELECT COUNT(*) FROM rpc_audit", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            auth_state_path: default_auth_state_path(),
            max_in_flight: DEFAULT_IN_FLIGHT,
            whitelist: HashSet::new(),
            method_scopes: HashMap::new(),
            nonce_ttl_seconds: default_nonce_ttl(),
            clock_skew_seconds: default_clock_skew(),
        }
    }
}
