# task_m100_2557 처리결과 보고서 — 스타일/번호 JSON 이스케이프 정합

- **이슈**: [#2557](https://github.com/edwardkim/rhwp/issues/2557)
- **브랜치**: `task/m100-2557-style-json-escape` (base `devel` @ `3c54abfd`)
- **범위**: `src/wasm_api.rs` JSON 방출 5곳
- **분류**: 결함 수정 (깨진 JSON → 편집기 예외)

## 1. 문제

스타일·번호 정의 JSON 방출부가 **큰따옴표만** 이스케이프했다. 역슬래시·개행·탭이 든 이름이면
문법적으로 깨진 JSON 이 나오고, TS 측은 가드 없이 `JSON.parse` 하므로 **예외**가 난다.

같은 파일에 올바른 헬퍼(`json_escape`, `helpers.rs:825` — `\ " \n \r \t` 처리)가 있고
`wasm_api.rs:4866-4868` 이 이미 쓰고 있었다. 아래 5곳만 누락이었다.

| 위치 | 함수 |
|---|---|
| `wasm_api.rs:5610`, `:5611` | `getStyleList` (local_name, english_name) |
| `wasm_api.rs:5979` | `getNumberingList` (level_formats) |
| `wasm_api.rs:6180`, `:6216` | `getStyleAt` + 셀 내 변형 |

## 2. 영향

HWP 스타일 이름에 역슬래시·개행은 적법하며 템플릿 문서에서 드물지 않다.

- `JSON.parse` 가 **throw** 한다(값 이상이 아니라 예외).
- 호출부에 try/catch 가 없다 — `ui/style-dialog.ts:137`, `ui/toolbar.ts:548`,
  `engine/input-handler.ts:4397`.
- `getStyleAt` 은 **커서 이동마다** 호출된다(`input-handler.ts:2060`, `:4386`, `:4524`)
  → 해당 문서에서 **키 입력마다 예외**가 나고 스타일 드롭다운·툴바가 채워지지 않는다.

## 3. 변경

5곳을 `json_escape(...)` 로 교체. 새 헬퍼를 만들지 않고 기존 것을 재사용했다.

**범위 밖으로 남긴 것**: `wasm_api.rs:4646`, `:4675`, `:4699`, `:5434` 는 역슬래시까지 2단계로
처리해(개행만 미처리) 위험도가 낮고 오류 메시지/키 경로다. 기존 테스트 기대값에 영향을 줄 수
있어 건드리지 않았다.

## 4. 검증

### red→green 실증

`style_json_survives_backslash_and_control_chars` 추가 — 스타일 이름에
`a\b`(역슬래시) + 개행 + 탭 + 큰따옴표를 넣고 `get_style_list()` / `get_style_at()` 결과가
`serde_json` 으로 파싱되는지, 그리고 왕복 후 원래 이름이 복원되는지 단언한다.

- `:5610` 을 나이브 이스케이프로 되돌리면 → **FAILED**
- 복원하면 → **passed**

### 회귀

`cargo test --lib wasm_api` 통과.

### 미실행 항목 (투명 고지)

- **TS 측 행위 검증 미실행** — 실제 편집기에서 키 입력마다 예외가 사라지는지는 브라우저
  왕복이 필요하다. 본 PR 은 근본 원인(Rust 방출부)을 고치고 Rust 단에서 JSON 유효성을
  단언하는 데 그쳤다.
- **PR CI 전체 검증**(`cargo clippy -- -D warnings` 등): 저장소 규약상 작업지시자 별도 승인
  사항이라 실행하지 않았다.
