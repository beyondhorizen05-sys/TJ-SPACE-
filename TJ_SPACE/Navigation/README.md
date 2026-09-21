# TJ SPACE — System 15: Navigation Mesh & Pathfinding

## D1 — Scope
Owns Citadel navigation generation, hierarchical navigation queries, localized dynamic-obstacle invalidation, navigation-area tags, traversal links, walkable-area queries, incremental rebuild requests, and connectivity validation. It does not own world geometry creation or rendering.

## D2 — Data model
FTJNavGenerationParams, FTJNavRegion, FTJPathOptions, FTJNavPathResult, FTJNavIslandReport, ETJNavAreaTag, dynamic obstacle bounds, and ATJNavTraversalLink.

## D3 — Interface contract
GenerateNavMesh(region, params); TagNavArea(region, tags); QueryPath(from, to, opts); AddDynamicObstacle(actor, bounds); RemoveDynamicObstacle(actor); GetWalkableArea(bounds); RebuildIncremental(changedRegion); ValidateConnectivity(). Doorway/elevator traversal uses registered off-mesh navigation links.

## D4 — Implementation
Uses Unreal NavigationSystem/RecastNavMesh over the Citadel world's collision geometry. Runtime generation is dynamic, path queries project endpoints to navigation, and dirty regions request localized rebuilding. The explicit operator navigation profile is 42 cm radius, 180 cm height, 45 cm step, 44 degree slope. Unreal documents collision-derived, tiled navigation and localized rebuilds. citeturn0search5turn0search8

## D5 — Integration
System 13: consumes the established UE5 rendering/world foundation. System 14: consumes its centimeter, right-handed Z-up Citadel coordinate convention and the same world collision geometry. No later system is referenced.

## D6 — Failure modes
Invalid parameters, missing navigation system, non-walkable endpoints, no route, disallowed partial route, invalid obstacle bounds, invalid rebuild region, and unavailable nav data return explicit failure states.

## D7 — Test cases
Parameter rejection; valid generation; all five area tags; reachable path; unreachable path; obstacle add/remove; incremental rebuild; connectivity validation; traversal-link registration; smoothing endpoint preservation.

## D8 — Acceptance checklist
[x] Eight requested functions implemented. [x] Configurable radius, step and slope. [x] Dynamic tiled navigation. [x] Collision geometry source. [x] Five area tags. [x] Polyline path result. [x] Dynamic obstacle dirtying. [x] Door/elevator traversal-link representation. [x] Walkable area and incremental rebuild APIs. [x] Connectivity API. [x] No later numbered system referenced.