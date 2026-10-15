---
kind: canonical
status: active
canonical: mydocs/tech/text-ir-v2.md
last_verified: 2026-10-14
---

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

P42 opens the first portable `GlyphRun` direct-replay subset in browser
CanvasKit. Normal lowering can attach a bounded, producer-positioned
`GlyphRun` when an exact embedded font face proves one-to-one nominal glyph
mapping. Layer JSON carries the referenced font bytes, CanvasKit verifies their
content identity and constructs the exact face, and every unsupported or
malformed case still selects the anchored `TextRun` fallback. Canvas2D remains
the public default renderer.

Schema 1.23 makes `PageLayerTree` the production paint contract. Explicit text
visuals carry the source span of their fallback slot, decoration endpoints are
trimmed by the producer, and editor object/structure labels are emitted as
`controlLabel` paint operations. Backends translate these decisions without
reconstructing visibility, clip, annotation, or text-visual policy.

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
  and `textDecoration`. Each carries the exact fallback `source` span.
- Producer-trimmed `textDecoration.positions`; consumers use the final exported
  position as the decoration endpoint.
- `controlLabel`, the explicit editor annotation for image, equation, table,
  text-box, header, footer, and footnote-area paint slots.
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

The explicit visual ops are canonical paint decisions. Production backends
suppress the corresponding `TextRun` mirror and replay each source-bound visual
exactly once. Duplicate, orphaned, or ambiguously bound visual ops are invalid.

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
- `TextRun` source ids are unique and export-local; sparse pre-bound ids are
  preserved. They must not be used as cross-document or cross-export stable ids.
- Field marker, paragraph-end, and line-break metadata also appear as source
  annotations.
- P12 enables the `GlyphRun` schema contract and native Skia contract guard.
  Native Skia selection remains disabled until it can instantiate the exact
  referenced font blob/face. P42 adds a separate browser CanvasKit subset for
  producer-positioned nominal glyphs; runs outside that subset remain
  `TextRun`-only unless another shaping pass inserts a proven alternative.
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

The first overlay inventory was deliberately conservative. Raster images,
equations, form controls, raw SVG fragments, placeholders, special text visual
ops, and effect-heavy `TextRun` payloads were made visible in the plan before
any hidden overlay was removed. Horizontal character overlap, control marks,
tab leaders, and decorations now have direct replay; unsupported vertical or
malformed variants remain explicit blockers. `GlyphRun` and
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
3. TextRun effects: horizontal decorations, emphasis dots, tab leaders, control
   markers, and character overlap are promoted through external visual ops.
   Vertical text, ratio scaling, shade, outline, shadow, emboss, and engrave
   remain guarded. Unsupported text effects must not trigger approximate
   `GlyphRun` replay.
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
- strict glyph의 경로·affine·baseline·gradient·stroke·bitmap/SVG geometry는
  유한한 f32 범위(`abs(value) <= f32::MAX`) 안에 있어야 합니다. 사각형은
  성분과 함께 `x + width`, `y + height`도 검사합니다. 공통 selector와 browser가
  Native Skia의 scalar 경계를 공유하며, 한 part라도 초과하면 해당 variant 전체를
  거부하고 `TextRun` fallback을 선택합니다. 이 검사는 자원 준비 또는 변환 후
  geometry에 대한 backend 검증을 대체하지 않습니다.
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

## P39 Positioned Text Visual Replay

Layer schema `1.19` adds `text.charOverlapOp.bounded`,
`text.controlMarkOp.positioned`, `text.controlMarkOp.bounded`,
`text.tabLeaderOp.bounded`, and `text.decorationOp.bounded`. The producer exports
space, tab, paragraph-end, and line-break marks with run-relative coordinates
and font sizes. Each operation is limited to 4,096 positioned items or source
characters and exports a completeness flag; an incomplete sidecar is never
eligible for direct replay. Decoration positions follow expanded PUA display
text and carry the adjusted baseline and font size for superscript/subscript;
tab-leader endpoints use the compatibility renderer's following-text clamp,
and combined-number overlap uses the compatibility renderer's digit-count
scale.
The same positioned list may appear on the
anchored `TextRun` as compatibility metadata and on its external
`TextControlMark` operation; `legacyVisuals.controlMarks: "mirror"` makes the
external operation the sole paint authority.

