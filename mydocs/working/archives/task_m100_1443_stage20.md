# Task M100 #1443 Stage 20 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `3b47d1a8 task 1443: 표 셀 로컬 resize 회귀 보정`

## 1. 목표

Stage 19에서 Shift 개별 셀 resize와 이후 일반 컬럼 resize의 row 흔들림을 보정했지만,
개별 셀 변형을 여러 번 만든 뒤 전체 셀/컬럼 경계를 이동할 때 한컴과 다른 표 구조가 남아 있다.

이번 Stage 20에서는 아래 재현을 기준으로 한컴 호환 동작을 다시 맞춘다.

## 2. 작업지시자 재현

작업지시자 스크린샷 기준:

1. `셀보호2.hwp`에서 여러 행의 개별 셀 경계를 Shift+마우스로 변형한다.
2. 이후 Shift 없이 전체 컬럼/전체 셀 경계를 이동한다.
3. rhwp는 일부 셀 경계가 개별 변형 상태와 전역 경계 이동 사이에서 어긋나며, 한컴 스크린샷처럼 정렬되지 않는다.

## 3. 현재 추정

- Stage 19는 “local resize row를 흔들지 않기”에 초점을 맞춰, local resize 이력이 있는 표의 일반 resize에서 전체 셀의 표시 폭 hint를 보존했다.
- 그러나 한컴 동작은 단순히 local row를 고정하는 것이 아니라, 전역 경계 이동 시 각 행의 같은 논리 경계를 기준으로 영향을 받을 segment와 보존할 segment를 다시 분류하는 것으로 보인다.
- 특히 이미 분리된 segment를 넘어 전체 컬럼 경계를 움직일 때:
  - 같은 논리 boundary에 속한 segment는 이동해야 한다.
  - 이전 Shift로 분리되어 다른 표시 좌표에 있는 segment는 유지되어야 한다.
  - 업데이트 대상이 아닌 셀은 전역 fallback에 의해 흔들리면 안 된다.

## 4. 수정 방향

1. headless 재현 스크립트를 Stage 20 케이스에 맞게 추가 계측한다.
   - 여러 local segment 생성 후 일반 resize 수행
   - 각 resize 호출 payload와 `getTableCellBboxes` 전후 delta를 비교
2. `findAlignedLogicalResizeAffectedCells`와 `isKnownLocalResizeSegment`의 역할을 분리한다.
   - “같은 논리 boundary”와 “현재 표시 좌표가 같은 boundary”를 모두 기록
   - 일반 resize에서는 한컴 기준으로 이동할 segment만 target/neighbor로 선택
3. render hint 생성 시 전체 셀 무조건 고정 대신, 실제 한컴처럼 이동해야 하는 논리 boundary segment와 보존해야 하는 segment를 구분한다.
4. `issue_493_cell_attrs.rs`에 Stage 20 순서 회귀 테스트를 추가한다.

## 5. 검증 계획

- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture`
- `cd rhwp-studio && npx tsc --noEmit`
- `wasm-pack build --target web --out-dir pkg`
- `localhost:7700` headless 실제 드래그 재현
- 필요 시:
  - `cargo test --profile release-test --test issue_1073_nested_table_split -- --nocapture`
  - `cargo test --profile release-test --test svg_snapshot -- --nocapture`
  - `cd rhwp-studio && npm test`
  - `cargo fmt --check`
  - `git diff --check`

## 6. 현재 상태

- Stage 19 변경은 커밋 완료.
- Stage 20 수정 진행 중.

## 7. 진행 결과

### 7.1 원인 확인

Headless 재현으로 다음 순서를 확인했다.

1. `cell[5]` 오른쪽 경계를 Shift+마우스로 80px 이동
2. `cell[10]` 오른쪽 경계를 Shift 없이 크게 이동

Stage 19 상태에서는 일반 resize의 이동 한계가 `TableResizeRenderer.computeBorderLines()`의 전체 선 목록을 기준으로 계산되었다.
이 때문에 `cell[5]`가 만든 local segment 선이 다음 경계선으로 취급되어, 전체 컬럼 경계가 해당 local segment를 넘어서 이동하지 못했다.

### 7.2 조치

- local resize 이력이 있는 표의 일반 resize에서는 전체 선 목록 기준 clamp를 쓰지 않는다.
- 대신 현재 resize의 `affectedCellIndices` target 셀과 그 보상 neighbor 셀 bbox만 기준으로 이동 한계를 계산한다.
- 이로써 전역 컬럼 경계가 다른 행의 분리된 local segment 선에 막히지 않고, 실제 target/neighbor 최소 크기 한계까지 이동할 수 있다.

### 7.3 Headless 재검증

동일 순서 재검증 결과:

- 첫 호출: Shift resize `cell[5]`, updates 5개
- 두 번째 호출: 일반 resize `cell[10]`, updates 25개
- 결과:
  - `cell[0]`, `cell[10]`, `cell[15]` 폭: +125.7px
  - `cell[1]`, `cell[11]`, `cell[16]` 폭: -125.7px
  - 전역 경계: `225.3px` → `351.0px`
  - 기존 local segment: `cell[5]` 경계 `304.7px` 유지
  - 마지막 행 뒤쪽 셀: `cell[22]`, `cell[23]`, `cell[24]` 이동/폭 변화 0px

### 7.4 현재 검증

- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `localhost:7700` headless 실제 드래그 재현
  - 통과
- `cd rhwp-studio && npm test`
  - 통과: 75 passed
- `git diff --check`
  - 통과
