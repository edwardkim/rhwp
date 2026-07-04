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

## P20 Glyph Payload Resource Proof

P20 keeps the P19 payload families behind the same gates, but makes their
resource identity and native font-construction blockers explicit.

- Each advanced `GlyphOutline` payload may export `payloadResourceKey`. The key
  includes the payload family (`colorLayers`, `bitmapGlyph`, or `svgGlyph`) and
  the replay-relevant placement/source metadata, so a bitmap `imageRef: 7` and a
  static SVG `svgRef: 7` cannot accidentally share a cache entry.
- Bitmap and SVG glyph payload fixtures are treated as resource identity proofs,
  not replay enablement. They can satisfy their strict payload contract while
  still requiring backend gates before direct replay.
- Native Skia now has a glyph-run replay proof matrix that separates portable
  variant contract checks from exact typeface construction. Missing font blob
  bytes, non-zero TTC/OTC face index, variation axes, and the intentionally
  unimplemented exact typeface constructor are reported distinctly.
- The public compatibility path is unchanged: unsupported or unconstructed
  glyph variants continue to use the `TextRun` fallback.

## P24 Strict Bitmap/SVG Glyph Producer Corpus

P24 widens the advanced glyph payload corpus without making bitmap or SVG glyphs
the default public replay path.

- `ResourceArena` can now intern image bytes and static SVG fragments alongside
  font blobs, giving producer-output fixtures a concrete resource path instead
  of only hand-authored numeric ids.
- `payloadResourceKey` keeps the existing payload-family/source/placement
  identity and appends the interned resource's `blake3` key when bytes are
  available. Two exports that reuse `imageRef: 0` or `svgRef: 0` for different
  producer bytes therefore do not share a strict glyph cache slot.
- The schema minor version and feature arrays advertise the P24 additions:
  `text.glyphOutline.payloadResourceDigestKey` and
  `text.glyphOutline.svgGlyph.vectorResourceId`.
- `BitmapGlyph` remains strict only for a single producer-selected strike with
  deterministic alpha, scaling, filtering, finite placement, non-empty text and
  glyph ranges, and no `backendDefault` scaling policy.
- `SvgGlyph` remains strict only for a static sanitized vector resource with a
  required finite positive `viewBox`; script, animation, external resources, and
  interactivity flags must all stay false. JSON keeps the compatibility `svgRef`
  field and also exposes `vectorResourceId` as the clearer static-vector alias.
- Even when a backend explicitly enables the bitmap or SVG glyph family, invalid
  strict payloads still reject and the schema-v1 compatibility export keeps the
  `TextRun` fallback.

## P25 Exact Font Replay Proof Corpus

P25 widens the exact-font replay proof corpus while keeping public glyph-run
fallback behavior conservative.

- CanvasKit selection now rejects variable-font glyph-run instances with
  `variationUnsupported` until an exact variation constructor is proven for the
  public backend.
- CanvasKit selection also rejects non-default TTC/OTC face indexes with
  `faceIndexUnsupported`; default face index `0` remains the positive control.
- Native Skia proof now distinguishes missing font blob bytes, exported
  `dataRef` mismatch, and digest mismatch between the interned bytes and the
  font metadata. Metadata mismatch is treated as a failed portable contract,
  not as a best-effort construction case.
- Native Skia still reports variation axes, non-zero collection face indexes,
  and the intentionally unimplemented exact typeface constructor as separate
  proof reasons. This keeps later exact-construction work from silently changing
  fallback policy.
- The glyph id field remains `u32` in Text IR, but backend selection/proof keeps
  the current range guard before direct glyph replay.

## P26 Guarded V2 Authority Follow-Ups

P26 does not promote a new replay family. It closes the authority gaps left by
the earlier v2 phases so experimental vocabulary cannot be mistaken for a stable
backend contract.

- `MixedPerGlyph`, non-horizontal glyph orientation, and `glyphTransforms`
  remain vocabulary for future vertical and per-glyph transform work.
  CanvasKit/native selection now reports `mixedPerGlyphAuthorityPending`,
  `verticalGlyphOrientationAuthorityPending`, or
  `glyphTransformAuthorityPending` and keeps the homogeneous `TextRun` fallback
  until cluster/grapheme orientation, transform replay, vertical fixtures, and
  backend fallback policy are proven together.
- `lineBreakRisks` stays report-only telemetry. Even under
  `fallbackFreeStrict`, line-break risk metadata does not become a validation
  error when the slot has a strict variant.
- The guarded COLRv1 subset remains limited to the P19 solid/gradient/transform
  graph contract. Composite, blend, clip, nested paint, partial sweep, and other
  future graph primitives require document-backed fixtures before writer or
  backend authority expands.
- Cross-scope variant vocabulary stays diagnostic-only. Variants still need a
  same-leaf default `TextRun` fallback before they can participate in schema-v1
  compatible export.
- Font metrics data and font-name resolution remain compatibility diagnostics,
  not portable replay proof. A resolver may use `font_metrics_data.rs` to compare
  shaped advances against legacy `TextRun` layout, but the future native strict
  replay proof should still be based on explicit `fontResources`/`ResourceArena`
  identity, resource bytes, digest/`dataRef`, face index, variation axes, and
  shaping proof. P26 keeps metrics/name resolution out of strict proof and leaves
  full ResourceArena enforcement to the resolver/proof follow-up.

## P27 Font Resolver and Proof Boundary

P27 closes the resolver/proof boundary without turning glyph-id replay into a
default path.

- A `GlyphRun` marked `Portable` is no longer enough for CanvasKit/native-style
  selection by itself. The shared selection report also requires a matching
  `FontFaceResource`, `FontBlobResource`, portable `dataRef`, interned blob
  bytes, and digest agreement before it can select the sidecar.
- Font name resolution, HWP `FontFace` substitution, runtime font fallback, and
  `font_metrics_data.rs` remain compatibility diagnostics. They may explain why
  `TextRun` layout or advances look plausible, but they are not portable
  glyph-id replay proof.
- Missing or incomplete proof now has deterministic shared reject reasons:
  `fontFaceMissing`, `fontBlobMissing`, `fontBlobNotPortable`,
  `fontBlobBytesMissing`, `fontBlobDataRefMismatch`, and
  `fontBlobDigestMismatch`.
- Variation axes and non-default collection faces remain separate gates. P27
  keeps those as exact-construction blockers rather than folding them into
  generic font-name matching.
- The public compatibility path is unchanged: when proof is missing, the backend
  keeps the `TextRun` fallback.

## CanvasKit Parity Plan Link

CanvasKit replay widening is tracked in
`docs/canvaskit-parity-implementation.md`. That plan keeps Text IR v2 sidecars
as guarded alternatives to the `TextRun` compatibility path, and it keeps
CanvasKit direct replay separate from hidden Canvas2D overlay behavior. Text IR
v2 changes that affect `GlyphRun`, `GlyphOutline`, `text.variantGroups`,
`ResourceArena`, or fallback-free profiles should update that plan when they
also change CanvasKit selection, cache, or render-diff expectations.

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
- Add document-backed resource table entries for image/SVG glyph payload bytes
  once writer emission starts.
- Promote renderer diagnostics from report-only to backend selection telemetry
  once CanvasKit/native glyph alternatives are actually consumed.
