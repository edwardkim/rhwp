# Task M100 #1491 Stage 3: Shift+마우스 세로 셀 resize 보정

- 이슈: #1491
- 브랜치: `local/task_m100_1491`
- 작성일: 2026-06-23
- 방법론: Hyper-Waterfall

## 배경

Stage 2에서 가로 방향 Shift+마우스 셀 resize와 복구본 재로드 경로를 보정했다. 작업지시자 추가 검증 결과, 세로 방향에서 Shift+마우스 드래그를 아래로 수행하면 행 높이 또는 대상 셀 높이가 기대대로 독립 조정되지 않는다.

## 재현 관찰

- 입력: `셀보호2.hwp` 계열 표
- 조작: 표 내부 가로 경계선에서 Shift+마우스 드래그 다운
- 증상:
  - 드래그 marker는 표시되지만 최종 반영이 기대한 단일 셀/행 높이 조정과 다르다.
  - 가로 resize 보정과 달리 세로 resize 경로가 정상적으로 분리되지 않는다.

## 가설

1. Stage 2의 Shift 후보 승격은 가로/세로 공통 상태를 다루지만, 세로 적용 단계에서 `singleCellTarget` 또는 `localResize/renderHeight` 힌트가 충분히 전달되지 않을 수 있다.
2. 행 경계 hit-test의 후보 셀 산정이 세로 resize에서 대상 셀의 `start/end` 의미를 올바르게 해석하지 못할 수 있다.
3. Rust 렌더러는 `local_resize_cols`와 `local_resize_cell_heights`를 저장/복구 후 추론하지 않아, 세로 복구본 또는 후속 렌더에서 로컬 높이 힌트가 사라질 수 있다.

## Stage 3 범위

- 세로 Shift+마우스 resize의 대상 산정, marker, finish 적용 경로를 점검한다.
- 필요한 경우 세로 로컬 resize 힌트 적용/복구를 보정한다.
- Stage 2 가로 resize 및 복구본 보정은 유지한다.

## 검증 계획

- `rhwp-studio/tests/table-mouse-resize-1491.test.ts`
  - 세로 Shift resize가 단일 셀 후보와 local height 힌트를 사용하는지 회귀 테스트 추가
- 필요 시 `tests/issue_493_cell_attrs.rs`
  - 세로 로컬 resize 저장/복구 렌더 회귀 테스트 추가
- `cd rhwp-studio && node --test tests/table-mouse-resize-1491.test.ts`
- `cd rhwp-studio && node --test tests/table-resize-undo-cache-1491.test.ts`
- `cd rhwp-studio && npx tsc --noEmit`
- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

## 구현 결과

- 세로 단일 셀 resize에서 같은 열의 나머지 셀에도 `renderHeight` 로컬 힌트를 보내도록 프론트 payload를 보강했다.
- 렌더러의 열별 y-grid 생성에서 `local_resize_cell_heights`를 읽도록 연결했다.
- 저장/복구 후 사라진 세로 로컬 resize 힌트를 추론하기 위해 `Table::inferred_local_resize_cols`를 추가했다.
- 전역 행 높이 계산과 열별 y-grid 생성에서 명시/추론된 로컬 resize 열을 분리하도록 보정했다.
- 세로 로컬 resize 회귀 테스트와 정적 TS 테스트를 추가했다.

## 검증 결과

- `cargo test --profile release-test --test issue_493_cell_attrs local_height_resize_render_height_keeps_target_column_independent -- --nocapture`
  - 1차 실패: 같은 행의 다른 열 높이가 `45.8 -> 61.8`로 같이 커짐을 확인.
  - 이후 로컬 resize 열만 독립 y-grid를 쓰도록 수정했으나, 작업지시자 시각 검증 기준으로 복구본 문제가 아직 남아 있다.
- `wasm-pack build --target web --out-dir pkg` 통과.
  - `pkg` 갱신 후 작업지시자가 7700에서 시각 검증.
  - 복구본 세로 Shift+마우스 resize가 여전히 정상 동작하지 않음을 확인.
- clippy는 사용자 지시로 취소했다. Stage 3 커밋 전 PR/CI 전체 검증은 수행하지 않는다.

## 결론

Stage 3 변경은 복구본 세로 resize 원인을 일부 좁혔지만 문제 해결에는 실패했다. 다음 Stage 4에서는 복구본의 실제 저장 모델 값, 선택된 resize target, `resizeTableCells` payload, 렌더 bbox 재계산을 한 경로로 묶어서 다시 분석한다.
