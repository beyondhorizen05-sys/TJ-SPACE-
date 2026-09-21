# TJ SPACE — System 2: Spatial World Kernel

## Purpose

Owns the spatial coordinate authority, world-space bounds, zone registration, spatial lookup, streaming origin, and deterministic zone load-state calculation.

## 3D presentation requirement

TJ SPACE is a real-time 3D spatial world. This kernel represents actual Unreal Engine world-space coordinates and bounds; it is not a 2D, pixel-art, tile-art, voxel, or sprite-based world. Spatial zones are volumetric 3D regions using FVector center/extent values. The visual appearance remains governed by the already-established TJ SPACE 3D foundation.

## Spatial contract

- Unreal world units: centimeters.
- Coordinate system: right-handed, Z-up.
- World origin: (0,0,0).
- Default streaming cell size: 25600 cm.
- Default loading range: 76800 cm.
- World Partition is required for initialized runtime worlds.

## Runtime behavior

UTJSpatialWorldKernel maintains registered spatial zones. ResolveZone performs deterministic bounds lookup. SetStreamingOrigin changes the spatial observation point. UpdateStreamingState evaluates every registered zone against the configured loading range and broadcasts only state transitions.

## Lower-system integration

The kernel consumes the visual foundation established by System 1 only through the existing Unreal project/runtime baseline. It does not redefine visual materials, rendering, or UI behavior.

## Invariants

1. Zone IDs are non-empty and unique.
2. Zone extents are strictly positive.
3. Streaming origin must contain finite coordinates.
4. A zone is loaded when its center is within the configured loading range.
5. State-change events fire only when a zone's loaded state changes.
