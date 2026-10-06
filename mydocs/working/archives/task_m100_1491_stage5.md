# Task M100 #1491 Stage 5: 세로 Shift resize 완료 경로 분석

- 이슈: #1491
- 브랜치: `local/task_m100_1491`
- 작성일: 2026-06-24
- 방법론: Hyper-Waterfall

## 배경

Stage 4에서 세로 Shift resize를 단일 셀 local height가 아니라 행 resize로 유지하도록 전환했지만, 작업지시자 시각 검증 결과 신규 문서와 복구 문서 모두에서 여전히 정상 동작하지 않았다.

화면에는 세로 resize용 가로 marker가 표시되지만 mouseup 후 한컴처럼 행 높이 조정이 반영되지 않는다. 따라서 Stage 5는 정책 추정보다 실제 완료 경로에서 payload가 어떤 분기로 만들어지는지 확인한다.

추가로 작업지시자가 가로 Shift+마우스 셀 너비 조절도 다시 동작하지 않는다고 보고했다. hover marker는 표시되지만 실제 mousedown/drag 시작 경로에서 단일 셀 resize target이 누락될 가능성을 함께 확인한다.

이후 작업지시자가 세로 Shift+마우스도 가로 크기 조절처럼 동작해야 한다고 재확인했다. 따라서 Stage 4의 "세로는 행 resize 유지" 가정은 폐기하고, Shift가 눌린 세로 경계도 `singleCellTarget` 기반 local height resize로 처리한다.

추가 시각 검증에서 세로 local height가 같은 열 전체 격자를 깨뜨리는 문제가 확인되었다. 세로 Shift resize는 해당 셀 경계만 조절되어야 한다. 이후 IAB 검증에서 가로 단일 셀 resize는 정상이나 세로 단일 셀 resize가 아직 개별 셀 단위로 맞지 않는다고 확인했다.

이후 작업지시자가 이전 PR에서는 정상 처리되었다고 알려주었다. 로컬 히스토리 비교 결과 참고 기준은 `5e101a6e task 1443: Shift 셀 단독 리사이즈 구현` 및 후속 `b7b1dd20 task 1443: 표 셀 로컬 resize 회귀 보정`으로 확인했다. 회귀는 `804c6ea0 task 1491: 세로 Shift resize 행 기준 전환`에서 `edge.type === 'col'` 제한을 추가해 세로 Shift가 `singleCellTarget`으로 들어가지 못하게 한 부분이 1차 원인이다.

단, 이전 PR의 세로 payload를 그대로 복원해 `heightDelta`와 `renderHeight`를 함께 보내면 현재 요구사항인 "해당 개별 셀높이만 조절"과 충돌한다. Rust API 검증에서 `heightDelta`를 함께 적용하면 같은 행의 옆 셀 bbox 높이가 함께 커졌다. 따라서 이전 PR에서는 `col` 제한 없는 Shift 단일 셀 경로를 참고하고, 실제 세로 payload는 대상/이웃에 `heightDelta: 0` + `renderHeight`를 보낸다. 같은 열의 나머지 셀에는 변경이 아니라 현재 표시 높이를 유지하는 보존 힌트를 함께 보내야 한다.

작업지시자가 이후 `셀 높이를 같게`와 `셀 너비를 같게` 모두 정상 동작하지 않는다고 확인했다. 두 메뉴 명령은 마우스 resize와 달리 `InputHandler.executeOperation`을 거치지 않고 `services.wasm.resizeTableCells`를 직접 호출해 refresh/undo/cache 경로를 우회하고 있었다. 또한 `셀 높이를 같게`는 모델 `cell.height` 평균과 `heightDelta`만 사용해 local render height가 있는 표에서 시각 결과가 맞지 않았다.

## 가설

