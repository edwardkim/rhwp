# Task M100 #1443 Stage 19 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `26c18a8b task 1443: 렌더링 회귀 테스트 보정`

## 1. 목표

Stage 18에서 해결한 렌더링 회귀 보정 이후 전체 회귀 테스트를 이어간다.
PR 문서는 아직 생성하지 않고, 모든 회귀 검증이 통과한 뒤 별도 단계에서 준비한다.

## 2. 검증 기준

- 기존 golden SVG를 갱신하지 않는다.
- 테스트가 생성한 `.actual.svg`는 산출물로만 보고 커밋하지 않는다.
- 실패가 발생하면 단독 테스트로 원인을 좁힌 뒤 최소 수정한다.
- #1443 기능 요구사항과 기존 문서 렌더링 호환성을 함께 만족시킨다.

## 3. 진행 계획

1. 전체 통합 테스트 실행
   - `cargo test --profile release-test --tests`
2. 실패 테스트 분리
   - 실패한 테스트만 `--nocapture`로 재실행
   - 필요 시 이전 stage 커밋과 비교
3. 렌더링 산출물 정리
   - `.actual.svg` 삭제
   - `git diff --check`
4. 최종 PR 준비 전 검증 대기
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`

## 4. 현재 상태

- Stage 18 커밋 완료.
- PR 관련 문서는 제거했고, PR 문서는 회귀 테스트 전체 통과 후 새로 생성한다.

## 5. 진행 결과

- 불필요한 임시 worktree 제거
  - `/private/tmp/rhwp-head-1443`
  - `/private/tmp/rhwp-pr1429-review`
  - `/private/tmp/rhwp-stage-scan-1443`
  - `/private/tmp/rhwp-upstream-1073`
  - `/private/tmp/rhwp_head_1443`
- 현재 `git worktree list` 기준 남은 worktree는 메인 작업 디렉터리 하나뿐이다.
- `cargo test --profile release-test --tests`
  - 통과
- `cargo fmt --check`
  - 최초 확인에서 `table_layout.rs` 줄바꿈 포맷 차이 1건 발견
  - `cargo fmt` 적용 후 통과
- `git diff --check`
  - 통과
- `tests/golden_svg/**/*.actual.svg`
  - 남은 산출물 없음

## 6. 추가 회귀: `셀보호2.hwp` shift 셀 크기 조절

### 6.1 증상

`samples/셀보호2.hwp`에서 Shift+마우스로 부분 셀 경계를 조절하면 표 모양이 다시 깨진다.
마지막 행은 `cell[20] c=0 cs=1`, `cell[21] c=1 cs=2`처럼 위쪽 행의 `c=0 cs=2` 구조와 다르게 분할되어 있어, 부분 경계가 이웃 경계를 넘어가면 전역 열폭 계산과 충돌한다.

### 6.2 조치

- Studio resize drag에서 Shift 개별 셀 resize는 전역 이전/다음 경계선 기준으로 막지 않는다.
  - 한컴처럼 대상 셀의 부분 경계가 다른 행/열의 경계를 넘어갈 수 있어야 한다.
  - 대신 대상 셀과 같은 행/열의 보상 이웃 셀 둘 중 하나가 최소 크기 `200HU` 미만으로 줄어드는 경우만 clamp한다.
  - drag marker와 실제 mouseup delta 계산 모두 같은 clamp 결과를 사용한다.
- `resizeTableCells` native 적용 시 실제 width/height delta 합이 0이면 table common width/height를 보존한다.
  - 보상 resize는 셀 내부 경계 이동이므로 한컴처럼 표 외곽 크기가 변하지 않아야 한다.
  - raw `CommonObjAttr` width/height와 `table.common`을 같이 복원한다.
- `tests/issue_493_cell_attrs.rs`에 `compensated_cell_resize_keeps_cellprotect2_table_common_size` 회귀 테스트를 추가했다.

### 6.3 추가 검증 결과

- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture`
  - 통과: 11 passed
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `cargo test --profile release-test --test svg_snapshot -- --nocapture`
  - 통과: 8 passed
- `cargo test --profile release-test --test issue_1073_nested_table_split -- --nocapture`
  - 통과: 3 passed
- `cd rhwp-studio && npm test`
  - 통과: 75 passed
- `cargo fmt --check`
  - 통과
- `git diff --check`
  - 통과
- `tests/golden_svg/**/*.actual.svg`
  - 남은 산출물 없음

### 6.4 후속 보정

- 최초 보정은 경계선이 전역 이전/다음 선을 넘지 못하게 해 한컴보다 Shift 컬럼 조절 범위가 좁았다.
- `computeResizePositionBounds`를 조정해 Shift 또는 이미 분리된 로컬 셀 segment resize에서는 전역 경계선이 아니라 대상 셀과 보상 이웃 셀의 현재 bbox를 기준으로 clamp한다.
- 이로써 아래 행의 분리된 셀 경계가 위쪽 행의 기존 열 경계를 넘어가도 허용하고, 표 외곽과 최소 셀 크기만 보호한다.
- Shift 개별 셀 resize의 드래그 마커가 표 전체 높이/너비로 표시되던 문제를 보정했다.
  - `TableResizeRenderer.showDragMarker`가 선택적 marker bbox 범위를 받을 수 있게 했다.
  - 로컬 셀 segment resize 중에는 대상 셀 bbox만 넘겨 한컴처럼 해당 셀 경계 선분만 움직이는 표시가 나오게 했다.
