import json
from pathlib import Path
root=Path(__file__).parents[1]
D=json.loads((root/"ServiceLifecycle"/"ServiceLifecycle.json").read_text())
def test_states(): assert set(D["states"])=={"Stopped","Starting","Running","Stopping","Error"}
def test_forced_stop(): assert D["forcedStop"]=={"leverHoldSeconds":3,"singleActionStops":False}
def test_critical_barrier(): assert D["criticalTaskBarrier"]=={"color":"red","blocksMainDoor":True,"dropsWhenResolved":True}
def test_surfaces(): assert len(D["surfaces"])==8
def test_api(): 
 h=(root/"ServiceLifecycle"/"ServiceLifecycle.h").read_text()
 for n in ["StartService","StopService","RenderSDKActions","ExecuteAction","RenderTasks","RenderHealthChecks","RenderServiceLogs","RenderConfiguration"]: assert n in h
def test_no_higher_system_refs():
 r=(root/"ServiceLifecycle"/"README.md").read_text()
 assert "System 8" not in r and "System 9" not in r