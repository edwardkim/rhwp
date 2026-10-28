# Task M100-258 Stage 1 — 누름틀 + 양식 모드 착수 조사

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15

## 1. 이슈 상태

- 상태: open
- 제목: 한글 누름틀 + 양식 모드 구현 요청
- assignee: `jangster77`
- label: `enhancement`
- 요청 요지:
  - 한글 누름틀(필드 입력) 지원
  - 한컴웹기안기의 양식 모드처럼 누름틀 지정 위치만 편집 허용
  - 일반 편집에서 사용자가 누름틀 자체를 삭제할 수 있는 문제 보완

## 2. 한컴 도움말 기준

확인한 공식 도움말:

- `필드 입력`: `[입력-개체-필드 입력]`에서 사용자 정보, 작성한 날짜, 문서 요약,
  파일 이름/경로 등의 필드를 삽입한다.
- `필드 입력: 누름틀`: 안내문, 메모, 필드 이름을 입력한다.
- 누름틀 안내문은 편집 화면에서 빨간색 기울임 글자 속성으로 나타난다.
- 안내문을 마우스로 누르면 안내문이 사라지고 낫표로 묶인 누름틀 입력 위치가 표시된다.
- `양식 모드에서 편집 가능`을 선택하면 양식 모드에서도 해당 개체를 편집할 수 있다.
- 양식 모드는 문서 내용을 보호하고, 양식 모드에서 편집 속성을 가진 개체만 편집 가능하게
  허용하는 모드로 설명된다.
- 작성한 날짜와 파일 이름/경로 필드도 같은 `양식 모드에서 편집 가능` 속성을 가진다.

판단:

- 파일 포맷의 `editable` 속성과 에디터 런타임의 `form mode`를 분리해야 한다.
- HWP/HWPX 파일만으로 한컴 데스크톱 일반 편집 모드의 누름틀 삭제를 절대 차단한다고
  보장하기는 어렵다.
- rhwp에서 구현할 수 있는 1차 목표는 rhwp-studio/HwpCtrl 계층에서 양식 모드로 열었을 때
  편집 가능 누름틀만 수정되도록 막는 것이다.

## 3. 기존 rhwp 구현 확인

### 모델/파서/직렬화

- `src/model/control.rs`
  - `FieldType::ClickHere`
  - `Field::guide_text()`
  - `Field::memo_text()`
  - `Field::field_name()`
  - `Field::is_editable_in_form()`
  - `Field::build_clickhere_command()`
- `src/parser/hwpx/section.rs`
  - HWPX `editable="1"` → `properties bit 0`
  - `CLICKHERE`, `DATE`, `DOC_DATE`, `PATH`, `SUMMARY`, `USER_INFO` 타입 매핑
- `src/serializer/hwpx/field.rs`
  - `<hp:fieldBegin ... editable="0/1">` 직렬화
- `mydocs/tech/hwp_save_guide.md`
  - ClickHere CTRL_HEADER, command, CTRL_DATA name 저장 규칙 기록
- `mydocs/tech/hwp_spec_errata.md`
  - memo_index, HelpState, CTRL_DATA name 관련 정오표 기록

### WASM/document core

- `src/document_core/queries/field_query.rs`
  - 필드 목록 조회
  - 필드 값 조회/설정
  - 현재 커서의 ClickHere field range 확인
- `src/wasm_api.rs`
  - `getFieldList`
  - `setFieldValue`, `setFieldValueByName`
  - `getFieldInfoAt`, `getFieldInfoAtInCell`, `getFieldInfoAtByPath`
  - `setActiveField`, `setActiveFieldInCell`, `setActiveFieldByPath`
  - `getClickHereProps`, `updateClickHereProps`

### rhwp-studio

- `rhwp-studio/src/ui/field-edit-dialog.ts`
  - 안내문/메모/필드 이름/양식 모드 편집 가능 UI 존재
- `rhwp-studio/src/command/commands/edit.ts`
  - `field:edit`
  - `field:remove`
