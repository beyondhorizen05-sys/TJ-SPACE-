# SYSTEM 11 — Logs, Events & Observability

## D1
Owns the Archive tower, live log ingestion, archive search, event-bus railway, per-actor audit ribbons, physical export crystals, retention shelf clearing, and Archive audio profile.

## D2
FTJLogRecord, FTJArchiveFilter, FTJRetentionPolicy, FTJExportRange, ETJLogFormat, ETJRetentionAction.

## D3
BuildArchiveTower(); StreamLogsToArchive(source); QueryArchive(filter); RenderEventBus(); RenderAuditTrail(actorId); ExportLogs(format,range); ConfigureLogRetention(policy).

## D4
Logs become streams of light entering the Archive. Archive queries filter stored records by source, topic prefix, and message contents. Event topics are validated against the tjs.<domain>.<event> convention. The event railway is a physical network; dropped-event presentation is reserved for derailed rail cars. Audit trails become actor-specific ribbons. Exports become physical data crystals. Retention clearing marks the shelf-clearing state.

Entering the Archive toggles a dedicated audio profile. Unreal supports spatial audio volumes and interpolated transitions for area-based audio processing; Audio Gameplay Volumes provide component-based area effects and enter/exit behavior. citeturn0search0turn0search5

## D5
System 1 supplies the visual/material foundation.
System 2 supplies the Citadel 3D world.
System 3 supplies service actor context.
System 4 supplies container actor context.
System 5 supplies network terrain context.
System 6 supplies package identity context.
System 7 supplies service interaction context.
System 8 supplies backup/service context.
System 9 supplies telemetry context.
System 10 supplies device/access identity context.

## D6
Empty sources reject ingestion. Invalid topics are rejected. Invalid export ranges are rejected. Empty actor IDs reject audit rendering. Negative retention ages reject policy changes. Retention clearing removes records older than the configured age. No record is silently accepted with a malformed event topic.

## D7
1. Build the Archive tower.
2. Ingest a source and create a tjs.* topic.
3. Query by source/topic/message.
4. Render event railway.
5. Render an actor audit ribbon.
6. Export a time range to a physical crystal state.
7. Configure shelf-clearing retention.
8. Verify audio-profile state.
9. Verify required API surfaces and no higher-system references.

## D8
- [x] Archive tower
- [x] Live light-stream ingestion
- [x] Physical archive search contract
- [x] Event railway
- [x] tjs.<domain>.<event> convention
- [x] Derailed-rail-car dropped-event representation
- [x] Actor audit ribbons
- [x] Physical data crystals
- [x] Retention shelf clearing
- [x] Archive audio profile
- [x] Machine-readable schema
- [x] Automated tests
- [x] CI workflow
- [x] Systems 1–10 integration
- [x] No higher-system references
