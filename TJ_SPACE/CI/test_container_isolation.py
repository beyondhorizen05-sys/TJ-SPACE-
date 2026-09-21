import json
from pathlib import Path

P=Path(__file__).parents[1]/"ContainerIsolation"/"ContainerIsolation.json"
D=json.loads(P.read_text())

def test_boundary_modes():
    assert set(D["boundaryModes"])=={"solid","xray","isolation","traffic"}

def test_health_states():
    assert set(D["healthStates"])=={"unknown","healthy","degraded","critical","stopped"}

def test_traffic_is_gated():
    assert "gated handshake" in D["trafficRule"]
    assert "free-flowing pipe" in D["trafficRule"]

def test_resource_limits():
    limits=D["manifest"]["limits"]
    assert {"cpuLimit01","memoryLimit01","pidsLimit01","ioLimit01"}<=set(limits)

def test_volume_mounts():
    assert D["manifest"]["volumeMounts"][0]["containerPath"]

def test_subcontainer_topology():
    assert isinstance(D["manifest"]["subcontainerIds"],list)
