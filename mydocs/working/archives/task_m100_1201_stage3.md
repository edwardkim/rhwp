# Task M100-1201 Stage 3 완료 보고 — masterPage@type 파싱 정합성 보강

## 작업 범위

구현계획서 Stage 3 범위에 따라 HWPX `masterPage@type` 값 해석을 보강했다.

## 변경 내용

### `src/parser/hwpx/section.rs`

- `HwpxMasterPageType` 내부 enum을 추가했다.
- `parse_hwpx_master_page_type()` helper를 추가했다.
- type 문자열은 ASCII 영숫자만 남긴 뒤 대문자로 정규화한다.

지원 표기:

```text
BOTH / Both / both
EVEN / Even / even
ODD / Odd / odd
LAST_PAGE / LastPage / lastPage
OPTIONAL_PAGE / OptionalPage / optionalPage
```

기존 의미 유지:

- `LAST_PAGE pageDuplicate="0"`는 기존처럼 `is_extension=true`, `overlap=true`, `replace_base=true`로 처리한다.
- `OPTIONAL_PAGE` 계열은 기존처럼 `is_extension=true`, `overlap=true`, `ext_flags=0x0007`로 처리한다.
- 알 수 없는 type 값은 기존 동작과 같이 `Both`로 닫힌다.

## 테스트

추가:

- `parser::hwpx::section::tests::test_parse_hwpx_master_page_type_accepts_official_and_sample_spellings`
  - 공식 문서식 CamelCase와 실제 샘플식 uppercase/underscore 표기를 함께 검증
- `parser::hwpx::section::tests::test_parse_master_page_mixed_case_type_attrs`
  - 실제 `parse_hwpx_master_page()` 경로에서 mixed-case type이 `MasterPage` 필드로 반영되는지 검증

기존 유지:

- `test_parse_master_page_last_page_extension`
- `test_parse_master_page_optional_page_extension`

## 검증

통과:

```text
cargo fmt --all --check
cargo test --lib test_parse_hwpx_master_page_type_accepts_official_and_sample_spellings
cargo test --lib test_parse_master_page_mixed_case_type_attrs
cargo test --lib test_parse_master_page_last_page_extension
cargo test --lib test_parse_master_page_optional_page_extension
cargo test --lib hwpx
```

## 확인 결과

- HWPX masterpage type은 XML의 명시값을 기준으로 매핑한다.
- HWP5 raw parser의 `[Both, Odd, Even]` 순서 기반 해석은 변경하지 않았다.
- #1201 샘플의 `EVEN`/`ODD` 표기와 공식 문서의 `Even`/`Odd` 표기 모두 같은 결과로 해석된다.

## 다음 단계

Stage 4에서는 실제 #1201 샘플을 대상으로 구조 및 시각 검증을 수행한다.

Stage 4 착수 전 작업지시자 승인이 필요하다.
