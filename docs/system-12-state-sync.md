# System 12 — State Synchronization Bridge

## Scope
The synchronization bridge carries authenticated backend state changes through bounded WebSocket and gRPC streams.

It owns:
- sync-session authentication;
- typed-diff streaming;
- per-client/session sequence allocation;
- bounded batching and path coalescing;
- gap detection;
- snapshot recovery;
- interpolation hints;
- reconnect replay;
- bounded slow-client queues and throttling.

## Wire surfaces
- WebSocket: `GET /api/v1/sync/ws?client_id=...&auth_token=...&prefix=...`
- gRPC: `tjspace.v1.StateSync/StreamDiffs`
- gRPC bind defaults to `127.0.0.1:8091`.

The WebSocket endpoint uses the standard WebSocket upgrade/framing model. The gRPC surface uses tonic/protobuf server-streaming.

## Ordering and recovery
Each client receives a monotonically increasing sequence. Reconnect retains recent per-client history in a bounded ring. If requested history is unavailable or a sequence gap is detected, the bridge returns a full Patch-DB snapshot instead of blocking the stream.

## Slow clients
Each session uses a bounded Tokio mpsc queue. Publishing uses non-blocking `try_send`; a full queue marks the client throttled and requests resynchronization rather than waiting on the client. This prevents a slow consumer from stalling the daemon.

## Diff batching
The stream coalesces repeated state paths during the configured latency window and caps the number of diffs per batch. The latest state for a repeated path wins within a batch.

## Interpolation
Set patches receive a smooth authoritative hint with a 100 ms default duration. Deletes receive a snap hint. These are advisory client hints; the backend diff remains authoritative.

## Security
The bridge uses the existing daemon authentication token for session admission. Invalid credentials are rejected before stream creation.

## Build
The gRPC bindings are generated from `proto/state_sync.proto` with the pure-Rust `protox` compiler and `tonic-prost-build`, avoiding a runtime dependency on protoc.
