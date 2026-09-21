import json
from pathlib import Path
R=Path(__file__).parents[1];D=json.loads((R/"Presence"/"AvatarCameraNavigation.json").read_text())
def test_vr(): assert D["vr"]["motionControllers"]=="first-class" and D["vr"]["snapTurn"] and D["vr"]["teleport"] and D["vr"]["seatedMode"]
def test_interaction(): assert D["interaction"]=="every interaction maps to /api/v1 backend RPC"
def test_camera(): assert set(D["camera"])=={"first_person","third_person","orbit","vr"}
def test_locomotion(): assert set(D["locomotion"])=={"smooth","snap_turn","teleport","seated"}
def test_inworld(): assert D["vr"]["inWorldPrompts"]
def test_api():
 h=(R/"Presence"/"AvatarCameraNavigation.h").read_text()
 for n in ["SpawnOperatorAvatar","SetCameraMode","NavigateTo","EnterBuilding","InteractWithObject","RenderPresence","SetCameraBookmark","EnableVRMode"]: assert n in h
def test_no_higher_refs():
 r=(R/"Presence"/"README.md").read_text()
 for n in range(15,16): assert f"System {n}" not in r