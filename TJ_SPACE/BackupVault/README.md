# SYSTEM 8 — Backup Vault, Snapshots & Restore

## D1
Owns the vault wing, physical backup chambers, differential snapshot monoliths, progress visualization, restore discharge, integrity verification, deliberate deletion, and calendar-obelisk scheduling.

## D2
Models: FTJBackupTarget, FTJBackupMonolith, FTJBackupSchedule, FTJRestoreInterlock, ETJBackupState.

## D3
Required functions:
- BuildBackupVault()
- RenderBackupTarget(targetId)
- CreateBackup(packageId,targetId)
- RenderBackupProgress(packageId,progress)
- RestoreBackup(monolithId,targetService)
- VerifyBackupIntegrity(monolithId)
- DeleteBackup(monolithId)
- RenderBackupSchedule()

Physical restore controls also expose interlock hold/release.

## D4
The vault is a 3D Unreal Actor composed from Components; Unreal Actors are world-placeable objects and Components provide attached functionality and renderable pieces. citeturn0search1turn0search2

Each backup target is a chamber. A new differential backup marks the previous monolith in that chamber as Superseded and creates a visible dissolution effect before the new monolith becomes current.

Progress is visualized through the vault material state. Dynamic Material Instances can change material parameters during gameplay, enabling live progress and discharge effects. citeturn0search0turn0search5

Restore requires a physical interlock held for at least 2 seconds. Integrity verification produces a physical verification seal.

Deletion deliberately creates an OnlyRestorePointWarning01 warning surface. The implementation records the deletion state only after the deletion request reaches that deliberate deletion operation.

Backup scheduling is represented by a calendar obelisk.

## D5
System 1: uses established photorealistic materials and runtime parameterization.
System 2: vault is a 3D spatial structure.
System 3: package IDs align with service identities.
System 4: target services remain compatible with containerized service representation.
System 5: vault/network visuals remain spatially compatible.
System 6: backups operate on acquired package identities.
System 7: restore targets align with service lifecycle identities.

## D6
Unknown target rejects backup creation. Empty package/target IDs reject. New backups supersede the previous monolith in the same chamber. Progress is clamped to 0–1. Restore without a held interlock is rejected. Unknown monoliths are rejected. Integrity verification cannot operate on deleted monoliths. Deletion of an unknown/already-deleted monolith is rejected.

## D7
CI validates differential mode, dissolution requirement, restore interlock duration, only-restore-point deletion warning, calendar-obelisk schedule, all required API names, and absence of higher-system references.

## D8
- [x] Vault wing
- [x] Backup chambers
- [x] Differential monolith creation
- [x] Previous monolith dissolution
- [x] Live backup progress
- [x] Restore discharge
- [x] Physical restore interlock
- [x] Integrity verification
- [x] Deliberate deletion
- [x] Only-restore-point warning
- [x] Calendar-obelisk schedule
- [x] System 1–7 integration
- [x] Machine-readable contract
- [x] Automated tests
- [x] CI workflow
