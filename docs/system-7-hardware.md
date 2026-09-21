# System 7 — Hardware, Disk & WiFi

## D1 — Scope

### Owns
- Physical block-device enumeration using Linux block-device/sysfs data.
- SMART health normalization and temperature/health readings.
- Volume creation, LVM, Linux MD RAID, filesystem formatting, resize, mount and unmount.
- WiFi scanning and configuration through NetworkManager.
- CPU, RAM, GPU and thermal inventory.
- Kernel driver/module discovery, loading and unloading.
- Destructive-operation confirmation and audit persistence.
- Volume metadata persistence in System 2 Patch-DB.
- Headless Rust APIs and RPC bindings.

### Explicitly does not own
- Service lifecycle, package handling, authentication policy, or persistence primitives outside System 2.
- Any 3D or graphical interface.

Integration is limited to Systems 1 and 2.

## D2 — Data model

Rust structures in `tjspace-core/src/hardware.rs`:

- `Disk`: id, path, model, serial, size_bytes, rotational, filesystem, mountpoint, health.
- `DiskHealthSummary`: status, temperature_c, power_on_hours, reallocated_sectors, pending_sectors, uncorrectable_sectors, smart_available.
- `SmartData`: normalized health fields plus attribute map and raw SMART JSON.
- `Volume`: id, name, device, disks, raid_level, filesystem, size_bytes, mountpoint, vg_name, lv_name, md_device, created_at.
- `WifiNetwork`: ssid, signal_percent, security, channel, frequency.
- `WifiCredentials`: password and optional identity.
- `CpuInfo`, `GpuInfo`, `ThermalReading`, `SystemHardware`.
- `DriverInfo`: module, loaded, description.
- `Confirmation`: operation_id, operation, issued_at, expires_at, confirmed.
- `AuditRecord`: id, operation, resource, destructive, confirmed, success, actor, timestamp, detail.

Patch-DB paths:
- `hardware.volumes.<volume-id>` — persisted volume metadata.
- `hardware.audit.<audit-id>` — immutable-by-convention operation records.

## D3 — Interface contract

Primary methods:

- `EnumerateDisks() -> Result<Vec<Disk>>`
- `GetDiskHealth(diskId) -> Result<DiskHealthSummary>`
- `GetSmartData(diskId) -> Result<SmartData>`
- `CreateVolume(name, disks[], raidLevel, fsType) -> Result<Volume>`
- `CreateVolumeConfirmed(name, disks[], raidLevel, fsType, confirmationId) -> Result<Volume>`
- `ResizeVolume(volumeId, newSize) -> Result<Volume>`
- `ResizeVolumeConfirmed(volumeId, newSize, confirmationId) -> Result<Volume>`
- `DeleteVolume(volumeId) -> Result<()> `
- `DeleteVolumeConfirmed(volumeId, confirmationId) -> Result<()> `
- `MountVolume(volumeId, mountPoint) -> Result<()> `
- `UnmountVolume(volumeId) -> Result<()> `
- `ScanWifiNetworks() -> Result<Vec<WifiNetwork>>`
- `ConfigureWifi(ssid, credentials) -> Result<()> `
- `GetSystemHardware() -> Result<SystemHardware>`
- `GetDrivers() -> Result<Vec<DriverInfo>>`
- `LoadDriver(module) -> Result<()> `
- `UnloadDriver(module) -> Result<()> `
- `UnloadDriverConfirmed(module, confirmationId) -> Result<()> `
- `request_confirmation(operation, resource) -> Result<Confirmation>`
- `confirm_destructive_operation(operationId) -> Result<()> `

RPC methods are registered on the Core dispatcher for all primary operations plus confirmation, disk, hardware, WiFi, and driver management.

## D4 — Implementation

### Disk and SMART
- `lsblk -J -b` supplies device identity, model, serial, capacity, filesystem and mount information.
- Stable disk IDs are derived from the device path and are resolved back through `/sys/block`.
- `smartctl -Aj` is parsed into normalized health fields.
- ATA attributes such as reallocated, pending and uncorrectable sectors are normalized when present.
- NVMe percentage-used is retained in `SmartData.wear_percent`.

