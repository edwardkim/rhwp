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

## CanvasKit Direct Replay and Overlay Policy

P15 promotes CanvasKit planning from an implicit Canvas2D-assisted preview idea
into an explicit replay policy. This still does not switch the public renderer.
Instead, `getCanvasKitReplayPlan(page, mode)` exposes a diagnostics-only plan
from the current `PageLayerTree` so a frontend can see which operations are
direct replay candidates, which operations need a transition overlay, and which
text variants select a strict sidecar or fall back to `TextRun`.

CanvasKit has two operational modes:

- `default`: native-preparation mode. CanvasKit should prefer direct replay and
  must not silently hide unsupported operations behind a Canvas2D overlay. If an
  operation is not covered yet, the plan reports `directRequired` with
  `hiddenOverlayForbidden` so the gap remains visible.
- `compat`: conservative direct replay mode. It may choose more conservative
  policy values such as clip padding or sampling, but it does not mean a hidden
  Canvas2D overlay fallback.

The first overlay inventory is deliberately conservative. Raster images,
equations, form controls, raw SVG fragments, placeholders, special text visual
ops, and effect-heavy `TextRun` payloads are visible in the plan before any
hidden overlay is removed. Basic page background, vector primitives, clipping,
and simple `TextRun` payloads are direct candidates. `GlyphRun` and
`GlyphOutline` stay under the P14 text variant selection diagnostics: if the
strict sidecar is not selected, CanvasKit must use the `TextRun` fallback.

Overlay removal should be staged from low-risk paint operations toward text and
variant-sensitive operations:

1. Raster image replay: crop, tile, image-effect preprocessing, filtering, and
   resource-cache behavior should match Canvas2D through direct CanvasKit image
   replay first. `compat` may keep an overlay until the direct path has a parity
   fixture; `default` should make the gap visible.
2. Equation and form-object replay: parity fixtures should decide whether the
   vector/layout-box path or an image fallback is the canonical replay for each
   operation before the overlay is removed.
3. TextRun effects: vertical text, rotation, synthetic style, ratio scaling,
   shade, outline, shadow, decorations, emphasis dots, tab leaders, and control
   markers should be promoted effect-by-effect. Unsupported text effects must
   not trigger approximate `GlyphRun` replay.
4. GlyphRun/GlyphOutline gates: CanvasKit should choose a strict variant only
   when the selection report says it is replayable. Opening outline replay in
   CanvasKit is a backend parity milestone, not a schema change.

## P19 Advanced Glyph Payload Gates

P19 adds schema-v1 vocabulary for richer `GlyphOutline` payload families without
turning them into default replay paths.

- `payloadKind: "colorLayers"` is reserved for producer-normalized color glyph
  data. `ColorLayers.colrV0` uses resolved solid layer paths. `ColorLayers.colrV1`
  starts with a bounded graph vocabulary where `node.kind` is validated before a
  backend may select the variant.
- CanvasKit may select only the bounded COLRv1 graph subset covered by this
  phase: solid paths, single linear/radial/sweep gradient paths, and transform
  chains that end in exactly one supported leaf. Composite, blend, clip, nested
  paint, partial sweep angle, and invalid gradient-stop graphs still reject
  deterministically and use the `TextRun` fallback.
- `payloadKind: "bitmapGlyph"` is reserved for one producer-selected image strike.
  It must carry deterministic placement, scaling, and filtering metadata. Backend
  default strike selection is not part of the strict contract.
- `payloadKind: "svgGlyph"` is reserved for sanitized static vector resources.
  Script, animation, external resource loading, and interactivity flags must stay
  false before a backend may consider the payload strict.
- Each `GlyphOutline` op may carry only one payload family. Mixing stroke,
  color-layer, bitmap, and SVG payload fields is a validation error.
- CanvasKit and Canvas2D report the same high-level reject reasons for advanced
  payload families: `unsupportedColorGlyph`, `unsupportedBitmapGlyph`, and
  `unsupportedSvgGlyph`. Canvas2D can still add
  `backendDoesNotSupportVariant`.

These gates keep writer emission closed until a payload family has a proof
fixture and backend-specific replay path. The required `TextRun` fallback remains
the compatibility path.

Every overlay removal requires a Canvas2D-vs-CanvasKit fixture. Rasterizer
output can use fuzzy PNG comparison, but semantic decisions must be exact:
selected variant id, fallback reason, resource resolution, effect preprocessing
diagnostics, and cache behavior should be asserted without tolerance. When a
direct CanvasKit path intentionally differs from Canvas2D because it is closer
to native Skia semantics, the fixture should label that as a Skia strict replay
improvement rather than a Canvas2D compatibility match.

## Follow-Ups

- Wire real document font blob extraction into `ResourceArena`.
- Expand CanvasKit glyph replay beyond the guarded COLRv1 solid/gradient subset.
- Add native glyph outline replay behind the strict `GlyphOutline` variant.
- Add resource table entries for font blobs and face identity.
- Promote renderer diagnostics from report-only to backend selection telemetry
  once CanvasKit/native glyph alternatives are actually consumed.