CanvasKit directly replays horizontal `CharOverlap`, `TextControlMark`,
`TabLeader`, and `TextDecoration` operations. It preserves all modeled line
shapes, bounds wave construction, honors paragraph marks, and keeps vertical,
rotated, malformed, incomplete, or over-limit payloads on deterministic
fallback.
`showControlCodes` remains an automatic-mode blocker because structural labels
such as table and image markers do not yet have equivalent paint ops. For
ordinary positioned `TextRun` replay, a missing glyph may use the prepared
default face or bounded old-Hangul subset for only the affected contiguous span
while retaining serialized positions. A run may create at most 4,096 fallback
spans, bounding alternating-glyph font probes and draw calls. Hancom boxed-number
PUA characters use a bounded vector box and digit fallback. If no prepared face
or explicit PUA fallback resolves a glyph, `textRun:glyphMapping` remains an
unexpected runtime diagnostic so automatic mode cannot silently accept missing
ink. The document-backed readiness set currently covers positioned paragraph
marks and PUA fallback; character overlap, tab leaders, and decorations are
held by synthetic replay contracts until focused source fixtures are added.

## P42 Portable GlyphRun Direct Replay

Layer schema `1.22` and resource table `1.6` add `resources.fontBlobs` and
`resources.fontBlobKeys`. A portable `FontBlobResource.dataRef` resolves to one
of those content-addressed payloads; the consumer must verify the declared
length and BLAKE3 digest before constructing a typeface.

Normal font-native lowering emits a `GlyphRun` only for a bounded horizontal,
unrotated, fill-only run whose source and display projection are identical.
Every scalar must be in the nominal-replay allowlist, use the same HWP language
font slot, map to a nonzero glyph in the exact embedded face, and need no
combining, bidi, complex-script, old-Hangul, emoji, variation, synthetic style,
or per-glyph transform handling. Positions and advances come from the producer;
the sidecar is marked `positionAdjusted`, while the original `TextRun` remains
in the same equivalence group as the compatibility fallback.

Browser CanvasKit verifies each font payload before variant selection, bounds
individual and document-wide font bytes, normalizes an exact TTC v1/v2 face to
a standalone SFNT when `faceIndex` is nonzero, and keeps bounded Typeface/Font
caches for the document generation. Selection rejects malformed glyph counts,
zero or out-of-range glyph ids, non-finite or Float32-overflowing geometry,
unsupported paint effects, variations, oversized resource tables, missing
resources, digest mismatch, and face construction failure before `drawGlyphs`
runs. Document reset and renderer disposal release every cached native object.

This phase does not make glyph ids the general text authority. Complex shaping,
vertical and mixed orientation, glyph transforms, variable instances, and
native Skia exact typeface construction remain guarded follow-ups. Canvas2D and
layered SVG continue to use `TextRun`, and the browser default does not change.

## CanvasKit Parity Plan Link

CanvasKit replay widening is tracked in
`mydocs/tech/canvaskit-parity-implementation.md`. That plan keeps Text IR v2 sidecars
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

- Expand producer-side glyph shaping beyond the bounded nominal-glyph subset.
- Expand CanvasKit glyph replay beyond the guarded COLRv1 solid/gradient subset.
- Add native glyph outline replay behind the strict `GlyphOutline` variant.
- Add document-backed resource table entries for image/SVG glyph payload bytes
  once writer emission starts.
- Promote renderer diagnostics from report-only to backend selection telemetry
  once CanvasKit/native glyph alternatives are actually consumed.

## P45 이관: 엄격한 글리프 자원 검증

`GlyphRun` 선택은 정확한 face의 `maxp.numGlyphs`를 대조하며, glyph 0과 범위 밖 ID는
`TextRun`으로 되돌린다. placement·position·advance는 저장형 `f64`의 유한성뿐 아니라
소비형 `f32`의 표현 범위를 만족해야 한다. 개별 윤곽선 명령과 윤곽선 수는 각각 4096개로
제한한다. Native SVG 준비는 파싱 불가능한 path를 부분적으로 생략하지 않고 전체 엄격한
대안을 거부한다. 중첩 SVG의 viewport와 leaf별 글리프 준비 예산은 기존 계약을 유지한다.

기존 selector 단위 검사에 표현 범위 양쪽 경계와 정확한 face 범위 검사를 추가했다.
현재 이관 단계에서는 개별 파일 rustfmt와 diff 검사만 수행했다. Cargo·Native·fresh WASM 및
독립 한컴 PDF 시각 검증은 저장 공간 확보 뒤 수행해야 하며, 과거 브랜치 결과를 재사용하지 않는다.

## P45 이관: 보고 전용 shaping 측정

