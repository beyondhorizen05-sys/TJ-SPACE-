import json
from pathlib import Path
R=Path(__file__).parents[1];D=json.loads((R/"Telemetry"/"TelemetryHealthMetrics.json").read_text())
def test_facilities(): assert set(D["facilities"])=={"cpu","ram","disk","network","uptime"}
def test_graph_free(): assert "no traditional graphs" in D["graphReplacement"]
def test_strain(): assert D["strainBinding"]=="DegradedStrained"
def test_alerts(): assert D["alerts"]=="world events"
def test_comparison(): assert D["containerComparison"]=="per-container comparative telemetry"
def test_api():
 h=(R/"Telemetry"/"TelemetryHealthMetrics.h").read_text()
 for n in ["RenderSystemMetrics","RenderCPULoad","RenderMemoryUsage","RenderDiskUsage","RenderNetworkThroughput","RenderUptime","TriggerSystemAlert","RenderContainerMetrics"]: assert n in h
def test_no_higher_refs():
 r=(R/"Telemetry"/"README.md").read_text()
 for n in range(10,16): assert f"System {n}" not in r