# System 8 — OS Installer & First Boot

## D1 — Scope
Owns bare-metal installation from a live image, GPT partitioning, EFI/root/data/swap layout validation, base-system deployment, bootloader configuration, first-boot setup, reinstall restore, and post-install verification. It is headless/scriptable and has no graphical-world dependency. Integration is limited to Systems 1–7.

## D2 — Data model
- `PartitionSpec`: name, size_bytes, filesystem, mountpoint, esp, swap.
- `PartitionLayout`: partitions, gpt.
- `InstallOptions`: layout, base_source, hostname, admin_public_key, network, headless, restore_source, confirm_id.
- `NetworkSetup`: ssid, password.
- `InstallResult`: install_id, disk, root, verified, bootloader_configured.
- `VerificationReport`: bootable, root_present, daemon_present, config_present, checks.
- Installer confirmation records are persisted under `installer.confirmations.<id>` in Patch-DB.
- Installation metadata is persisted under `installer.installation`.

## D3 — Interface contract
Implemented:
- `RunInstaller(targetDisk, options)`
- `PartitionDisk(diskId, layout)`
- `WriteBaseSystem(partition, source)`
- `ConfigureBootloader(diskId)`
- `FirstBootWizard()`
- `RestoreFromBackup(backupSource)`
- `VerifyInstallation()`
- `RequestInstallerConfirmation(operation, resource)`
- `ConfirmInstallerOperation(operationId)`

The Core RPC dispatcher exposes all primary installer operations and confirmation operations.

## D4 — Implementation
### Partitioning
Uses GPT and `sgdisk` to create:
- one EFI System Partition
- one root partition
- optional data partition(s)
- optional swap partition

Layout validation requires exactly one EFI partition and a root mountpoint.

### Base system
The live environment mounts the selected root partition and extracts the supplied root filesystem archive into it. Installation sources are validated before execution.

### Boot
UEFI bootloader installation uses `grub-install --target=x86_64-efi` and generates the boot configuration.

The Linux kernel's documented boot process supports an early userspace/initramfs phase that locates and mounts the real root filesystem before normal userspace starts. citeturn0search0turn0search3

### First boot
Interactive mode prompts for:
- hostname
- admin public key

Headless mode is enabled with `TJS_HEADLESS_FIRST_BOOT=1` and consumes:
- `TJS_HOSTNAME`
- `TJS_ADMIN_PUBLIC_KEY`
- optional `TJS_NETWORK_SSID`
- optional `TJS_NETWORK_PASSWORD`

It writes first-boot configuration, hostname, optionally configures NetworkManager, and enables/starts the headless daemon.

### Restore
A supplied backup archive is extracted into the installed root during reinstall flow.

### Verification
Checks:
- target root exists
- boot directory exists
- `/usr/local/bin/tjs-box` exists
- `/etc/tjspace/tjspace.yaml` exists

## D5 — Integration
System 1:
- Installer is owned by the Core.
- Installer RPC methods use the existing Core authenticated dispatcher.

System 2:
- Installation metadata and confirmation state are persisted in Patch-DB.

System 7:
- The installer validates and operates on physical disk targets using the established disk identity conventions.

## D6 — Failure modes
- Installer invoked outside the designated live environment: rejected unless running in dry-run mode.
- Invalid target disk/partition/source: rejected before mutation.
- Missing EFI or root partition: rejected.
- Missing explicit installation confirmation: rejected.
- Failed partitioning command: installation stops with an error.
- Failed root filesystem deployment: installation stops.
- Failed bootloader configuration: installation is not reported successful.
- Failed restore: installation result is not reported successful.
- Verification detects missing daemon/config/boot tree and reports failure.

Destructive installation requires an explicit confirmation record bound to both operation and target.

## D7 — Test cases
Implemented in `tjspace-core/tests/os_installer.rs`:
1. EFI/root layout validation.
2. Confirmation issuance and confirmation.
3. Missing installation confirmation rejection.
4. Invalid partition-target rejection.
5. Post-install verification detecting an incomplete installation.

All tests are headless and dry-run; they do not modify a physical disk.

## D8 — Acceptance checklist
- [x] Live-image execution guard.
- [x] Fully scriptable headless mode.
- [x] GPT disk partitioning.
- [x] EFI partition.
- [x] Root partition.
- [x] Data partition layout support.
- [x] Swap partition layout support.
- [x] Root filesystem deployment.
- [x] UEFI bootloader configuration.
- [x] Interactive first-boot wizard.
- [x] Headless first-boot configuration.
- [x] Hostname setup.
- [x] Admin public-key setup.
- [x] Network setup.
- [x] Initial daemon enable/start.
- [x] Backup restore during reinstall.
- [x] Installation integrity verification.
- [x] Explicit destructive confirmation.
- [x] Patch-DB persistence.
- [x] Core RPC integration.
- [x] Automated tests.
