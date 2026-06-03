# Stage 1 보고서 — Task M100-1198: Rust native path 기반 내부 붙여넣기

## 목표

중첩 표 셀에서 내부 클립보드 붙여넣기 대상이 바깥 셀로 해석되지 않도록, Rust native/WASM API에 `cellPath` 기반 내부 붙여넣기 경로를 추가한다.

## 변경

### `src/document_core/commands/text_editing.rs`

- `get_cell_paragraphs_mut_by_path(...)` 추가.
- 기존 `get_cell_paragraph_mut_by_path(...)`는 새 문단 목록 헬퍼를 재사용하도록 정리.
- `mark_cell_control_dirty(...)`를 `pub(crate)`로 변경해 clipboard 명령에서도 기존 path 편집 API와 같은 dirty 처리를 사용할 수 있게 했다.

### `src/document_core/commands/clipboard.rs`

- 셀 문단 목록에 클립보드 문단을 삽입하는 공통 헬퍼 `paste_paragraphs_into_cell_paragraphs(...)` 추가.
- 기존 `paste_internal_in_cell_native(...)`는 얕은 표/글상자/캡션 경로를 유지하면서 공통 헬퍼를 사용하도록 정리.
- 신규 `paste_internal_in_cell_by_path_native(...)` 추가.
  - `cellPath`가 가리키는 최종 중첩 셀 문단 목록에 삽입.
  - 최외곽 표 dirty, `raw_stream` 무효화, section dirty, pagination 갱신 수행.
  - 반환 JSON은 기존 셀 붙여넣기와 같은 `cellParaIdx`, `charOffset` 형식.

### `src/wasm_api.rs`

- `pasteInternalInCellByPath(section, parentPara, pathJson, charOffset)` 바인딩 추가.

### `tests/issue_1198_nested_cell_paste.rs` (신규)

- `samples/exam_social.hwp`의 상단 `성명` 입력칸 hit-test 경로가 `[(4,0,3),(0,1,0)]`임을 확인.
- 문서 본문의 첫 유효 글자를 내부 클립보드에 복사.
- 신규 path API로 `성명` 중첩 셀에 붙여넣고, `get_text_in_cell_by_path`로 실제 중첩 셀 삽입을 검증.

## 검증

```text
cargo test --test issue_1198_nested_cell_paste -- --nocapture
```

결과:

```text
1 passed
```

```text
cargo test --test issue_850_answer_sheet_name_hit_test issue_850_exam_social_answer_sheet_name_cell_keeps_outer_path -- --nocapture
```

결과:

```text
1 passed
```

```text
cargo fmt --check
git diff --check
cargo test --lib
```

결과:

```text
cargo fmt --check: success
```

업스트림 반영 전 1차 검증에서는 `cargo test --lib`가 통과했다. 이후 `upstream/devel` `c884205d`
기준으로 재동기화하고 Stage 2/최종 검증에서 전체 결과를 다시 확인했다.

## 남은 작업

Stage 2에서 다음을 진행한다.

- HTML 붙여넣기용 `pasteHtmlInCellByPath` 추가.
- `rhwp-studio/src/core/wasm-bridge.ts` 신규 API 래퍼 추가.
- `rhwp-studio/src/engine/input-handler-keyboard.ts`에서 `cellPath.length > 1`일 때 path 기반 붙여넣기 라우팅.
- 붙여넣기 후 `cellParaIdx`와 `cellPath` 마지막 엔트리 갱신.
