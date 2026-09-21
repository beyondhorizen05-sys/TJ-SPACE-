# TJ SPACE — System 2: Spatial World Kernel

## D1 — Scope

The Spatial World Kernel owns the Citadel world assembly boundary, the bidirectional backend-ID/3D-actor registry, district streaming state, environment reactivity from system health, layout persistence, spatial querying, and world reset.

It does not own service behavior, application lifecycle, authentication, networking policy, logs, updates, HUD implementation, avatars, or extension/modding contracts.

## D2 — Data model

Runtime structures are declared in `SpatialWorldKernel.h`:

- `FTJSpatialServerConfig`: world ID, initial origin, district cell size, loading range, streaming flag.
- `FTJSpatialDistrictCoord`: integer X/Y district coordinate.
- `FTJSpatialEntityState`: backend ID, actor transform, district ID, registration state.
- `FTJSpatialDistrictState`: district ID, coordinate, center, loaded state.
- `FTJSpatialWorldLayout`: world ID, streaming origin, entity snapshots, district snapshots.
- `FTJSystemHealthState`: normalized health, load, and fault values.
- `FTJSpatialQueryFilter`: backend prefix, district ID, optional 3D bounds.
- `FTJSpatialQueryResult`: matching entity states and match count.

The authoritative configuration is `SpatialWorldKernel.json`; its machine-readable contract is `SpatialWorldKernel.schema.json`.

## D3 — Interface contract

The public callable surface is:

```cpp
bool BootCitadel(const FTJSpatialServerConfig& serverConfig);
bool RegisterSpatialEntity(const FString& backendId, AActor* actor);
bool StreamDistrict(const FTJSpatialDistrictCoord& coord);
bool SetEnvironmentState(const FTJSystemHealthState& systemHealth);
bool PersistWorldLayout(const FTJSpatialWorldLayout& layout);
FTJSpatialQueryResult QuerySpatialState(const FTJSpatialQueryFilter& filter) const;
bool ResetCitadel(bool preserveData);
AActor* ResolveBackendId(const FString& backendId) const;
FString ResolveActorBackendId(AActor* actor) const;
```

Delegates report entity, district, and environment state transitions.

## D4 — Implementation

`SpatialWorldKernel.cpp` provides the concrete implementation. Boot validates the Unreal world and streaming capability, establishes the Citadel origin, initializes the district registry, restores persisted district layout when valid, and activates the origin district.

The registry keeps both backend-to-actor and actor-to-backend maps. Registration is rejected for empty IDs, duplicate IDs, invalid actors, or actors already registered.

District streaming uses integer world-space cells and the configured loading radius. The implementation is compatible with Unreal World Partition's grid/cell streaming model; World Partition itself uses a persistent world subdivided into streamable grid cells and distance-based streaming sources. citeturn0search0turn0search3

Environment health is normalized to 0..1 and drives the existing System 1 material scalar `DegradedStrained`, plus optional health/load/fault material parameters when present.

Persistence writes a deterministic JSON snapshot under the project's Saved directory. Reset either preserves the snapshot or removes it, then reconstructs the origin district.

## D5 — Integration

The only cross-system integration is with System 1:

- System 1's UE5 rendering/material baseline remains authoritative.
- System 1's `DegradedStrained` scalar is driven from `1 - Health01`.
- System 1's existing 3D material and pipeline rules remain unchanged.
- No replacement UI or rendering stack is introduced here.

World Partition is used only as the Unreal runtime spatial substrate for the Citadel. Unreal documents World Partition as a distance-based grid-cell streaming system and exposes streaming configuration and streaming-source concepts. citeturn0search0turn0search2

## D6 — Failure modes

1. Invalid server configuration → boot returns false; no world mutation occurs.
2. Missing/non-streamable world when streaming is enabled → boot returns false.
3. Empty or duplicate backend ID → registration returns false.
4. Invalid actor or already-registered actor → registration returns false.
5. Non-finite health values → environment update returns false.
6. Corrupt or mismatched persisted layout → layout load is rejected without replacing the live registry.
7. Persistence write failure → persistence/reset operation returns false.
8. Invalid district cell size → district streaming returns false.

## D7 — Test cases

1. Boot rejects a non-positive district cell size.
2. Boot rejects a non-finite initial origin.
3. Registry rejects duplicate backend IDs.
4. Bidirectional registry resolves backend ID to actor and actor to backend ID.
5. District streaming loads the requested district and unloads districts outside the configured radius.
6. Health 0.0 maps to `DegradedStrained=1.0`; health 1.0 maps to 0.0 stress.
7. Spatial query filters by backend prefix, district, and optional 3D bounds.
8. Reset with preserve=true keeps the persisted snapshot; preserve=false deletes it.

## D8 — Acceptance checklist

- [x] Citadel boot path exists and validates runtime prerequisites.
- [x] Backend ID ↔ 3D actor registry is bidirectional.
- [x] District coordinates and streaming state are persistent runtime state.
- [x] Environment health drives System 1's `DegradedStrained` material parameter.
- [x] Layout persistence uses a concrete JSON file format.
- [x] Spatial query API returns deterministic entity snapshots.
- [x] World reset supports preserve-data and destructive modes.
- [x] 3D world-space coordinates are authoritative; no pixel-art/tile-art/voxel representation is used.
- [x] CI tests cover happy, boundary, and failure behavior.
