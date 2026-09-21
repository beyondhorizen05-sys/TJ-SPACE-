import json
from pathlib import Path
R=Path(__file__).parents[1];D=json.loads((R/"Identity"/"IdentityAuthDevices.json").read_text())
def test_visual_mapping(): assert D["device"]=="avatar" and D["keycard"]=="permission stripes"
def test_auth_visuals(): assert D["authorizationBeam"]=="green" and D["deniedPlacard"]=="red"
def test_revocation(): assert D["revocation"]=="stone and removal"
def test_surfaces(): assert D["fleet"]=="gatehouse control room" and D["ledger"]=="physical access ledger"
def test_api():
 h=(R/"Identity"/"IdentityAuthDevices.h").read_text()
 for n in ["RenderDeviceAvatar","RenderKeycard","AuthenticateDevice","RenderLocalAuthCookie","ManageDeviceFleet","RenderAccessLog"]: assert n in h
def test_no_higher_refs():
 r=(R/"Identity"/"README.md").read_text()
 for n in range(11,16): assert f"System {n}" not in r