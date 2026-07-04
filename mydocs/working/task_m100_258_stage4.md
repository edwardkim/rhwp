# Task M100-258 Stage 4 — 누름틀 삽입 대화상자

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `8aa4c52c` (`task 258: 양식 모드 누름틀 이동 구현`)

## 1. 작업 목적

`insert:field` 스텁을 실제 누름틀 삽입 대화상자로 교체한다. 한컴 도움말의
`필드 입력: 누름틀` 기준에 맞춰 안내문, 메모, 필드 이름, 양식 모드 편집 가능 속성을
입력하고 현재 커서 위치에 ClickHere 필드를 생성한다.

## 2. 구현 범위

- rhwp-studio `insert:field` 명령을 스텁에서 실제 명령으로 교체
- 누름틀 삽입 대화상자 추가
- WASM API로 현재 커서 위치에 ClickHere field range/command/CTRL_DATA 생성
- body text 우선, 가능한 경우 표 셀/글상자 경로까지 기존 텍스트 삽입 경로와 맞춰 지원
- 삽입 후 필드 목록/양식 모드 이동 대상에 새 필드가 포함되는지 확인

## 3. 비범위

- 사용자 정보/문서 요약/작성한 날짜/파일 이름·경로 탭 전체 구현
- HwpCtrl `CreateField` 호환 액션
- 배포 문서/보호 문서 기능

## 4. 예상 수정 파일

- `src/document_core/queries/field_query.rs`
- `src/wasm_api.rs`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/command/commands/insert.ts`
- `rhwp-studio/src/ui/field-insert-dialog.ts` 신규 후보

## 5. 검증 계획

- `cargo fmt --check`
- 누름틀 삽입 focused test
- `npm run build`
- `git diff --check`

## 6. 진행 기록

- Stage 4 문서 작성 후 구현 착수.
- `insert:field` 스텁을 `FieldInsertDialog` 기반 실제 명령으로 교체했다.
- Rust `DocumentCore`에 본문/셀/중첩 cellPath용 ClickHere 삽입 API를 추가했다.
- 삽입 시 `Control::Field`, `FieldRange`, `ctrl_data_records` 인덱스를 함께 정렬하고,
  빈 누름틀의 FIELD_BEGIN/FIELD_END 갭이 직렬화 순서에 맞게 유지되도록 `char_offsets`
  재생성 로직을 보정했다.
- WASM API와 `WasmBridge.insertClickHereField()` 라우팅을 추가했다.
- `tests/issue_258_clickhere_form_mode.rs`에 빈 editable ClickHere 삽입 회귀 테스트를 추가했다.

## 7. 검증 결과

- `cargo fmt --check` 통과
- `cargo test --test issue_258_clickhere_form_mode` 통과
- `cargo test --test issue_838_field_set_value` 통과
- `npm run build` 통과
- `git diff --check` 통과
