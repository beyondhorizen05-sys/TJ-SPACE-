# TJ SPACE — System 3: Service Embodiment Layer

## D1 — Scope

This layer turns an installed package manifest into a persistent photorealistic 3D service building. It owns the building shell, enterable interior modules, physical interface exits, dependency conduits, resource-footprint visualization, lifecycle-state architecture, and graceful archival.

It does not own package acquisition, networking policy, authentication, telemetry collection, logging, updates, HUD behavior, avatars, or extension/modding contracts.

## D2 — Data model

The runtime model is declared in ServiceEmbodimentLayer.h:

- FTJServiceManifest
- FTJServiceInterfaceManifest
- FTJServiceDependencyManifest
- FTJServiceResourceManifest
- FTJServiceTransitionSpec
- ETJServiceState
- ETJServiceTransitionCurve

A service manifest supplies the building/interior mesh paths, interface definitions, dependency definitions, and CPU/memory/storage/network footprint.

## D3 — Interface contract

The required public surface is:

```cpp
ATJServiceBuilding* InstantiateServiceBuilding(
    const FString& PackageId,
    const FTJServiceManifest& Manifest
);

bool BindServiceState(
    const FString& PackageId,
    ETJServiceState State
);

bool RenderServiceInterior(const FString& PackageId);
bool DisplayServiceInterfaces(const FString& PackageId);
bool ShowServiceDependencies(const FString& PackageId);
bool ShowResourceDraw(const FString& PackageId);
bool ArchiveServiceBuilding(const FString& PackageId);
```

## D4 — Implementation

### Photorealistic building embodiment

InstantiateServiceBuilding validates the package identity, creates an Unreal Actor, loads the manifest-provided building mesh, and constructs the architectural shell. The shell remains governed by the System 1 photorealistic material and rendering foundation.

### Enterable interior

Interior meshes and module meshes from the manifest are attached beneath an interior scene root with collision enabled. This makes the interior an actual 3D spatial component rather than a UI representation.

### Physical service interfaces

Each declared service interface becomes a physical architectural exit positioned around the building perimeter. Protocol, port, and activity values are available as material parameters.

### Dependency pipes

Each dependency becomes a physical conduit attached to the building. Throughput controls the conduit activity parameter.

### Resource footprint

CPU, memory, storage, and network values are normalized into a physical footprint scale and resource-activity parameter.

### Graceful archive

Archiving hides the building, disables collision and ticking, and retains its runtime object rather than abruptly destroying it. This allows the service embodiment to leave the active Citadel while preserving the building object for controlled lifecycle handling.

### Lifecycle architecture — BindServiceState

Unreal Dynamic Material Instances are used for runtime parameter changes; Epic documents these as runtime-editable material instances suitable for changing visual properties during gameplay. citeturn0search0turn0search2

The five architectural states are:

| State | Architecture |
|---|---|
| stopped | dormant structure, low activity, high residual stress |
| starting | rising architectural activity, controlled pulse |
| running | stable illuminated architecture and active interfaces |
| error | visibly stressed architecture and degraded activity |
| updating | controlled intermediate activity with update pulse |

No lifecycle state is swapped instantaneously. Every state change enters the transition matrix and interpolates continuously on Tick.

Transition curves:

- Linear — unchanged/self-state.
- CubicEaseInOut — smooth acceleration/deceleration.
- CubicEaseOut — fast architectural response with a soft settle.
- QuinticEaseOut — rapid initial response with a long smooth settle.

Every ordered state transition has an explicit duration in `ServiceEmbodiment.json`, including all 25 matrix entries.

### Exact transition durations

