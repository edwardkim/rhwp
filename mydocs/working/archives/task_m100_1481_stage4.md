# Task M100 #1481 Stage 4

- 이슈: #1481 표 줄/칸 편집 회귀 및 표 resize 표시 높이 회귀
- 브랜치: `task_m100_1481`
- 작성일: 2026-06-22
- 상태: 하단선 resize 회귀 보정 및 로컬 검증 완료

## 증상

일반 표를 생성한 직후 맨 아랫줄을 드래그해 표 높이를 조절하면 행 간격이 셀 저장 height 합으로 다시 계산되어 표가 납작하게 줄어든다.

## 원인

`create_table_native()`로 만든 일반 표는 저장 셀 height 합보다 큰 `Table.common.height`를 갖고, 렌더러는 이 표시 height를 기준으로 행을 늘려 보여준다.

그런데 하단선 resize 경로는 `resize_table_cells_native()`에서 셀 `heightDelta`만 갱신한 뒤 `Table::update_ctrl_dimensions()`를 호출한다. 이 함수는 `get_row_heights()` 합으로 `common.height`와 raw common height를 다시 쓰므로, 생성 직후 표의 표시 height가 셀 저장 height 합으로 붕괴한다.

## 구현 방향

- resize 전 row height 합과 `common.height`를 기록한다.
- resize 후 row height 합 변화량을 계산한다.
- resize 전 `common.height > row_height_sum`인 표는 `common.height = 기존 common.height + row_height_sum_delta`로 동기화한다.
- raw common height도 같은 값으로 갱신한다.
- 내부 경계 보상 resize처럼 row height 합 변화가 0인 경우에는 기존 표시 height를 유지한다.

## 검증 계획

```bash
cargo test --release issue_1481_resize --lib
cargo test --release issue_1481 --lib
cargo fmt --check
wasm-pack build --target web --out-dir pkg
git diff --check
```

## 구현 결과

- `resize_table_cells_native()`에서 resize 전 row height 합과 표시 height를 기록한다.
- 세로 resize 후 row height 합 변화량만 기존 표시 height에 반영한다.
- `common.height`가 셀 저장 height 합보다 큰 일반 표는 resize 후에도 표시 height를 보존한다.
- raw common height도 같은 값으로 동기화한다.
- 회귀 테스트 `issue_1481_resize_bottom_row_keeps_create_table_display_height`를 추가했다.

## 검증 결과

```bash
cargo test --release issue_1481_resize --lib
# 1 passed

cargo test --release issue_1481 --lib
# 5 passed

cargo fmt --check

wasm-pack build --target web --out-dir pkg
# Done in 1m 14s

git diff --check
```
