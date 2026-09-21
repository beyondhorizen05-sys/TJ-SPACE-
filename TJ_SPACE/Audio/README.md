# TJ SPACE — System 26: Audio Design System

## D1 — Scope
Owns photoreal spatial audio, district ambience, service machinery/hum, alerts, UI feedback, operator footsteps, future voice playback binding, dialog ducking, acoustic profiles and audio routing. It does not own visual world creation, service lifecycle state, telemetry generation, or UI construction.

## D2 — Data model
FTJSpatialAudioParams, FTJAudioReverbProfile, FTJAudioZoneState, FTJVoiceStreamBinding, channel routing, ambience beds, service-transition cue map, alert-signature map and acoustic profiles.

## D3 — Interface contract
LoadAmbienceBed(zoneId, bedAsset); SetAudioZone(zoneId); PlaySpatialSound(actor, cue, params); PlayUISound(panel, cue); DuckForDialog(conversationId, amount); SetMasterVolume(channel, value); ApplyAcousticReverb(zoneId, reverbProfile); StreamVoice(agentId, audioStream); TriggerAlarm(severity, source).

## D4 — Implementation
Spatial playback uses Unreal Audio Components and attached sound playback; UI playback uses non-spatialized 2D playback. Zone transitions use AudioComponent FadeIn/FadeOut. Reverb uses the world's audio device reverb manager with priority, volume and fade time. Sound-mix overrides provide dialog ducking and channel volume control. Voice streaming is an explicit placeholder binding that records the agent and byte count without inventing a decoder.

## D5 — Integration
System 13 supplies the established UE5 world/audio spatial foundation. System 14 supplies Citadel spatial regions. System 16 supplies service-state transition delegates. Systems 17–21 can route machinery, network, marketplace, lifecycle and backup interaction cues. System 22 maps alert severity/source to mandatory sound signatures. System 23 routes authentication/access feedback. System 24 routes observability notifications. System 25 routes update/signing/integrity notifications. No system numbered 27 or higher is referenced.

## D6 — Failure modes
Invalid or missing assets reject operations; unknown audio zones reject transitions; missing alert signatures reject alarms; missing transition cues fail coverage validation; unavailable audio device fails reverb/master-volume operations; empty voice streams are rejected; duck values are clamped.

## D7 — Test cases
1. Load two ambience beds and verify crossfade.
2. Play spatial machinery from a service actor.
3. Play a console/UI sound.
4. Apply dialog ducking.
5. Set channel volumes at 0, 0.5 and 1.
6. Apply distinct Tor, Archive and Vault reverb profiles.
7. Trigger Info, Warning and Critical alarms.
8. Reject an alarm without a signature.
9. Register every service transition cue and verify coverage.
10. Bind a voice stream and verify byte count.
11. Verify a service transition invokes its mapped cue.
12. Verify invalid inputs fail safely.

## D8 — Acceptance checklist
- [x] All nine requested functions implemented.
- [x] Spatial audio and attenuation path implemented.
- [x] Per-zone ambience and crossfade implemented.
- [x] Service transition cue binding implemented.
- [x] Alert signature routing implemented.
- [x] UI sound path implemented.
- [x] Dialog ducking implemented.
- [x] Channel/master volume routing implemented.
- [x] Runtime acoustic reverb implemented.
- [x] Voice stream placeholder implemented.
- [x] Tor, Archive and Vault profiles defined.
- [x] Integration is limited to Systems 13–25.
