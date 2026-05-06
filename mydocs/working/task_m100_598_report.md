# Task #598 최종 결과보고서 — rhwp-studio 본문 각주 마커 이동/삭제

## 작업 개요

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **브랜치**: `local/task598`
- **범위**: `(1) hit test + 커서 이동`, `(2) 삭제 API/UI`

본문에 렌더링되는 각주 마커를 커서 이동 단위로 취급하고, 본문 마커 기준으로 각주 편집 모드 진입 및 각주 삭제를 지원하도록 구현했다.

## 구현 요약

### 1차: hit test + 커서 이동

- `ComposedParagraph.footnote_positions` 에 실제 `para.controls` 인덱스를 보존하도록 확장했다.
- `FootnoteMarkerNode.control_index` 가 실제 각주 control index 를 가리키도록 보정했다.
- `hitTestBodyFootnoteMarker()` / `hit_test_body_footnote_marker_native()` 를 추가했다.
- 본문 각주/미주 마커를 inline cursor unit 으로 취급하도록 탐색 길이와 문단 control position 폴백을 보정했다.
- `get_cursor_rect_native()` 가 각주 마커 왼쪽/오른쪽 caret 위치를 반환하도록 보정했다.
- rhwp-studio 마우스 처리에서 본문 각주 마커 클릭 시 각주 편집 모드로 진입하도록 연결했다.

### 2차: 삭제 API/UI

- `getFootnoteAtCursor()` / `get_footnote_at_cursor_native()` 를 추가했다.
  - Backspace: `direction="backward"`
  - Delete: `direction="forward"`
- `deleteFootnote()` / `delete_footnote_native()` 를 추가했다.
  - 본문 각주 control 검증
  - 8 UTF-16 code unit 컨트롤 슬롯 제거에 맞춘 `char_offsets` / `char_count` 보정
  - `controls` / `ctrl_data_records` 동시 제거
  - 각주 번호 재계산
  - 본문 reflow, section recompose, pagination, page tree cache 무효화
- `DocumentEvent::FootnoteDeleted` 를 추가했다.
- rhwp-studio Backspace/Delete 처리에서 일반 텍스트 삭제 전에 각주 마커 삭제를 우선 처리하도록 연결했다.
- 삭제 작업은 `SnapshotCommand` 로 실행해 Undo/Redo 경로를 사용한다.

## 검증 결과

실행 명령:

```bash
cargo test --test issue_598_footnote_marker_nav
cargo test navigable_text_len_counts_trailing_footnote_marker
cargo build
cd rhwp-studio && npm run build
docker-compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
git diff --check
```

결과:

- `cargo test --test issue_598_footnote_marker_nav`: 3 passed
- `cargo test navigable_text_len_counts_trailing_footnote_marker`: 1 passed
- `cargo build`: 통과
- `npm run build`: 통과
- `docker-compose --env-file .env.docker run --rm wasm`: 통과
- 새 WASM 반영 후 `npm run build`: 통과
- `git diff --check`: 통과

추가 확인:

- `pkg/rhwp.js` 와 rhwp-studio 번들에 `hitTestBodyFootnoteMarker`, `getFootnoteAtCursor`, `deleteFootnote` 가 포함됨을 확인했다.

참고:

- `cargo test navigable_text_len_counts_trailing_footnote_marker` 실행 시 기존 테스트 코드의 warning 이 함께 출력됐다. 이번 변경과 무관한 기존 warning 이며 테스트는 통과했다.
- `npm run build` 실행 시 Vite chunk size warning 이 출력됐다. 기존 번들 크기 경고이며 빌드는 성공했다.

## 수동 검증

작업지시자가 `http://localhost:7700/` 에서 rhwp-studio 를 실행해 확인했다.

확인된 동작:

- `samples/footnote-01.hwp` 로드
- 본문 각주 마커 클릭 시 하단 각주 영역으로 caret 이동
- 본문 각주 마커 뒤 커서 위치에서 Backspace 로 각주 삭제
- 본문 각주 마커 앞 커서 위치에서 `Fn+Delete` 로 각주 삭제
- `Fn+Delete` 삭제 후 첫 번째 각주 본문 제거 및 기존 두 번째 각주가 `1)` 로 재번호화됨

## 산출물

- 구현 파일:
  - `src/document_core/queries/cursor_rect.rs`
  - `src/document_core/commands/footnote_ops.rs`
  - `src/document_core/commands/object_ops.rs`
  - `src/document_core/helpers.rs`
  - `src/document_core/queries/doc_tree_nav.rs`
  - `src/model/paragraph.rs`
  - `src/model/event.rs`
  - `src/renderer/composer.rs`
  - `src/renderer/layout/paragraph_layout.rs`
  - `src/wasm_api.rs`
  - `rhwp-studio/src/core/types.ts`
  - `rhwp-studio/src/core/wasm-bridge.ts`
  - `rhwp-studio/src/engine/input-handler-mouse.ts`
  - `rhwp-studio/src/engine/input-handler-text.ts`
- 테스트:
  - `tests/issue_598_footnote_marker_nav.rs`
- 문서:
  - `mydocs/plans/task_m100_598.md`
  - `mydocs/plans/task_m100_598_impl.md`
  - `mydocs/plans/task_m100_598_delete_impl.md`
  - `mydocs/working/task_m100_598_stage1.md`
  - `mydocs/working/task_m100_598_stage3_1.md`
  - `mydocs/working/task_m100_598_stage3_2.md`
  - `mydocs/working/task_m100_598_stage3_3.md`
  - `mydocs/working/task_m100_598_stage3_4.md`
  - `mydocs/working/task_m100_598_stage4_1.md`
  - `mydocs/working/task_m100_598_stage4_2.md`

## 남은 확인 항목

- PR 전 최종 diff 리뷰
- 작업 단위 커밋 작성
