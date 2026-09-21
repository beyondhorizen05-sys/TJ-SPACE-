# SYSTEM 9 — Telemetry, Health & System Metrics

## D1
Owns the Citadel utility-infrastructure visualization for CPU, RAM, disk, network throughput, uptime, world-event alerts, and comparative per-container telemetry. It deliberately replaces traditional graphs with readable physical infrastructure.

## D2
Models: FTJSystemMetrics, FTJContainerMetrics, FTJSystemAlert, ETJAlertSeverity.

## D3
Required functions:
- RenderSystemMetrics()
- RenderCPULoad()
- RenderMemoryUsage()
- RenderDiskUsage()
- RenderNetworkThroughput()
- RenderUptime()
- TriggerSystemAlert(severity,source)
- RenderContainerMetrics(containerId)

## D4
CPU is a power plant. RAM is pressure-vessel infrastructure. Disk is storage-silo infrastructure. Network throughput is a network exchange. Uptime is an uptime monument. Alerts materialize as world events. Containers receive comparative facility markers.

RenderSystemMetrics is implemented as the central material-state application path. CPU, RAM, disk, and network values are applied directly to their physical facilities rather than requiring a graph or panel. Unreal supports runtime scalar material parameters through Dynamic Material Instances, allowing material state to change during gameplay. citeturn0search2turn0search10

Each facility receives a DegradedStrained scalar derived from its normalized load. The current threshold model is 0.55: values at or below it have zero strain; values above it ramp linearly to 1.0 at full load.

The actor ticks so live metric state can continuously update the world. Unreal's Actor Tick executes per frame when enabled. citeturn0search0turn0search1

## D5
System 1: drives the required DegradedStrained material parameter.
System 2: telemetry facilities occupy the 3D Citadel.
System 3: service embodiments remain visually compatible with live health state.
System 4: per-container metrics are displayed comparatively.
System 5: network throughput is represented by the network exchange.
System 6: package/service identities remain available as spatial context.
System 7: service health state can be represented through the same physical world metrics.
System 8: backup-related load can be reflected through the same infrastructure state.

## D6
Missing world prevents Citadel construction. Empty container IDs reject comparative rendering. Metric values are clamped to 0–1. Empty alert source rejects alert creation. Uptime is maintained from the telemetry actor's runtime. Every metric update reapplies the physical facility state and strain scalar.

## D7
CI validates five physical metric facilities, graph replacement, DegradedStrained binding, world-event alerts, comparative container telemetry, all required APIs, and absence of higher-system references.

## D8
- [x] Power plant CPU display
- [x] Pressure-vessel RAM display
- [x] Storage-silo disk display
- [x] Network-exchange throughput display
- [x] Uptime monument
- [x] World-event alerts
- [x] Per-container comparative telemetry
- [x] Traditional graphs replaced by world infrastructure
- [x] Real-time facility updates
- [x] DegradedStrained binding
- [x] System 1–8 integration
- [x] Machine-readable schema
- [x] Automated tests
- [x] CI workflow
- [x] No System 10+ references
