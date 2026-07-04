# PR #1438 검토 — 폰트 리소스 증명 게이트로 glyph 선택 (P27, seo-rii)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1438
- 작성자: `seo-rii` (Seohyun Lee) — 렌더 백엔드 핵심 컨트리뷰터(P시리즈)
- 상태: open / base `devel` (head: `render-p27`, 작업 커밋 4 + devel 동기화 merge)
- 변경: 6파일 +742/-77 (`paint/text_v2.rs`, `paint/json.rs`, `renderer/layer_renderer.rs`,
  `docs/text-ir-v2.md`, README/README_EN)
- 연결: P27 (멀티 렌더러 #536 계열, closes 아님)

## 2. 변경 요약

폰트 resolver 진단과 portable glyph replay proof를 분리(P27). CanvasKit/native-style
glyph 선택이 `fontResources`(blob `dataRef`·interned bytes·digest 일치) 증명을 갖출 때만
`Portable` glyph run을 선택하도록 게이트. 증명이 불완전하면 `TextRun` fallback 유지
(하위 호환). `glyph_run_is_strict(run, resources)` + `fallback_reason` 우선순위 병합으로
사유(`fontFaceMissing` 등)를 진단에 명시.

## 3. CI 실패 → 작성자 수정 경위

직전 커밋 `63f30d4e` 에서 `paint::json::serializes_optional_glyph_run_variant_with_text_run_fallback`
가 실패(빈 fontResources 에서 `strictVariantAvailable:true` 기대). 게이트 강화로 빈 리소스
→ strict 비가용이 된 것과 테스트 기대값 불일치였다. 메인테이너가 원인을 로컬 재현해
코멘트로 전달(음성/양성 분리 + OFL 폰트 또는 합성 리소스 fixture 제안).

작성자가 커밋 `0cbfa737 (test: update glyph-run JSON proof diagnostics)` 로 반영:
- **음성**: `serializes_optional_glyph_run_variant_with_text_run_fallback` — 빈 리소스 →
  `strictVariantAvailable:false` + `fallbackReason:"fontFaceMissing"`.
- **양성**: `serializes_strict_glyph_run_variant_when_font_resources_are_proven` — 신규.
  `add_portable_font_resources`(합성 FontBlob/FontFace, Embedded source)로 증명을 채운 뒤
  strict variant 가용 검증. **라이선스 폰트 불필요·CI 안전** (제안 의도 충족).

## 4. 검증 (로컬, `pr1438-review2` = local/devel + cherry-pick 4커밋)

- `paint::json` 14 passed (음성·양성 둘 다 ok) — 이전 실패 해소.
- `paint::text_v2` 15 / `renderer::layer_renderer` 33 passed.
- 전체 lib **1853 passed / 0 failed** (회귀 0).
- `cargo fmt --check` OK, `cargo clippy --lib` 0.
- GitHub CI: Build & Test / Analyze(rust·js·python) / CodeQL 전부 pass.

## 5. 평가

- 설계 일관 — proof 게이트로 strict glyph 선택을 좁히되 `TextRun` fallback 유지(하위 호환).
  fallback_reason 우선순위 병합으로 진단 명확.
- CI 실패를 메인테이너 진단 → 작성자가 음성/양성 분리로 정확히 수정. 양성 fixture가
  합성 폰트 리소스라 CI에서 라이선스 폰트 없이 strict 경로 검증(라이선스 우려 해소).
- README/docs로 P24~P27 경계 문서화. 핵심 컨트리뷰터의 단계 작업.

## 6. 판단

**merge 권고**. proof 게이트가 안전(fallback 유지·회귀 0), CI 실패 해소 확인, 테스트가
라이선스 폰트 없이 strict 경로를 검증. base 가 `devel`(BEHIND 이나 머지 깨끗 예상).
세부는 `pr_1438_report.md`.
