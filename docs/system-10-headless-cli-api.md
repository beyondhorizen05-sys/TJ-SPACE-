# System 10 — Headless CLI & API

## D1 — Scope
System 10 owns the standalone `tjspace-cli` binary, scriptable command output, stable CLI exit codes, and the authenticated public REST command contract.

It wraps backend RPC methods. It does not implement a second backend or a graphical interface.

## D2 — Data model
- `Cli`: endpoint, JWT/token credentials, JSON/quiet flags, selected command.
- Command enums: service, package, registry, volume, backup, device, update, diagnostic.
- `ApiEnvelope`: RPC method, parameters, trace ID.
- `RpcResponse`: existing Core response envelope.
- JWT claims: subject, issued-at, expiry, scope.
- Stable exit codes:
  - 0 success
  - 2 usage/invalid input
  - 3 authentication failure
  - 4 resource not found
  - 5 conflict/already exists
  - 6 permission/confirmation failure
  - 7 local I/O/transport failure
  - 8 remote backend failure
  - 70 internal failure

JWT is a compact signed claims representation as defined by RFC 7519; this implementation uses HS256 with an explicitly configured secret and expiry. citeturn0search0

## D3 — Interface contract

### CLI
`tjspace-cli status`

Service:
- `service list`
- `service start <package-id>`
- `service stop <package-id> [--force]`
- `service restart <package-id>`
- `service logs <package-id> [--lines N]`
- `service config <package-id>`

Package:
- `package install <path>`
- `package uninstall <package-id> [--purge-data]`
- `package list`
- `package inspect <package-id>`
- `package verify <package-id> --public-key-hex <key>`

Registry:
- `registry add <url>`
- `registry remove <url>`
- `registry list`
- `registry search <query>`

Volume:
- `volume create <name> --disks <disk...> [--raid-level <level>] [--fs-type <fs>]`
- `volume resize <volume-id> <new-size>`
- `volume delete <volume-id>`
- `volume list`

Backup:
- `backup create <package-id> <target-id>`
- `backup restore <monolith-id>`
- `backup list`
- `backup verify <monolith-id>`

Device:
- `device add <device-id>`
- `device revoke <device-id>`
- `device list`

Update:
- `update check`
- `update apply <release-id>`
- `update rollback <target-version>`

Diagnostics:
- `diag dump`
- `diag restore <snapshot-file>`

Every command supports global `--json` and `--quiet`.

### REST
All command endpoints use `/api/v1/*`, POST request bodies carrying RPC parameters, and Bearer JWT authentication:
- `/api/v1/status`
- `/api/v1/service/{list,start,stop,restart,logs,config}`
- `/api/v1/package/{install,uninstall,list,inspect,verify}`
- `/api/v1/registry/{add,remove,list,search}`
- `/api/v1/volume/{create,resize,delete,list}`
- `/api/v1/backup/{create,restore,list,verify}`
- `/api/v1/device/{add,revoke,list}`
- `/api/v1/update/{check,apply,rollback}`
- `/api/v1/diag/{dump,restore}`

Axum supports route-level authorization middleware, which is used for the public API layer. citeturn1search0turn1search3

## D4 — Implementation
- Added `tjspace-core/src/headless_api.rs`.
- Added `tjspace-core/src/cli.rs`.
- Registered the `tjspace-cli` Cargo binary.
- CLI sends authenticated requests to the REST command surface; the REST handlers translate commands back into existing RPC calls.
- JSON mode emits compact machine-readable JSON.
- Quiet mode suppresses normal output and errors.
- JWTs can be supplied through `--token` / `TJS_JWT_TOKEN`, or generated from `--jwt-secret` / `TJS_JWT_SECRET`.
- Server JWT signing secret is configured as `jwt_secret` in `tjspace.yaml`.
- Service logs use the system journal.
- Package inspection/verification uses the stored package artifact and existing package integrity tooling.
- Registry endpoints and device state are persisted through Patch-DB.
- Volume listing reads persisted hardware volume metadata.
- Backup list/verify query the existing backup records.
- Diagnostic dump exports the Patch-DB snapshot; restore applies its state as authenticated patches.
- All commands execute through Core RPC handlers rather than directly manipulating backend state from the CLI.

## D5 — Integration
System 10 integrates only with Systems 1–9:
- Core RPC dispatcher
- Patch-DB
- RPC authentication/transport
- existing backend service/package/container operations
- existing package/registry operations
- existing hardware/volume operations
- existing backup operations
- existing device/update state
- existing installer/update recovery state

No graphical-world dependency exists.

## D6 — Failure modes
- Missing JWT: exit 3.
- Expired/invalid JWT: HTTP 401 and exit 3.
- Unknown CLI syntax: exit 2.
- Missing resource: exit 4.
- Conflict: exit 5.
- Permission/confirmation failure: exit 6.
- Local transport/I/O failure: exit 7.
- Backend RPC failure: exit 8.
- Unexpected internal error: exit 70.

The CLI never treats an unsuccessful backend response as success.

## D7 — Test cases
1. JWT generation produces a three-part compact token.
2. `--json` and `--quiet` parse after a command.
3. Service stop parses package ID and force mode.
4. REST paths map to their exact backend RPC methods.
5. Volume creation parses multiple disks and filesystem/RAID options.

## D8 — Acceptance checklist
- [x] `tjspace-cli` binary
- [x] Status
- [x] Service list/start/stop/restart/logs/config
- [x] Package install/uninstall/list/inspect/verify
- [x] Registry add/remove/list/search
- [x] Volume create/resize/delete/list
- [x] Backup create/restore/list/verify
- [x] Device add/revoke/list
- [x] Update check/apply/rollback
- [x] Diagnostic dump/restore
- [x] `--json`
- [x] `--quiet`
- [x] Stable exit codes
- [x] JWT-authenticated REST API
- [x] REST paths mirror CLI command groups
- [x] CI-testable without a graphical world
