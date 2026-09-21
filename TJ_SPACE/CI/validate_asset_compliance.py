#!/usr/bin/env python3
import json,sys
from pathlib import Path
REQUIRED={"M_Metal_Brushed","M_Glass_Smart","M_Concrete_Sovereign","M_Emissive_Circuit","M_Holo_Panel","M_Data_Flow","M_Shield_TLS","M_Energy_Conduit"}
FORBIDDEN={"pixel_art","cel_shading","voxels","low_poly","unlit"}

def ValidateAssetCompliance(asset):
 e=[]
 for k,v in {"renderer":"Unreal Engine 5","geometry":"Nanite","lighting":"Lumen","shading":"PBR","toneMapping":"ACES"}.items():
  if asset.get(k)!=v:e.append(f"{k} must be {v}")
 m=asset.get("material",{})
 if m.get("name") not in REQUIRED:e.append("material is not in the master material library")
 if "DegradedStrained" not in m.get("parameters",{}):e.append("material must expose DegradedStrained scalar")
 s=str(asset.get("style","")).lower()
 for x in FORBIDDEN:
  if x in s:e.append(f"forbidden style: {x}")
 return e

def main():
 if len(sys.argv)!=2:return 2
 e=ValidateAssetCompliance(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")))
 for x in e:print("ERROR:",x,file=sys.stderr)
 return 1 if e else 0
if __name__=="__main__":raise SystemExit(main())