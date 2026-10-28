# 완료 보고서 — Task M100-823

- 이슈: #823
- 제목: PNG 렌더가 headless macOS에서 미설치 폰트 fallback 시 hang
- 작성일: 2026-06-04
- 브랜치: `local/task_m100_823`

## 1. 완료 내용

Skia PNG/raster 렌더 경로에서 시스템에 없는 font family를 CoreText
`match_family_style`에 넘기지 않도록 수정했다.

`SkiaLayerRenderer` 생성 시 `FontMgr::family_names()` 결과를 시스템 family 캐시로
수집하고, 본문 text replay, mark font, form control, equation 렌더의 시스템 폰트
fallback이 모두 이 캐시를 통과하도록 했다. custom font path로 로드된 typeface는
기존처럼 먼저 사용되며, 시스템 캐시에 없다는 이유로 배제되지 않는다.

## 2. 주요 변경

- `src/renderer/skia/font_lookup.rs`
  - `SystemFontFamilies` 타입 alias 추가
  - `collect_system_families`, `has_system_family`, `match_system_family_style` 추가
  - missing family skip 단위 테스트 추가
- `src/renderer/skia/renderer.rs`
  - `system_families` 필드 추가
  - text replay/equation/form control 경로에 안전 lookup 적용
- `src/renderer/skia/text_replay.rs`
  - 본문 fallback chain에서 custom typeface 후 시스템 family를 안전 lookup
  - `DejaVu Sans` mark font 단발 lookup도 사전 필터링
- `src/renderer/skia/equation_conv.rs`
  - 수식 fallback chain이 시스템 family 캐시를 전달받도록 변경

## 3. 검증 결과

통과:

- `cargo fmt --check`
- `cargo check --lib`
- `cargo check --lib --features native-skia`
- `cargo test --lib --features native-skia font_lookup`
  - 2 passed
- `cargo test --release --lib --features native-skia font_lookup`
  - 2 passed
- `cargo test --release --lib`
  - 1562 passed, 6 ignored
- `git diff --check`

추가 확인:

- `rg -n "match_family_style" src/renderer/skia -g '*.rs'`
  - 직접 호출은 `font_lookup.rs` helper 한 곳으로 제한됨
- 작업지시자 macOS 직접 테스트
  - PNG export가 hang 없이 완료됨을 확인

## 4. 리스크

- 시스템 family membership은 exact match다. 이슈의 직접 원인은 missing family를
  CoreText에 넘기는 것이므로 exact membership skip으로 해결한다.
- 일부 family alias가 miss될 수 있으나, custom typeface 우선 처리와 후속 fallback,
  마지막 `legacy_make_typeface(None, style)` fallback은 유지했다.
- headless macOS hang 자체는 로컬에서 재현하지 않았다. 대신 CoreText 호출 전 차단
  구조와 native-skia 테스트로 회귀를 가드했다.

## 5. 결론

Task M100-823 구현, 검증, 작업지시자 macOS 직접 확인을 완료했다. 이슈를 close할 수 있다.
