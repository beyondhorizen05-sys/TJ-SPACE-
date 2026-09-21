# SYSTEM 34 — Build & Release Pipeline

## D1 Scope
Owns monorepo build orchestration, Rust cross-compilation, web UI builds, SDK publication, installer images, release packaging, signing/verification, publication and CI release stages. Primary environment: Debian/Ubuntu; macOS secondary; Windows via WSL2.

## D2 Data model
FTJReleaseArtifact and FTJRelease track artifact name/path/hash/signature/target, version/channel, artifact set, signed state and published state.

## D3 Interface
BuildRustBinaries(targetTriple); BuildWebUIs(appId); BuildSdk(); BuildInstallerImage(targetTriple); PackageRelease(version,channel); SignRelease(artifact,releaseKey); PublishRelease(release); VerifyRelease(artifact).

## D4 Implementation
Supported targets: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu and riscv64gc-unknown-linux-gnu. Rust artifacts cover tjs-box, tjs-container and tjs-cli. Web UI, SDK and installer artifacts are release-manifest inputs. Packaging requires version, channel and artifacts. Publication fails unless the release is signed; verification fails for missing signatures.

## D5 Integration
Systems 1–12 supply backend, RPC, containers, packaging, registry, hardware/installer, OS update, headless API, federation and sync artifacts. Systems 13–33 supply the world/runtime, services, navigation, networking, marketplace, lifecycle, backup, telemetry, identity, observability, updates, HUD, presence, SDK, audio, accessibility, performance and degradation deliverables assembled into the release. System 25 consumes signed/integrity-verified release artifacts. No system beyond 33 is referenced.

## D6 Failure modes
Unsupported targets, empty app/version/channel, missing artifacts, missing release key, unsigned publication and unverifiable artifacts fail closed.

## D7 Test cases
1. Build each Rust target.
2. Reject unsupported target.
3. Build a web UI.
4. Build @tjspace/start-sdk.
5. Build each installer target.
6. Package a version/channel.
7. Reject empty release.
8. Sign an artifact.
9. Reject signing without a key.
10. Verify a signed artifact.
11. Reject unsigned verification.
12. Reject unsigned publication.
13. Publish a signed release.
14. Validate PR/main/tag/nightly CI definitions.

## D8 Acceptance
- [x] Monorepo orchestration
- [x] x86_64
- [x] ARM64
- [x] RISCV64
- [x] tjs-box
- [x] tjs-container
- [x] tjs-cli
- [x] Web UI builds
- [x] @tjspace/start-sdk
- [x] Installer image builds
- [x] Release packaging
- [x] Signing
- [x] Verification
- [x] Update-server publication contract
- [x] PR build/test
- [x] Main version/tag/build
- [x] Tag sign/publish
- [x] Nightly cross-architecture matrix
- [x] Debian/Ubuntu primary
- [x] macOS secondary
- [x] Windows WSL2
- [x] Systems 1–33 integration only
