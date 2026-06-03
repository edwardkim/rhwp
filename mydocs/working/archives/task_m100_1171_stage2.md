# Stage 2 완료보고서 — Task #1171

- **이슈**: [#1171](https://github.com/edwardkim/rhwp/issues/1171)
- **브랜치**: `local/task1171`
- **단계 목표**: 백엔드 by_path picture getter/setter 가 글상자(Shape text_box) path
  (마지막 세그먼트=Shape, cell_index=0 sentinel)도 해석하여 속성 read/write 가능하게.
- **작성일**: 2026-06-02

## 변경 내용 (`src/document_core/commands/object_ops.rs`)

### 1. 글상자 mut 헬퍼 import
- `use super::super::helpers::{get_textbox_from_shape, get_textbox_from_shape_mut};`

### 2. getter `get_cell_picture_properties_by_path_native`
- `resolve_cell_by_path`(마지막 세그먼트=표 셀 요구) + 수동 `cell.paragraphs[last.2]` 접근을
  → `resolve_paragraph_by_path`(cursor_nav.rs:584, 표 셀과 글상자 모두 처리, 최종 문단 직접 반환)
  한 줄로 교체. 이후 `cell_para.controls[inner_control_idx]` 의 Picture 조회는 동일.
- 표 셀 동작 불변(resolve_paragraph_by_path 의 Table arm 이 기존과 동일 navigation).

### 3. setter 공통 헬퍼 `resolve_cell_paragraph_mut` 에 Shape arm 추가
- 기존 Table 전용(비-Table 거부) → immutable 짝 `resolve_paragraph_by_path` 와 동일하게
  `Control::Table` + `Control::Shape`(cell_index=0 검증 + `get_textbox_from_shape_mut`) 모두 처리.
- 헬퍼 doc 주석에 "[Task #1171] 이후 표 셀과 글상자를 모두 처리(immutable 짝과 동일)" 명시.
- picture setter `set_cell_picture_properties_by_path_native` 가 이 헬퍼를 통해 글상자 path 처리.

## 범위 메모 (Shape setter 부수 효과)

`resolve_cell_paragraph_mut` 는 picture setter 와 shape setter
(`set_cell_shape_properties_by_path_native`) 가 **공유**한다. Shape arm 추가로 shape setter 도
글상자 안 (중첩) 도형 path 를 처리할 수 있게 되었다(부수 효과, 회귀 아님 — 기존 표 셀 동작
불변, 능력만 추가). 단 shape **getter**(`get_cell_shape_properties_by_path_native`)는 여전히
`resolve_cell_by_path`(표 전용)를 사용하므로, 글상자 안 도형의 get/set 은 완전히 배선되지
않았다. 본 이슈 범위(picture)에서는 프런트가 글상자 안 도형 path 를 보내지 않으므로 무해하며,
글상자 안 도형 편집은 후속(수행계획서 §8 범위 밖)으로 남긴다.

## 검증

- `cargo build` 통과.
- 신규 테스트 `picture_in_textbox_get_set_by_path` (tests/issue_1171_textbox_picture_cellpath.rs):
  - 섹션0 문단25 글상자 picture(inner ctrl 0) 조회 → width+12345 변경 set → 재조회 반영 확인.
  - 두번째 picture(inner ctrl 1) 분리 조회 확인(inner_control_idx 정합).
- **`cargo test --lib`: 1527 passed, 0 failed** (object_ops by_path 셀 테스트 포함 — getter/setter
  교체 회귀 0).
- 셀 by_path/이미지 통합 회귀: issue_1161_image_cellpath, issue_1198_nested_cell_paste,
  issue_717_table_cell_hit_test, issue_table_vpos_01_page5_cell_hit_test 전부 통과.
- `cargo clippy --lib` 변경 파일/라인 경고 0.

## 다음 단계 (Stage 3)

프런트엔드 hit-test: 글상자 내부 클릭이 텍스트 편집으로 단락(input-handler-mouse.ts:744)되기
전에 글상자 안 picture 를 선제 hit-test 하여 picture 객체선택으로 보낸다(picture 우선).
