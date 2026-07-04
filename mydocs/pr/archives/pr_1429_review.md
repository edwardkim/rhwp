# PR #1429 리뷰 기록

## PR 정보

- PR: https://github.com/edwardkim/rhwp/pull/1429
- 제목: `render: keep v2 authority gaps on text fallback`
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/536 (`Refs #536`, 자동 close 아님)
- 작성자: `seo-rii`
- base: `edwardkim/rhwp:devel`
- head: `seo-rii/rhwp:render-p26`
- 상태: Open, Draft 아님
- mergeable: `MERGEABLE`
- merge state: `CLEAN`
- 작성 시점: 2026-06-18 KST
- 변경 규모: 4 files, +346 / -21

## 변경 범위

- Text IR v2 P26 단계에서 아직 권한이 증명되지 않은 glyph replay vocabulary를 strict `GlyphRun`으로 선택하지 않도록 막았다.
- `MixedPerGlyph`는 `mixedPerGlyphAuthorityPending`, `glyphTransforms`는 `glyphTransformAuthorityPending`, vertical orientation은 `verticalGlyphOrientationAuthorityPending`으로 거부한다.
- 위 케이스에서는 기존 `TextRun` fallback을 유지하도록 `TextV2Diagnostics`와 `layer_renderer`의 선택 사유를 맞췄다.
- `lineBreakRisks`는 strict variant가 있을 때 `fallbackFreeStrict`에서도 report-only telemetry로 유지된다.
- font resolution/metrics만으로 public `GlyphRun`을 내보내지 않는 회귀 테스트와 `docs/text-ir-v2.md` P26 authority 문서를 추가했다.

## 로컬 검토 결과

Blocking finding 없음.

검토 포인트:

- `src/paint/text_v2.rs`의 strict 판정은 diagnostics뿐 아니라 horizontal orientation과 glyph transform 부재를 함께 요구한다.
- `src/renderer/layer_renderer.rs`의 reject reason은 기존 generic `unsupportedPaintEffect` 대신 authority pending reason을 명시한다.
- glyph outline strict 판정은 기존 diagnostics 기반 helper를 유지해 이번 glyph run orientation gate에 잘못 묶이지 않는다.
- 문서의 P26 설명과 구현의 fallback 정책이 일치한다.

## 로컬 검증

통과 확인:

```text
CARGO_TARGET_DIR=/tmp/rhwp-pr1429-target cargo test --lib paint::text_v2 -- --nocapture
CARGO_TARGET_DIR=/tmp/rhwp-pr1429-target cargo test --lib renderer::layer_renderer -- --nocapture
CARGO_TARGET_DIR=/tmp/rhwp-pr1429-target cargo test --lib paint::text_shape::tests::font_resolution_without_shaping_proof_never_emits_public_glyph_runs -- --nocapture
```

GitHub Actions 확인:

- `Build & Test`: pass
- `Canvas visual diff`: pass
- `CodeQL`: pass
- `Analyze (javascript-typescript)`: pass
- `Analyze (python)`: pass
- `Analyze (rust)`: pass
- `WASM Build`: skipped

문서 추가 커밋은 문서 전용 변경이므로 `git diff --check`와 변경 범위 확인으로 검증한다.

## 리뷰 결론

PR #1429는 보수적인 fallback gate를 강화하는 변경이며, Text IR v2의 future vocabulary를 안정 replay contract로 오인하지 않도록 막는다. 로컬 targeted test와 GitHub Actions가 통과했으므로, 리뷰 문서/오늘할일 커밋을 PR head에 포함해 CI 재확인 후 merge 가능으로 판단한다.
