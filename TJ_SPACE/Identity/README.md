# SYSTEM 10 — Identity, Auth, Devices & Access Control

## D1
Owns device avatars, permission keycards, signed-request door authentication, local authentication passes, device-fleet gatehouse control, access history, and device revocation.

## D2
FTJDeviceIdentity, FTJKeycard, FTJLocalAuthSession, FTJAccessLedgerEntry, ETJDeviceState, ETJDoorAuthState.

## D3
RenderDeviceAvatar(deviceId), RenderKeycard(deviceId,permissions), AuthenticateDevice(deviceId,signature), RenderLocalAuthCookie(session), ManageDeviceFleet(), RenderAccessLog(). Additional registration/revocation functions support the owned device lifecycle.

## D4
Devices materialize as avatars. Permissions materialize as keycard stripes. Authentication compares the presented signed-request value with the registered device credential, rejects unknown/revoked devices, records every attempt, and opens the physical door on success. A successful authorization drives AuthorizationBeam01 and DoorOpen01. A failed request drives DeniedPlacard01 and leaves the door closed. Runtime Dynamic Material Instances are used for these state changes; Unreal documents MIDs as runtime-editable material instances. citeturn0search1turn0search4

Revoked devices change to the stone state and are removed from visibility. Local auth sessions become physical passes carrying validity state. The fleet control room exposes device and revoked-device counts. The access ledger records timestamp, device, result, and reason.

## D5
System 1 supplies the photorealistic material/runtime visual foundation.
System 2 supplies the 3D Citadel spatial context.
System 3 keeps service architecture compatible with identity-driven entry.
System 4 keeps container identity spatially compatible.
System 5 supplies the established network/security terrain context.
System 6 supplies package identity context.
System 7 supplies service interaction context.
System 8 supplies backup/service identity context.
System 9 supplies telemetry context for the gatehouse environment.

## D6
Unknown device: authentication denied and logged.
Revoked device: authentication denied and logged.
Empty signature: denied and logged.
Wrong signature: denied and logged.
Empty device ID: avatar/keycard operations reject.
Invalid local session: physical pass is rendered inactive.
Duplicate registration: rejected.
Revocation: device state becomes Revoked, stone transition activates, avatar visibility is removed.

## D7
1. Register and render an active device.
2. Render permission stripes.
3. Authenticate a matching signature and open the door.
4. Reject a wrong signature and show the red denial placard.
5. Revoke a device and verify stone/removal state.
6. Render a valid and revoked local auth pass.
7. Record access history and fleet counts.
8. Verify all required API surfaces and absence of higher-system references.

## D8
- [x] Device avatars
- [x] Permission keycard stripes
- [x] Signed-request authentication
- [x] Smooth authorization beam state
- [x] Invalid-signature red placard state
- [x] Local auth physical passes
- [x] Device fleet gatehouse
- [x] Physical access ledger
- [x] Revoked-device stone/removal state
- [x] Runtime material-state implementation
- [x] Machine-readable schema
- [x] Automated tests
- [x] System 1–9 integration
- [x] No higher-system references
