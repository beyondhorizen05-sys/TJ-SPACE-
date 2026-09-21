import json
from pathlib import Path
R=Path(__file__).parents[1];D=json.loads((R/"SDK"/"VisualExtensionSDK.json").read_text())
def test_art_bible(): assert D["artBible"]["renderer"]=="Unreal Engine 5" and D["artBible"]["geometry"]=="Nanite" and D["artBible"]["lighting"]=="Lumen" and D["artBible"]["shading"]=="PBR" and D["artBible"]["toneMapping"]=="ACES"
def test_forbidden(): assert set(D["artBible"]["forbidden"])=={"pixel art","cel shading","voxel geometry","unlit materials"}
def test_signing(): assert D["signing"]=="signed visual mod distribution"
def test_hot_reload(): assert "without process restart" in D["hotReload"]
def test_registry(): assert D["registry"]=="signed visual extension manifests"
def test_api():
 h=(R/"SDK"/"VisualExtensionSDK.h").read_text()
 for n in D["functions"]: assert n in h
def test_validator(): 
 from SDK.VisualCompliance import ValidateVisualCompliance
 assert not ValidateVisualCompliance({"mod_id":"bad","signature":"x","assets":["pixel art"]})["compliant"]
def test_no_forward_refs():
 for p in [R/"SDK"/"README.md",R/"SDK"/"VisualExtensionSDK.json"]:
  s=p.read_text()
  for n in range(16,20): assert f"System {n}" not in s