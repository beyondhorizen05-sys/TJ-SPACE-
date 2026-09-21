# System 9 — System Updates & Recovery

## D1 — Scope
System 9 owns headless OS release discovery, download, integrity verification, A/B staging, safe-mode state, factory reset of the managed OS deployment area, OS rollback, failed-boot recovery, and persistent update history.

It does not own application/package updates, container lifecycle, hardware management, installation partitioning, or UI.

## D2 — Data model
- OsUpdateConfig: release channel, release index URL, state root, optional Ed25519 public key, dry-run flag, command timeout, maximum boot attempts.
- OsRelease: release ID, version, channel, artifact URL, SHA-256, optional Ed25519 signature, publication time, notes.
- OsReleaseIndex: channel, current version, release list.
- UpdateRecord: update ID, release ID, source version, target version, timestamps, status, slot, error.
- Internal OsState: active slot/version, boot slot, previous version, pending release/slot, boot-attempt counter, safe-mode flag.
- FactoryResetResult: reset ID, preserve-data flag, wipe result, preserved-volume location.

State is persisted under the configured OS state root. Update history is append-oriented JSON; state files are atomically replaced.

## D3 — Interface contract
Public manager methods:
- CheckForOsUpdate() -> Result<Option<OsRelease>>
- DownloadOsUpdate(releaseId) -> Result<PathBuf>
- VerifyOsUpdate(releaseId) -> Result<bool>
- ApplyOsUpdate(releaseId) -> Result<UpdateRecord>
- EnterSafeMode() -> Result<()>
- ExitSafeMode() -> Result<()>
- FactoryReset(preserveData) -> Result<FactoryResetResult>
- RollbackOs(targetVersion) -> Result<()>
- GetUpdateHistory() -> Result<Vec<UpdateRecord>>

Recovery support:
- BootHealthAck(version) commits the new slot after the running OS passes boot-health checks.
- RecoverFailedBoot() increments the pending-boot attempt counter and automatically selects the previous slot after the configured threshold.

RPC methods expose the same operations through the existing authenticated transport.

CLI commands expose check/download/verify/apply/safe-mode/reset/rollback/history operations.

## D4 — Implementation
- Release discovery is channel-scoped and rejects a channel mismatch.
- Downloads are written to a temporary .part file, hashed, then atomically renamed.
- Verification requires the published SHA-256 and validates the TAR bundle contains usr/local/bin/tjs-box; when both signature and trusted public key are configured, the SHA-256 digest is also Ed25519 verified.
- Updates are staged into the inactive A/B slot. The active slot is never overwritten.
- Boot selection is persisted atomically in boot-slot.
- A pending update remains pending until BootHealthAck(version) succeeds.
- Failed boot attempts are counted. Once the threshold is exceeded, the previous slot/version is selected automatically and history records automatic_rollback.
- If staging or boot-selection fails, the previous slot is restored immediately.
- Safe mode persists a durable flag and Patch-DB state.
- Factory reset requires recovery mode outside dry-run, optionally copies managed volumes to a preservation area, clears both managed OS slots, and leaves the daemon in safe mode.
- Rollback only targets the current/previous managed OS version and validates the target slot before switching.
- System 8 installer remains the installation/partitioning boundary; System 9 consumes the resulting headless installation and manages subsequent OS generations.

Linux documentation describes atomic operations and live-update mechanisms that preserve system state across transitions; TJ SPACE uses the same safety principle of staging the replacement separately and committing only a controlled state transition. See the Linux kernel documentation for atomic operations and live update. 

## D5 — Integration
- System 1 Core: Core owns OsUpdateManager and exposes the authenticated RPC methods.
- System 2 Patch-DB: safe-mode, active-version, and factory-reset state transitions are persisted as patches.
- System 3 RPC Transport & Auth: OS-update RPCs pass through the existing authenticated RPC path.
- System 4 Container Runtime: no container is required to update the OS.
- System 5 Service Packaging Toolchain: not used for OS packages; OS artifacts are a separate TAR-based release format.
- System 6 Registry Server: not required; the OS release channel is an independently configured HTTP index.
- System 7 Hardware: factory reset/update logic does not bypass its hardware safety controls.
- System 8 OS Installer & First Boot: provides the installed headless base that System 9 subsequently updates.

## D6 — Failure modes
| Failure | Detection | Recovery |
|---|---|---|
| Release index unavailable | HTTP error | active OS unchanged |
| Channel mismatch | index validation | reject release |
| Download corruption | SHA-256 mismatch | delete partial file |
| Signature failure | Ed25519 verification error | reject artifact |
| Invalid OS bundle | required daemon missing | reject before slot switch |
| Slot staging failure | extraction/validation error | active slot untouched |
| Boot-selection failure | selector/error | restore old selector/state |
| New OS fails health acknowledgement | pending state remains | boot-attempt recovery selects old slot |
| Repeated failed boots | counter threshold | automatic rollback |
| Factory reset outside recovery mode | environment guard | refuse reset |
| Concurrent update/reset | lock creation failure | refuse second operation |

The A/B rule is the central safety invariant: the currently active OS slot is never modified while an update is being staged.

## D7 — Test cases
1. Default manager state has empty update history.
2. Safe-mode entry and exit persist the durable flag.
3. Factory reset with preserveData=true preserves the managed volume tree.
4. Rollback rejects an unavailable target version.
5. Bundle verification rejects an archive without usr/local/bin/tjs-box.
6. SHA-256 verification rejects corrupted downloads.
7. A pending update exceeding the boot-attempt threshold switches back to the previous slot and records automatic rollback.
8. Recovery-mode protection rejects destructive reset when recovery mode is absent.

## D8 — Acceptance checklist
- [x] Release-channel query implemented.
- [x] OS artifact download implemented.
- [x] SHA-256 verification implemented.
- [x] Optional Ed25519 release verification implemented.
- [x] A/B inactive-slot staging implemented.
- [x] Automatic failed-boot rollback state machine implemented.
- [x] Safe-mode state implemented.
- [x] Factory-reset preservation path implemented.
- [x] OS rollback implemented.
- [x] Update history implemented.
- [x] Authenticated RPC integration implemented.
- [x] Headless CLI integration implemented.
- [x] No 3D-world dependency.
- [x] No package-update mechanism is used for OS updates.
