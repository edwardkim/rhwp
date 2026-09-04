# Task M100 #1443 Stage 17 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `29136874 task 1443: 안 여백 off 리플로우 기준 보정`

## 1. 목표

`samples/셀보호2.hwp`에서 표 객체 선택 후 마우스로 표를 이동할 수 있게 하고, 표/셀 속성 기본 탭 값이 한컴과 맞도록 확인/수정한다.

## 2. 사용자 확인 현상

- 표 객체 선택 상태에서 마우스로 드래그해도 표가 이동하지 않는다.
- 표/셀 속성 기본 탭 값이 한컴과 다르다.
  - rhwp 스샷: 너비 `146.2mm`, 높이 `61.6mm`, 세로 기준 `-5.0mm`
  - 한컴 스샷: 너비 `152.36mm`, 높이 `63.26mm`, 세로 기준 `78.08mm`

## 3. 1차 분석

### 3.1 마우스 표 이동

현재 `rhwp-studio/src/engine/input-handler-mouse.ts`는 표 객체 선택 상태에서 좌클릭이 선택된 표 내부 셀에 hit되면 즉시 표 객체 선택을 풀고 셀 편집 진입으로 넘긴다.

현재 흐름:

1. 표 객체 선택 상태
2. 표 내부 좌클릭
3. `hitTest`가 같은 표 셀을 반환
4. border click이 아니면 `enterSelectedTableCell = true`
5. 표 이동 드래그 시작 전에 셀 편집 진입

따라서 `moveDragState`가 만들어지지 않아 마우스 이동이 동작하지 않는다.

한컴처럼 처리하려면:

- `mousedown` 시에는 표 이동/셀 진입 후보 상태로 잡는다.
- `mousemove`가 임계값 이상이면 표 이동 드래그를 시작한다.
- `mouseup`까지 움직임이 거의 없으면 기존처럼 셀 편집 진입으로 처리한다.

### 3.2 표속성 값

`get_table_properties_native`는 raw `CommonObjAttr`에서 width/height/v_offset/h_offset을 읽는다.

`samples/셀보호2.hwp` 덤프 기준:

- `common.width = 43190HU` → 약 `152.4mm`
- `common.height = 17932HU` → 약 `63.3mm`
- `vertical_offset = 22133HU` → 약 `78.1mm`

즉 현재 코드가 최신 WASM으로 실행된다면 한컴 스샷과 같은 값이 내려와야 한다. rhwp 스샷의 `146.2mm`, `61.6mm`, `-5.0mm`는 다음 중 하나일 수 있다.

- Stage 14 이전 WASM/pkg 또는 브라우저 캐시가 동작 중
- 표 이동/속성 저장 과정에서 raw/common 값이 잘못 갱신됨
- UI가 table props가 아닌 렌더 bbox/셀 합산값을 표시하는 경로가 있음

## 4. 수정 계획

- 표 마우스 이동:
  - 표 객체 선택 상태에서 내부 좌클릭 시 즉시 셀 진입하지 않고 후보 상태를 둔다.
  - drag threshold를 넘으면 기존 `moveDragState`를 생성해 `moveTableOffset` 경로를 사용한다.
  - 클릭만 하면 기존처럼 셀 진입을 유지한다.
- 표속성:
  - `samples/셀보호2.hwp`의 `getTableProperties` 반환값을 테스트로 고정한다.
  - `tableWidth/tableHeight/vertOffset/horzOffset`가 한컴 기준 raw/common 값과 맞는지 확인한다.
  - 필요하면 `raw_ctrl_data` 대신 `table.common`을 우선 반환하거나, 표 이동/속성 저장 시 raw/common 동기화를 보정한다.

## 5. 검증 계획

- Rust:
  - `cargo test --test issue_493_cell_attrs -- --nocapture`
- Studio:
  - `cd rhwp-studio && npx tsc --noEmit`
  - 표 객체 선택 후 내부 드래그로 `moveTableOffset` 호출 및 선택 유지 확인
- WASM:
  - `wasm-pack build --target web --out-dir pkg`
- 수동 시각 검증:
  - `셀보호2.hwp` 표 선택 → 마우스 드래그 이동
  - 표/셀 속성 기본 탭 값 확인

## 6. 수정 결과

- `moveTableOffset`가 `raw_ctrl_data`만 갱신하던 문제를 수정했다.
  - `table.common.vertical_offset`
  - `table.common.horizontal_offset`
- `setTableProperties`에서 위치/배치 속성의 raw/common 이중 표현을 같이 갱신하도록 보정했다.
  - `treatAsChar`
  - `textWrap`
  - `vertRelTo`, `vertAlign`
  - `horzRelTo`, `horzAlign`
  - `vertOffset`, `horzOffset`
  - `restrictInPage`, `allowOverlap`, `keepWithAnchor`
  - 바깥 여백
- 표 객체 선택 상태에서 표 내부를 누르는 경우:
  - `mousedown` 시 표 이동 후보 상태를 만든다.
  - 3px 이상 움직이면 표 이동 드래그로 처리한다.
  - 움직임 없이 `mouseup`하면 기존처럼 해당 셀 편집 상태로 진입한다.
- `samples/셀보호2.hwp`의 한컴 기준 표 속성값을 회귀 테스트로 고정했다.
  - `tableWidth = 43190`
  - `tableHeight = 17932`
  - `horzOffset = 4`
  - `vertOffset = 22133`
  - `horzRelTo = Column`
  - `vertRelTo = Para`
  - `textWrap = TopAndBottom`

## 7. 검증 결과

- `cargo test --test issue_493_cell_attrs -- --nocapture`
  - 통과: 10 passed
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `wasm-pack build --target web --out-dir pkg`
  - 통과
- `git diff --check`
  - 통과
