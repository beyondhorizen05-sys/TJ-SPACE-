# TJ SPACE — System 29: Accessibility & Localization

## D1 — Scope
Owns internationalization, locale loading/fallback, keyboard-layout registration, colorblind semantic status remapping, motion comfort, screen-reader descriptions, subtitles/captions, and UI text scaling.

## D2 — Data model
FTJKeyboardLayout, FTJMotionComfortOptions, FTJCaptionStyle, FTJStatusColorMap, locale state, active keyboard layout, subtitle-channel set and UI scale.

## D3 — Interface contract
LoadLocale(localeCode); ApplyLocale(localeCode); GetAvailableLocales(); RegisterKeyboardLayout(layout); SetActiveLayout(layout); EnableColorblindMode(mode); ConfigureMotionComfort(opts); DescribeEntity(actor); EnableSubtitles(channel); ConfigureCaptionStyle(style); ScaleUIText(scale). Additional LocalizeKey and GetStatusColorMap expose the accessibility binding layer.

## D4 — Implementation
Uses Unreal FInternationalization and FText for culture-aware runtime text. Unreal's localization system uses FText for user-facing localized text and supports runtime culture switching and prioritized culture fallback. citeturn0search0turn0search1turn0search6

Locale fallback resolves an exact culture, then its two-letter language, then English. Keyboard layouts are registered as explicit data records. Colorblind mode changes the semantic status palette for Info/Warning/Critical/Healthy/Denied/Authorized rather than relying on hue alone. Motion options expose snap-turn, teleport, vignette and FOV limits. Entity descriptions return localized FText suitable for a screen-reader bridge. Subtitle channels and caption style are independently configurable. UI scale is constrained to a readable 0.75–3.0 range.

## D5 — Integration
System 13: localized visual/UI labels and accessibility styling. System 14: localized spatial entity descriptions. Systems 15–25: navigation, audio, service, container, networking, marketplace, lifecycle, backup, telemetry, identity, observability and update surfaces use the locale/accessibility contract for user-facing strings. System 26: voice/subtitle/caption channel binding. Systems 16, 22 and 23 use the semantic status-color map so color changes preserve meaning. No system numbered 30 or higher is referenced.

## D6 — Failure modes
Invalid locale falls back through language to English; missing layout is rejected; invalid FOV/scale is rejected; empty entity returns empty FText; invalid caption style is rejected; unknown localization keys retain an explicit fallback rather than exposing an unresolved null value.

## D7 — Test cases
1. Load exact locale.
2. Load unavailable regional locale and verify language fallback.
3. Load unavailable language and verify English fallback.
4. Register/set keyboard layout.
5. Test all four colorblind modes and verify all semantic status entries change as defined.
6. Configure snap turn, teleport, vignette and FOV clamp.
7. Describe a valid and invalid actor.
8. Enable each subtitle channel and configure caption style.
9. Scale UI at minimum, normal and maximum; reject out-of-range values.
10. Resolve localized keys and verify FText output.
11. Verify status-color map exposes Info/Warning/Critical/Healthy/Denied/Authorized independently of raw hue.
12. Verify Systems 16, 22 and 23 can consume the same semantic map.

## D8 — Acceptance checklist
- [x] Locale loading
- [x] Locale fallback
- [x] Runtime locale application
- [x] Available-locale enumeration
- [x] Keyboard layout registration/selection
- [x] Protanopia
- [x] Deuteranopia
- [x] Tritanopia
- [x] Achromatopsia
- [x] Motion comfort
- [x] Snap turn
- [x] Teleport
- [x] Vignette
- [x] FOV clamp
- [x] Screen-reader entity description
- [x] Subtitle channels
- [x] Caption styling
- [x] UI scaling
- [x] Semantic status-color remapping
- [x] Integration limited to Systems 13–28
