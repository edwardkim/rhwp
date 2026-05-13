# Task M100 #871 Stage 4 완료 보고서

## 개요

Stage 4에서는 Stage 2/3에서 수정한 외부 클립보드 우선순위 로직을 실제 macOS 웹 환경에서 검증했다. 일반 본문 텍스트 붙여넣기는 정상화되었으나, 답안지 상단 `성명` 칸처럼 중첩 표 내부에 있는 입력 위치에서 외부 텍스트가 바깥 표 문단에 삽입되는 문제가 추가로 확인되었다.

## 추가 진단

첨부 재현 화면의 `성명` 칸은 단일 표 셀이 아니라 다음 구조다.

```text
본문 문단
  └─ 큰 표
      └─ 셀 내부 문단
          └─ 중첩 표
              └─ 성명 입력 셀
```

PR #865(Task #850)는 이 답안지 영역의 hit-test 결과가 루트 기준 전체 `cellPath`를 보존하도록 수정한 작업이다. `local/task871`에는 해당 변경이 없었으므로, 먼저 #865의 코드 변경만 현재 브랜치에 반영했다.

## 추가 반영 내용

### 1. PR #865 코드 반영

반영 파일:

- `src/document_core/queries/cursor_rect.rs`
- `src/document_core/queries/rendering.rs`
- `src/wasm_api.rs`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/src/view/page-renderer.ts`
- `tests/issue_850_answer_sheet_name_hit_test.rs`

Task #850 문서 파일은 Task #871 문서와 충돌을 피하기 위해 반영하지 않았다.

### 2. 중첩 표 외부 HTML 붙여넣기 보정

PR #865를 반영한 뒤에도 외부 클립보드가 `text/html`을 함께 제공하면 `onPaste()`가 `pasteHtmlInCell()` 경로를 탔다. 이 API는 단일 표 셀 인덱스만 받아 중첩 표의 전체 `cellPath`를 표현할 수 없다.

따라서 `rhwp-studio/src/engine/input-handler-keyboard.ts`에 다음 정책을 추가했다.

- 커서가 중첩 표 위치(`cellPath.length > 1`)에 있고 외부 `text/html`과 `text/plain`이 함께 있으면 `pasteHtmlInCell()`을 사용하지 않는다.
- 대신 `text/plain`을 기존 `InsertTextCommand` 경로로 삽입한다.
- `InsertTextCommand`는 이미 `insertTextInCellByPath()`를 사용하므로 중첩 표 위치를 정확히 보존한다.

이 보정은 중첩 표에서 서식보다 위치 정확성을 우선하는 임시 안전장치다. 중첩 표 내부 HTML 서식 보존까지 필요하면 별도 `pasteHtmlInCellByPath` 계열 API가 필요하다.

## 검증 결과

### 자동 검증

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
```

- 3 passed

```bash
cd rhwp-studio
npm run build
```

- `tsc && vite build` 성공
- 기존 Vite chunk size warning만 발생

```bash
docker-compose --env-file .env.docker run --rm wasm
```

- WASM `pkg/` 빌드 성공

### 수동 검증

환경:

- macOS 브라우저
- Vite dev server: `http://127.0.0.1:7700/`
- 문서: `exam_social.hwp`
- 위치: 1쪽 답안지 상단 `성명` 칸

검증 내용:

1. 외부 앱/시스템 클립보드에서 일반 텍스트 복사
2. rhwp-studio의 `성명` 칸에 커서를 둔 뒤 붙여넣기
3. 텍스트가 바깥 표 문단이 아니라 `성명` 입력 셀 내부에 삽입됨 확인

작업지시자가 동일 환경에서 “이제 정상적으로 동작”한다고 확인했다.

## 남은 제약

- 중첩 표 내부 외부 HTML 붙여넣기는 현재 plain text로 fallback한다.
- rhwp-studio 내부 복사 marker가 있는 경우에는 기존 내부 클립보드 경로를 유지한다.
- headless Chrome e2e 실실행은 sandbox의 Chrome 실행 제한으로 이번 환경에서는 완료하지 못했다.

## 다음 단계

최종 보고서를 작성하고 오늘할일 문서를 갱신한다. 작업지시자 승인 후 필요 시 issue close는 별도 승인 절차로 진행한다.
