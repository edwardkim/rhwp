# PR #1438 최종 보고서 — 폰트 리소스 증명 게이트 glyph 선택 (P27, seo-rii)

## 1. 결정

**merge 수용** — 작업 커밋 4개를 `local/devel` 에 cherry-pick → `devel` push.

## 2. 변경 본질

폰트 resolver 진단과 portable glyph replay proof 분리(P27). glyph 선택이 `fontResources`
증명(blob `dataRef`·interned bytes·digest 일치)을 갖출 때만 `Portable` glyph run 선택,
증명 불완전 시 `TextRun` fallback 유지. 6파일 +742/-77 (text_v2/json/layer_renderer +
docs/README).

## 3. CI 실패 해소

이전 CI(`Build & Test`)가 `paint::json` 테스트의 `strictVariantAvailable:true` 기대로
실패. 메인테이너가 로컬 재현해 음성/양성 분리 + 라이선스 없는 fixture를 제안(코멘트).
작성자가 `0cbfa737` 로 반영:
- 음성: 빈 fontResources → `strictVariantAvailable:false` + `fallbackReason:"fontFaceMissing"`.
- 양성(신규): `add_portable_font_resources`(합성 FontBlob/FontFace)로 증명을 채워 strict
  variant 검증. 라이선스 폰트 없이 CI에서 strict 경로 검증 — "CI 라이선스 폰트 불가"
  제약 해소.

## 4. 검증

- cherry-pick(4커밋) → local/devel: 충돌 없음(author seorii 보존).
- `paint::json` 14 / `paint::text_v2` 15 / `renderer::layer_renderer` 33 passed.
- 전체 lib 1853 passed / 0 failed (회귀 0). fmt OK, clippy 0.
- GitHub CI: Build & Test / Analyze / CodeQL 전부 pass.

## 5. merge 방식 — cherry-pick

base=devel·CI 통과였으나 검증된 작업 커밋 4개만 cherry-pick(무관 변경 0, author 보존)
하여 push. PR 은 "devel 에 포함됨" 으로 close.

## 6. 후속

- P28+ (CanvasKit glyph replay, native glyph replay, 실문서 폰트 blob 추출 등) 후속 PR.
- `Refs #536`(멀티 렌더러 트래킹) 유지.
