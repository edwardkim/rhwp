# Task M100 #2215 최종 보고 — 거대 셀 드래그 선택 page 범위와 split fragment 정합

- 이슈: [#2215](https://github.com/edwardkim/rhwp/issues/2215)
- 브랜치: `issue-2215-selection-page-range`
- 기준: `upstream/devel@af5902b6`
- 검증일: 2026-07-19
- 판정: **완료 — 정확성·성능·호환 회귀 통과**

## 결론

115쪽에 걸친 거대 표 셀에서 마우스 드래그할 때 매 갱신마다 전체 host page tree를 탐색하던
경로를 endpoint page 범위로 제한했다. Studio가 이미 보존하는 두 endpoint page를 optional
hint로 전달하고, native는 유효한 same-page 선택은 1장, cross-page 선택은 두 endpoint 사이의
host page만 cached build한다.

수동 검증에서 추가로 발견된 동일 cell paragraph의 page 경계 오류도 같은 범위에서 해결했다.
한 line segment의 leading/trailing cursor를 서로 다른 page의 첫 hit로 조합하지 않고 같은 page
tree 안에서 pair로 해소하며, segment page를 단조롭게 진행시킨다. 그 결과 UI 114→115에서
페이지 밖으로 뻗던 rect와 26.99초 fallback이 사라졌다.

page hint는 정확성 계약이 아닌 성능 hint로 유지했다. 누락·한쪽만 존재·host 밖 hint는 기존
전체 탐색을 사용하고, hinted 후보에서 필요한 segment를 모두 해소하지 못해도 전체 host page로
한 번 재시도한다. 기존 positional API와 JSON 반환 형식은 바꾸지 않았다.

## 원인

문제는 두 층으로 나뉘었다.

1. `get_selection_rects_native()`는 셀의 host page를 모두 빌드·순회했다. 115쪽 표에서는
   pointer move마다 같은 작업이 반복돼 selection 갱신이 수초 단위로 늘어날 수 있었다.
2. split paragraph 경계 offset은 이전 page trailing과 다음 page leading 양쪽에서 유효하다.
   기존 구현은 한 line segment의 left/right cursor를 page 후보 전체에서 각각 독립적으로
   첫-hit 탐색해, 서로 다른 page 좌표계를 하나의 rect 폭에 섞을 수 있었다. 불완전한 결과는
   다시 115쪽 full fallback을 유발했다.

pagination, LineSeg, line-break semantic, Canvas clipping 또는 mouse rAF 정책은 원인이 아니었고
이번 변경에서도 수정하지 않았다.

## 구현

### native/WASM

- `SelectionPagePlan`으로 hinted page와 full fallback을 분리했다.
- `getSelectionRectsInCellEx`에 optional `startPageHint`/`endPageHint`를 추가했다.
- 두 hint가 실제 host page에 모두 속할 때만 inclusive 후보 범위를 사용한다.
- hinted 후보만 shared page-tree cache를 사용하고 positional/fallback의 기존 로컬 수명은
  유지한다.
- 기대한 non-empty segment 수와 실제 rect segment 수가 다르면 hints 없이 한 번 재시도한다.
- 각 line segment의 cursor pair를 동일 `PageRenderTree`에서 찾고 page 진행을 단조롭게 유지한다.

### Studio

- endpoint page와 `Ex` API가 모두 있을 때만 hinted options 호출을 사용한다.
- hint가 없거나 불완전하거나 구버전 WASM에 `Ex`가 없으면 positional API로 복구한다.
- `InputHandler.updateSelection()`은 같은 셀 선택에서 기존 `cursorRect.pageIndex`만 전달한다.
- auto-scroll, selection overlay, pagination 및 Canvas refresh 정책은 변경하지 않았다.

## 정확성 회귀

`tests/issue_2215_selection_page_range.rs`가 HWP/HWPX에 대해 다음 계약을 고정한다.

| 범위 | 계약 |
|------|------|
| 정상 first/middle same-page | rect page·좌표·copy oracle 유지 |
| 서로 다른 문단 p54→p55 | 45 rect, 두 page, copy 1,517자 유지 |
| split same-page p1/p56/p114 | pointer fragment page와 page 폭 안 rect |
| 동일 split paragraph UI 1→2, 56→57, 114→115 | 양쪽 endpoint page rect, 모든 rect가 자기 page 폭 안 |
| missing/one-sided/invalid/stale hint | positional/full fallback과 동일 |

HWP와 HWPX의 rect/copy 결과도 byte-level BLAKE3 동등성을 확인한다. 기존
`issue_658_text_selection_rects` 2건도 유지됐다.

## 실제 pointer 검증과 성능

Chrome에서 HWP/HWPX를 실제 `mouse.down → mouse.move → mouse.up` 경로로 검증했다.

| 시나리오 | drag callback p95 | rect p95 | warm long task |
|------|------:|------:|------:|
| first/middle/late same-page | 최대 1.2ms | 최대 0.2ms | 0 |
| p54→p55 exact cross-page | 최대 3.0ms | 최대 2.4ms | 0 |
| p54→p55 edge auto-scroll | 최대 3.4ms | 최대 2.7ms | 0 |
| UI 114→115 split paragraph 재검증 | 0.8ms | 0.3ms | 0 |

UI 114→115는 양쪽 page에 3개 highlight가 표시되고 mouseup 후 유지됐으며, HWP/HWPX copy가
byte 동등했다. 선택 중 pagination, `refreshLayout`, `refreshPages`, `document-changed`는 모두
0회였다.

같은 cell paragraph 전체 범위는 수정 전 115장 fallback으로 26.99초였으나, 수정 후 cold
571.3ms, warm 2.56ms였다. 환경별 wall-clock은 CI gate로 두지 않고 결정적 후보·fallback·rect
계약을 source test로 고정했다.

## Stage 4 통합 검증

2026-07-19 최신 `upstream/devel@af5902b6` 기준으로 실행했다.

| 검증 | 결과 |
|------|------|
| `cargo test --offline --verbose` | 전체 library/integration/doc test 통과, 실패 0 (`lib`: 2,287 pass, 7 ignored) |
| `cargo clippy --offline --all-targets -- -D warnings` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `npm --prefix rhwp-studio test` | 365 pass, 실패 0 |
| `npm --prefix rhwp-studio run build` | TypeScript + Vite production build 통과 |

## 후속 이슈

실제 검증 중 UI 114쪽 마지막 텍스트 줄을 단일 클릭하면 캐럿 대신 표 객체가 선택되는 별도
결함을 확인해 [#2400](https://github.com/edwardkim/rhwp/issues/2400)으로 분리했다.

이는 #2215의 selection rect/page 후보 문제가 아니다. Studio의 `isTableBorderClick()`이 현재
page 정보 없이 `getTableBBox()`를 호출하고, native가 같은 다중-page 표의 첫 fragment bbox를
반환하는 경로가 원인 후보다. #2215의 드래그 정확성과 성능은 정상이며, #2400에서 page-scoped
bbox와 텍스트 hit/경계 hit 우선순위를 별도로 다룬다.

## 변경 파일

- native/WASM: `src/document_core/queries/cursor_nav.rs`, `src/wasm_api.rs`
- Studio: `rhwp-studio/src/core/selection-page-hints.ts`, `wasm-bridge.ts`,
  `engine/input-handler.ts`
- 회귀: `tests/issue_2215_selection_page_range.rs`,
  `rhwp-studio/tests/selection-page-hints.test.ts`
- 계획·근거: `mydocs/plans/task_m100_2215*.md`,
  `mydocs/working/task_m100_2215_stage*.md`

로컬 계측 probe와 raw 결과는 `/private/tmp` 산출물로만 유지하고 PR에는 포함하지 않는다.
검증용 `node_modules` 및 HWPX symlink도 변경 목록에서 제외한다.