`TextShapeReport`의 run·line·paragraph·page 측정과 `LineBreakShadowReport`는 진단값이다.
기존 `shape_glyph_run` 결과를 한 번 관찰하며 추가 shaping 호출이나 레이아웃·variant 선택
분기를 만들지 않는다. 줄 소속과 문맥은 호출자가 제공하며 `Unknown`을 값 없음과 구분한다.
수평 advance span의 차이는 잉크 경계나 수직 메트릭의 정확성 증명이 아니다.

합성 resolver 검사는 호출 수, 원래 배치 보존, 명시적 소속 집계와 문맥별 위험 보고를 검사한다.
이 단계의 실행 검증은 파일 rustfmt·diff 확인까지이며 Cargo 실행은 대기 중이다.

## P45 이관: 명시적 schema v2 내보내기와 borrowed slots

기존 `to_json()`은 schema v1을 유지한다. `to_json_v2_compat()`는 원래 TextRun 자리에
모든 대안을 묶고, `to_json_v2_strict_glyph_outline()`과 `to_json_v2_strict_glyph_run()`은
공유 selector의 자원·geometry 검증을 통과한 대안만 내보낸다. 잘못된 공개 variant 메타데이터,
불완전한 part와 남는 sidecar는 구조화된 오류로 거부한다. 진단 플래그만으로 엄격한 출력을
허용하지 않는다. 엄격한 export는 fallback이 없으므로 schema v1 자동 downgrade를 금지한다.

`LayerTextPaintOpV2`와 variant/part view는 원래 Rust PaintOp를 읽기 전용으로 빌린다.
JSON 검증 경로를 공유하므로 직렬화 없는 API는 아니며, resource와 조상 clip은 원래 tree에 남는다.
검사는 기존 bitmap/font producer를 사용한 자원 보존, source·paint order, 엄격한 거부와
lossless compatibility downgrade에 한정한다. 새 font SVG producer 검사는 시각 증거 확보 후
별도로 도입한다. 이관 파일의 rustfmt·diff 확인을 마쳤으며 실행 검증은 대기 중이다.

## P45 이관: WASM object API

`getPageLayerTreeValue*`는 정본 JSON writer의 필드를 재사용하고 resource의 그림·글꼴 bytes를
`Uint8Array`로 제공한다. SVG resource는 문자열이며, 소비자가 이미 가진 content key의 slot만
생략한다. resource hint는 최대 4096개 문자열, 각 UTF-8 문자열 1024 bytes로 제한한다.
기본 JSON API와 Canvas2D 기본 선택은 바꾸지 않는다. v2 JSON/object API는 모두 명시적이다.

기존 resource-key API와 연결할 수 있도록 v1 object API의 마지막 인수는 선택적인
`omit_font_bytes`와 `omit_image_bytes`이다. 이미지 옵션은 기존 정본 writer의 source-key
규칙을 따르며, 되찾을 key가 없는 payload를 무조건 생략하는 옵션이 아니다. paint 생성,
직렬화와 resource 변환은 같은 hot render 경계 안에서 수행한다.

hot boundary 등록 검사를 보완했다. rustfmt·정적 API 대조는 완료했으며 wasm32 빌드와 fresh
WASM의 실제 object·편집 무효화 검사는 아직 실행하지 않았다. 직렬화 비용 감소를 주장하지 않는다.

## P45 이관: COLRv0 producer helper

`decode_colrv0_color_layers_payload`는 bounded COLR/CPAL 테이블에서 확정된 path·색상을 만들고
CPAL digest를 resource identity에 포함한다. 전경색 palette entry, 선택 face와 palette, source
범위를 검증하며 font 32 MiB, layer 256개, 전체 path command 65536개를 상한으로 둔다.
이 helper는 일반 문서 lowerer에 자동 연결되어 있지 않다. 정상 문서의 COLRv0 출력 지원 완료를
뜻하지 않는다. 합성 font producer 검사 패치는 독립 한컴 PDF 증거를 확보할 때까지 별도 보존하며,
현재 source 확인·rustfmt 외 빌드와 출력 검증은 미실행이다.

## P45 이관: bounded color graph와 Native 재생

