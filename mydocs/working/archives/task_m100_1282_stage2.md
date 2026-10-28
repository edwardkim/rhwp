# Task 1282 Stage 2 — 셀 높이 자동 증가 구현

## 목적

Stage 1 red test에서 확인된 실패를 최소 범위로 수정한다.

## 변경 내용

- `tests/issue_1282_rotated_cell_picture_resize.rs` 추가
  - `samples/ta-pic-001-r.hwp`의 회전된 셀 내부 그림을 `height=30000`으로 리사이즈한다.
  - 소유 cell height가 `picture.vertOffset + picture.common.height + padding` 이상인지 검증한다.
  - table common height가 증가한 cell height를 따라오는지 검증한다.
  - export HWP를 다시 parse해 cell height 증가가 저장 결과에도 보존되는지 검증한다.
- `src/document_core/commands/object_ops.rs` 보강
  - `set_cell_picture_properties_by_path_native`가 picture 속성 적용 후 직접 소유 cell 높이를 증가시킨다.
  - path 길이 1인 표 셀 picture에만 적용한다.
  - 글상자 path와 깊은 중첩 path는 기존 동작을 유지한다.
  - shrink 조작에서는 cell height를 자동 감소시키지 않는다.
  - `Table::update_ctrl_dimensions()`를 재사용해 `raw_ctrl_data`와 `table.common.height`를 같이 동기화한다.

## 검증

통과:

```text
cargo fmt --check
cargo test --test issue_1282_rotated_cell_picture_resize -- --nocapture
cargo test --test issue_1279_picture_rotation_save
```

## 다음 확인

- rhwp-studio 리사이즈 드래그 경로에서 화면 bbox와 저장 속성 반영이 같은 좌표계로 유지되는지 확인한다.
- 필요하면 Stage 3에서 TypeScript 쪽 중심/크기 안정화 보정을 별도 진행한다.
