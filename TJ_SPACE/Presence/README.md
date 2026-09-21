# SYSTEM 14 — Avatar, Camera, Navigation & Presence

## D1
Owns the operator avatar, camera modes, navigation/fast travel, building entry, unified interaction, multi-device presence, camera bookmarks, and VR mode.

## D2
ETJCameraMode, ETJLocomotionMode, FTJPresencePeer, FTJCameraBookmark, FTJInteractionRequest, ATJOperatorAvatar, UTJAvatarCameraNavigation.

## D3
SpawnOperatorAvatar(deviceId); SetCameraMode(mode); NavigateTo(targetId); EnterBuilding(packageId); InteractWithObject(actor,interaction); RenderPresence(peers); SetCameraBookmark(name,transform); EnableVRMode().

## D4
VR is first-class. The implementation defines motion-controller interaction, teleport locomotion, snap turn, and seated mode as mandatory comfort capabilities. Epic's UE VR Template documents teleport and snap-turn locomotion and motion-controller grabbing, while OpenXR provides cross-device controller profiles and grip/aim poses. citeturn0search0turn0search1

The operator is an Unreal Pawn, matching Unreal's model of a Pawn as the physical representation of a player. citeturn0search2
EnableVRMode selects VR camera mode and teleport locomotion. Seated mode is explicitly available as a locomotion/tracking configuration contract. Epic's VR guidance distinguishes seated and standing tracking origins. citeturn0search8turn0search11

Interaction prompts are defined as in-world spatial elements, never screen-space. Every interaction is routed through /api/v1 backend RPC semantics.

Navigation, building entry, and object interaction therefore produce explicit backend routes rather than local-only visual actions.

## D5
System 1 supplies the 3D visual foundation.
System 2 supplies spatial world coordinates.
System 3 supplies service buildings and entry targets.
System 4 supplies container interaction context.
System 5 supplies network terrain.
System 6 supplies package identity/context.
System 7 supplies service interaction context.
System 8 supplies backup/restore context.
System 9 supplies telemetry context.
System 10 supplies device identity and access context.
System 11 supplies observability context.
System 12 supplies update/integrity context.
System 13 supplies holographic in-world panel context.

## D6
Empty device/target/package/interaction values reject requests. Unsupported or empty RPC routes reject execution. VR activation requires a valid backend route. Bookmarks require names. Presence is state-replaced atomically.

## D7
1. Spawn operator avatar from a device ID.
2. Switch camera modes.
3. Navigate using backend RPC route.
4. Enter a service building through backend RPC.
5. Execute an object interaction through backend RPC.
6. Publish multi-device presence.
7. Create a camera bookmark.
8. Enable VR.
9. Verify snap-turn, teleport, and seated comfort contracts.
10. Verify in-world prompt contract.
11. Verify no higher-system references.

## D8
- [x] Operator avatar
- [x] Camera mode system
- [x] Navigation/fast-travel RPC
- [x] Building entry RPC
- [x] Unified interaction RPC
- [x] Multi-device presence
- [x] Camera bookmarks
- [x] VR flagship mode
- [x] Motion-controller first-class contract
- [x] Snap turn
- [x] Teleport locomotion
- [x] Seated mode
- [x] In-world interaction prompt contract
- [x] Backend RPC routing
- [x] Machine-readable schema
- [x] Automated tests
- [x] Systems 1–13 integration
- [x] No higher-system references
