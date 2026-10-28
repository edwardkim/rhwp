# Task M100 #1491 Stage 2: Shift+마우스 단일 셀 resize 재보정

- 이슈: #1491
- 브랜치: `local/task_m100_1491`
- 작성일: 2026-06-23
- 방법론: Hyper-Waterfall

## 배경

Stage 1에서는 undo/redo 뒤 표 resize 런타임 캐시를 비우고, 행/열 경계 hit-test가 교차점에서 행 경계를 우선 잡는 문제를 보정했다.

작업지시자 검증 결과, 특정 표에서 첫 화면의 두 번째 줄 첫 컬럼 경계를 Shift+마우스 왼쪽 드래그하면 두 번째 화면처럼 일반 컬럼 resize처럼 보이는 변화가 남는다. 기대 동작은 Shift resize가 현재 셀 또는 현재 행의 단일 셀 경계 조정으로 제한되고, 다른 행의 컬럼 경계가 같이 이동하지 않는 것이다.

추가 검증에서 사용자가 지적한 핵심 증상은 원본 샘플이나 새 표가 아니라 자동 저장 복구본에서 발생했다. 원본 문서에서 Shift resize 후 저장/복구를 거치면 Studio 런타임 힌트(`local_resize_rows`, `local_resize_cell_widths`)는 직렬화되지 않고 셀 모델 폭만 남는다. 이후 복구본 렌더러가 변경된 셀 폭을 전역 컬럼 폭 제약으로 해석해, 복구 직후부터 대상 행의 경계가 어긋나고 다음 Shift resize에서 이전 변형이 다시 재현된다.

## 재현 관찰

- 입력: 셀 보호 샘플 계열 표
- 조작: 두 번째 줄 첫 컬럼 경계에서 Shift+마우스 왼쪽 버튼 드래그
- 증상:
  - 단일 셀 resize가 아닌 전체 컬럼 resize처럼 보이는 변형이 발생한다.
  - 사용자가 보기에는 Shift resize 정상 동작이 아니다.

## 가설

1. Shift modifier가 mousedown 시점에만 판정되어, 드래그 중/종료 시점의 Shift 상태가 반영되지 않을 수 있다.
2. 단일 셀 후보는 계산되지만 `resizeDragState`에 일반 resize 상태로만 남아 finish 단계에서 전체 컬럼 resize 경로로 떨어질 수 있다.
3. 행/열 경계 hover는 개선되었지만, 단일 셀 resize 후보와 최종 적용 경로가 분리되어 있어 사용자 조작과 결과가 일치하지 않을 수 있다.
4. hover cache가 없는 상태에서 Shift를 먼저 누르고 표 경계로 mousedown하면 일반 경계 클릭 분기가 표 bbox를 복구하지 못해 resize가 시작되지 않을 수 있다.
5. HWP5에는 Studio 내부 로컬 resize 힌트를 저장할 필드가 없어, 자동 저장 복구본에서는 같은 행 패턴 안의 소수 폭 벡터를 행 단위 resize 결과로 다시 추론해야 한다.

## Stage 2 범위

- resize drag 상태에 단일 셀 후보를 보존한다.
- mousedown 이후 Shift가 감지되는 경우에도 최종 적용을 단일 셀 resize로 전환할 수 있게 한다.
- marker 표시와 finish 적용 경로가 같은 단일 셀 후보를 사용하게 맞춘다.
- 일반 표 경계 mousedown에서 hover cache가 없으면 현재 좌표의 table hit/layout으로 bbox를 복구한다.
- 저장/복구 후 사라진 로컬 resize 힌트를 렌더 시점에서 보수적으로 복원한다.
- 기존 일반 컬럼 resize, 표 외곽 선택 전환, undo/redo 캐시 정리 동작은 유지한다.

## 검증 계획

