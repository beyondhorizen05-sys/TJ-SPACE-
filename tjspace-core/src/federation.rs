use anyhow::{anyhow, Result};
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Key, Nonce};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr}, path::{Path, PathBuf}, sync::{Arc, RwLock}, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::{net::UdpSocket, time::timeout};
use uuid::Uuid;
use x25519_dalek::{PublicKey as XPublicKey, StaticSecret};

const SERVICE: &str = "_tjspace._tcp.local";
const MDNS_ADDR: &str = "224.0.0.251:5353";
const PROTOCOL: &str = "tjs-federation-v1";
const MAX_ENVELOPE: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    #[serde(default = "default_state_path")] pub state_path: String,
    #[serde(default = "default_mdns_timeout_ms")] pub mdns_timeout_ms: u64,
    #[serde(default = "default_peer_port")] pub peer_port: u16,
}
fn default_state_path() -> String { "tjspace-federation-identity.json".into() }
fn default_mdns_timeout_ms() -> u64 { 1800 }
fn default_peer_port() -> u16 { 8090 }

impl Default for FederationConfig {
    fn default() -> Self { Self { state_path: default_state_path(), mdns_timeout_ms: default_mdns_timeout_ms(), peer_port: default_peer_port() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub peer_id: String,
    pub endpoint: String,
    pub discovered: bool,
    pub trusted: bool,
    pub ed25519_public_key_hex: Option<String>,
    pub x25519_public_key_hex: Option<String>,
    pub last_seen_unix: u64,
    pub registry_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustExchange {
    pub endpoint: String,
    pub peer_ed25519_public_key_hex: String,
    pub peer_x25519_public_key_hex: String,
    pub challenge_hex: String,
    pub peer_signature_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustOffer {
    pub peer_id: String,
    pub endpoint: String,
    pub ed25519_public_key_hex: String,
    pub x25519_public_key_hex: String,
    pub challenge_hex: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdentityFile {
    node_id: String,
    ed25519_secret_hex: String,
    x25519_secret_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerStore {
    peers: HashMap<String, Peer>,
}

#[derive(Clone)]
pub struct FederationManager {
    config: FederationConfig,
    identity: Arc<Identity>,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
}

struct Identity {
    node_id: String,
    signing: SigningKey,
    x_secret: StaticSecret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Envelope {
    protocol: String,
    sender_id: String,
    request_id: String,
    nonce_hex: String,
    ciphertext_hex: String,
    signature_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerRequest {
    operation: String,
    params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerResponse {
    ok: bool,
    result: Option<Value>,
    error: Option<String>,
}

impl FederationManager {
    pub fn new(config: FederationConfig) -> Result<Self> {
        let identity = load_or_create_identity(Path::new(&config.state_path))?;
        let peers = load_peers(&PathBuf::from(format!("{}.peers.json", config.state_path)))?;
        Ok(Self { config, identity: Arc::new(identity), peers: Arc::new(RwLock::new(peers)) })
    }

    pub fn node_id(&self) -> &str { &self.identity.node_id }

    pub fn DiscoverPeers(&self) -> Result<Vec<Peer>> {
        let discovered = discover_mdns(self.config.mdns_timeout_ms, self.config.peer_port)?;
        let mut store = self.peers.write().map_err(|_| anyhow!("peer store poisoned"))?;
        for mut peer in discovered {
            peer.trusted = store.get(&peer.peer_id).map(|p| p.trusted).unwrap_or(false);
            if let Some(old) = store.get(&peer.peer_id) {
                peer.ed25519_public_key_hex = old.ed25519_public_key_hex.clone();
                peer.x25519_public_key_hex = old.x25519_public_key_hex.clone();
                peer.registry_scopes = old.registry_scopes.clone();
            }
            store.insert(peer.peer_id.clone(), peer);
        }
        self.persist_peers(&store)?;
        Ok(store.values().cloned().collect())
    }

    pub fn AddManualPeer(&self, peer_id: &str, endpoint: &str) -> Result<Peer> {
        validate_peer_id(peer_id)?;
        validate_endpoint(endpoint)?;
        let peer = Peer { peer_id: peer_id.into(), endpoint: endpoint.into(), discovered: false, trusted: false, ed25519_public_key_hex: None, x25519_public_key_hex: None, last_seen_unix: now(), registry_scopes: vec![] };
        let mut store = self.peers.write().map_err(|_| anyhow!("peer store poisoned"))?;
        store.insert(peer_id.into(), peer.clone());
        self.persist_peers(&store)?;
        Ok(peer)
    }

    pub async fn EstablishTrust(&self, peer_id: &str, exchange: TrustExchange) -> Result<TrustOffer> {
        validate_peer_id(peer_id)?;
        validate_endpoint(&exchange.endpoint)?;
        let peer_ed = decode32(&exchange.peer_ed25519_public_key_hex)?;
        let peer_x = decode32(&exchange.peer_x25519_public_key_hex)?;
        let challenge = hex::decode(&exchange.challenge_hex)?;
        if challenge.len() < 16 || challenge.len() > 64 { return Err(anyhow!("challenge must be 16..64 bytes")); }
        let transcript = trust_transcript(self.node_id(), peer_id, &self.public_ed(), &hex::encode(self.public_x()), &hex::encode(peer_ed), &hex::encode(peer_x), &exchange.endpoint, &challenge);
        VerifyingKey::from_bytes(&peer_ed)?.verify(&transcript, &Signature::from_slice(&hex::decode(exchange.peer_signature_hex)?)?)?;
        let local_sig = self.identity.signing.sign(&transcript);
        let peer = Peer { peer_id: peer_id.into(), endpoint: exchange.endpoint.clone(), discovered: false, trusted: true, ed25519_public_key_hex: Some(hex::encode(peer_ed)), x25519_public_key_hex: Some(hex::encode(peer_x)), last_seen_unix: now(), registry_scopes: vec![] };
        let mut store = self.peers.write().map_err(|_| anyhow!("peer store poisoned"))?;
        let scopes = store.get(peer_id).map(|p| p.registry_scopes.clone()).unwrap_or_default();
        let mut peer = peer;
        peer.registry_scopes = scopes;
        store.insert(peer_id.into(), peer);
        self.persist_peers(&store)?;
        Ok(TrustOffer { peer_id: self.node_id().into(), endpoint: self.local_endpoint_hint(), ed25519_public_key_hex: self.public_ed(), x25519_public_key_hex: hex::encode(self.public_x()), challenge_hex: exchange.challenge_hex, signature_hex: hex::encode(local_sig.to_bytes()) })
    }

    pub fn RevokeTrust(&self, peer_id: &str) -> Result<()> {
        let mut store = self.peers.write().map_err(|_| anyhow!("peer store poisoned"))?;
        let p = store.get_mut(peer_id).ok_or_else(|| anyhow!("peer not found"))?;
        p.trusted = false;
        p.ed25519_public_key_hex = None;
        p.x25519_public_key_hex = None;
        p.registry_scopes.clear();
        p.last_seen_unix = now();
        self.persist_peers(&store)
    }

    pub fn ListPeers(&self) -> Result<Vec<Peer>> {
        Ok(self.peers.read().map_err(|_| anyhow!("peer store poisoned"))?.values().cloned().collect())
    }

    pub async fn ReplicateBackup(&self, monolith_id: &str, peer_id: &str, db: &sqlx::PgPool) -> Result<Value> {
        let row = sqlx::query_as::<_, (String, String, String, String)>("SELECT monolith_id,package_id,target_id,state FROM backups WHERE monolith_id=$1")
            .bind(monolith_id).fetch_optional(db).await?.ok_or_else(|| anyhow!("backup not found"))?;
        self.call_peer(peer_id, "backup.replicate", json!({"monolith_id":row.0,"package_id":row.1,"target_id":row.2,"state":row.3})).await
    }

    pub async fn RestoreFromPeer(&self, peer_id: &str, monolith_id: &str, db: &sqlx::PgPool) -> Result<Value> {
        let value = self.call_peer(peer_id, "backup.get", json!({"monolith_id":monolith_id})).await?;
        let obj = value.as_object().ok_or_else(|| anyhow!("peer returned invalid backup"))?;
        let pkg = obj.get("package_id").and_then(Value::as_str).ok_or_else(|| anyhow!("peer backup missing package_id"))?;
        let target = obj.get("target_id").and_then(Value::as_str).unwrap_or("peer");
        let state = obj.get("state").and_then(Value::as_str).unwrap_or("replicated");
        sqlx::query("INSERT INTO backups(monolith_id,package_id,target_id,state) VALUES($1,$2,$3,$4) ON CONFLICT (monolith_id) DO UPDATE SET package_id=excluded.package_id,target_id=excluded.target_id,state=excluded.state")
            .bind(monolith_id).bind(pkg).bind(target).bind(state).execute(db).await?;
        Ok(json!({"restored":monolith_id,"source_peer":peer_id}))
    }

    pub async fn ShareRegistry(&self, peer_id: &str, registry_scope: &str) -> Result<Value> {
        validate_scope(registry_scope)?;
        {
            let store = self.peers.read().map_err(|_| anyhow!("peer store poisoned"))?;
            let p = store.get(peer_id).ok_or_else(|| anyhow!("peer not found"))?;
            if !p.trusted { return Err(anyhow!("peer is not trusted")); }
        }
        let mut store = self.peers.write().map_err(|_| anyhow!("peer store poisoned"))?;
        let p = store.get_mut(peer_id).unwrap();
        if !p.registry_scopes.iter().any(|s| s == registry_scope) { p.registry_scopes.push(registry_scope.into()); }
        self.persist_peers(&store)?;
        self.call_peer(peer_id, "registry.share", json!({"scope":registry_scope})).await
    }

    pub async fn QueryPeerStatus(&self, peer_id: &str) -> Result<Value> {
        self.call_peer(peer_id, "status", json!({})).await
    }

    pub async fn DelegateServiceControl(&self, peer_id: &str, service_id: &str, scopes: &[String]) -> Result<Value> {
        validate_peer_id(peer_id)?;
        if service_id.is_empty() || service_id.len() > 255 { return Err(anyhow!("invalid service_id")); }
        if scopes.is_empty() { return Err(anyhow!("at least one scope is required")); }
        for s in scopes { validate_scope(s)?; }
        self.call_peer(peer_id, "service.delegate", json!({"service_id":service_id,"scopes":scopes})).await
    }

    async fn call_peer(&self, peer_id: &str, operation: &str, params: Value) -> Result<Value> {
        let peer = {
            let store = self.peers.read().map_err(|_| anyhow!("peer store poisoned"))?;
            let p = store.get(peer_id).ok_or_else(|| anyhow!("peer not found"))?;
            if !p.trusted { return Err(anyhow!("peer trust revoked")); }
            p.clone()
        };
        let body = self.encrypt_for_peer(&peer, &PeerRequest { operation: operation.into(), params })?;
        let url = format!("{}/api/v1/federation/envelope", peer.endpoint.trim_end_matches('/'));
        let client = reqwest::Client::builder().connect_timeout(Duration::from_secs(5)).timeout(Duration::from_secs(30)).build()?;
        let response = client.post(url).header("content-type","application/octet-stream").body(serde_json::to_vec(&body)?).send().await?;
        if !response.status().is_success() { return Err(anyhow!("peer returned HTTP {}", response.status())); }
        let envelope: Envelope = response.json().await?;
        let peer_response: PeerResponse = self.decrypt_from_peer(&peer, &envelope)?;
        if peer_response.ok { peer_response.result.ok_or_else(|| anyhow!("peer returned no result")) } else { Err(anyhow!(peer_response.error.unwrap_or_else(|| "peer operation failed".into()))) }
    }

    fn encrypt_for_peer(&self, peer: &Peer, request: &PeerRequest) -> Result<Envelope> {
        let peer_x = XPublicKey::from(decode32(peer.x25519_public_key_hex.as_ref().ok_or_else(|| anyhow!("peer x25519 key missing"))?)?);
        let shared = self.identity.x_secret.diffie_hellman(&peer_x);
        let key_bytes = key_for(&shared.to_bytes(), self.node_id(), &peer.peer_id);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
        let mut nonce_bytes = [0u8;12]; OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = serde_json::to_vec(request)?;
        let aad = format!("{}|{}|{}", PROTOCOL, self.node_id(), peer.peer_id);
        let ciphertext = cipher.encrypt(nonce, chacha20poly1305::aead::Payload { msg:&plaintext, aad:aad.as_bytes() }).map_err(|_| anyhow!("peer encryption failed"))?;
        let mut signed = aad.into_bytes();
        signed.extend_from_slice(&nonce_bytes); signed.extend_from_slice(&ciphertext);
        let sig = self.identity.signing.sign(&signed);
        Ok(Envelope { protocol:PROTOCOL.into(), sender_id:self.node_id().into(), request_id:Uuid::new_v4().to_string(), nonce_hex:hex::encode(nonce_bytes), ciphertext_hex:hex::encode(ciphertext), signature_hex:hex::encode(sig.to_bytes()) })
    }

    fn decrypt_from_peer(&self, peer: &Peer, envelope: &Envelope) -> Result<PeerResponse> {
        if envelope.protocol != PROTOCOL || envelope.sender_id != peer.peer_id { return Err(anyhow!("invalid federation envelope")); }
        let peer_ed = VerifyingKey::from_bytes(&decode32(peer.ed25519_public_key_hex.as_ref().ok_or_else(|| anyhow!("peer identity key missing"))?)?)?;
        let nonce = decode12(&envelope.nonce_hex)?;
        let ciphertext = hex::decode(&envelope.ciphertext_hex)?;
        if ciphertext.len() > MAX_ENVELOPE { return Err(anyhow!("federation payload too large")); }
        let aad = format!("{}|{}|{}", PROTOCOL, peer.peer_id, self.node_id());
        let mut signed = aad.clone().into_bytes(); signed.extend_from_slice(&nonce); signed.extend_from_slice(&ciphertext);
        peer_ed.verify(&signed, &Signature::from_slice(&hex::decode(&envelope.signature_hex)?)?)?;
        let peer_x = XPublicKey::from(decode32(peer.x25519_public_key_hex.as_ref().ok_or_else(|| anyhow!("peer x25519 key missing"))?)?);
        let shared = self.identity.x_secret.diffie_hellman(&peer_x);
        let key_bytes = key_for(&shared.to_bytes(), peer.peer_id.as_str(), self.node_id());
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
        let plaintext = cipher.decrypt(Nonce::from_slice(&nonce), chacha20poly1305::aead::Payload {msg:&ciphertext,aad:aad.as_bytes()}).map_err(|_| anyhow!("peer decryption failed"))?;
        Ok(serde_json::from_slice(&plaintext)?)
    }

    pub fn local_endpoint_hint(&self) -> String { format!("http://127.0.0.1:{}", self.config.peer_port) }
    fn public_ed(&self) -> String { hex::encode(self.identity.signing.verifying_key().to_bytes()) }
    fn public_x(&self) -> [u8;32] { XPublicKey::from(&self.identity.x_secret).to_bytes() }
    fn persist_peers(&self, peers: &HashMap<String,Peer>) -> Result<()> { save_peers(&PathBuf::from(format!("{}.peers.json", self.config.state_path)), peers) }
}

pub async fn handle_envelope(core: Arc<crate::Core>, manager: Arc<FederationManager>, body: Vec<u8>) -> Result<Vec<u8>> {
    if body.len() > MAX_ENVELOPE { return Err(anyhow!("federation payload too large")); }
    let envelope: Envelope = serde_json::from_slice(&body)?;
    if envelope.protocol != PROTOCOL { return Err(anyhow!("unsupported federation protocol")); }
    let peer = {
        let store = manager.peers.read().map_err(|_| anyhow!("peer store poisoned"))?;
        let p = store.get(&envelope.sender_id).ok_or_else(|| anyhow!("unknown peer"))?;
        if !p.trusted { return Err(anyhow!("peer is not trusted")); }
        p.clone()
    };
    let request: PeerRequest = manager.decrypt_request(&peer, &envelope)?;
    let response = match request.operation.as_str() {
        "status" => PeerResponse { ok:true, result:Some(serde_json::to_value(core.get_system_state().await?)?), error:None },
        "backup.replicate" => {
            let p=request.params["package_id"].as_str().ok_or_else(|| anyhow!("package_id required"))?;
            let t=request.params["target_id"].as_str().unwrap_or("peer");
            let m=request.params["monolith_id"].as_str().ok_or_else(|| anyhow!("monolith_id required"))?;
            let state=request.params["state"].as_str().unwrap_or("replicated");
            sqlx::query("INSERT INTO backups(monolith_id,package_id,target_id,state) VALUES($1,$2,$3,$4) ON CONFLICT (monolith_id) DO UPDATE SET package_id=excluded.package_id,target_id=excluded.target_id,state=excluded.state")
                .bind(m).bind(p).bind(t).bind(state).execute(&core.db).await?;
            PeerResponse {ok:true,result:Some(json!({"replicated":m})),error:None}
        }
        "backup.get" => {
            let m=request.params["monolith_id"].as_str().ok_or_else(|| anyhow!("monolith_id required"))?;
            let row=sqlx::query_as::<_,(String,String,String,String)>("SELECT monolith_id,package_id,target_id,state FROM backups WHERE monolith_id=$1")
                .bind(m).fetch_optional(&core.db).await?.ok_or_else(|| anyhow!("backup not found"))?;
            PeerResponse {ok:true,result:Some(json!({"monolith_id":row.0,"package_id":row.1,"target_id":row.2,"state":row.3})),error:None}
        }
        "registry.share" => {
            let scope=request.params["scope"].as_str().ok_or_else(|| anyhow!("scope required"))?;
            validate_scope(scope)?;
            PeerResponse {ok:true,result:Some(json!({"shared":scope})),error:None}
        }
        "service.delegate" => {
            let service_id=request.params["service_id"].as_str().ok_or_else(|| anyhow!("service_id required"))?;
            let scopes:Vec<String>=serde_json::from_value(request.params["scopes"].clone())?;
            if !scopes.iter().all(|s| matches!(s.as_str(),"read"|"start"|"stop"|"restart"|"logs"|"config")) { return Err(anyhow!("unsupported service scope")); }
            let mut results=Vec::new();
            if scopes.iter().any(|s|s=="start") { core.start_service(service_id).await?; results.push("started"); }
            if scopes.iter().any(|s|s=="stop") { core.stop_service(service_id,true).await?; results.push("stopped"); }
            if scopes.iter().any(|s|s=="restart") { core.restart_service(service_id).await?; results.push("restarted"); }
            if scopes.iter().any(|s|s=="read"||s=="config"||s=="logs") { results.push("authorized"); }
            PeerResponse {ok:true,result:Some(json!({"service_id":service_id,"applied":results})),error:None}
        }
        _ => PeerResponse {ok:false,result:None,error:Some("unsupported federation operation".into())}
    };
    let out = manager.encrypt_response(&peer, &response)?;
    Ok(serde_json::to_vec(&out)?)
}

impl FederationManager {
    fn decrypt_request(&self, peer:&Peer, envelope:&Envelope)->Result<PeerRequest>{
        if envelope.protocol != PROTOCOL || envelope.sender_id != peer.peer_id { return Err(anyhow!("invalid federation envelope")); }
        let peer_ed=VerifyingKey::from_bytes(&decode32(peer.ed25519_public_key_hex.as_ref().ok_or_else(|| anyhow!("peer identity key missing"))?)?)?;
        let nonce=decode12(&envelope.nonce_hex)?;
        let ciphertext=hex::decode(&envelope.ciphertext_hex)?;
        if ciphertext.len()>MAX_ENVELOPE{return Err(anyhow!("payload too large"))}
        let aad=format!("{}|{}|{}",PROTOCOL,peer.peer_id,self.node_id());
        let mut signed=aad.clone().into_bytes();signed.extend_from_slice(&nonce);signed.extend_from_slice(&ciphertext);
        peer_ed.verify(&signed,&Signature::from_slice(&hex::decode(&envelope.signature_hex)?)?)?;
        let peer_x=XPublicKey::from(decode32(peer.x25519_public_key_hex.as_ref().ok_or_else(|| anyhow!("peer x25519 key missing"))?)?);
        let shared=self.identity.x_secret.diffie_hellman(&peer_x);
        let key=key_for(&shared.to_bytes(),peer.peer_id.as_str(),self.node_id());
        let cipher=ChaCha20Poly1305::new(Key::from_slice(&key));
        let plaintext=cipher.decrypt(Nonce::from_slice(&nonce),chacha20poly1305::aead::Payload{msg:&ciphertext,aad:aad.as_bytes()}).map_err(|_|anyhow!("peer decryption failed"))?;
        Ok(serde_json::from_slice(&plaintext)?)
    }
    fn encrypt_response(&self, peer:&Peer, response:&PeerResponse)->Result<Envelope>{
        let peer_x=XPublicKey::from(decode32(peer.x25519_public_key_hex.as_ref().ok_or_else(|| anyhow!("peer x25519 key missing"))?)?);
        let shared=self.identity.x_secret.diffie_hellman(&peer_x);
        let key=key_for(&shared.to_bytes(),self.node_id(),peer.peer_id.as_str());
        let cipher=ChaCha20Poly1305::new(Key::from_slice(&key));
        let mut nonce=[0u8;12];OsRng.fill_bytes(&mut nonce);
        let aad=format!("{}|{}|{}",PROTOCOL,self.node_id(),peer.peer_id);
        let ciphertext=cipher.encrypt(Nonce::from_slice(&nonce),chacha20poly1305::aead::Payload{msg:&serde_json::to_vec(response)?,aad:aad.as_bytes()}).map_err(|_|anyhow!("response encryption failed"))?;
        let mut signed=aad.into_bytes();signed.extend_from_slice(&nonce);signed.extend_from_slice(&ciphertext);
        let sig=self.identity.signing.sign(&signed);
        Ok(Envelope{protocol:PROTOCOL.into(),sender_id:self.node_id().into(),request_id:Uuid::new_v4().to_string(),nonce_hex:hex::encode(nonce),ciphertext_hex:hex::encode(ciphertext),signature_hex:hex::encode(sig.to_bytes())})
    }
}

fn trust_transcript(local_id:&str, peer_id:&str, local_ed:&str, local_x:&str, peer_ed:&str, peer_x:&str, endpoint:&str, challenge:&[u8])->Vec<u8>{
    let mut ids=[local_id,peer_id]; ids.sort();
    let mut keys=[local_ed,peer_ed]; keys.sort();
    let mut xkeys=[local_x,peer_x]; xkeys.sort();
    format!("{}|trust|{}|{}|{}|{}|{}|{}",PROTOCOL,ids[0],ids[1],keys[0],xkeys[0],xkeys[1],endpoint,hex::encode(challenge)).into_bytes()
}
fn key_for(shared:&[u8;32], a:&str, b:&str)->[u8;32]{
    let mut h=Sha256::new();h.update(PROTOCOL.as_bytes());h.update(shared);let mut ids=[a,b];ids.sort();h.update(ids[0].as_bytes());h.update(ids[1].as_bytes());h.finalize().into()
}
fn now()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()}
fn decode32(s:&str)->Result<[u8;32]>{let b=hex::decode(s)?;b.try_into().map_err(|_|anyhow!("expected 32 bytes"))}
fn decode12(s:&str)->Result<[u8;12]>{let b=hex::decode(s)?;b.try_into().map_err(|_|anyhow!("expected 12 bytes"))}
fn validate_peer_id(s:&str)->Result<()> { if s.is_empty() || s.len()>128 || !s.chars().all(|c|c.is_ascii_alphanumeric()||matches!(c,'-'|'_'|'.')) { return Err(anyhow!("invalid peer id")); } Ok(()) }
fn validate_endpoint(s:&str)->Result<()> { let u=reqwest::Url::parse(s)?; if !matches!(u.scheme(),"http"|"https") || u.host_str().is_none() {return Err(anyhow!("endpoint must be http(s)"));} Ok(()) }
fn validate_scope(s:&str)->Result<()> { if s.is_empty() || s.len()>128 || s.chars().any(|c|c.is_control()||c==' ') {return Err(anyhow!("invalid scope"));} Ok(()) }

fn load_or_create_identity(path:&Path)->Result<Identity>{
    if path.exists(){
        let f:IdentityFile=serde_json::from_slice(&std::fs::read(path)?)?;
        let ed=decode32(&f.ed25519_secret_hex)?;let x=decode32(&f.x25519_secret_hex)?;
        return Ok(Identity{node_id:f.node_id,signing:SigningKey::from_bytes(&ed),x_secret:StaticSecret::from(x)});
    }
    let signing=SigningKey::generate(&mut OsRng);let x_secret=StaticSecret::random_from_rng(OsRng);
    let f=IdentityFile{node_id:Uuid::new_v4().to_string(),ed25519_secret_hex:hex::encode(signing.to_bytes()),x25519_secret_hex:hex::encode(x_secret.to_bytes())};
    if let Some(parent)=path.parent(){if !parent.as_os_str().is_empty(){std::fs::create_dir_all(parent)?;}}
    std::fs::write(path,serde_json::to_vec_pretty(&f)?)?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(path,std::fs::Permissions::from_mode(0o600))?; }
    Ok(Identity{node_id:f.node_id,signing,x_secret})
}

fn load_peers(path:&Path)->Result<HashMap<String,Peer>>{
    if !path.exists(){return Ok(HashMap::new())}
    let store:PeerStore=serde_json::from_slice(&std::fs::read(path)?)?;Ok(store.peers)
}
fn save_peers(path:&Path, peers:&HashMap<String,Peer>)->Result<()>{
    if let Some(parent)=path.parent(){if !parent.as_os_str().is_empty(){std::fs::create_dir_all(parent)?;}}
    std::fs::write(path,serde_json::to_vec_pretty(&PeerStore{peers:peers.clone()})?)?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(path,std::fs::Permissions::from_mode(0o600))?; }
    Ok(())
}

fn discover_mdns(timeout_ms:u64, default_port:u16)->Result<Vec<Peer>>{
    let socket=std::net::UdpSocket::bind(("0.0.0.0",0))?;
    socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))?;
    let query=mdns_ptr_query(SERVICE);
    socket.send_to(&query,MDNS_ADDR)?;
    let mut buf=[0u8;65535];let mut peers=HashMap::<String,Peer>::new();
    loop{
        match socket.recv_from(&mut buf){
            Ok((n,src))=>{for (name,port) in parse_srv_records(&buf[..n],SERVICE){let endpoint=format!("http://{}:{}",src.ip(),if port==0{default_port}else{port});let id=name.trim_end_matches('.').to_string();peers.entry(id.clone()).or_insert(Peer{peer_id:id,endpoint,discovered:true,trusted:false,ed25519_public_key_hex:None,x25519_public_key_hex:None,last_seen_unix:now(),registry_scopes:vec![]});}},
            Err(e) if e.kind()==std::io::ErrorKind::WouldBlock||e.kind()==std::io::ErrorKind::TimedOut=>break,
            Err(e)=>return Err(e.into())
        }
    }
    Ok(peers.into_values().collect())
}
fn mdns_ptr_query(name:&str)->Vec<u8>{let mut out=vec![0,0,0,0,0,1,0,0,0,0,0,0];for p in name.trim_end_matches('.').split('.') {out.push(p.len() as u8);out.extend_from_slice(p.as_bytes());}out.push(0);out.extend_from_slice(&12u16.to_be_bytes());out.extend_from_slice(&1u16.to_be_bytes());out}
fn parse_srv_records(data:&[u8],service:&str)->Vec<(String,u16)>{
    if data.len()<12{return vec![]} let qd=u16::from_be_bytes([data[4],data[5]]) as usize;let an=u16::from_be_bytes([data[6],data[7]]) as usize;let mut off=12;
    for _ in 0..qd{if skip_name(data,&mut off).is_err()||off+4>data.len(){return vec![];}off+=4;}
    let mut out=Vec::new();for _ in 0..an{let _=read_name(data,&mut off);if off+10>data.len(){break;}let typ=u16::from_be_bytes([data[off],data[off+1]]);let class=u16::from_be_bytes([data[off+2],data[off+3]]);let rdlen=u16::from_be_bytes([data[off+8],data[off+9]]) as usize;off+=10;if off+rdlen>data.len(){break;}if typ==33&&class&0x7fff==1&&rdlen>=7{let port=u16::from_be_bytes([data[off+4],data[off+5]]);let mut noff=off+6;if let Ok(target)=read_name(data,&mut noff){let id=target.trim_end_matches(".local.").trim_end_matches('.').to_string();if !service.is_empty(){out.push((id,port));}}}off+=rdlen;}out}
fn skip_name(data:&[u8],off:&mut usize)->Result<()> {let _=read_name(data,off)?;Ok(())}
fn read_name(data:&[u8],off:&mut usize)->Result<String>{let mut pos=*off;let mut jumped=false;let mut next=*off;let mut labels=Vec::new();let mut depth=0;loop{if pos>=data.len(){return Err(anyhow!("dns name out of bounds"))}let len=data[pos];if len&0xc0==0xc0{if pos+1>=data.len(){return Err(anyhow!("dns pointer"))}let ptr=(((len as usize)&0x3f)<<8)|data[pos+1] as usize;if !jumped{next=pos+2;jumped=true}pos=ptr;depth+=1;if depth>20{return Err(anyhow!("dns pointer loop"))}continue}if len==0{pos+=1;if !jumped{next=pos}break}if len>63||pos+1+len as usize>data.len(){return Err(anyhow!("dns label"))}labels.push(String::from_utf8_lossy(&data[pos+1..pos+1+len as usize]).into_owned());pos+=1+len as usize;}*off=next;Ok(if labels.is_empty(){String::new()}else{format!("{}.",labels.join("."))})}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_key_is_symmetric() {
        let shared=[7u8;32];
        assert_eq!(key_for(&shared,"a","b"),key_for(&shared,"b","a"));
    }

    #[test]
    fn decode32_accepts_exact_key_length() {
        assert!(decode32(&"aa".repeat(32)).is_ok());
        assert!(decode32("aa").is_err());
    }

    #[test]
    fn scope_rejects_control_and_space() {
        assert!(validate_scope("registry:read").is_ok());
        assert!(validate_scope("registry read").is_err());
        assert!(validate_scope("registry\nread").is_err());
    }

    #[test]
    fn endpoint_requires_http_or_https() {
        assert!(validate_endpoint("https://peer.local:8090").is_ok());
        assert!(validate_endpoint("ftp://peer.local:21").is_err());
    }

    #[test]
    fn mdns_query_contains_service_question() {
        let q=mdns_ptr_query(SERVICE);
        assert_eq!(u16::from_be_bytes([q[4],q[5]]),1);
        assert!(q.windows(8).any(|w| w==b"_tjspace"));
    }
}
