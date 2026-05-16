# Text IR v2 Migration Contract

This document records the P11/P12 text paint contract for the layered renderer.
The goal is to make source identity and future text variants explicit without
breaking the existing `TextRun` replay path.

## Current Position

`TextRun` remains the compatibility paint contract. It still carries the text
projection, style, explicit positions, HWP text flags, and legacy visual
payloads that SVG, Canvas2D, and native Skia can replay with existing string
APIs.

P12 adds the first guarded `GlyphRun` variant contract. Glyph ids are still not
canonical by default: `TextRun` remains the fallback replay path, and a
`GlyphRun` may only be selected when the variant is complete, the diagnostics
are exact or position-adjusted, the font resource is self-contained, and the
paint style is fill-only. Native Skia deliberately keeps using the `TextRun`
fallback in P12 because exact blob-backed typeface construction is not wired
yet.

P13 closes the first diagnostics layer for this contract. The export is still
schema v1 and still keeps `TextRun` fallback as the replay baseline, but it now
also reports `textV2` compatibility diagnostics: slot-level variant state,
structured validation issues, the v1 downgrade path, fallback-free profile
guards, and line-break risk telemetry for text runs whose shaped replay could
affect layout-sensitive behavior.

P14 adopts the first backend-facing text variant policy. It adds a
`GlyphOutline` strict sidecar contract for producer-resolved glyph paths and a
shared backend selection diagnostic that can explain why CanvasKit/native-style
replay selects a strict variant or falls back to `TextRun`. This is still a
guarded contract, not a public default path switch.

## Export Contract

Layer JSON now provides additive text metadata:

- `schemaMinorVersion` and `resourceTableMinorVersion` for compatible schema
  growth under major version 1.
- `usedFeatures`, `requiredFeatures`, `optionalFeatures`, and `knownFeatures`
  so consumers can decide what they can safely replay.
- `textSources`, an export-local table of source text entries.
- `TextRun.source`, a span into `textSources`.
- `TextRun.paintStyle`, the paint-visible style projection.
- `TextRun.projectionKind`, describing how `TextRun.text` relates to source.
- `TextRun.placement`, run-local-to-page transform metadata.
- `TextRun.clusterBasis` and `TextRun.clusters`, additive layout placement
  clusters. These are not shaped glyph clusters.
- `TextRun.legacyVisuals`, marking legacy inline visual payloads as mirrors
  when a separate visual op exists.
- Explicit special visual ops: `charOverlap`, `textControlMark`, `tabLeader`,
  and `textDecoration`.
- `fontResources`, an additive table for font blob/face identity.
- Optional `GlyphRun` sidecar ops with `variant`, `shapeKey`, glyph ids,
  glyph positions, shaped clusters, and replay diagnostics.
- Optional `GlyphOutline` sidecar ops with `variant`, `anchorOpId`,
  `payloadKind`, placement, outline paths, strict stroke metadata when present,
  and replay diagnostics. These sidecars are text alternatives, not generic
  shape paths.
- `textV2`, an additive diagnostics object with:
  - `compatibilityProfile`, currently `v1Compat` for normal exports.
  - `fallbackRequired`, which stays true for the v1 compatibility writer.
  - `downgradePath=schemaV1FlattenedTextRunAndGlyphRun`.
  - `slotDiagnostics`, one entry per v1 text variant group.
  - `validationIssues`, using stable issue codes and severity.
  - `lineBreakRisks`, report-only telemetry for complex text runs.

The explicit visual ops are additive. Existing renderers skip them and keep
drawing the paired `TextRun` mirror, so visual output does not double-paint.
Future backends can choose the explicit op and suppress the corresponding
legacy mirror.

`GlyphRun` is also additive. Backends must choose a single variant set per
`equivalenceGroup`. If a glyph variant is unsupported, incomplete, or fails its
diagnostics/resource guard, the backend must paint the default `TextRun`
fallback instead.

`GlyphOutline` follows the same variant rule but is anchored to the same
paint-order slot through `anchorOpId`. The strict subset currently allows
monochrome fill outlines and a small fill/stroke subset with deterministic
stroke style. Backends that cannot preserve that payload must reject the
sidecar and use `TextRun`.

## Invariants

- `schemaVersion` and `resourceTableVersion` stay major integer versions for
  v1 compatibility.
- Compatible changes use minor versions and feature arrays.
- Source ranges are UTF-8 byte ranges. UTF-16 ranges are also exported for JS
  and DOM consumers.
- `TextRun.text` is a replay projection, not the long-term source identity.
- `TextRun.placement` and clusters are metadata while
  `text.placementAuthority` is `compatibilityProjection`.
- `TextRun` source ids are dense and export-local. They must not be used as
  cross-document or cross-export stable ids.
- Field marker, paragraph-end, and line-break metadata also appear as source
  annotations.
- P12 enables the `GlyphRun` schema contract and native Skia contract guard,
  but native Skia selection remains disabled until it can instantiate the exact
  referenced font blob/face. Normal layer lowering still emits `TextRun` only
  unless a shaping pass explicitly inserts glyph alternatives.
- P13 `textV2` diagnostics are additive and report-only for normal exports.
  They must not change renderer output or make `GlyphRun` the canonical path.
- P14 `GlyphOutline` is a strict sidecar. It must carry `anchorOpId`, stay in
  the same `equivalenceGroup`, and complete every declared variant part before
  selection. In schema v1 the `equivalenceGroup` is also the paint-order slot id
  because fallback `TextRun` ops do not yet have stable per-op ids.
- P14 backend selection diagnostics are deterministic and report-only. They
  explain CanvasKit/native eligibility, glyph-id range limits, font portability,
  missing glyphs, cluster mismatch, unsupported text effects, incomplete
  variants, and outline payload/stroke rejection.
- A fallback-free text profile is only valid when every text variant slot has a
  strict visual variant. In schema v1 the default writer still exports the
  fallback, and the fallback-free profile is only exposed as a guard/validator.
- `slotDiagnostics.strictVariantAvailable` requires exact or position-adjusted
  quality, strict visual eligibility, replayable font eligibility, no missing
  glyphs, no cluster mismatch, and no unsplit fallback-font use.
- `lineBreakRisks` is explanatory telemetry. It marks cases such as char
  overlap, vertical/rotated text, ratio/spacing changes, tab leaders, visible
  text effects, field markers, and explicit line/paragraph-end markers. It is
  not a layout decision source.
- Canvas2D/layered SVG keep using the `TextRun` fallback and ignore glyph
  sidecars.
- Glyph ids require portable font identity. Consumers must not replay glyph ids
  against an arbitrary local font just because the family name matches.

## Follow-Ups

- Wire real document font blob extraction into `ResourceArena`.
- Add CanvasKit glyph replay behind the same variant gate.
- Add native glyph outline replay behind the strict `GlyphOutline` variant.
- Add resource table entries for font blobs and face identity.
- Promote renderer diagnostics from report-only to backend selection telemetry
  once CanvasKit/native glyph alternatives are actually consumed.
