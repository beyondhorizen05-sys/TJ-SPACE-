import json
from pathlib import Path

ROOT = Path(__file__).parents[1]
CONFIG = ROOT / "ServiceEmbodiment" / "ServiceEmbodiment.json"

STATES = {"stopped", "starting", "running", "error", "updating"}

def load_config():
    return json.loads(CONFIG.read_text())

def test_all_service_states_exist():
    data = load_config()
    assert set(data["states"]) == STATES

def test_transition_matrix_is_complete():
    data = load_config()
    transitions = {(x["from"], x["to"]) for x in data["transitionMatrix"]}
    assert len(transitions) == 25
    assert transitions == {(a, b) for a in STATES for b in STATES}

def test_all_non_self_transitions_are_animated():
    data = load_config()
    for item in data["transitionMatrix"]:
        if item["from"] != item["to"]:
            assert item["durationSeconds"] > 0
            assert item["curve"] in {"cubicEaseInOut", "cubicEaseOut", "quinticEaseOut"}

def test_self_transitions_are_noops():
    data = load_config()
    for item in data["transitionMatrix"]:
        if item["from"] == item["to"]:
            assert item["durationSeconds"] == 0.0
            assert item["curve"] == "linear"

def test_resource_manifest_is_non_negative():
    resources = load_config()["manifest"]["resources"]
    assert all(value >= 0 for value in resources.values())

def test_manifest_has_interfaces_dependencies_and_resources():
    manifest = load_config()["manifest"]
    assert "interfaces" in manifest
    assert "dependencies" in manifest
    assert "resources" in manifest
