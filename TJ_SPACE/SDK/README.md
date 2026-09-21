# SYSTEM 15 — SDK, Modding & Extension

## D1
Owns the package visual specification extension, Citadel themes, visual plugin API, signed visual-mod packaging, hot visual reload, Art Bible compliance validation, and visual extension registry.

## D2
FTJServiceVisualSpec, FTJCitadelThemeSpec, FTJVisualPluginSpec, FTJVisualModManifest, FTJComplianceReport, FTJVisualExtensionManifest.

## D3
CreateServiceVisualSpec(packageId); CreateCitadelTheme(spec); CreateVisualPlugin(spec); PackageVisualMod(dir); HotReloadVisual(packageId); ValidateVisualCompliance(mod); RegisterVisualExtensionPoint(manifest).

## D4
The SDK fixes the visual contract to UE5, Nanite, Lumen, PBR, 4K–8K textures and ACES, with explicit rejection of pixel art, cel shading, voxel geometry and unlit materials. Unreal's plugin model supports code/data extensions, while Editor Python can automate content-production workflows. citeturn0search4turn0search0

Visual mods require a signed seal and are represented by a machine-readable manifest. HotReloadVisual provides the runtime/editor integration point for visual iteration; actual asset replacement remains subject to the engine/plugin lifecycle.

ValidateVisualCompliance emits a detailed report containing every detected violation and warning. Unreal's Data Validation system is designed for custom scripted asset rules and CI validation, including C++/Blueprint/Python validators. citeturn0search1turn0search3

## D5
System 1 supplies the Art Bible and master visual foundation.
System 2 supplies the Citadel spatial world.
System 3 supplies service-building embodiment.
System 4 supplies container visualization.
System 5 supplies network terrain.
System 6 supplies package acquisition and signature context.
System 7 supplies lifecycle interaction.
System 8 supplies backup/restore context.
System 9 supplies telemetry context.
System 10 supplies identity/access context.
System 11 supplies observability.
System 12 supplies update/signing/integrity context.
System 13 supplies HUD/panel/inspection surfaces.
System 14 supplies avatar/camera/navigation/VR presence.

## D6
Empty package/spec/manifest fields reject creation. Missing mod signatures fail compliance. Forbidden visual descriptors produce explicit compliance errors. Empty extension registration fields reject registry insertion. Hot reload rejects empty package IDs.

## D7
1. Create service visual spec.
2. Create Citadel theme.
3. Create visual plugin.
4. Package a visual mod.
5. Hot-reload a named package.
6. Reject pixel art.
7. Reject cel shading.
8. Reject voxel geometry.
9. Reject unlit materials.
10. Reject unsigned visual mods.
11. Produce detailed compliance report.
12. Register signed extension manifest.
13. Verify Art Bible constants and required APIs.

## D8
- [x] Package visual specification extension
- [x] Citadel theme packs
- [x] Visual plugin API
- [x] Signed visual mod distribution
- [x] Hot reload integration point
- [x] Art Bible enforcement
- [x] Pixel-art rejection
- [x] Cel-shading rejection
- [x] Voxel-geometry rejection
- [x] Unlit-material rejection
- [x] Detailed compliance reports
- [x] Visual extension registry
- [x] Machine-readable schema
- [x] Automated compliance tests
- [x] CI validation
- [x] Systems 1–14 integration
- [x] No higher-system references
