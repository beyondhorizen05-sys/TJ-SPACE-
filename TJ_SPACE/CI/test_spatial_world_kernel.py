import json
from pathlib import Path

ROOT = Path(__file__).parents[1]

def load_config():
    return json.loads((ROOT / "SpatialWorld" / "SpatialWorldKernel.json").read_text())

def zone_contains(zone, point):
    c = zone["center"]
    e = zone["extent"]
    return all(abs(point[i] - c[i]) <= e[i] for i in range(3))

def test_world_units_and_coordinate_system():
    data = load_config()
    assert data["world"]["units"] == "centimeters"
    assert data["world"]["coordinateSystem"] == "right-handed-z-up"

def test_streaming_defaults_are_positive():
    data = load_config()
    assert data["streaming"]["enabled"] is True
    assert data["streaming"]["cellSize"] > 0
    assert data["streaming"]["loadingRange"] > 0

def test_core_zone_contains_origin():
    data = load_config()
    core = data["zones"][0]
    assert zone_contains(core, [0, 0, 0])

def test_core_zone_boundary_is_inclusive():
    data = load_config()
    core = data["zones"][0]
    e = core["extent"]
    assert zone_contains(core, [e[0], e[1], e[2]])

def test_duplicate_zone_ids_are_rejected_by_configuration():
    data = load_config()
    ids = [z["id"] for z in data["zones"]]
    assert len(ids) == len(set(ids))