1. row edge에서도 셀 선택 모드이면 `finishResizeDrag`가 선택 셀 전용 분기로 들어가 행 전체 resize가 되지 않는다.
2. mouseup 좌표 또는 clamp 결과로 `heightDelta`가 0 처리되어 적용이 생략된다.
3. `resizeTableCells`는 호출되지만 렌더/캐시 갱신 때문에 화면에 반영되지 않는다.
4. hover는 4px 허용폭으로 경계를 표시하지만 `findSingleCellResizeTarget`은 1px만 허용해 Shift+마우스 시작 시 resize state가 만들어지지 않는다.
5. `promoteResizeDragToSingleCell`와 `shouldResizeSingleCell`이 `col` 경계로 제한되어 세로 Shift resize가 local height 경로에 들어가지 못한다.
6. 세로 Shift에서 같은 열의 나머지 셀 현재 표시 높이를 보존하지 않으면 복구본/기존 조정 표에서 모델 height로 되돌아가 경계가 흔들릴 수 있다.
7. 이전 PR처럼 세로 Shift를 `singleCellTarget`으로 보내되, 현재 요구사항에서는 모델 `heightDelta` 없이 표시 높이만 바꿔야 한다.
8. 셀 높이/너비 균등화 메뉴는 마우스 resize와 같은 operation 경로를 타야 화면 갱신과 undo/cache 정리가 일관된다.

## 진행 계획

1. row edge에서 셀 선택 모드 전용 분기를 타는지 먼저 차단하고, 행 전체 일반 resize 분기로 보내도록 수정한다.
2. 정적 테스트로 row edge가 cell-selection resize 분기를 타지 않는다는 회귀 방지를 추가한다.
3. 가로 Shift 단일 셀 resize target 판정과 hover 경계 판정의 허용폭 불일치를 보정한다.
4. Shift 단일 셀 resize 승격을 가로/세로 공통 경로로 복원한다.
5. 세로 Shift payload를 대상/이웃 셀의 `heightDelta: 0` + `renderHeight` 변경과 같은 열 나머지 셀의 보존 `renderHeight` hint로 좁힌다.
6. 셀 높이/너비 균등화 명령을 bbox 표시 크기 기준과 `executeOperation` 경로로 맞춘다.
7. 최소 Rust 테스트와 Studio 테스트를 수행한다.
8. `wasm-pack build --target web --out-dir pkg`를 먼저 수행하고 작업지시자 시각 검증을 받는다.

## 현재 분석

- Stage 4 이후에도 `finishResizeDrag`에는 셀 선택 모드 전용 분기가 남아 있었다.
- 이 분기는 row edge에서도 선택 셀과 같은 열의 아래 이웃만 `heightDelta` 보상 대상으로 삼는다.
- 따라서 커서/선택 상태가 셀 선택 모드인 경우, 세로 경계 조작이 한컴식 행 전체 높이 조절로 가지 못하고 선택 셀 기준 부분 높이 조절로 흐를 수 있다.
- hover marker 판정은 `hitTestBorder`의 기본 허용폭 4px로 성공하지만, 실제 단일 셀 resize target 계산은 1px 허용폭만 사용하고 있었다.
- 경계선 가까이 또는 교차점 근처에서 marker는 표시되는데 `startResizeDrag`가 target을 얻지 못해 resize state가 만들어지지 않을 수 있다.
- Stage 4에서 `edge.type === 'col'` 제한을 넣으면서 세로 Shift resize가 `singleCellTarget`으로 승격되지 않았다.
- 이전 정상 PR(`b7b1dd20`)에서는 세로도 `edge.type === 'col'` 제한 없이 Shift 단일 셀 경로로 들어갔다.
- 해당 PR의 세로 payload처럼 `heightDelta`를 실제 모델 변화량으로 보내면 현재 검증 샘플에서는 같은 행의 옆 셀 높이가 함께 변한다.
- `e5880112` 이후 같은 열 나머지 셀에도 `pushLocalResizeHeightHint`를 보내는 변경이 들어갔다.
- 같은 열 나머지 셀의 보존 힌트는 필요하지만, 대상/이웃 셀에 모델 `heightDelta`까지 적용하면 같은 행 옆 셀이 흔들린다.
- 따라서 복원 기준은 `b7b1dd20`처럼 세로 Shift를 단일 셀 경로로 보내고, 대상/이웃은 `heightDelta: 0`으로 모델 행 높이를 흔들지 않으며, 같은 열 나머지 셀은 현재 표시 높이 보존 힌트만 보내는 형태다.
- 셀 높이/너비 균등화 명령은 선택 셀 평균 계산은 bbox 표시 크기 기준으로 수행하고, 적용은 `ih.executeOperation({ kind: 'snapshot' })` 내부에서 `resizeTableCells`를 호출해야 한다.

## 현재 수정

