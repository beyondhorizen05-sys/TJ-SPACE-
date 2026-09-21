# TJ SPACE — System 32: Save/Load & Session Persistence

## D1 — Scope
Owns operator session persistence, camera bookmarks, session history/UI state, 3D interface world snapshots, restart continuity, and world-layout restoration. It does not own service-data backups, container filesystems, backend databases, or package backup/restore.

## D2 — Data model
FTJCameraBookmark stores client-local name, transform, and timestamp. FTJSessionState stores client ID, bookmarks, history, opaque UI JSON, and schema version. UTJSessionSaveGame is the Unreal USaveGame representation. World snapshots store world ID, capture time, streaming origin, environment health, registered spatial transforms, and district state. Storage is separated into Sessions and WorldSnapshots. Unreal documents USaveGame as the base object for saved state and supports asynchronous save/load for larger saves. citeturn0search0turn0search2

## D3 — Interface contract
Required: SaveSession(clientId), LoadSession(clientId), SnapshotWorld(), RestoreWorld(snapshotId), PersistBookmark(clientId,name,transform), ListBookmarks(clientId), ClearSession(clientId). Runtime adapters: Initialize, SetSessionState, GetSessionState, GetBookmark.

## D4 — Implementation
Sessions use versioned JSON and atomic temporary-file replacement. Client IDs are sanitized before becoming filenames. SaveSession persists bookmarks, history, and UI state; LoadSession validates schema, kind, and client identity. PersistBookmark upserts by client plus name. SnapshotWorld captures the current System 14 spatial layout into a unique world-interface snapshot. RestoreWorld validates the snapshot, restores the spatial layout, reapplies environment health, and applies saved transforms through System 14 backend-ID resolution. Unreal's JSON serializer supports runtime object serialization/deserialization. citeturn1search0turn1search4

## D5 — Integration
System 13 provides the visual/presentation baseline but is not copied into the snapshot. System 14 is the authoritative spatial registry and world-layout boundary. Systems 16–31 are not copied wholesale; any session/UI metadata enters only through explicit session state. System 21 remains authoritative for backend service-data backups. System 32 never treats a world snapshot as a service backup.

## D6 — Failure modes
Missing world/kernel, empty IDs, corrupt or wrong-schema files, invalid transforms, unknown actor IDs, and disk failures fail closed. ClearSession is idempotent when the file is already absent.

## D7 — Test cases
1. Save/load history, UI JSON, and two bookmarks.
2. Upsert a bookmark and verify the new transform.
3. Reject empty IDs/names.
4. Snapshot, recreate the persistence object, and restore.
5. Corrupt a snapshot and verify no restore occurs.
6. Move a registered actor and restore its captured transform.
7. Clear a session and verify memory/disk removal.
8. Verify no System 21 backup path is read or written.

## D8 — Acceptance checklist
- Session bookmarks persist per client.
- Session history and UI state persist.
- World snapshots have a separate namespace/schema.
- World snapshots contain 3D interface state, not service backup data.
- World restoration resolves actors through System 14.
- Corrupt/mismatched state fails closed.
- Integration is limited to Systems 13–31.
