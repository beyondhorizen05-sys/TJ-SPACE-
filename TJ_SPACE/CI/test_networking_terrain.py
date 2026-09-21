import json
from pathlib import Path
D=json.loads((Path(__file__).parents[1]/"NetworkingTerrain"/"NetworkingTerrain.json").read_text())
def test_four_strategies(): assert set(D["strategies"])=={"LAN","Tor","Clearnet","WireGuard"}
def test_all_layers(): assert {"streets","subterraneanTunnels","openWorldBridge","privateRoads","tlsShields","dnsBeams","trafficPulses"}<=set(D["layers"])
def test_tor_modalities(): assert all(D["torMode"][k] for k in ("lighting","audio","fog"))
def test_construction(): assert D["construction"]["enableDisable"] and D["construction"]["blendSeconds"]>0
def test_traffic_rule(): assert "light pulses" in D["trafficRule"] and "coexist" in D["trafficRule"]
def test_api_names():
    h=(Path(__file__).parents[1]/"NetworkingTerrain"/"NetworkingTerrain.h").read_text()
    for n in ["BuildLANTerrain","BuildTorTunnels","BuildClearnetHighway","BuildVPNRoads","RenderTLSCertificate","VisualizeDNSResolution","ShowTrafficFlow","ConfigureNetworkStrategy"]: assert n in h