- `rhwp-studio/src/command/commands/insert.ts`
  - `insert:field`는 현재 스텁
- `rhwp-studio/src/engine/input-handler-text.ts`
  - 필드 시작/끝 경계에서 Backspace/Delete 차단 일부 존재
- `rhwp-studio/src/engine/input-handler.ts`
  - 현재 커서의 필드 정보 표시와 active field 전환 존재

## 4. MVP 정책

Stage 2의 양식 모드 MVP에서 허용할 동작:

- 양식 모드 OFF:
  - 기존 동작 유지
- 양식 모드 ON:
  - `FieldType::ClickHere`이고 `editable=true`인 누름틀 내부 텍스트 입력
  - 같은 조건의 누름틀 내부 Backspace/Delete
  - 같은 조건의 누름틀 내부 선택/복사
  - 편집 가능 누름틀로 커서 이동

Stage 2에서 차단할 동작:

- 일반 본문 텍스트 입력
- editable이 아닌 ClickHere 수정
- 누름틀 자체 삭제
- 선택 범위 삭제가 필드 밖을 포함하는 경우
- 문단 분할/병합
- 일반 Tab 문자 삽입
- 붙여넣기/잘라내기 중 필드 밖 변경
- 표/그림/도형/수식 삭제 또는 이동
- 일반 삽입 명령과 서식 변경 명령

## 5. Stage 2 수정 후보

우선순위 높은 파일:

- `rhwp-studio/src/command/types.ts`
  - `EditorContext`에 `editMode` 또는 `isFormMode` 추가
- `rhwp-studio/src/main.ts`
  - 편집 모드 상태 보관과 context 반영
- `rhwp-studio/src/command/dispatcher.ts`
  - 양식 모드에서 문서 변경 명령 차단
- `rhwp-studio/src/engine/input-handler-keyboard.ts`
  - 직접 키 처리 경로 차단
- `rhwp-studio/src/engine/input-handler-text.ts`
  - IME/input/raw insert/delete 경로 차단
- `rhwp-studio/src/command/commands/edit.ts`
  - `field:remove` 차단
- `rhwp-studio/src/core/wasm-bridge.ts`
  - field info 타입 확장

필요 시 Rust 수정:

- `src/document_core/queries/field_query.rs`
  - field info JSON에 `editableInForm` 추가
- `src/wasm_api.rs`
  - JS 바인딩 반환 contract 유지

## 6. 구현 전 확인할 결정 사항

1. 양식 모드 토글 명령 위치:
   - 후보: `view:form-mode`
   - 후보: `tool:form-mode`
2. 양식 모드에서 Tab:
   - 후보 A: 다음 편집 가능 누름틀로 이동
   - 후보 B: Stage 2에서는 삽입만 차단하고 Stage 3에서 이동 구현
3. 초기 MVP의 편집 가능 대상:
   - 후보 A: ClickHere만
   - 후보 B: ClickHere + editable cell

Stage 2는 위험을 줄이기 위해 후보 B를 모두 미루고 ClickHere만 처리하는 편이 적절하다.

## 7. 검증 계획

문서/코드 공통:

- `git diff --check`

Rust:

- `cargo fmt --check`
- ClickHere 관련 focused test
- `cargo test --test issue_838_field_set_value`

Studio:

- package script 확인 후 타입 체크 또는 build
- 수동 확인:
  - 양식 모드 OFF 기존 입력
  - 양식 모드 ON 본문 입력 차단
  - 양식 모드 ON editable ClickHere 입력 허용
  - 양식 모드 ON `field:remove` 차단

## 8. 현재 판단

구현 가능하다. 다만 이번 작업의 본질은 저장 포맷 파싱보다 에디터 런타임 보호 정책이다.
기존 ClickHere 저장/편집 기반이 있으므로 Stage 2는 rhwp-studio의 입력/명령 가드를 중심으로
진행하면 된다.

소스 변경은 작업지시자 승인 후 시작한다.
