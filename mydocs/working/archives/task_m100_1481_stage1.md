# Task M100 #1481 Stage 1

- 이슈: #1481 표 열 추가 후 일반 표 높이가 납작해지는 회귀
- 브랜치: `task_m100_1481`
- 작성일: 2026-06-22
- 상태: 구현 및 focused 검증 완료. 작업지시자 확인 대기.

## 목표

일반 표에서 `왼쪽에 칸 추가하기`, `오른쪽에 칸 추가하기`, `칸 지우기`를 실행했을 때 표 전체
높이가 셀 저장 height 합 수준으로 납작해지는 회귀를 보정한다.

추가로 한컴 도움말 기준 `줄/칸 추가하기 <Alt+Insert>`와 `줄/칸 지우기 <Alt+Delete>` 대표 기능이
단순 메뉴 표시가 아니라 대화상자 흐름으로 동작하도록 보강한다.

## 범위

- 열 추가/삭제 시 기존 `Table.common.height`와 raw `HEIGHT` 값을 보존한다.
- 열 추가/삭제로 실제 변경되어야 하는 width는 기존 `update_ctrl_dimensions()` 결과를 유지한다.
- focused 회귀 테스트를 추가해 일반 `createTable()` 계열 표의 높이 보존을 고정한다.
- 상단 표 메뉴에 `줄/칸 추가하기`, `줄/칸 지우기` 대표 항목을 추가한다.
- `Alt+Insert`, `Alt+Delete`는 한컴 도움말처럼 대표 대화상자를 열도록 매핑한다.
- `줄/칸 추가하기`는 방향과 추가 개수(1~63)를 선택해 반복 적용한다.
- `줄/칸 지우기`는 현재 셀이 포함된 줄 또는 칸을 선택해 삭제한다.

## 구현 순서

1. `issue_1481` 재현 테스트를 먼저 추가한다.
2. `Table::insert_column()` 열 추가 경로에서 height 축을 보존한다.
3. 사용자 추가 제보에 따라 `Table::delete_column()` 열 삭제 경로도 같은 방식으로 보존한다.
4. `cargo test --release issue_1481 --lib`로 focused 검증한다.
5. `cargo test --release table --lib`로 표 주변 회귀를 확인한다.
6. 한컴 도움말 기준 `줄/칸 추가하기`, `줄/칸 지우기` 대화상자 명령을 구현한다.
7. 프론트 타입/단축키 테스트를 실행한다.
8. `git diff --check`로 문서/소스 diff를 확인한다.

## 구현 결과

- `src/model/table.rs`
  - `insert_column()`에서 기존 표 외곽 height를 저장한 뒤, 폭 갱신 후 height만 복원한다.
  - `delete_column()`도 동일하게 열 삭제 후 height를 복원한다.
  - `raw_ctrl_data[HEIGHT]`와 `table.common.height`를 함께 복원한다.
- `src/wasm_api/tests.rs`
  - `issue_1481_insert_column_keeps_create_table_height`
  - `issue_1481_delete_column_keeps_create_table_height`
- `rhwp-studio/src/ui/table-row-column-dialog.ts`
  - `줄/칸 추가하기` 대화상자 추가
  - `줄/칸 지우기` 대화상자 추가
- `rhwp-studio/src/command/commands/table.ts`
  - `table:insert-row-col`, `table:delete-row-col` 대표 명령 추가
  - 기존 단건 줄/칸 추가·삭제 명령은 빠른 실행 경로로 유지
- `rhwp-studio/index.html`, `rhwp-studio/src/command/shortcut-map.ts`
  - `Alt+Insert`, `Alt+Delete`를 대표 대화상자 명령으로 이동

## 검증 결과

```bash
cargo test --release issue_1481 --lib
# 2 passed

cargo test --release table --lib
# 254 passed, 3 ignored

cargo fmt --check

wasm-pack build --target web --out-dir pkg

cd rhwp-studio && npx tsc --noEmit

cd rhwp-studio && node --test tests/shortcut-map.test.ts tests/menu-shortcut-labels.test.ts
# 12 passed

cd rhwp-studio && npm test
# 118 passed

IAB http://localhost:7700/
# 새 문서 생성 후 상단 표 메뉴에 `줄/칸 추가하기` / `줄/칸 지우기`와 `⌥Insert` / `⌥Delete` 표시 확인
# IAB 콘솔 error 로그 없음

git diff --check
```

## 메모

일반 `createTable()`은 셀 height와 표 외곽 `common.height`가 서로 다른 의미를 가진다.
열 추가/삭제는 행 수가 바뀌지 않는 구조 편집이므로, 표 외곽 height를 셀 height 합으로 다시 쓰면 안 된다.
