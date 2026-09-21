# SYSTEM 4 — Container Isolation Visualization (LXC)

## D1
Owns the 3D visualization of LXC containers as sealed pods, X-ray/isolation boundaries, subcontainer topology, volume mounts, cgroup governors, health vital signs, and explicit gated cross-container handshakes.

The visual layer is not the LXC runtime itself. LXC uses Linux namespaces and cgroups to isolate/restrict system containers, and its documentation identifies mount, PID, UTS, IPC, user, and network namespaces as container contexts. citeturn0search0turn0search8

## D2
The data model contains:
- FTJContainerManifest
- FTJContainerVolumeMount
- FTJContainerResourceLimit
- ETJContainerBoundaryMode
- ETJContainerHealth
- ATJContainerPod

Cgroup/resource-limit visualization includes CPU, memory, PID, and I/O limit ratios. LXC documents CPU, memory and PID cgroup limits as relevant controls for preventing resource exhaustion. citeturn0search1

## D3
Required API:
- RenderContainerPod(containerId)
- ShowContainerBoundary(containerId, mode)
- RenderSubcontainers(packageId)
- ShowVolumeMounts(containerId)
- VisualizeResourceLimits(containerId)
- ShowContainerHealth(containerId)

## D4
The pod is a real Unreal Actor with:
- sealed shell
- layered X-ray boundary
- subcontainer nodes
- volume-mount conduits
- physical governor bars
- animated vital-sign element

Boundary modes are explicit:
- Solid
- XRay
- Isolation
- Traffic

Cross-container traffic is represented by a **gated handshake**, never by a free-flowing pipe. The boundary is visually emphasized before any handshake can be displayed.

Volume mounts are represented as controlled mount channels. LXC documentation describes host volumes mounted into container paths through bind-style mount configuration. citeturn0search2

Health uses continuous pulse animation:
- Healthy: stable strong pulse
- Degraded: slower/reduced pulse
- Critical: rapid unstable pulse
- Stopped: no pulse
- Unknown: neutral pulse

## D5
System 1 integration:
- uses the established photorealistic UE5 material/rendering foundation
- runtime material parameters drive X-ray, isolation, governor, volume, and vital-sign visuals

System 2 integration:
- pods are ordinary 3D actors in the Citadel spatial world
- their transforms remain compatible with the spatial-world coordinate model

System 3 integration:
- container pods can be associated with a service package through PackageId
- container topology is therefore visually subordinate to its owning service embodiment

## D6
Failure handling:
1. Empty container ID → rejected.
2. Unknown manifest → pod creation rejected.
3. Invalid world → pod creation rejected.
4. Duplicate pod → existing pod returned.
5. Unknown pod for visualization → operation rejected.
6. Missing material parameter → only that visual parameter is skipped.
7. Empty subcontainer list → topology remains valid with no child nodes.
8. Empty volume list → no mount channels are created.
9. Out-of-range resource values → clamped to 0–1.
10. Critical health → vital pulse changes continuously rather than an instant visual swap.

## D7
CI tests verify:
1. all boundary modes exist
2. all health states exist
3. traffic rule explicitly requires a gated handshake
4. resource-limit schema exists
5. volume-mount schema exists
6. subcontainer topology fields exist

## D8
Acceptance checklist:
- [x] Each LXC container is rendered as a sealed 3D pod.
- [x] Solid boundary exists.
- [x] X-ray overlay exists.
- [x] Isolation boundary exists.
- [x] Traffic boundary mode exists.
- [x] Cross-container traffic is explicitly gated.
- [x] No free-flowing cross-container pipe is used.
- [x] Multi-subcontainer topology is rendered.
- [x] Volume mounts are rendered.
- [x] CPU cgroup limit is visualized.
- [x] Memory cgroup limit is visualized.
- [x] PID limit is visualized.
- [x] I/O limit is visualized.
- [x] Container health has animated vital signs.
- [x] System 1 integration is implemented.
- [x] System 2 integration is implemented.
- [x] System 3 integration is implemented.
- [x] No later-system integration is introduced.
- [x] Machine-readable contracts are committed.
