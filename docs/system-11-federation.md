# System 11 — Federation / Multi-Citadel

## Scope
Owns peer discovery, explicit trust, encrypted peer RPC, cross-device backup replication/restore, registry sharing grants, peer status, and delegated service-control grants.

Does not own local service lifecycle, package format, local registry implementation, Patch-DB implementation, or local backup creation; those are consumed through lower-level interfaces.

## Security
- Discovery uses mDNS _tjspace._tcp.local on UDP 5353.
- Discovery data is untrusted metadata only.
- Trust is explicit and revocable.
- Each node has an Ed25519 signing identity and X25519 static key.
- Trust records pin both peer public keys.
- Post-trust requests use ChaCha20-Poly1305 AEAD with an X25519-derived key.
- Every encrypted envelope is additionally Ed25519-signed.
- AAD binds protocol, sender and recipient identities.
- Identity and peer stores are mode 0600 on Unix.
- Unknown or revoked peers are rejected before decryption/dispatch.
- Payloads are bounded to 8 MiB.

## RPC surface
DiscoverPeers, AddManualPeer, EstablishTrust, RevokeTrust, ListPeers, ReplicateBackup, RestoreFromPeer, ShareRegistry, QueryPeerStatus, DelegateServiceControl.

## Peer operations
Encrypted peer operations are status, backup.replicate, backup.get, registry.share, and service.delegate.

## Persistence
Identity is stored in tjspace-federation-identity.json; trusted peer metadata is stored beside it as .peers.json.

## Failure behavior
Network errors, bad signatures, revoked trust, invalid keys, oversized envelopes, unavailable backups, invalid scopes, and unsupported peer operations return errors without terminating the daemon.
