# SYSTEM 6 — Marketplace District & Package Acquisition

## D1
Owns the walkable marketplace hall, registry catalog visualization, package inspection, installation staging, Ed25519 signature-seal visualization, update renovation, two-step uninstall/data custody, and registry-source management.

## D2
Models: FTJPackageManifest, FTJRegistrySource, FTJTargetPlot, FTJRemovalState, ETJPackageVerification, ETJInstallStage, ETJRemovalStage.

## D3
Required functions:
- BuildMarketplaceDistrict()
- RenderRegistryCatalog(registryUrl)
- InspectPackage(packageId)
- InstallPackage(packageId,targetPlot)
- VerifyPackageSignature(packageId)
- UpdatePackage(packageId,version)
- RemovePackage(packageId)
- ManageRegistrySources()

## D4
The hall is a real Unreal actor. Catalog entries are physical panels. Inspection creates an inspection stand. Unverified packages create a red quarantine barrier and cannot install. Verified packages receive a physical Ed25519 seal. Installation is staged at the target plot. Updates appear as renovation activity.

Unreal's Asset Manager provides discovery and controlled loading of primary assets, while Data Assets provide structured package metadata. citeturn0search1turn0search2

The package contract records Ed25519 public-key/signature material. Verification is a cryptographic gate: only a package that reaches Verified state may cross the installation boundary.

Removal is deliberately destructive only after:
1. arming the removal action;
2. holding the physical data-custody lever continuously for at least 3 seconds.
A click or release before 3 seconds never destroys data.

## D5
System 1: established photorealistic UE5 material/runtime foundation.
System 2: marketplace is a 3D district actor in the spatial world.
System 3: package identity aligns with service-package identity.
System 4: installed package/container relationships remain visually compatible with container pods.
System 5: registry/network endpoints can be represented in the existing networking terrain.

## D6
Empty IDs are rejected. Unknown packages cannot be inspected, installed, updated, or removed. Unverified packages cannot install. Missing signature/public key leaves a package quarantined. Lever release before 3 seconds cannot destroy data. Registry management rejects empty registry identifiers or URLs.

## D7
CI validates Ed25519 contract, three-second lever rule, no single-click destruction, red quarantine requirement, registry structure, and all required API names.

## D8
- [x] Walkable marketplace hall
- [x] Live registry catalog representation
- [x] Pre-install inspection
- [x] Physical installation sequence
- [x] Ed25519 signature seal
- [x] Red quarantine for unverified packages
- [x] Updates represented as renovation
- [x] Two-step uninstall/data custody
- [x] Three-second physical lever
- [x] No single-click data destruction
- [x] Registry source management
- [x] System 1–5 integration boundary
- [x] Machine-readable contract
- [x] Automated tests
