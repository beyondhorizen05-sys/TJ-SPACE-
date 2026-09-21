# SYSTEM 13 — HUD, Panels, Inspection & Debug Overlays

## D1
Owns the holographic HUD, service/container inspection panels, network topology, structural X-ray, dependency graph, developer debug layers, and universal search/navigation.

## D2
FTJHUDState, FTJPanelState, FTJDebugLayer, ETJOverlayMode, and UTJHUDClientStateStore.

## D3
RenderHUD(state); RenderServicePanel(packageId); RenderContainerPanel(containerId); RenderNetworkOverlay(mode); RenderXRayOverlay(mode); RenderDependencyOverlay(); RenderDebugOverlay(layer); RenderSearchAndNavigate(query).

## D4
The visual contract is physically present holography: thin glass panels, emissive data lines, depth-aware placement, and no flat 2D game HUD. State is held in a client state store. UI consumers subscribe to state-change notifications rather than Tick-polling. Epic's UMG Viewmodel documentation describes event-driven View Bindings that notify widgets when Viewmodel variables change, while Epic's UMG optimization guidance recommends event-driven updates instead of per-frame attribute polling. citeturn0search1turn0search9

Service and container panels publish inspection state. Network, X-ray, and dependency modes are mutually explicit overlay states. Debug layers are independently toggled. Search publishes a query into the client state store so navigation consumers can react to it.

## D5
System 1 supplies the holographic material/UI foundation.
System 2 supplies the spatial world context.
System 3 supplies service architecture inspection context.
System 4 supplies container inspection context.
System 5 supplies network topology context.
System 6 supplies package acquisition context.
System 7 supplies service interaction context.
System 8 supplies backup context.
System 9 supplies telemetry state for HUD values.
System 10 supplies identity/access inspection context.
System 11 supplies logs/events observability context.
System 12 supplies update/signing/integrity inspection context.

## D6
Empty target/query rejects the request. Unsupported overlay mode rejects the request. State-store creation failure prevents rendering. UI state is not refreshed by Tick; publishers explicitly emit state changes.

## D7
1. Publish HUD state and receive a state-change event.
2. Render service inspection.
3. Render deep container inspection.
4. Enable topology overlay.
5. Enable structural X-ray overlay.
6. Enable dependency overlay.
7. Enable a developer debug layer.
8. Publish universal search query.
9. Verify no Tick-polling contract.
10. Verify no higher-system references.

## D8
- [x] Thin glass holographic HUD
- [x] Emissive data lines
- [x] No flat 2D game HUD
- [x] Client state store
- [x] Event-driven state publication
- [x] No Tick polling
- [x] Service inspector
- [x] Deep container inspector
- [x] Network topology overlay
- [x] Structural X-ray overlay
- [x] Dependency overlay
- [x] Developer debug layers
- [x] Universal search/navigation state
- [x] Machine-readable schema
- [x] Automated tests
- [x] Systems 1–12 integration
- [x] No higher-system references
