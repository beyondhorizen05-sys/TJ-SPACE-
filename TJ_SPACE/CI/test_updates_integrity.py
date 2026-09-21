import json
from pathlib import Path
R=Path(__file__).parents[1];D=json.loads((R/"Updates"/"UpdatesSigningIntegrity.json").read_text())
def test_mapping(): assert D["delivery"]=="update notification delivery" and D["renovation"]=="renovation crew"
def test_gate(): assert D["signatureGate"]=="mandatory verified seal"
def test_rollback(): assert D["rollback"]=="architectural restoration"
def test_merkle(): assert "cracked corrupt branches" in D["merkle"]
def test_history(): assert D["history"]=="miniature architectural models"
def test_api():
 h=(R/"Updates"/"UpdatesSigningIntegrity.h").read_text()
 for n in ["RenderUpdateAvailable","RenderUpdateInProgress","VerifyUpdateSignature","RenderRollback","RenderPackageIntegrityCheck","RenderUpdateHistory"]: assert n in h
def test_no_higher_refs():
 r=(R/"Updates"/"README.md").read_text()
 for n in range(13,16): assert f"System {n}" not in r