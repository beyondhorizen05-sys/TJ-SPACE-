import json
from pathlib import Path
R=Path(__file__).parents[1]
D=json.loads((R/"BackupVault"/"BackupVault.json").read_text())
def test_differential(): assert D["backupMode"]=="differential" and D["differentialDissolve"]
def test_interlock(): assert D["restoreInterlock"]["holdSeconds"]==2
def test_delete_warning(): assert "only restore point" in D["deletionWarning"]
def test_schedule(): assert D["schedule"]=="calendar obelisk"
def test_api():
 h=(R/"BackupVault"/"BackupVault.h").read_text()
 for n in ["BuildBackupVault","RenderBackupTarget","CreateBackup","RenderBackupProgress","RestoreBackup","VerifyBackupIntegrity","DeleteBackup","RenderBackupSchedule"]: assert n in h
def test_no_higher_refs():
 r=(R/"BackupVault"/"README.md").read_text()
 for n in range(9,16): assert f"System {n}" not in r