| From | To | Curve | Duration |
|---|---|---|---:|
| stopped | starting | CubicEaseInOut | 1.20s |
| stopped | running | QuinticEaseOut | 2.40s |
| stopped | error | CubicEaseInOut | 0.90s |
| stopped | updating | CubicEaseInOut | 1.40s |
| starting | stopped | CubicEaseInOut | 0.80s |
| starting | running | CubicEaseOut | 1.10s |
| starting | error | CubicEaseOut | 1.10s |
| starting | updating | CubicEaseInOut | 0.95s |
| running | stopped | QuinticEaseOut | 0.75s |
| running | starting | CubicEaseInOut | 0.70s |
| running | error | CubicEaseInOut | 0.65s |
| running | updating | CubicEaseInOut | 1.00s |
| error | stopped | CubicEaseInOut | 1.30s |
| error | starting | CubicEaseOut | 1.00s |
| error | running | QuinticEaseOut | 2.00s |
| error | updating | CubicEaseInOut | 0.90s |
| updating | stopped | CubicEaseInOut | 1.00s |
| updating | starting | CubicEaseInOut | 0.85s |
| updating | running | QuinticEaseOut | 1.60s |
| updating | error | CubicEaseOut | 0.80s |

Self-transitions are zero-duration no-op transitions.

## D5 — Integration

### System 1

- Uses the existing photorealistic UE5 rendering/material foundation.
- Uses runtime material parameters rather than replacing the material architecture.
- Architectural state is expressed through dynamic material parameters such as `ServiceStress01`, `ServiceActivity01`, and `ServiceStateBlend`.
- This follows the System 1 material-instance model; Unreal's documentation confirms dynamic material instances can be modified at runtime. citeturn0search0turn0search6

### System 2

- Service buildings are Unreal 3D actors placed inside the spatial world.
- The embodiment layer is compatible with System 2's spatial registration boundary.
- Building locations remain ordinary Unreal world-space transforms.
- Service building archival does not redefine the Citadel spatial coordinate system.

No higher-system integration is introduced.

## D6 — Failure modes

1. Empty package ID → instantiation rejected.
2. Manifest/package ID mismatch → instantiation rejected.
3. Missing Unreal world → instantiation rejected.
4. Invalid building mesh → building is destroyed and instantiation fails.
5. Missing interior assets → shell remains valid; interior construction reports failure.
6. Invalid lifecycle transition request on an archived building → rejected.
7. Duplicate package ID → existing embodiment is returned instead of creating a duplicate.
8. Invalid material slot → architectural state continues while that individual material parameter is skipped.
9. Archive request for an unknown package → rejected.
10. State changes are never applied as instantaneous visual swaps; transition timing is governed by the explicit transition matrix.

## D7 — Test cases

1. **Manifest validation** — empty package ID is rejected.
2. **Building instantiation** — valid manifest produces one service building.
3. **Duplicate protection** — repeated package instantiation does not create a second building.
4. **Lifecycle transition** — stopped → starting uses a 1.20-second CubicEaseInOut transition.
5. **Lifecycle transition** — running → error uses a 0.65-second CubicEaseInOut transition.
6. **Lifecycle transition** — error → running uses a 2.00-second QuinticEaseOut transition.
7. **Interior generation** — manifest interior modules become attached 3D components with collision.
8. **Interface generation** — every manifest interface creates a physical exit.
9. **Dependency visualization** — every dependency creates a physical conduit.
10. **Resource visualization** — CPU/memory/storage/network values affect the physical footprint.
11. **Graceful archive** — archived buildings become hidden and non-collidable without abrupt destruction.

## D8 — Acceptance checklist

- [x] Installed-package manifest becomes a photorealistic 3D building.
- [x] Building shell uses manifest-defined geometry.
- [x] Enterable interior geometry is generated from the manifest.
- [x] Network interfaces become physical architectural exits.
- [x] Dependencies become physical conduits.
- [x] CPU/memory/storage/network footprint becomes spatial geometry.
- [x] Five distinct service states are implemented.
- [x] BindServiceState uses continuous animation.
- [x] All 20 non-self state transitions have explicit curves and durations.
- [x] Five self-transitions are explicitly defined.
- [x] Runtime material parameters drive architectural state.
- [x] Graceful service archival is implemented.
- [x] System 1 integration is implemented.
- [x] System 2 spatial integration boundary is respected.
- [x] No higher-system references are introduced.
- [x] Machine-readable manifest and transition contracts are committed.
- [x] CI/test configuration is prepared for the Service Embodiment Layer.
