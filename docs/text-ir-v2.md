# Text IR v2 Migration Contract

This document records the P11 text paint contract for the layered renderer.
The goal is to make source identity and future text variants explicit without
breaking the existing `TextRun` replay path.

## Current Position

`TextRun` remains the compatibility paint contract. It still carries the text
projection, style, explicit positions, HWP text flags, and legacy visual
payloads that SVG, Canvas2D, and native Skia can replay with existing string
APIs.

P11 does not make glyph ids canonical. Portable `GlyphRun` replay needs exact
font bytes or a verified font face, face index, synthetic style flags, shaping
features, script/language, fallback policy, and diagnostics. Those remain a
follow-up phase.

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

The explicit visual ops are additive. Existing renderers skip them and keep
drawing the paired `TextRun` mirror, so visual output does not double-paint.
Future backends can choose the explicit op and suppress the corresponding
legacy mirror.

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
- P11 does not enable `GlyphRun`, CanvasKit glyph replay, or native glyph
  outline replay. Those require a guarded variant selection contract.

## Follow-Ups

- Add guarded `GlyphRun` variants only when exact or verified font identity is
  available.
- Add native glyph outline/glyph run replay behind the variant gate.
- Add resource table entries for font blobs and face identity.
- Promote renderer diagnostics once glyph alternatives exist.
