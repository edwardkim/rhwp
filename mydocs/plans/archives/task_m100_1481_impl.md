# Task M100 #1481 구현 계획서

- 이슈: #1481
- 작업 브랜치: `task_m100_1481`
- 작성일: 2026-06-22
- 상태: Stage 1 구현 진행 중

## 1. 구현 목표

일반 표에서 열을 추가하거나 삭제해도 표 전체 높이가 기존 시각 높이를 유지하도록 보정한다.
또한 한컴 도움말 기준 `줄/칸 추가하기 <Alt+Insert>`와 `줄/칸 지우기 <Alt+Delete>` 대표 명령을
대화상자 흐름으로 구현한다.

핵심은 `cell.height`와 `Table.common.height`의 의미 차이를 구조 편집 경로에서 보존하는 것이다.
일반 `createTable()`은 셀의 저장 height를 `282HU`로 두고 표 외곽 높이를 `1282HU * 행수`로 둔다.
따라서 열 추가/삭제처럼 행 수가 바뀌지 않는 작업에서는 `common.height`를 셀 height 합으로 다시 쓰면 안 된다.

## 2. 변경 후보 파일

### 2.1 `src/model/table.rs`

후보 A:

- `Table::insert_column()` / `Table::delete_column()` 시작 시 기존 `common.height`와 raw `HEIGHT` 값을 보존한다.
- 기존 로직이 `update_ctrl_dimensions()`로 폭/높이를 재계산한 뒤, 열 추가/삭제에서는 height만 기존 값으로 복원한다.
- width는 열 추가/삭제로 실제 변경되어야 하므로 `update_ctrl_dimensions()` 결과를 유지한다.

후보 B:

- `update_ctrl_dimensions()`를 일반 갱신 함수와 width/height 선택 갱신 함수로 분리한다.
- 열 추가/삭제는 width만 갱신하고 height는 보존한다.
- 행 추가는 height 증가 정책을 별도 계산한다.

우선 후보 A가 가장 좁고 PR #1446 Stage 19의 복원 패턴과 유사하다.

### 2.2 `src/document_core/commands/table_ops.rs`

- 필요하면 native 명령 레벨에서 원래 common height를 보존한다.
- `resizeTableCells`의 `original_height` 복원 패턴을 참고한다.
- `raw_ctrl_data[common_obj_offsets::HEIGHT]`와 `table.common.height`를 함께 복원한다.

### 2.3 `src/wasm_api/tests.rs` 또는 관련 Rust test 파일

회귀 테스트 후보:

- `issue_1481_insert_column_keeps_create_table_height`
  - `HwpDocument::create_empty()`
  - `create_table_native(0, 0, 0, 3, 5)`
  - 표의 `common.height` 기록
  - `insert_table_column_native(0, 0, 0, 0, false)`
  - `common.height`가 기존 값과 같고, `common.width`는 증가했는지 확인
- `issue_1481_delete_column_keeps_create_table_height`
  - `HwpDocument::create_empty()`
  - `create_table_native(0, 0, 0, 3, 5)`
  - 표의 `common.height` 기록
  - `delete_table_column_native(0, 0, 0, 0)`
  - `common.height`가 기존 값과 같고, `common.width`는 감소했는지 확인
- `issue_1481_insert_row_increases_height_by_visual_row_height`
  - 행 추가 정책을 확정한 뒤 추가
  - 기준 행의 시각 높이를 어떻게 정의할지 먼저 구현 중 확정

### 2.4 `rhwp-studio` 프론트

- `rhwp-studio/src/ui/table-row-column-dialog.ts`
  - `줄/칸 추가하기` 대화상자
  - `줄/칸 지우기` 대화상자
- `rhwp-studio/src/command/commands/table.ts`
  - `table:insert-row-col`, `table:delete-row-col` 대표 명령 추가
  - 기존 단건 `위/아래 줄 추가`, `왼/오른쪽 칸 추가`, `줄/칸 지우기`는 빠른 실행으로 유지
- `rhwp-studio/src/command/shortcut-map.ts`, `rhwp-studio/index.html`
  - `Alt+Insert`, `Alt+Delete`를 대표 대화상자 명령으로 이동

## 3. 구현 순서

1. 실패 재현 테스트를 먼저 추가한다.
2. 열 추가/삭제 경로에서 `common.height` 보존 보정을 적용한다.
3. 행 추가 경로는 다음 중 하나로 결정한다.
   - 기존 렌더 행 높이를 기준으로 `common.height` 증가
   - 현 stage에서는 열 추가/삭제 회귀만 보정하고 행 추가는 별도 stage 후보로 남김
4. focused 테스트를 실행한다.
5. 한컴 도움말 기준 대표 대화상자 명령을 구현한다.
6. 프론트 타입/단축키 테스트를 실행한다.
7. Stage 1 보고서를 작성하고 작업지시자 확인을 받는다.

## 4. 테스트 명령

구현 직후:

```bash
cargo test --release issue_1481 --lib
git diff --check
```

테이블 주변 회귀:

```bash
cargo test --release table --lib
cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture
```

Studio 경로 확인이 필요하면:

```bash
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && node --test tests/shortcut-map.test.ts tests/menu-shortcut-labels.test.ts
wasm-pack build --target web --out-dir pkg
```

## 5. 시각 검증 계획

- IAB 또는 Browser에서 `http://localhost:7700/` 접속
- 일반 표 생성
- 셀 우클릭 후 `왼쪽에 칸 추가하기`
- 같은 표에서 `오른쪽에 칸 추가하기`
- 같은 표에서 `칸 지우기`
- `Alt+Insert`로 `줄/칸 추가하기` 대화상자 실행
- `Alt+Delete`로 `줄/칸 지우기` 대화상자 실행
- 표 높이가 납작해지지 않는지 확인

## 6. 리스크와 대응

- `update_ctrl_dimensions()`는 저장 raw bytes와 `table.common`을 동시에 갱신하는 공용 함수다.
  - 공용 함수 자체를 급하게 바꾸기보다, 열 추가 경로에서 필요한 축만 보존하는 좁은 보정을 우선한다.
- 행 추가까지 한 번에 일반화하면 row height 의미가 섞일 수 있다.
  - Stage 1은 열 추가/삭제 재현 회귀를 먼저 고정하고, 행 추가는 테스트로 관찰한 뒤 범위를 확정한다.
- PR #1446의 local resize hint와 충돌할 수 있다.
  - `local_resize_rows`, `local_resize_cols`, `local_resize_cell_heights`가 있는 표는 기존 동작 보존을 별도로 확인한다.

## 7. 커밋 계획

Stage 1 승인 후 구현한다.

커밋 제목 후보:

```text
task 1481: 표 열 구조 편집 높이 축소 회귀 보정
```
