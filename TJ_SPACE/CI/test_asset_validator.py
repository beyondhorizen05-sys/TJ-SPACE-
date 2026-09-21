from validate_asset_compliance import ValidateAssetCompliance

def valid():return {"renderer":"Unreal Engine 5","geometry":"Nanite","lighting":"Lumen","shading":"PBR","toneMapping":"ACES","material":{"name":"M_Holo_Panel","parameters":{"DegradedStrained":0.0}},"style":"photorealistic"}
def test_valid():assert ValidateAssetCompliance(valid())==[]
def test_missing_scalar():a=valid();del a["material"]["parameters"]["DegradedStrained"];assert "material must expose DegradedStrained scalar" in ValidateAssetCompliance(a)
def test_bad_renderer():a=valid();a["renderer"]="Other";assert "renderer must be Unreal Engine 5" in ValidateAssetCompliance(a)
def test_bad_material():a=valid();a["material"]["name"]="M_Unknown";assert "material is not in the master material library" in ValidateAssetCompliance(a)
def test_forbidden_style():a=valid();a["style"]="pixel_art";assert "forbidden style: pixel_art" in ValidateAssetCompliance(a)