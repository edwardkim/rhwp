# Task M100 #1491 Stage 4: 복구본 세로 Shift resize 재분석

- 이슈: #1491
- 브랜치: `local/task_m100_1491`
- 작성일: 2026-06-24
- 방법론: Hyper-Waterfall

## 배경

Stage 3에서 세로 Shift resize의 `renderHeight` 힌트와 로컬 열 추론을 보강했지만, 작업지시자 시각 검증 결과 복구본에서는 여전히 정상 동작하지 않았다. 특히 자동 저장 복구본을 다시 복구한 뒤 세로 경계를 Shift+마우스로 아래로 이동하면 표의 세로 경계와 행 높이가 기대와 다르게 커진다.

## Stage 4 목표

- 복구본에서 실제 저장된 셀 `height`, 렌더 bbox, `resizeTableCells` payload를 한 흐름으로 확인한다.
- 세로 Shift resize가 선택한 target/neighbor가 사용자 조작 위치와 일치하는지 확인한다.
- 프론트 payload 문제가 아니라 Rust 렌더/복구 추론 문제인지, 또는 양쪽 모두인지 분리한다.
- 수정 후 반드시 `wasm-pack build --target web --out-dir pkg`를 먼저 수행하고 시각 판단을 받는다.

## 분석 계획

1. `셀보호2.hwp`를 기준으로 세로 Shift resize → export/reload를 스크립트로 재현한다.
2. 복구본에서 주요 셀의 `height`, bbox `y/h`, target/neighbor 관계를 기록한다.
3. 현재 `inferred_local_resize_cols`가 실제 복구본에서 어떤 열을 추론하는지 확인한다.
4. 프론트에서 세로 Shift resize payload가 `localResize/renderHeight`를 어떤 셀에 보내는지 검증한다.
5. 원인을 좁힌 뒤 최소 수정한다.

## 검증 계획

- 수정 후 `wasm-pack build --target web --out-dir pkg`를 먼저 수행한다.
- 시각 검증 대기.
- 시각 통과 후 필요한 단위 테스트와 타입체크를 수행한다.

## 현재 분석

- 신규 파일에서도 세로 Shift resize가 깨지는 것으로 확인되어, 복구본 전용 문제가 아니라 세로 resize 정책 문제로 재분류했다.
- Stage 3의 세로 local-height 대칭 보정은 오판이었다. 한컴 동작은 세로 경계에서 단일 열만 독립 높이로 분리하지 않고, 행 경계가 유지되는 row resize로 동작한다.
- 따라서 row edge에서는 Shift가 눌려도 `singleCellTarget`으로 승격하지 않도록 수정했다.
- 저장/복구 파일에서 세로 로컬 열을 추론하는 `inferred_local_resize_cols`도 제거했다. HWP 저장 파일에는 열별 독립 행 높이를 보존할 근거가 없으므로, 복구본은 행 높이 기준으로 정렬 렌더해야 한다.

## 현재 검증

- `cd rhwp-studio && node --test tests/table-mouse-resize-1491.test.ts` 통과: 7개
- `cargo test --profile release-test --test issue_493_cell_attrs vertical_resize_keeps_row_cells_aligned -- --nocapture` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- 작업지시자 시각 검증 실패
  - 신규 문서와 복구 문서 모두에서 Shift+마우스 세로 셀 높이 조정이 여전히 정상 동작하지 않는다.
  - 화면에는 가로 경계 marker가 표시되지만, mouseup 후 한컴처럼 셀 높이 변경이 반영되지 않는다.

## 결론

Stage 4의 “세로 Shift는 행 resize로 유지” 수정만으로는 부족하다. 다음 Stage 5에서는 resize 정책보다 먼저 실제 이벤트 완료 경로를 확인한다. 특히 row edge에서 `resizeTableCells` payload가 생성/전송되는지, `heightDelta`가 0으로 clamp되는지, 또는 적용 후 렌더 캐시/복구 캐시 때문에 화면에 반영되지 않는지를 분리한다.