Linux exposes discovered block devices through `/sys/block`, which is the kernel-supported device model interface. citeturn0search1turn0search3

### Volumes
- `none` or `lvm`: LVM PVs -> VG -> LV.
- `raid0`, `raid1`, `raid5`, `raid6`, `raid10`: Linux MD RAID -> PV -> VG -> LV.
- Filesystems are formatted with a validated filesystem type.
- Resize uses LV growth/reduction followed by filesystem growth where supported.
- Mounted filesystem shrinking is refused.
- Volume metadata is written to System 2 Patch-DB after successful state changes.

Linux LVM provides PV/VG/LV abstraction, while Linux MD exposes RAID arrays as block devices. citeturn0search5turn0search0

### WiFi
- Network scanning uses `nmcli device wifi list --rescan yes`.
- Configuration uses NetworkManager's headless `nmcli device wifi connect` interface.
- Passwords are passed to the subprocess but never placed in audit records.

NetworkManager documents `nmcli` for headless systems and WiFi scan/connect operations. citeturn1search0turn1search1

### Hardware and drivers
- CPU/RAM data comes from Linux procfs.
- Thermal zones come from `/sys/class/thermal`.
- GPU inventory uses `lspci` when available.
- Loaded modules come from `/proc/modules`.
- Driver load/unload uses `modprobe` and `modprobe -r`.

### Safety
- Every destructive volume operation and driver unload requires a time-limited confirmation.
- Confirmation is bound to the operation/resource prefix and consumed only when the destructive operation executes.
- Destructive operations write an audit record before returning success.
- Command execution is bounded by timeouts and failures return errors instead of terminating the daemon.
- Device, filesystem, volume, mount-point and module identifiers are validated.

## D5 — Integration

System 1 integration:
- HardwareManager is owned by the Core.
- Hardware RPCs use the existing Core dispatcher and inherit its authenticated transport.

System 2 integration:
- Volume records are persisted as Patch-DB patches.
- Audit records are persisted as Patch-DB patches.
- Patch-DB revision changes therefore become visible through the existing Core state revision.

No other system is required for operation.

## D6 — Failure modes

- Missing `lsblk`, `smartctl`, `mdadm`, LVM, filesystem tools, `nmcli`, `lspci`, or `modprobe`: operation returns an explicit command failure.
- SMART unavailable or unsupported: normalized health becomes `unknown` rather than fabricating health.
- Mounted disk selected for volume creation: creation is refused.
- Invalid RAID/filesystem/name/device/module/mount point: validation fails before mutation.
- Missing, expired, unconfirmed, or operation-mismatched destructive confirmation: mutation is refused.
- Volume resize below current size while mounted: refused.
- Unsupported filesystem shrink: refused.
- WiFi command timeout or NetworkManager failure: error returned and no success audit is emitted.
- Patch-DB persistence failure: operation returns an error instead of claiming metadata was saved.

## D7 — Test cases

Implemented in `tjspace-core/tests/hardware.rs`:
1. Confirmation issuance and consumption.
2. Double-confirmation rejection.
3. Expired confirmation rejection.
4. Unsupported filesystem rejection.
5. Invalid driver-module rejection.
6. Invalid WiFi SSID rejection.
7. Patch-DB audit persistence.

The test suite is designed to run headlessly and does not require a physical disk mutation.

## D8 — Acceptance checklist

- [x] Disk enumeration returns model, serial, size and normalized health.
- [x] SMART data is normalized and raw data retained.
- [x] LVM-backed volume creation is implemented.
- [x] MD RAID-backed volume creation is implemented.
- [x] Filesystem formatting is implemented with allowlisted types.
- [x] Volume resize, deletion, mount and unmount are implemented.
- [x] Volume metadata is persisted in Patch-DB.
- [x] WiFi scanning and configuration are implemented through NetworkManager.
- [x] CPU, RAM, GPU and thermal inventory is implemented.
- [x] Driver discovery, loading and unloading are implemented.
- [x] Destructive operations require explicit confirmation.
- [x] Destructive operations are audit logged.
- [x] Headless RPC surfaces are registered through Core.
- [x] Failure paths return errors without daemon termination.
- [x] At least five automated tests cover safety and failure boundaries.
