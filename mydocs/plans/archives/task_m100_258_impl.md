# 구현 계획서 — Task M100-258: 한글 누름틀 + 양식 모드 구현

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 수행 계획서: `mydocs/plans/task_m100_258.md`
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15

## 1. 구현 원칙

- 1차 구현은 기존 ClickHere 문서를 대상으로 한 양식 모드 MVP로 제한한다.
- 저장 포맷의 ClickHere 의미는 기존 HWP5/HWPX 라운드트립 규칙을 유지한다.
- 에디터 보호 정책은 rhwp-studio의 런타임 상태로 구현한다.
- 새 정책은 일반 편집 모드 동작을 바꾸지 않는다.

## 2. Stage 1 — 착수 조사 보고서

작성 파일:

- `mydocs/working/task_m100_258_stage1.md`

기록 내용:

- 한컴 도움말 기준 요약
- 이슈 요구사항 해석
- 기존 코드 기반 목록
- MVP 허용/차단 동작 목록
- Stage 2 소스 변경 후보 파일

검증:

- `git diff --check`
- 변경 파일 범위 확인

## 3. Stage 2 — 양식 모드 상태와 편집 정책

예상 수정 파일:

- `rhwp-studio/src/command/types.ts`
- `rhwp-studio/src/command/dispatcher.ts`
- `rhwp-studio/src/command/commands/view.ts` 또는 `rhwp-studio/src/command/commands/tool.ts`
- `rhwp-studio/src/main.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/src/engine/input-handler-text.ts`
- `rhwp-studio/src/engine/input-handler-mouse.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`
- 필요 시 `src/document_core/queries/field_query.rs`, `src/wasm_api.rs`

구현 항목:

1. `EditorContext`에 편집 모드 상태를 추가한다.
   - 후보: `editMode: 'normal' | 'form'`
   - 후보: `isFormMode: boolean`
2. 현재 커서가 양식 모드에서 수정 가능한 ClickHere 안인지 판정한다.
   - Rust JSON에 `editableInForm`을 추가하거나,
   - 기존 `getClickHereProps(fieldId).editable`을 프론트엔드에서 조회한다.
3. `CommandDispatcher`에 문서 변경 명령 차단 정책을 추가한다.
   - 양식 모드에서 명령별 `canExecute` 누락으로 우회되는 경로를 줄인다.
   - 우선 문서 구조 변경 명령과 `field:remove`를 차단한다.
4. keyboard/input handler에서 직접 처리되는 변경 경로를 차단한다.
   - 문자 입력
   - IME composition 입력
   - Backspace/Delete
   - Enter/Shift+Enter
   - Tab 삽입
   - paste/cut/delete selected object
5. editable ClickHere 내부에서는 기존 텍스트 편집 흐름을 유지한다.
6. 양식 모드 토글 명령을 추가하고 UI 상태 갱신 이벤트를 연결한다.

검증:

- 일반 편집 모드에서 기존 ClickHere 입력/삭제가 유지되는지 확인
- 양식 모드에서 editable ClickHere 내부만 수정 가능한지 확인
- `field:remove`가 양식 모드에서 차단되는지 확인
- `cargo test --test issue_838_field_set_value` 또는 관련 ClickHere 테스트
- `npm` 기반 스튜디오 타입 체크/빌드 명령은 현재 package script 확인 후 수행

## 4. Stage 3 — 편집 가능 누름틀 이동 UX

예상 수정 파일:

- `src/document_core/queries/doc_tree_nav.rs`
- `src/wasm_api.rs`
- `rhwp-studio/src/engine/cursor.ts`
- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/src/main.ts`

구현 항목:

- 다음/이전 편집 가능 ClickHere 탐색 정책을 확정한다.
- Tab을 필드 이동으로 쓸지, 기존 탭 문자 삽입을 유지할지 양식 모드에서만 분리한다.
- 상태 표시줄 이벤트에 양식 모드와 필드 이름을 반영한다.

검증:

- 본문 ClickHere 이동
- 표 셀 내부 ClickHere 이동
- 글상자 내부 ClickHere 이동 가능 여부 확인

## 5. Stage 4 — 필드 입력 대화상자 1차

예상 수정 파일:

- `rhwp-studio/src/command/commands/insert.ts`
- `rhwp-studio/src/ui/field-insert-dialog.ts` 신규 후보
- `rhwp-studio/src/core/wasm-bridge.ts`
- `src/wasm_api.rs`
- `src/document_core/commands/` 또는 `src/document_core/queries/field_query.rs`

구현 항목:

- `insert:field` 스텁 제거
- 누름틀 탭 대화상자 추가
- 안내문/메모/필드 이름/양식 모드 편집 가능 값 입력
- 커서 위치에 ClickHere fieldBegin/fieldEnd와 command/CTRL_DATA 생성

검증:

- 새 누름틀 삽입 후 화면 표시
- 누름틀 고치기에서 동일 값 확인
- 저장 후 재열기 또는 파싱으로 command/CTRL_DATA 확인

## 6. Stage 5 — 보고와 PR 준비

작성 파일:

- `mydocs/working/task_m100_258_stageN.md`
- `mydocs/report/task_m100_258_report.md`

검증 후보:

- `git diff --check`
- `cargo fmt --check`
- 관련 ClickHere/field 통합 테스트
- `cargo build --bin rhwp`
- `wasm-pack build --target web --out-dir pkg`
- rhwp-studio 빌드/타입 체크

PR 전 전체 로컬 CI급 검증은 작업지시자 별도 승인 후 수행한다.

## 7. 승인 요청

Stage 1 착수 조사 보고서를 작성한 뒤, Stage 2 양식 모드 MVP 구현으로 들어간다.
