use anyhow::{anyhow, Result};
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Query, State},
    response::{Response, IntoResponse},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::{HashMap, VecDeque}, sync::{Arc, RwLock}, time::{Duration, Instant}};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response as GrpcResponse, Status};

use crate::patch_db::{DiffFilter, PatchDb, Snapshot, TypedDiff};

pub const SYNC_PROTOCOL: &str = "tjs-sync-v1";
const SESSION_QUEUE: usize = 64;
const HISTORY_LIMIT: usize = 4096;
const DEFAULT_BATCH_LATENCY_MS: u64 = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_grpc_bind")]
    pub grpc_bind: String,
    #[serde(default = "default_batch_latency_ms")]
    pub batch_latency_ms: u64,
    #[serde(default = "default_max_batch_diffs")]
    pub max_batch_diffs: usize,
    #[serde(default = "default_session_queue")]
    pub session_queue: usize,
}
fn default_grpc_bind() -> String { "127.0.0.1:8091".into() }
fn default_batch_latency_ms() -> u64 { DEFAULT_BATCH_LATENCY_MS }
fn default_max_batch_diffs() -> usize { 128 }
fn default_session_queue() -> usize { SESSION_QUEUE }

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            grpc_bind: default_grpc_bind(),
            batch_latency_ms: default_batch_latency_ms(),
            max_batch_diffs: default_max_batch_diffs(),
            session_queue: default_session_queue(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFilter {
    pub prefix: Option<String>,
}
impl From<SyncFilter> for DiffFilter {
    fn from(v: SyncFilter) -> Self { DiffFilter { prefix: v.prefix } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolationHint {
    pub mode: String,
    pub duration_ms: u64,
    pub authoritative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencedDiff {
    pub sequence: u64,
    pub diff: TypedDiff,
    pub interpolation: InterpolationHint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffBatch {
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub diffs: Vec<SequencedDiff>,
    pub max_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEnvelope {
    pub protocol: String,
    pub session_id: String,
    pub server_revision: u64,
    pub batch: Option<DiffBatch>,
    pub snapshot: Option<Snapshot>,
    pub resync_required: bool,
    pub throttled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectResult {
    pub session_id: String,
    pub replay: Vec<SequencedDiff>,
    pub snapshot: Option<Snapshot>,
    pub resync_required: bool,
}

struct Session {
    client_id: String,
    filter: SyncFilter,
    tx: mpsc::Sender<SyncEnvelope>,
    next_sequence: u64,
    last_ack: u64,
    throttled: bool,
    last_throttle_notice: Instant,
}

#[derive(Clone)]
pub struct SyncBridge {
    patch_db: PatchDb,
    config: SyncConfig,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    history: Arc<RwLock<HashMap<String, VecDeque<SequencedDiff>>>>,
    next_client_sequence: Arc<RwLock<HashMap<String, u64>>>,
    authorized_token: String,
}

impl SyncBridge {
    pub fn new(patch_db: PatchDb, config: SyncConfig, authorized_token: String) -> Self {
        Self {
            patch_db,
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
            next_client_sequence: Arc::new(RwLock::new(HashMap::new())),
            authorized_token,
        }
    }

    pub fn OpenSyncSession(&self, client_id: &str, auth_token: &str) -> Result<(String, mpsc::Receiver<SyncEnvelope>)> {
        if client_id.is_empty() || client_id.len() > 128 {
            return Err(anyhow!("invalid client_id"));
        }
        if !self.authorized_token.is_empty() && auth_token != self.authorized_token {
            return Err(anyhow!("sync authentication failed"));
        }
        let session_id = uuid::Uuid::new_v4().to_string();
        let capacity = self.config.session_queue.max(4);
        let (tx, rx) = mpsc::channel(capacity);
        let next = self.next_client_sequence.read().map_err(|_| anyhow!("sequence lock poisoned"))?
            .get(client_id).copied().unwrap_or(1);
        self.sessions.write().map_err(|_| anyhow!("session lock poisoned"))?.insert(session_id.clone(), Session {
            client_id: client_id.into(),
            filter: SyncFilter { prefix: None },
            tx,
            next_sequence: next,
            last_ack: next.saturating_sub(1),
            throttled: false,
            last_throttle_notice: Instant::now() - Duration::from_secs(60),
        });
        Ok((session_id, rx))
    }

    pub fn StreamDiffs(&self, session_id: &str, filter: SyncFilter) -> Result<mpsc::Receiver<SyncEnvelope>> {
        let mut sessions = self.sessions.write().map_err(|_| anyhow!("session lock poisoned"))?;
        let session = sessions.get_mut(session_id).ok_or_else(|| anyhow!("sync session not found"))?;
        session.filter = filter.clone();
        let rx = {
            let (tx, rx) = mpsc::channel(self.config.session_queue.max(4));
            session.tx = tx;
            rx
        };
        let patch_db = self.patch_db.clone();
        let sessions_ref = self.sessions.clone();
        let history_ref = self.history.clone();
        let next_client_sequence_ref = self.next_client_sequence.clone();
        let client_id = session.client_id.clone();
        let session_id_owned = session_id.to_string();
        tokio::spawn(async move {
            let mut source = patch_db.subscribe_to_diffs(filter.into());
            while let Ok(diff) = source.recv().await {
                let send = {
                    let mut map = match sessions_ref.write() { Ok(m) => m, Err(_) => break };
                    let s = match map.get_mut(&session_id_owned) { Some(s) => s, None => break };
                    let seq = s.next_sequence;
                    s.next_sequence = s.next_sequence.saturating_add(1);
                    let hint = ApplyInterpolationHint(&diff);
                    let item = SequencedDiff { sequence: seq, diff: diff.clone(), interpolation: hint };
                    {
                        let mut h = match history_ref.write() { Ok(v) => v, Err(_) => break };
                        let q = h.entry(client_id.clone()).or_default();
                        q.push_back(item.clone());
                        while q.len() > HISTORY_LIMIT { q.pop_front(); }
                    }
                    if let Ok(mut cursors) = next_client_sequence_ref.write() { cursors.insert(client_id.clone(), s.next_sequence); }
                    let envelope = SyncEnvelope {
                        protocol: SYNC_PROTOCOL.into(),
                        session_id: session_id_owned.clone(),
                        server_revision: diff.revision,
                        batch: Some(DiffBatch {
                            first_sequence: seq,
                            last_sequence: seq,
                            diffs: vec![item],
                            max_latency_ms: DEFAULT_BATCH_LATENCY_MS,
                        }),
                        snapshot: None,
                        resync_required: false,
                        throttled: false,
                    };
                    match s.tx.try_send(envelope) {
                        Ok(()) => s.throttled = false,
                        Err(_) => {
                            s.throttled = true;
                            if s.last_throttle_notice.elapsed() >= Duration::from_secs(1) {
                                s.last_throttle_notice = Instant::now();
                                let notice = SyncEnvelope {
                                    protocol: SYNC_PROTOCOL.into(),
                                    session_id: session_id_owned.clone(),
                                    server_revision: diff.revision,
                                    batch: None,
                                    snapshot: None,
                                    resync_required: true,
                                    throttled: true,
                                };
                                let _ = s.tx.try_send(notice);
                            }
                        }
                    }
                };
                if send.is_err() { break; }
            }
        });
        Ok(rx)
    }

    pub fn BatchDiffs(&self, diffs: &[SequencedDiff], max_latency_ms: u64) -> Vec<DiffBatch> {
        if diffs.is_empty() { return vec![]; }
        let limit = self.config.max_batch_diffs.max(1);
        let mut batches = Vec::new();
        let mut current: Vec<SequencedDiff> = Vec::new();
        let mut indexes: HashMap<String, usize> = HashMap::new();
        for diff in diffs.iter().cloned() {
            let path = diff.diff.patch.path.clone();
            if let Some(index) = indexes.get(&path).copied() {
                current[index] = diff;
                continue;
            }
            if current.len() >= limit {
                batches.push(make_batch(std::mem::take(&mut current), max_latency_ms));
                indexes.clear();
            }
            indexes.insert(path, current.len());
            current.push(diff);
        }
        if !current.is_empty() { batches.push(make_batch(current, max_latency_ms)); }
        batches
    }

    pub fn AssignSequence(&self, session_id: &str, diff: TypedDiff) -> Result<SequencedDiff> {
        let mut sessions = self.sessions.write().map_err(|_| anyhow!("session lock poisoned"))?;
        let s = sessions.get_mut(session_id).ok_or_else(|| anyhow!("sync session not found"))?;
        let seq = s.next_sequence;
        s.next_sequence = s.next_sequence.saturating_add(1);
        let item = SequencedDiff { sequence: seq, diff: diff.clone(), interpolation: ApplyInterpolationHint(&diff) };
        let mut next = self.next_client_sequence.write().map_err(|_| anyhow!("sequence lock poisoned"))?;
        next.insert(s.client_id.clone(), s.next_sequence);
        Ok(item)
    }

    pub fn DetectGap(&self, client_seq: u64, server_seq: u64) -> bool {
        server_seq > client_seq.saturating_add(1)
    }

    pub fn RequestFullSnapshot(&self, session_id: &str) -> Result<SyncEnvelope> {
        let revision = self.patch_db.get_revision()?;
        let snapshot = self.patch_db.get_snapshot()?;
        let mut sessions = self.sessions.write().map_err(|_| anyhow!("session lock poisoned"))?;
        let s = sessions.get_mut(session_id).ok_or_else(|| anyhow!("sync session not found"))?;
        s.last_ack = s.next_sequence.saturating_sub(1);
        Ok(SyncEnvelope {
            protocol: SYNC_PROTOCOL.into(),
            session_id: session_id.into(),
            server_revision: revision,
            batch: None,
            snapshot: Some(snapshot),
            resync_required: false,
            throttled: false,
        })
    }

    pub fn ApplyInterpolationHint(&self, diff: &TypedDiff) -> InterpolationHint {
        ApplyInterpolationHint(diff)
    }

    pub fn CloseSyncSession(&self, session_id: &str) -> Result<()> {
        self.sessions.write().map_err(|_| anyhow!("session lock poisoned"))?.remove(session_id)
            .ok_or_else(|| anyhow!("sync session not found"))?;
        Ok(())
    }

    pub fn Ack(&self, session_id: &str, sequence: u64) -> Result<()> {
        let mut sessions = self.sessions.write().map_err(|_| anyhow!("session lock poisoned"))?;
        let s = sessions.get_mut(session_id).ok_or_else(|| anyhow!("sync session not found"))?;
        if sequence > s.last_ack { s.last_ack = sequence; }
        Ok(())
    }

    pub fn HandleReconnect(&self, client_id: &str, last_ack_seq: u64, auth_token: &str) -> Result<ReconnectResult> {
        let (session_id, _rx) = self.OpenSyncSession(client_id, auth_token)?;
        let history = self.history.read().map_err(|_| anyhow!("history lock poisoned"))?
            .get(client_id).cloned().unwrap_or_default();
        let replay: Vec<SequencedDiff> = history.iter().filter(|d| d.sequence > last_ack_seq).cloned().collect();
        let resync_required = !replay.is_empty() && replay.first().map(|d| d.sequence > last_ack_seq.saturating_add(1)).unwrap_or(false);
        if resync_required {
            return Ok(ReconnectResult { session_id, replay: vec![], snapshot: Some(self.patch_db.get_snapshot()?), resync_required: true });
        }
        Ok(ReconnectResult { session_id, replay, snapshot: None, resync_required: false })
    }

    pub fn install_replay(&self, session_id: &str, replay: Vec<SequencedDiff>) -> Result<()> {
        let sessions = self.sessions.read().map_err(|_| anyhow!("session lock poisoned"))?;
        let s = sessions.get(session_id).ok_or_else(|| anyhow!("sync session not found"))?;
        let envelope = SyncEnvelope {
            protocol: SYNC_PROTOCOL.into(),
            session_id: session_id.into(),
            server_revision: replay.last().map(|d| d.diff.revision).unwrap_or(self.patch_db.get_revision()?),
            batch: if replay.is_empty() { None } else { Some(make_batch(replay, self.config.batch_latency_ms)) },
            snapshot: None,
            resync_required: false,
            throttled: false,
        };
        s.tx.try_send(envelope).map_err(|_| anyhow!("session queue full"))
    }

    pub fn patch_db(&self) -> PatchDb { self.patch_db.clone() }
}

fn make_batch(diffs: Vec<SequencedDiff>, max_latency_ms: u64) -> DiffBatch {
    DiffBatch {
        first_sequence: diffs.first().map(|d| d.sequence).unwrap_or(0),
        last_sequence: diffs.last().map(|d| d.sequence).unwrap_or(0),
        diffs,
        max_latency_ms,
    }
}

pub fn ApplyInterpolationHint(diff: &TypedDiff) -> InterpolationHint {
    let mode = match diff.patch.op {
        crate::patch_db::PatchOp::Set => "smooth",
        crate::patch_db::PatchOp::Delete => "snap",
    };
    InterpolationHint {
        mode: mode.into(),
        duration_ms: if mode == "smooth" { 100 } else { 0 },
        authoritative: true,
    }
}

#[derive(Debug, Deserialize)]
pub struct SyncQuery {
    pub client_id: String,
    pub auth_token: String,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub last_ack_seq: Option<u64>,
}

pub async fn websocket_handler(
    State(bridge): State<Arc<SyncBridge>>,
    Query(query): Query<SyncQuery>,
) -> Response {
    let result = bridge.OpenSyncSession(&query.client_id, &query.auth_token);
    match result {
        Ok((session_id, _)) => {
            let bridge2 = bridge.clone();
            let prefix = query.prefix.clone();
            let last_ack = query.last_ack_seq;
            ws.on_upgrade(move |socket| async move {
                handle_websocket(socket, bridge2, session_id, prefix, last_ack).await;
            })
        }
        Err(e) => (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error":e.to_string()}))).into_response(),
    }
}

async fn handle_websocket(mut socket: WebSocket, bridge: Arc<SyncBridge>, session_id: String, prefix: Option<String>, last_ack: Option<u64>) {
    let filter = SyncFilter { prefix };
    let mut rx = match bridge.StreamDiffs(&session_id, filter) {
        Ok(r) => r,
        Err(_) => return,
    };
    if let Some(seq) = last_ack {
        let _ = bridge.Ack(&session_id, seq);
    }
    let hello = json!({"protocol":SYNC_PROTOCOL,"session_id":session_id,"type":"opened"}).to_string();
    if socket.send(Message::Text(hello.into())).await.is_err() { let _=bridge.CloseSyncSession(&session_id); return; }
    loop {
        tokio::select! {
            inbound = socket.recv() => {
                match inbound {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(v)=serde_json::from_str::<Value>(&text) {
                            if let Some(seq)=v.get("ack_seq").and_then(Value::as_u64) { let _=bridge.Ack(&session_id,seq); }
                            if v.get("resync").and_then(Value::as_bool)==Some(true) {
                                if let Ok(snapshot)=bridge.RequestFullSnapshot(&session_id) {
                                    if let Ok(payload)=serde_json::to_string(&snapshot) { let _=socket.send(Message::Text(payload.into())).await; }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_)))|None => break,
                    _ => {}
                }
            }
            outbound = rx.recv() => {
                match outbound {
                    Some(envelope) => match serde_json::to_string(&envelope) {
                        Ok(payload) => if socket.send(Message::Text(payload.into())).await.is_err(){break;},
                        Err(_) => break,
                    },
                    None => break,
                }
            }
        }
    }
    let _=bridge.CloseSyncSession(&session_id);
}

pub mod grpc {
    tonic::include_proto!("tjspace.v1");

    use super::*;
    use futures_util::StreamExt;

    #[derive(Clone)]
    pub struct SyncGrpcService {
        pub bridge: Arc<SyncBridge>,
        pub authorized_token: String,
    }

    #[tonic::async_trait]
    impl state_sync_server::StateSync for SyncGrpcService {
        type StreamDiffsStream = ReceiverStream<Result<DiffMessage, Status>>;

        async fn stream_diffs(
            &self,
            request: Request<SyncRequest>,
        ) -> Result<GrpcResponse<Self::StreamDiffsStream>, Status> {
            let req=request.into_inner();
            let (session_id,_)=self.bridge.OpenSyncSession(&req.client_id,&req.auth_token)
                .map_err(|e|Status::unauthenticated(e.to_string()))?;
            let filter=SyncFilter{prefix:if req.filter_prefix.is_empty(){None}else{Some(req.filter_prefix)}};
            let mut rx=self.bridge.StreamDiffs(&session_id,filter).map_err(|e|Status::internal(e.to_string()))?;
            let (tx,out)=mpsc::channel(32);
            let bridge=self.bridge.clone();
            tokio::spawn(async move {
                while let Some(env)=rx.recv().await {
                    let payload=match serde_json::to_vec(&env){Ok(v)=>v,Err(e)=>{let _=tx.send(Err(Status::internal(e.to_string()))).await;break;}};
                    if tx.try_send(Ok(DiffMessage{session_id:session_id.clone(),server_revision:env.server_revision,sequence:env.batch.as_ref().map(|b|b.last_sequence).unwrap_or(0),payload_json:payload,resync_required:env.resync_required,throttled:env.throttled})).is_err(){break;}
                }
                let _=bridge.CloseSyncSession(&session_id);
            });
            Ok(GrpcResponse::new(ReceiverStream::new(out)))
        }
    }
}
