import json
from pathlib import Path
R=Path(__file__).parents[1];D=json.loads((R/"Observability"/"LogsEventsObservability.json").read_text())
def test_mapping(): assert D["archive"]=="Archive tower" and D["droppedEventVisual"]=="derailed rail cars"
def test_topic(): assert D["eventTopicPrefix"]=="tjs."
def test_audio(): assert D["audio"]=="Archive audio profile"
def test_formats(): assert set(D["exportFormats"])=={"JSON","NDJSON","CSV"}
def test_retention(): assert D["retention"]=="shelf clearing"
def test_api():
 h=(R/"Observability"/"LogsEventsObservability.h").read_text()
 for n in ["BuildArchiveTower","StreamLogsToArchive","QueryArchive","RenderEventBus","RenderAuditTrail","ExportLogs","ConfigureLogRetention"]: assert n in h
def test_no_higher_refs():
 r=(R/"Observability"/"README.md").read_text()
 for n in range(12,16): assert f"System {n}" not in r