# Stage 1 보고서 — Task M100-823

- 이슈: #823
- 작성일: 2026-06-04
- 브랜치: `local/task_m100_823`

## 1. 작업 요약

headless macOS PNG 렌더에서 미설치 font family가 CoreText downloadable font lookup을
트리거하지 않도록 Skia 시스템 font lookup 경로를 안전화했다.

핵심 변경은 `FontMgr::family_names()` 결과를 `SkiaLayerRenderer` 생성 시 캐시하고,
시스템 family가 캐시에 있을 때만 `match_family_style`을 호출하도록 만든 것이다.
custom font path로 로드한 typeface는 기존처럼 시스템 캐시와 무관하게 우선 사용한다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `src/renderer/skia/font_lookup.rs` | 시스템 family 캐시 타입과 안전 lookup helper 추가 |
| `src/renderer/skia/mod.rs` | `font_lookup` 모듈 연결 |
| `src/renderer/skia/renderer.rs` | `system_families` 필드 추가, form/equation/text replay 경로 전달 |
| `src/renderer/skia/text_replay.rs` | 본문 fallback chain 및 mark font lookup 필터링 |
| `src/renderer/skia/equation_conv.rs` | 수식 렌더 fallback chain에 시스템 family 캐시 전달 |

## 3. 구현 세부

- `collect_system_families(&FontMgr)`로 시스템 family 목록을 `HashSet<String>`에 수집했다.
- `match_system_family_style(...)` helper가 family membership을 확인한 뒤에만
  `FontMgr::match_family_style`을 호출한다.
- 본문 fallback chain은 custom typeface 후보를 먼저 chain에 넣고, 시스템 family 후보는
  캐시에 존재할 때만 CoreText로 넘긴다.
- 문단/제어문자 mark font의 `DejaVu Sans` 단발 lookup도 helper를 통과하도록 바꿨다.
- form control fallback과 equation fallback도 같은 helper를 사용하도록 정렬했다.
- 마지막 `legacy_make_typeface(None, style)` fallback은 유지했다.

## 4. 검증

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
- `rg -n "match_family_style" src/renderer/skia -g '*.rs'`
  - 실제 호출은 `src/renderer/skia/font_lookup.rs` helper 한 곳만 남음

비고:

- `cargo test --lib --features native-skia font_lookup`와 release native-skia 테스트는
  Skia 바이너리 다운로드 확인 때문에 네트워크 허용으로 실행했다.
- 시각 판정은 대상이 아니다. 이번 이슈는 headless macOS CoreText IPC hang 방지이며,
  구조상 missing family가 CoreText lookup으로 전달되지 않는지를 테스트로 검증했다.

## 5. 남은 작업

- 최종 보고서 작성
- 커밋
- 작업지시자 승인 후 이슈 close
