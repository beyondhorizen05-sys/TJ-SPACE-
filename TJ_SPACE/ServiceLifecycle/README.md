# SYSTEM 7 — Service Lifecycle & Interaction

## D1 — Scope
Owns physical service start/stop controls, SDK action consoles, action feedback, task beacons and critical-door barriers, health monitors, service ticker logs, and configuration control rooms. It integrates only with Systems 1–6.

## D2 — Data model
FTJServiceRuntime, FTJSDKAction, FTJServiceTask, FTJHealthCheck, FTJServiceLogEntry, FTJServiceConfig, ETJServiceLifecycleState, ETJTaskSeverity.

## D3 — Interface
StartService(packageId)
StopService(packageId, graceful)
RenderSDKActions(packageId)
ExecuteAction(packageId, actionId, input)
RenderTasks(packageId)
RenderHealthChecks(packageId)
RenderServiceLogs(packageId)
RenderConfiguration(packageId)
Plus physical forced-stop lever hold/release and task resolution controls.

## D4 — Implementation
A real Unreal Actor owns the physical control-room surfaces and service interaction components. Unreal Actors are the native 3D world objects and Components provide renderable/interactive pieces. citeturn0search0turn0search2

Start is represented by a physical ignition sequence. Graceful stop uses a controlled shutdown console. Forced stop is armed but cannot complete until the physical lever has been held for at least 3 seconds.

SDK actions appear as physical consoles. Executing a registered action writes an in-world ticker-log entry and activates an action-feedback material parameter. Runtime material feedback uses Dynamic Material Instances, which Unreal supports for changing material parameters during gameplay. citeturn0search1turn0search3turn0search4

Tasks appear as alert beacons. A Critical unresolved task sets CriticalDoorBarrier01 to 1, creating a red barrier field across the main door. Resolving that task recalculates the barrier; it drops only when no Critical task remains unresolved.

Health checks are represented at building level. Logs are maintained per package. Configuration is represented by a physical control-room surface.

## D5 — Integration
System 1: established photorealistic materials and runtime visual foundation.
System 2: lifecycle interaction exists as 3D world actors.
System 3: package IDs correspond to service embodiments.
System 4: lifecycle visuals remain compatible with containerized service presentation.
System 5: action/health/log surfaces remain spatially represented.
System 6: package lifecycle begins from the acquired package identity.

## D6 — Failure modes
Unknown package rejects lifecycle operations. Starting an already running service is rejected. Graceful stop of an already stopped service is idempotent. Forced stop cannot complete before the 3-second lever hold. Unknown actions are rejected. Critical barriers remain active while any critical task is unresolved. Task resolution recalculates the door barrier.

## D7 — Tests
CI validates lifecycle states, forced-stop timing, no single-action forced stop, critical red barrier semantics, all eight visual surfaces, all required API names, and absence of higher-system references.

## D8 — Acceptance
- [x] Physical ignition
- [x] Controlled graceful shutdown
- [x] Forced-stop lever hold
- [x] 3-second forced-stop requirement
- [x] SDK action consoles
- [x] In-world action feedback
- [x] Critical task alert beacons
- [x] Red main-door barrier
- [x] Barrier drops only after critical tasks resolve
- [x] Building health monitors
- [x] Per-service ticker logs
- [x] Configuration control room
- [x] System 1–6 integration only
- [x] Machine-readable schema
- [x] Automated tests