- row edge에서는 셀 선택 모드 여부와 무관하게 일반 행 전체 resize 분기로 진행하도록 변경했다.
- cell-selection 전용 보상 분기는 가로 col edge에만 유지했다.
- `table-mouse-resize-1491.test.ts`에 row edge가 cell-selection 전용 분기로 들어가지 않는 조건을 추가했다.
- `findSingleCellResizeTarget`의 허용폭을 hover와 같은 4px로 맞춰, 표시된 경계가 mousedown에서도 Shift 단일 셀 resize 대상으로 잡히도록 했다.
- `table-mouse-resize-1491.test.ts`에 단일 셀 target 판정 허용폭 회귀 방지를 추가했다.
- `promoteResizeDragToSingleCell`의 `col` 제한을 제거해 세로 Shift resize도 가로처럼 단일 셀 local height 경로로 승격되게 했다.
- `shouldResizeSingleCell`의 `col` 제한을 제거해 mousedown 시 Shift가 눌린 세로 경계도 즉시 `singleCellTarget`을 갖도록 했다.
- 세로 Shift 단일 셀 resize에서는 같은 열 나머지 셀의 현재 높이 보존 hint를 유지했다.
- 세로 Shift 단일 셀 resize에서는 대상/이웃 셀에 `heightDelta: 0`과 `renderHeight`를 전달한다.
- Shift 없는 일반 세로 경계는 계속 행 전체 resize 경로를 사용한다.
- `셀 높이를 같게`는 bbox 표시 높이 평균을 계산하고 각 대상 셀에 `heightDelta: 0`, `localResize: true`, `renderHeight: avgHeight`를 보낸다.
- `셀 너비를 같게`는 기존 bbox 표시 폭/localResize 방식은 유지하되, 직접 WASM 호출 대신 `executeOperation` 경로로 적용한다.

## 현재 검증

- `cd rhwp-studio && node --test tests/table-mouse-resize-1491.test.ts` 통과: 8개
- `cd rhwp-studio && node --test tests/table-cell-width-equal-1491.test.ts` 통과: 6개
- `cd rhwp-studio && node --test tests/table-resize-undo-cache-1491.test.ts` 통과: 3개
- `cargo test --profile release-test --test issue_493_cell_attrs vertical_resize_keeps_row_cells_aligned -- --nocapture` 통과
- `cargo test --profile release-test --test issue_493_cell_attrs vertical_shift_local_height_keeps_unrelated_cells_stable -- --nocapture` 통과
- `cargo fmt --check` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- `cd rhwp-studio && npm run build` 통과
- IAB 7700 스모크 통과: `http://localhost:7700/`, title `rhwp-studio`, console error/warn 0건
- 균등화 명령 수정 후 IAB 7700 reload 통과: console error/warn 0건
- IAB screenshot capture는 `Page.captureScreenshot` timeout으로 생략했다.
- 7700 개발 서버는 작업지시자가 수동으로 띄운 프로세스이므로 건드리지 않았다.

## 회귀 위치 결론

- 이전 정상 PR 참고점: `5e101a6e`, `b7b1dd20`
- 회귀 도입 커밋: `804c6ea0`
- 직접 원인: `promoteResizeDragToSingleCell`과 `shouldResizeSingleCell`에 `edge.type === 'col'` 제한이 들어가 세로 Shift resize가 단일 셀 경로로 승격되지 않았다.
- 보조 원인: 같은 열의 나머지 셀을 현재 표시 높이로 보존하지 않으면 복구본/기존 조정 표에서 개별 셀 경계가 흔들릴 수 있다.
- 균등화 메뉴 원인: `셀 높이를 같게`/`셀 너비를 같게`가 직접 WASM 호출로 적용되어 operation refresh/undo/cache 경로를 우회했다. 높이 균등화는 추가로 모델 높이 기준이라 local render height 표와 맞지 않았다.
- 검증 메모: 이전 PR의 모델 `heightDelta`까지 그대로 복원하면 같은 행 옆 셀 높이가 바뀌는 실패가 재현되었다. 따라서 현재 요구사항에는 `heightDelta: 0` + `renderHeight`가 맞다.

## 시각 검증 대기

- 7700에서 신규 문서와 복구 문서 모두 세로 경계를 Shift+마우스로 아래/위 조절했을 때 해당 셀 경계만 조절되고 무관한 셀/행 경계가 깨지지 않는지 확인이 필요하다.
