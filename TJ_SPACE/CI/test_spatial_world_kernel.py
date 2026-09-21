import json
from pathlib import Path

ROOT = Path(__file__).parents[1]
CONFIG = ROOT / "SpatialWorld" / "SpatialWorldKernel.json"

def load_config():
    return json.loads(CONFIG.read_text())

def test_world_is_real_3d():
    data = load_config()
    assert data["world"]["units"] == "centimeters"
    assert data["world"]["coordinateSystem"] == "right-handed-z-up"
    assert data["citadel"]["streamingEnabled"] is True

def test_citadel_streaming_defaults_are_valid():
    data = load_config()
    assert data["citadel"]["districtCellSize"] > 0
    assert data["citadel"]["districtLoadingRange"] > 0
    assert data["citadel"]["districtLoadingRange"] >= data["citadel"]["districtCellSize"]

def test_origin_district_exists():
    data = load_config()
    district = data["districts"][0]
    assert district["id"] == "district_0_0"
    assert district["x"] == 0
    assert district["y"] == 0

def test_environment_range_is_normalized():
    health = load_config()["environment"]
    assert 0.0 <= health["health01"] <= 1.0
    assert 0.0 <= health["load01"] <= 1.0
    assert 0.0 <= health["fault01"] <= 1.0

def test_entity_ids_are_unique():
    data = load_config()
    ids = [item["backendId"] for item in data["entities"]]
    assert len(ids) == len(set(ids))

def test_schema_has_all_required_runtime_surfaces():
    schema = json.loads((ROOT / "SpatialWorld" / "SpatialWorldKernel.schema.json").read_text())
    assert set(schema["required"]) == {
        "version", "world", "citadel", "entities", "districts", "environment"
    }

def test_no_2d_or_pixel_art_world_mode_is_configured():
    data = load_config()
    serialized = json.dumps(data).lower()
    assert "pixel" not in serialized
    assert "voxel" not in serialized
