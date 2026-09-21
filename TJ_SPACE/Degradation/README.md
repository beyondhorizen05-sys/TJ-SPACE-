# SYSTEM 31 — Error & Degradation Handling

## D1 Scope
Owns explicit failure/degraded-state policy, error surfacing, cached read-only behavior, sync pending state, world-load placeholders, service-crash handling, renderer/UI degradation state and aggregate health.

## D2 Data model
FTJErrorRecord, FTJSystemHealth, FTJAggregateHealth, health registry, active errors, pending local actions and degraded-state flags.

## D3 Interface
HandleBackendLoss(reason); HandleSyncBridgeLoss(); HandleWorldLoadFailure(region,error); HandleServiceCrash(packageId,containerId); EnterDegradedMode(scope,reason); ExitDegradedMode(); RecoverWorldState(); SurfaceError(error,severity); GetSystemHealth(). Registration/resolution helpers are included.

## D4 Implementation
Backend loss enables read-only cached mode and explicitly reports that writes are disabled. Sync loss marks local actions pending and does not claim remote confirmation. World load failure records the failure and requires a placeholder-void presentation. Service crashes record a critical service failure and require the service to enter System 16 Error state. Generic degraded mode records a visible actionable reason. Recovery is blocked when authoritative backend/sync state is unavailable. Aggregate health reports Healthy/Degraded/Failed/Unknown and retains active errors.

Renderer stalls and UI lockups are represented through the Renderer/UI degraded scopes and must surface an explicit degraded state rather than silently presenting stale success.

## D5 Integration
System 13 provides the visual degraded-state presentation contract. System 14 provides world-region context. System 16 receives service-crash Error-state transitions. System 22 contributes health/alert information. System 24 receives error events for observability. System 27 surfaces actionable error/degraded panels. System 28 preserves operator interaction paths where possible. System 29 provides accessible localized error/caption presentation. System 30 supplies performance/degradation signals. System 12 is the authoritative state-sync recovery source.

## D6 Failure modes
Repeated backend loss is idempotent; sync loss never reports confirmation; world recovery cannot be confirmed while authoritative connectivity is unavailable; unresolved critical errors keep aggregate health Failed; degraded mode cannot be exited while known backend/sync blockers remain.

## D7 Tests
1. Backend loss enters read-only cached mode.
2. Sync loss creates pending state and prevents false confirmation.
3. World load failure records region-specific failure and placeholder requirement.
4. Service crash records critical failure and Error-state requirement.
5. Renderer/UI degraded scopes remain visible and actionable.
6. RecoverWorldState rejects unavailable authoritative sync.
7. Successful recovery clears the world degraded state.
8. ResolveError removes an active error.
9. Aggregate health becomes Failed for any failed system.
10. Aggregate health becomes Degraded for degraded systems without failures.
11. Empty/invalid error inputs are rejected.
12. Verify no failure path is silently swallowed.

## D8 Acceptance
- [x] Backend loss
- [x] Read-only cached mode
- [x] Sync bridge loss
- [x] Pending local actions
- [x] World load failure
- [x] Placeholder-void contract
- [x] Service crash handling
- [x] System 16 Error-state integration
- [x] Renderer stall degraded scope
- [x] UI lockup degraded scope
- [x] Visible/honest/actionable degraded modes
- [x] World recovery from authoritative state sync
- [x] User-facing error surface
- [x] Aggregate health
- [x] Systems 13–30 only
- [x] No System 32+ references