schema v1 minor 24는 `sourceOver` composite와 path clip을 명시한다. 64개 고유 node,
깊이 64, 공유 subtree를 포함한 최대 4096회 방문을 검증하며 cycle·도달 불가능 node·혼합
payload·같은 직접 source/backdrop child를 거부한다. Native는 backdrop 뒤 source 순으로
재생하고 조상 transform으로 계산한 page-space clip을 자식 transform과 분리한다.
leaf별 사전 준비 예산과 실패 후보의 비용 차감은 유지한다. 현재 Native renderer는
각 leaf에서 예산을 만들며, 페이지 전체가 하나의 카운터를 공유한다는 의미는 아니다.

기존 public payload 검사도 layer 수와 정규화 RGBA·opacity를 확인한다. 새로운 graph의
렌더링 회귀 검사는 원래 브랜치의 합성 IR 계약과 독립 한컴 출력 증거가 서로 다르므로 현재
활성 테스트에 추가하지 않았다. 별도 패치에 보존했으며 실제 Native·fresh WASM 실행,
독립 출력 및 페이지별 90% gate는 미검증이다. 기존 scalar 검사는 신설 optional 필드만 채웠다.

## P45 이관: font SVG baseline viewport

bitmap producer가 성립하지 않을 때만 정적인 font SVG 후보를 검토한다. 단일 square viewBox와
단일 root를 요구하며 root sizing/transform과 중첩 viewport는 거부한다. 통과한 payload에는
page-space `fontEmBox=(run.x, run.y+baseline, fontSize, fontSize)`를 기록한다. Native와 browser
consumer는 이를 같은 viewport로 쓰며, 일반 SVG decoder가 만든 `fontEmBox=None`은 기존 bbox
배치를 유지한다. viewBox·em-box와 끝점은 공통 portable scalar 계약을 만족해야 한다.

bitmap의 페이지 누적 예산을 decode 전에 확인하며, bitmap/SVG 자원은 scratch arena에서
검증을 마친 뒤에만 실제 arena에 넣는다. 새 font SVG·color 출력 fixture 및 Native pixel 검사는
별도 패치로 보존하고 시각 증거 확보 전에는 활성 회귀에 추가하지 않았다. 현재 확인은 rustfmt,
신설 필드의 기존 생성자 정합성, 기존 producer JSON 필드·SVG depth·준비 예산 보존까지다.
실제 빌드·출력 일치·성능 결과는 아직 없다.

## P45/P47 통합 검토: gradient 자원 비용

공유 DAG는 하나의 gradient leaf를 여러 번 준비할 수 있다. path 비용만 합산하면 shader가
보유하는 색상·offset 배열이 64 MiB 준비 예산과 P47 cache 비용에서 빠진다. 예를 들어 두 leaf와
`composite(n)=(n-1,n-2)`로 구성한 root16은 3193회 방문에 gradient leaf0를610회 재생한다.
8192개의 RGBA-f32 색상과 f32 offset은 shader 배열만으로도 이 경우 약95 MiB를 차지한다.

`gradient_stop_byte_cost(stop_count)`는 adapter의 color/offset 배열과 shader가 보유할
배열을 함께 보수적으로 추정한다. shader에는 암묵적인 양 끝 stop 두 개의 여유를 더한다.
이 값은 allocator 실측이 아니며, temporary 배열이 해제된 뒤에도 그 비용을 retained estimate에
포함한다. linear/radial/sweep 모두 배열·shader를 만들기 전에 checked 연산으로 예약하며,
실패하면 기존 `TextRun` fallback을 유지한다. P47 picture cache도 같은 helper의 expanded
합계를 보유 추정에 포함해야 한다. 세 allocation 계약 검사는 버퍼 생성 없이 상한·반복 방문·
overflow 거부를 검증하도록 후보 패치에 보존했다. source unit-test 수를 늘리는 정책 변경은
하지 않으며, private Native 준비 검사의 실제 실행은 아직 하지 않았다.

## P45/P47 통합 검토: Native admission 경계

공유 selector는 개별 monochrome path를4096개 명령으로 제한하지만 Native의 일반 path
builder는100000개까지 처리하므로 준비 입구에도 `has_portable_geometry()` 검사가 필요하다.
이 검사를 Native의 자원 준비 전에 적용해4096은 수용하고4097은 TextRun fallback으로 남긴다.
같은 경계의 공개 selector 검사는 기존 integration case에 추가했다. surface를 만들지 않는
private Native preparation 검사는 source unit-test 정책에 따라 후보 패치에 보존했다.
P47 cache는 거부된 준비 결과를 저장하지 않으며 실제 선택 결과로 진단을 갱신해야 한다.
새 검사는 아직 실행하지 않았으며, source unit-test 정책과 변경 파일의 rustfmt·diff를 확인한다.
