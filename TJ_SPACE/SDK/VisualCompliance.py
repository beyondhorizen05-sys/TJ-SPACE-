import json
from dataclasses import dataclass,asdict
FORBIDDEN=("pixel art","cel shading","voxel geometry","unlit materials")
@dataclass
class ComplianceReport:
    mod_id:str
    compliant:bool
    errors:list
    warnings:list
def ValidateVisualCompliance(mod:dict)->dict:
    text=json.dumps(mod).lower()
    errors=[f"Rejected Art Bible violation: {x}" for x in FORBIDDEN if x in text]
    if not mod.get("signature"): errors.append("Missing signed visual-mod seal")
    return asdict(ComplianceReport(mod.get("mod_id",""),not errors,errors,[]))