- `localhost:7700` 실제 조작 재현 중 추가로 확인한 문제를 보정했다.
  - 이벤트와 marker는 정상으로 들어왔지만, `cell[20]` 폭을 키운 뒤 `colSpan=2` 이웃 `cell[21]`을 줄이면 렌더러의 전역 column 폭 계산 때문에 `cell[22]` 시작 x가 같이 밀렸다.
  - `resizeTableCells`가 실제 보상 resize가 발생한 행/열을 `Table.local_resize_rows`, `Table.local_resize_cols` transient hint에 기록하도록 했다.
  - `build_row_col_x`는 이 hint가 있는 행에 한해 셀 순서 기반 x 좌표를 사용한다.
  - 로드만 한 기존 문서는 기존 golden 렌더 경로를 유지한다.

후속 보정 뒤 재검증:

- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture`
  - 통과: 11 passed
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `cd rhwp-studio && npm test`
  - 통과: 75 passed
- `cargo test --profile release-test --test issue_1073_nested_table_split -- --nocapture`
  - 통과: 3 passed
- `cargo test --profile release-test --test svg_snapshot -- --nocapture`
  - 통과: 8 passed
- `wasm-pack build --target web --out-dir pkg`
  - 통과
- `localhost:7700` headless 실제 Shift 드래그 재현
  - `cell[20]` 오른쪽 경계 125px 이동
  - marker: 대상 셀 선분 높이 56px
  - `cell[20]` bbox 폭: 79.3px → 204.3px
  - `cell[21]` bbox 폭: 160.9px → 35.9px
  - `cell[22]` x 이동량: 0px

### 6.5 추가 회귀: 로컬 셀 조절 뒤 일반 컬럼 조절

작업지시자 재현:

1. `셀보호2.hwp`에서 두 번째 줄 첫 셀을 Shift+마우스로 오른쪽 조절
2. 이후 전체 첫 번째 컬럼 경계를 Shift 없이 조절
3. 한컴은 기존에 분리된 로컬 셀 segment와 업데이트 대상이 아닌 뒤쪽 행을 유지하지만, rhwp는 일부 뒤쪽 셀 폭이 같이 변했다.

원인:

- Shift resize 후 row 1은 `localResize` hint로 보존되었지만, 이어지는 일반 resize는 모델 `widthDelta`만 전달했다.
- 렌더러의 전역 `col_widths` fallback이 업데이트 대상이 아닌 row 4에도 다시 적용되면서 `cell[23]`, `cell[24]`가 흔들렸다.
- 일반 resize의 target/neighbor도 모델 폭 기준 delta를 써서 표시 폭 기준으로는 target 증가분과 neighbor 감소분이 정확히 상쇄되지 않았다.

조치:

- Studio 일반 resize에서 해당 표에 Shift local resize 이력이 있으면 표시 bbox 기준으로 target/neighbor desired width를 계산한다.
- 이 경우 `resizeTableCells` 호출에 전체 셀의 현재 표시 width를 `renderWidth` hint로 싣는다.
  - target/neighbor는 desired 표시 width와 모델 width 차이를 `widthDelta`로 전달한다.
  - 나머지 셀은 `widthDelta=0`, `localResize=true`, `renderWidth=<현재 표시 폭>`으로 전달한다.
- Rust 적용부는 `widthDelta=0`인 render hint만 받은 행도 `local_resize_rows`에 기록한다.
- 표시 width hint가 있는 local row는 `build_row_col_x`에서 row cell 순서 기반 x 좌표로 렌더링한다.

Headless 실제 조작 재검증:

- 첫 호출: Shift resize `cell[5]`, updates 5개
- 두 번째 호출: 일반 resize `cell[10]`, updates 25개
- 결과:
  - `cell[0]`, `cell[10]` 폭: +59.4px
  - `cell[1]`, `cell[11]` 폭: -59.4px
  - 이전 Shift row: `cell[5]`, `cell[6]`, `cell[7]` 이동/폭 변화 0px
  - 마지막 행 뒤쪽 셀: `cell[22]`, `cell[23]`, `cell[24]` 이동/폭 변화 0px

### 6.6 추가 검증 결과

- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture`
  - 통과: 14 passed
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `wasm-pack build --target web --out-dir pkg`
  - 통과
- `localhost:7700` headless 실제 드래그 재현
  - 통과
- `cargo test --profile release-test --test issue_1073_nested_table_split -- --nocapture`
  - 통과: 3 passed
- `cargo test --profile release-test --test svg_snapshot -- --nocapture`
  - 통과: 8 passed
- `cd rhwp-studio && npm test`
  - 통과: 75 passed
- `cargo fmt --check`
  - 통과
- `git diff --check`
  - 통과
- `tests/golden_svg/**/*.actual.svg`
  - 남은 산출물 없음

## 7. 다음 액션

- 필요 시 `cargo clippy --all-targets -- -D warnings` 재실행
- 최종 통과 후 PR 문서 신규 생성
