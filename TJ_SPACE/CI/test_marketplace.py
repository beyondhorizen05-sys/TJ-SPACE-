import json
from pathlib import Path
D=json.loads((Path(__file__).parents[1]/"Marketplace"/"MarketplaceDistrict.json").read_text())
def test_ed25519(): assert D["verification"]=="Ed25519"
def test_three_second_lever(): assert D["removal"]["leverHoldSeconds"]==3
def test_no_single_click_destroy(): assert D["removal"]["singleClickDestroys"] is False
def test_quarantine(): assert "red barrier" in D["quarantine"]
def test_registry(): assert isinstance(D["registry"]["sources"],list)
def test_api():
 h=(Path(__file__).parents[1]/"Marketplace"/"MarketplaceDistrict.h").read_text()
 for n in ["BuildMarketplaceDistrict","RenderRegistryCatalog","InspectPackage","InstallPackage","VerifyPackageSignature","UpdatePackage","RemovePackage","ManageRegistrySources"]: assert n in h
