import unreal
MATERIALS=["M_Metal_Brushed","M_Glass_Smart","M_Concrete_Sovereign","M_Emissive_Circuit","M_Holo_Panel","M_Data_Flow","M_Shield_TLS","M_Energy_Conduit"]
ROOT="/Game/TJ_SPACE/Materials"

def _scalar(mat,name,default):
 p=unreal.MaterialEditingLibrary.create_material_expression(mat,unreal.MaterialExpressionScalarParameter,-900,200); p.parameter_name=name; p.default_value=default; return p

def BuildMaterialLibrary():
 tools=unreal.AssetToolsHelpers.get_asset_tools()
 for name in MATERIALS:
  path=f"{ROOT}/{name}"; mat=unreal.EditorAssetLibrary.load_asset(path)
  if not mat: mat=tools.create_asset(name,ROOT,unreal.Material,unreal.MaterialFactoryNew())
  _scalar(mat,"DegradedStrained",0.0)
  mat.set_editor_property("shading_model",unreal.MaterialShadingModel.MSM_DEFAULT_LIT)
  unreal.EditorAssetLibrary.save_asset(path)
 return True

if __name__=="__main__": BuildMaterialLibrary()