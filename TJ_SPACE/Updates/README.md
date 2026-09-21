# SYSTEM 12 — Updates, Signing & Package Integrity
## D1
Owns update delivery, mandatory signature gating, renovation visualization, rollback restoration, Merkle integrity visualization, and miniature version history.
## D2
FTJUpdateRecord, FTJMerkleNode, ETJUpdateState, ETJIntegrityState.
## D3
RenderUpdateAvailable(packageId,version); RenderUpdateInProgress(packageId); VerifyUpdateSignature(packageId,sig); RenderRollback(packageId,targetVersion); RenderPackageIntegrityCheck(packageId); RenderUpdateHistory(packageId).
## D4
An update arrives as a delivery. The renovation crew remains physically refused until its signature seal is verified. A verified signature releases the crew. Runtime Dynamic Material Instances support gameplay-time parameter changes. citeturn0search0turn0search4
Rollback becomes architectural restoration. Integrity is represented by a Merkle tree; corrupted nodes are represented by dark cracked branches carrying the affected chunk identifier. Version history is a row of miniature architectural models.
The prototype signature gate uses the deterministic contract sig:<packageId>:<version>.
## D5
System 1 material foundation; System 2 Citadel world; System 3 service architecture; System 4 container context; System 5 network context; System 6 package/signature context; System 7 lifecycle context; System 8 backup context; System 9 telemetry context; System 10 identity/access context; System 11 archive/event context.
## D6
Missing package/version rejects update rendering. Missing or invalid signature blocks renovation. Rollback requires a target version. Empty package ID rejects integrity/history rendering.
## D7
Tests cover update delivery, pre-verification crew refusal, valid/invalid signature gating, rollback, Merkle representation, history models, required APIs, and higher-system reference exclusion.
## D8
- [x] Update delivery
- [x] Mandatory signature gate
- [x] Physical renovation crew refusal before verification
- [x] Verified seal releases renovation
- [x] Rollback as architectural restoration
- [x] Merkle tree contract
- [x] Corrupt-node dark cracked branch/chunk identification contract
- [x] Version history miniature architecture
- [x] Runtime material-state implementation
- [x] Machine-readable schema
- [x] Automated tests
- [x] Systems 1–11 integration
- [x] No higher-system references
