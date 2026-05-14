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

The explicit visual ops are additive. Existing renderers skip them and keep
drawing the paired `TextRun` mirror, so visual output does not double-paint.
Future backends can choose the explicit op and suppress the corresponding
legacy mirror.

`GlyphRun` is also additive. Backends must choose a single variant set per
`equivalenceGroup`. If a glyph variant is unsupported, incomplete, or fails its
diagnostics/resource guard, the backend must paint the default `TextRun`
fallback instead.

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
- Canvas2D/layered SVG keep using the `TextRun` fallback and ignore glyph
  sidecars.
- Glyph ids require portable font identity. Consumers must not replay glyph ids
  against an arbitrary local font just because the family name matches.

## Follow-Ups

- Wire real document font blob extraction into `ResourceArena`.
- Add CanvasKit glyph replay behind the same variant gate.
- Add native glyph outline replay behind a separate strict visual variant.
- Add resource table entries for font blobs and face identity.
- Promote renderer diagnostics once glyph alternatives exist.
