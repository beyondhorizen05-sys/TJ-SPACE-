import json
from pathlib import Path
R=Path(__file__).parents[1];D=json.loads((R/"HUD"/"HUDPanelsInspection.json").read_text())
def test_style(): assert D["hudStyle"]["flat2D"] is False and D["hudStyle"]["glassPanels"]
def test_binding(): assert "no tick polling" in D["binding"]
def test_overlays(): assert set(D["overlays"])=={"topology","xray","dependencies","debug"}
def test_search(): assert D["search"]=="universal search and navigate"
def test_api():
 h=(R/"HUD"/"HUDPanelsInspection.h").read_text()
 for n in D["functions"]: assert n in h
def test_no_higher_refs():
 r=(R/"HUD"/"README.md").read_text()
 for n in range(14,16): assert f"System {n}" not in r