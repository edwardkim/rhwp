# Task M100 #1452 Stage 9 기록

- 이슈: #1452 `rhwp-studio: 그림 삽입/배치 속성 및 Shift+Tab 내어쓰기 개선`
- 브랜치: `local/task_m100_1452`
- 작성일: 2026-06-21
- 선행 커밋:
  - `b3e0d49d task 1452: 파일 열기 취소 재오픈 방지`

## 1. 배경

`samples/투명도0-50.hwp`를 열면 한컴은 파일에 저장된 마지막 커서 위치인 두 번째 그림 뒤에 캐럿을
두지만, rhwp-studio는 문서 첫 줄 시작 위치에 캐럿을 둔다.

## 2. 초기 확인

- `initializeDocument()`는 문서 로드 후 `inputHandler.activateWithCaretPosition()`을 호출한다.
- `activateWithCaretPosition()`은 WASM `getCaretPosition()` 결과를 사용한다.
- Rust 모델은 HWP `DOCUMENT_PROPERTIES`의 `caret_list_id`, `caret_para_id`, `caret_char_pos`를 이미 보존한다.
- `get_caret_position_native()`는 `caret_char_pos`를 `char_offsets` 기반으로 logical char offset으로 변환한다.
- `투명도0-50.hwp`의 본문 문단은 텍스트가 비어 있고 TAC 그림 컨트롤 두 개가 본문 흐름을 차지한다.
  이 경우 `char_offsets`가 비어 있어 기존 UTF-16 위치 변환이 0으로 떨어질 수 있다.

## 3. 개선 목표

- 텍스트 없이 인라인 컨트롤만 있는 문단에서도 저장된 `caret_char_pos`를 logical char offset으로 복원한다.
- `samples/투명도0-50.hwp` 로드 시 캐럿이 두 번째 그림 뒤 위치로 초기화되게 한다.
- 일반 텍스트 문단의 기존 `char_offsets` 기반 변환은 유지한다.

## 4. 검증 계획

- `cargo test --lib <신규 focused 테스트> -- --nocapture`
- 필요 시 `cd rhwp-studio && npx tsc --noEmit`
- 작업지시자 수동 시각 검증

## 5. 변경 내용

- `src/document_core/queries/cursor_nav.rs`
  - `char_offsets`가 비어 있는 문단에서 저장된 `caret_char_pos`를 원시 컨트롤 개수 기준으로 해석한다.
  - 원시 HWP 위치는 모든 컨트롤을 8 UTF-16 code unit으로 세지만, Studio logical 커서는
    `SectionDef`/`ColumnDef` 같은 구조 컨트롤을 제외하고 본문 흐름 인라인 개체만 한 글자처럼 센다.
  - 따라서 `투명도0-50.hwp`의 `caret_char_pos=32`는 구조 컨트롤 2개 + 그림 2개 뒤 위치이고,
    Studio logical `charOffset=2`로 복원된다.
- `tests/issue_1452_saved_caret.rs`
  - `samples/투명도0-50.hwp`의 원시 `DOCUMENT_PROPERTIES` 캐럿 값과 `getCaretPosition()` 변환 결과를 검증한다.
  - `getCursorRect(0,0,2)`가 첫 줄 시작보다 아래쪽, 즉 두 번째 TAC 그림 줄에 있음을 함께 검증한다.
- `src/document_core/queries/cursor_rect.rs`
  - 문단부호/조판부호 표시 중 빈 문단 끝 TextRun의 캐럿 y 좌표를 문단부호 기준선에 맞춰 아래로 보정한다.
  - 인라인 TAC 그림만 있는 문단에서 마지막 그림 뒤 커서도 같은 y 보정을 받게 했다.
  - 한컴 기준 수동 비교 결과 x 위치는 유지하고 y 위치만 추가 조정했다.
- `src/renderer/web_canvas.rs`, `src/renderer/svg.rs`, `rhwp-studio/src/view/canvaskit-renderer.ts`, `rhwp-studio/src/view/canvaskit/policy.ts`
  - 문단부호/조판부호가 페이지 오른쪽 여백 밖에 그려질 때 잘리지 않도록 렌더 클립의 오른쪽 여유를 둔다.
- `rhwp-studio/src/core/user-settings.ts`, `rhwp-studio/src/command/commands/view.ts`, `rhwp-studio/src/main.ts`, `src/wasm_api.rs`
  - 문단부호/조판부호 표시 상태를 사용자 설정으로 저장하고 문서 로드 시 복원한다.

## 6. 검증 결과

- `cargo test --test issue_1452_saved_caret -- --nocapture` 통과.
- `cargo test --test issue_1071_tac_cursor_nav -- --nocapture` 통과.
- `cargo test --lib logical_positions_do_not_double_count_control_only_fallback -- --nocapture` 통과.
- `cargo fmt --check` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
- `cd rhwp-studio && npx tsc --noEmit` 통과.
- `cd rhwp-studio && node --test tests/user-settings.test.ts` 통과.
- 브라우저 검증:
  - URL: `http://localhost:7700/?url=/samples/투명도0-50.hwp&filename=투명도0-50.hwp`
  - 앱 로드/DOM 스냅샷 정상, 콘솔 `error`/`warn` 없음.
  - 문단부호 표시 상태에서 DOM `.caret` style은 최종 `left=700.45px`, `top=310.9px`, `height=12px`.
  - 한컴 기준 수동 비교 피드백에 따라 x는 유지하고 y만 아래로 추가 조정했다.
