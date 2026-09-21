# TJ SPACE Core Backend

This is the headless Rust foundation for TJ SPACE.

## Build
Run cargo build --release inside tjspace-core. The single executable is target/release/tjs-box. The install script creates tjsd and tjs-cli symlinks to that same executable.

## RPC
POST /api/v1/rpc with JSON fields method and params. When auth_token is configured, send Authorization: Bearer <token>. Every RPC response contains a trace_id. JSON logs contain trace_id and service_id.

## Database
PostgreSQL is required. Startup applies the embedded migration.

## S9PK
An S9PK is a tar or tar.gz archive containing manifest.json with package_id and version.

## Signing
Artifacts use Ed25519. The 96-byte returned signed artifact contains the 32-byte public key followed by the 64-byte signature, allowing VerifyArtifact(bytes,sig) to verify without a separate key argument.
