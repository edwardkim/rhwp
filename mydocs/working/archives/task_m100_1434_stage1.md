# Task M100 #1434 — 1단계 완료 보고서 (command 한컴 포맷 정정)

- 브랜치: `local/task1434`
- 작성일: 2026-06-19
- 수정 파일: `src/model/control.rs`, `src/document_core/queries/field_query.rs`, `src/wasm_api.rs`

## 1. 수정 내용

### `build_clickhere_command` 한컴 포맷 정정 (`control.rs:310~`)

시그니처 `(guide, memo, name)` → `(guide, memo)`. 세 가지 어긋남 정정:

| # | 항목 | 정정 |
|---|------|------|
| ① | Name 키 | command 에서 제거 (이름은 CTRL_DATA 0x57 전담) |
| ② | trailing 공백 | HelpState 뒤 1개 → **2개** |
| ③ | set 길이 | inner 전체 → **inner 글자수 − 1** |

### 호출처 정합 (2곳)

- `field_query.rs:1247` (누름틀 삽입): `build_clickhere_command(guide, memo, name)` →
  `(guide, memo)`. name 은 `ctrl_data_name: Some(name)` 로 이미 별도 저장(불변).
- `wasm_api.rs:4045` (안내문 수정): `(guide, memo, "")` → `(guide, memo)` (동작 불변).

## 2. 검증

- 단위 테스트 4건 (model::control):
  - `task1434_clickhere_command_hancom_format`: 한컴 원본 2케이스(여기에 입력/제목 입력)
    문자열 **바이트 동형** (set:48/set:47, 공백 2개, Name 부재).
  - `task1434_command_has_no_name_key`: command 에 Name 키 부재 (회귀 가드).
  - `task1434_command_guide_memo_roundtrip`: 생성 command → guide_text/memo_text 재추출 정합.
  - `task1434_set_length_excludes_trailing_space`: set = inner − 1 규칙.
- `model::control` 7 passed / `field_query` 4 passed (시그니처 변경 회귀 0).
- lib build exit 0, unused 경고 없음.
- **한컴 샘플 `field-01.hwp` dump 불변** — 한컴 원본 command 파싱 회귀 0.

## 3. 다음 단계

- 2단계: 누름틀(안내문 포함) IR → HWP 저장 → 재파싱 → guide/memo/name 보존 round-trip +
  전수/CI.
- 3단계: 한컴 판정용 샘플 산출(작업지시자 Windows 한컴 편집기 바인딩 판정) + 문서·보고서.