- `rhwp-studio/tests/table-mouse-resize-1491.test.ts`
  - Shift 상태가 finish 시점에 확인되어 단일 셀 후보로 적용되는지 정적 회귀 테스트 추가
  - marker 표시가 동적 Shift 단일 셀 후보를 사용하도록 정적 회귀 테스트 추가
- `rhwp-studio/tests/table-resize-undo-cache-1491.test.ts`
  - Stage 1 캐시 정리 테스트 유지
- `tests/issue_493_cell_attrs.rs`
  - 원본 Shift resize → exportHwp → 재로드(복구본 상당) → 두 번째 Shift resize 경로 회귀 테스트 추가
- `cd rhwp-studio && npm test`
- `cd rhwp-studio && npx tsc --noEmit`
- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture`
- `git diff --check`

## 구현 결과

- `resizeDragState`에 `resizeTarget`을 추가해 mousedown 시점에 계산한 단일 셀 후보를 보존했다.
- `promoteResizeDragToSingleCell`을 추가해 drag 중 또는 mouseup 시점에 Shift가 확인되면 보존된 후보를 `singleCellTarget`으로 승격한다.
- 승격 시 단일 셀 기준 resize bounds를 다시 계산해 marker와 최종 적용 좌표가 같은 기준을 사용하게 했다.
- 작은 드래그에서 외곽 표 선택으로 전환하는 조건도 승격된 단일 셀 resize를 기준으로 판정하게 했다.
- IAB 경로는 현재 세션에서 `iab` 브라우저가 노출되지 않아 사용할 수 없었다. 대체로 기존 rhwp Studio E2E helper의 headless Chromium을 사용했다.
- 브라우저 자동화 중 hover cache가 비어 있으면 Shift 시작 mousedown에서 resize state가 생성되지 않는 버그를 발견했다.
- `input-handler-mouse.ts`에 `resolveTableResizeHit`을 추가해 일반 경계 mousedown 시 `hitTest`와 `getPageControlLayout`으로 table bbox를 복구하게 했다.
- `Table::inferred_local_resize_rows`를 추가했다. 같은 셀 배치 패턴을 공유하는 행들 중 다수의 폭 벡터와 다른 소수 행만 저장/복구 후 사라진 행 단위 resize 결과로 추론한다.
- `resolve_column_widths`와 `build_row_col_x`가 추론된 행을 기존 `local_resize_rows`와 동일하게 취급하도록 연결했다. 전역 컬럼 폭 계산에서는 제외하고, 해당 행의 x-grid는 셀 모델 폭 순서로 재구성한다.
- 복구본 재현 테스트에서 `셀보호2.hwp` 원본 Shift resize 후 export/reload를 거치면 기존에는 cell 5 폭이 `31.9px -> 111.9px`, cell 7 x가 `353.7px -> 433.7px`로 변했지만, 수정 후 저장 전 행 단위 폭이 유지된다.

## 검증 결과

- `cd rhwp-studio && node --test tests/table-mouse-resize-1491.test.ts` 통과
- `cd rhwp-studio && node --test tests/table-resize-undo-cache-1491.test.ts` 통과
- `cargo test --profile release-test --test issue_493_cell_attrs recovered_shift_resize_row_keeps_independent_widths -- --nocapture` 통과
- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture` 통과: 16개
- `cargo fmt --check` 통과
- `cd rhwp-studio && npx tsc --noEmit` 통과
- `cd rhwp-studio && npm test` 통과: 141개
- `git diff --check` 통과
- `VITE_URL=http://127.0.0.1:7700 node --input-type=module - --mode=headless ...` 브라우저 자동화 통과
  - `셀보호2.hwp`, `sec=0, para=1, control=0`
  - row 1 첫 셀 경계 Shift drag: cell 5 `dw=-22.6px`, cell 6 `dw=+22.6px`
  - 영향 없어야 하는 cell 0/10/20/21/22: `dw=0px`
  - console error/warn 없음
  - 스크린샷: `/tmp/rhwp-1491-shift-resize-before.png`, `/tmp/rhwp-1491-shift-resize-after.png`
