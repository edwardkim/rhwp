# 완료 보고서 — Task M100-2839

- 이슈: #2839
- 제목: HWPX `<hh:style>` `lockForm` 속성이 시리얼라이저에서 항상 "0"으로 하드코딩되어 원본 값이 유실됨
- 작성일: 2026-07-22
- 브랜치: `task/m100-2839-hwpx-style-lockform-roundtrip`

## 1. 문제

`<hh:style>` 의 `lockForm` 속성(HWPX spec 표 47, 양식 필드 잠금 여부)이:

- `src/parser/hwpx/header.rs::parse_style` 에서 아예 읽히지 않았고,
- `src/model/style.rs::Style` IR 구조체에 이를 담을 필드가 없었으며,
- `src/serializer/hwpx/header.rs::write_style` 에서 값과 무관하게 항상
  문자열 리터럴 `"0"` 이 방출되고 있었다.

결과적으로 원본 HWPX 문서에서 `lockForm="1"` 로 저장된 스타일을 rhwp로 파싱 후
재시리얼라이즈(round-trip)하면 예외 없이 `lockForm="0"` 으로 바뀌어, 양식 잠금
의미가 조용히 유실되는 문서 무결성 손상이 발생했다.

## 2. 근거

`git log -S "lockForm" -- src/` 로 추적한 결과, 이 하드코딩은 커밋 `6c3983b5`
("Task #182 Stage 1: header.xml IR 기반 동적 생성")에서 `hh:style` 요소를 동적
생성하기 시작한 시점부터 존재했고, 이후 어떤 커밋에서도 실제 값을 반영하도록
수정된 적이 없었다. 같은 함수의 `langID` 는 유사한 하드코딩 버그였다가 최근
"Task #1058 후속" 으로 수정되었지만 `lockForm` 은 남아 있었다. 오늘 이미 수정된
charPr/paraPr 관련 이슈(#2695, #2777)와도 무관한 별개 속성이다.

## 3. 수정 내용

- `src/model/style.rs`: `Style` 구조체에 `pub lock_form: bool` 필드 추가.
- `src/parser/hwpx/header.rs::parse_style`: `b"lockForm" => style.lock_form =
  attr_str(&attr) == "1"` 매칭 추가.
- `src/serializer/hwpx/header.rs::write_style`: `("lockForm", "0")` 하드코딩을
  `("lockForm", bool01(st.lock_form))` 로 교체해 IR 값을 그대로 방출.
- 아래 파일들은 `Style` 구조체 리터럴에 신규 필드를 명시적으로 채워 컴파일을
  통과시켰다 (HWP3 바이너리 파서와 wasm API의 신규 스타일 생성 경로는 `lockForm`
  개념이 없어 `false` 기본값으로 채움):
  - `src/parser/doc_info.rs` (HWP3 바이너리 STYLE 레코드 파싱)
  - `src/wasm_api.rs` (wasm 스타일 신규 생성 API)
  - `src/serializer/doc_info/tests.rs` (기존 단위 테스트 2건)

## 4. 테스트 (TDD)

`src/serializer/hwpx/header.rs::tests::write_style_emits_ir_lock_form` 추가:
`Style { lock_form: true, .. }` 을 시리얼라이즈했을 때 `lockForm="1"` 이 방출되는지
확인하는 단발 assertion.

- Red: 수정 전 실행 시 다음과 같이 실패함을 확인.
  ```
  thread '...write_style_emits_ir_lock_form' panicked at src\serializer\hwpx\header.rs:1368:9:
  Style.lock_form 이 방출돼야 함: <hh:style ... lockForm="0"/>
  ```
- Green: 수정 후 `cargo test --lib write_style_emits_ir_lock_form` → `ok. 1 passed`.

## 5. 검증 결과

통과:

- `cargo build --lib`
- `cargo test --lib write_style_emits_ir_lock_form` (1 passed)
- `cargo clippy --all-targets --profile release-test -- -D warnings` (경고 0건;
  최초 `bool01(...).to_string()` 에서 `unnecessary_to_owned` 지적을 받아
  `bool01(st.lock_form)` 로 수정 후 통과)
- `rustfmt --edition 2021` — 변경된 6개 파일에만 적용
  (`src/model/style.rs`, `src/parser/hwpx/header.rs`,
  `src/serializer/hwpx/header.rs`, `src/parser/doc_info.rs`,
  `src/serializer/doc_info/tests.rs`, `src/wasm_api.rs`)
