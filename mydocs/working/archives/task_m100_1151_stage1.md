# Task #1151 Stage 1 (A) 완료 보고서 — insert_picture_native floating 분기 + WASM export

수행계획서: [task_m100_1151.md](../plans/task_m100_1151.md) · 구현계획서: [task_m100_1151_impl.md](../plans/task_m100_1151_impl.md)

## 1. 변경 내용

### src/document_core/commands/object_ops.rs
- `insert_picture_native` 시그니처에 `cell_path: &[(usize, usize, usize)]` 인자 추가.
- 본문 / 셀 분기:
  - `cell_path.is_empty()` → 기존 inline 본문 삽입 (tac=true) 동작 그대로 유지.
  - `cell_path` 있음 → **floating picture 분기**: 셀의 page 좌표를 조회해 `horz_rel_to=Page`, `vert_rel_to=Page`, `horizontal_offset/vertical_offset` HWPUNIT 으로 환산. Picture 는 표가 들어있는 paragraph 의 sibling control 로 `controls.push()`. tac=false, wrap=Square (어울림), z_order=1.
- 신규 helper `compute_cell_page_offset(section_idx, parent_para_idx, cell_path) -> (i32, i32)`:
  - 전 페이지 render tree 순회 → TableCell 매칭 → bbox.x/bbox.y px → ×75 HWPUNIT.
  - 매칭 실패 시 (0, 0) fallback.
- 셀 분기 사후처리: outer Table.dirty=true + mark_section_dirty + paginate_if_needed (insert_text_in_cell_native 패턴 정합).

### src/wasm_api.rs
- `insertPicture` WASM export 시그니처에 `cell_path_json: &str` 추가.
- 빈 문자열 또는 `"[]"` → 본문 inline. 그 외 → `DocumentCore::parse_cell_path` 파싱 후 native 호출.

### 테스트 (src/document_core/commands/object_ops.rs 내 `#[cfg(test)] mod issue_1151_cell_picture_insert_tests`)
- `issue1151_insert_picture_into_table_cell_is_floating_sibling`:
  - 1×1 표 → 셀 안에 picture 삽입 → 셀 paragraph 의 controls 는 비어있음 + table 같은 paragraph 에 sibling Picture 존재 + `picture.common.treat_as_char == false` + `text_wrap == Square` 단언.
- `issue1151_insert_picture_body_keeps_existing_inline_behavior`:
  - cell_path=&[] 로 본문 삽입 → body paragraph 에 inline Picture (`treat_as_char == true`) 회귀 확인.
- `issue1151_invalid_cell_path_returns_error`:
  - 범위 초과 셀 인덱스 → Err.

## 2. TDD 사이클

| Step | 결과 |
|------|------|
| RED | 3 테스트 작성 → 컴파일 에러 (cell_path 인자 부재) ✓ |
| GREEN | helper + floating 분기 + WASM export 동기 → 3/3 PASS ✓ |
| REFACTOR | 공통 자원(shape_attr, border, crop, image_attr) 을 분기 위에 추출하여 두 분기에서 공유 |

## 3. 검증 결과

- `cargo test --lib issue_1151`: **3 passed, 0 failed**
- `cargo test --lib`: **1425 passed, 0 failed, 6 ignored** (회귀 0)
- `cargo test --tests`: 통합 테스트 그룹 모두 통과
- `cargo clippy --lib -- -D warnings`: **무경고**
- `cargo fmt --all -- --check`: **clean** (변경 파일 fmt 적용 후)

## 4. 한컴 정합 검증

`incellpicture.hwp` 의 floating picture 와 동일 패턴:
- `tac=false` ✓
- `text_wrap = Square` (어울림) ✓
- `horz_rel_to = Page`, `vert_rel_to = Page` ✓
- `horizontal_offset / vertical_offset` 으로 위치 지정 ✓
- 표가 있는 paragraph 의 sibling control 로 배치 ✓

(한컴 원본은 `horz_rel_to=Paper` 인데 본 구현은 `Page` 로 단순화. Page 와 Paper 의 의미상 차이가 layout 에 영향이 있는지는 Stage C 시각 검증에서 확인.)

## 5. Stage 2 진입 조건

- 시그니처 확정 ✓
- 본문 회귀 0 ✓
- 셀 floating 단위 검증 통과 ✓

→ Stage 2 (TS bridge + handler) 진행 가능.
