# Task #1151 구현계획서 — Floating picture 방식

수행계획서: [task_m100_1151.md](task_m100_1151.md)

## 0. 설계 결정

### 0-1. cell_path 전달 방식 (WASM)
- `cell_path_json: &str` — wasm_api.rs 의 9개 셀 API 와 동일 패턴 (`controlIndex/cellIndex/cellParaIndex` long form). devel 에 이미 존재하는 `parse_cell_path` 재사용.

### 0-2. Picture 속성 (floating, 한컴 정합)
- `treat_as_char = false`
- `horz_rel_to = Page` (Paper 와 거의 동등, 단순화 위해 Page 사용; section 의 page_def 기준)
- `vert_rel_to = Page`
- `text_wrap = Square` (어울림)
- `horizontal_offset / vertical_offset` = 셀의 page bbox 좌상단 (HWPUNIT)
- z_order 는 기존 패턴 (1+ 적절히)

### 0-3. 셀 좌표 조회
- `cursor_rect.rs:get_table_cell_bboxes_by_path_native` 패턴 차용 — render tree 에서 cell bbox 를 page-relative px 로 얻음
- px → HWPUNIT 환산: `× 75` (1px = 75 HWPUNIT at 96 DPI)
- 신규 helper: `compute_cell_page_offset` (object_ops.rs 내부) 또는 인라인

### 0-4. 삽입 위치 (section.paragraphs 중 어디에)
- `incellpicture.hwp` 분석: table 과 같은 paragraph (section.paragraphs[0]) 의 sibling control 로 삽입됨 (인덱스는 table 다음)
- 우리 코드: parent_para_idx (= 표가 들어있는 paragraph) 의 controls 마지막에 picture 를 append. ctrl_data_records 도 동기.

### 0-5. 본문 vs 셀 분기
- `cell_path.is_empty()` → 기존 inline 본문 삽입 (변경 없음)
- `cell_path` 있음 → 신규 floating 분기

---

## Stage A — Rust `insert_picture_native` floating 분기 + WASM export + 테스트

### A-1. helper 추가 (object_ops.rs 내부)
`compute_cell_page_bbox(section_idx, parent_para_idx, cell_path) -> Option<(i32 offset_x_hu, i32 offset_y_hu)>`
- `build_page_tree` 순회 (모든 페이지) → TableCell 매칭 → bbox.x/y 추출 → ×75 환산
- 매칭 실패 시 None (호출 측은 fallback 으로 0,0)

### A-2. insert_picture_native 시그니처 확장
```rust
pub fn insert_picture_native(
    &mut self,
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    cell_path: &[(usize, usize, usize)],  // 신규
    image_data: &[u8], width: u32, height: u32,
    natural_width_px: u32, natural_height_px: u32,
    extension: &str, description: &str,
) -> Result<String, HwpError>
```
- 본문 (`cell_path.is_empty()`): 기존 로직 그대로 (inline tac=true, body paragraph 삽입)
- 셀 (`!cell_path.is_empty()`): floating 분기
  - 셀 좌표 조회 (A-1)
  - Picture 생성 (tac=false, wrap=Square, horz/vert_rel_to=Page, offsets)
  - `section.paragraphs[parent_para_idx].controls.push(Control::Picture)` + `ctrl_data_records.push(None)` + raw_stream=None
  - Table.dirty 마킹, mark_section_dirty, paginate_if_needed

### A-3. WASM export 동기
`src/wasm_api.rs` `insertPicture` 에 `cell_path_json: &str` 추가. 빈 문자열/`"[]"` → 본문, 그 외 `parse_cell_path` → slice.

### A-4. 단위 테스트
`src/document_core/commands/object_ops.rs` 내 `#[cfg(test)] mod`:
1. 1×1 표 셀에 picture 삽입 → table 같은 paragraph 에 sibling Control::Picture, picture.common.treat_as_char==false
2. 본문 picture 삽입 (cell_path=&[]) → 기존 inline 동작 회귀
3. 잘못된 cell_path → Err

### A-5. 검증
- `cargo test --lib issue_1151`: GREEN
- `cargo test --lib`: 회귀 0
- `cargo clippy --lib -- -D warnings`, `cargo fmt --all -- --check`

---

## Stage B — TS bridge + handler

### B-1. wasm-bridge.ts
`insertPicture(sec, paraIdx, charOffset, cellPathJson, imageData, ...)` 시그니처 확장. 반환 타입은 기존 본문 패턴 그대로 (`{ok, paraIdx, controlIdx}`).

### B-2. input-handler-table.ts (finishImagePlacement)
- `hit.cellPath` 존재 시 `JSON.stringify(hit.cellPath)` 와 `hit.parentParaIndex` 사용
- 본문: 빈 cellPathJson

### B-3. input-handler-keyboard.ts (paste image)
- `cursor.getPosition()` 의 cellPath 동일 처리

### B-4. 검증
- `npx tsc --noEmit` (사전 무관 canvaskit 에러 제외)
- WASM 빌드 (`docker compose --env-file .env.docker run --rm wasm`)

---

## Stage C — 수동 검증 + 회귀 + 최종 보고서 + PR

### C-1. 수동 시나리오
1. 신규 문서 → 표 (3×3) 만들기 → 셀 클릭 → 이미지 삽입
2. 이미지가 셀 영역에 floating 으로 표시되는지
3. **다른 셀 클릭 시 cursor 이동 정상** (#1151 핵심)
4. 본문 클릭 → 이미지 삽입 (회귀)
5. Ctrl+V paste 이미지 (셀/본문)

### C-2. 자동 회귀
- `cargo test`, `cargo clippy --lib -- -D warnings`, `cargo fmt --all -- --check`
- rhwp-studio `npx tsc --noEmit`

### C-3. 보고서 + PR
- `mydocs/working/task_m100_1151_stage{1,2,3}.md` (단계별)
- `mydocs/report/task_m100_1151_report.md` (최종)
- `git push -u origin local/task1151`
- `gh pr create --repo edwardkim/rhwp --base devel --head johndoekim:local/task1151 --title "Task #1151: ..."`

---

## 단계별 보고서 위치
각 Stage 완료 시 `mydocs/working/task_m100_1151_stage{N}.md` 와 함께 커밋.
