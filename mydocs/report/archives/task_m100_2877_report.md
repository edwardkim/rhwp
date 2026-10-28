# task_m100_2877 처리 결과 보고

- 이슈: #2877 — HWPX 문단 번호 형식(numFormat) 리터럴 6종 미인식 + HANGUL_JAMO/HANGUL_SYLLABLE 코드 충돌로 데이터 유실
- 브랜치: `task/m100-2877-numbering-circled-literal` (origin/devel 기준)

## 근거 요약

`mydocs/tech/한글문서파일형식_5.0_revision1.3.md:978-993` "표 41: 문단 번호 형식"과
`mydocs/manual/OWPML SCHEMA/Core XML schema.xml`의 `NumberType1` enumeration(5~83행)은
문단 번호 형식 코드 0~14와 리터럴 15종이 1:1 대응함을 명시한다. 수정 전
`src/parser/hwpx/header.rs`의 `parse_numbering_format_code`는 이 중 6개 리터럴
(`CIRCLED_LATIN_CAPITAL`, `CIRCLED_LATIN_SMALL`, `CIRCLED_HANGUL_SYLLABLE`,
`CIRCLED_HANGUL_JAMO`, `HANGUL_PHONETIC`, `CIRCLED_IDEOGRAPH`)을 인식하지 못해
`_ => value.parse().unwrap_or(0)` 폴백으로 `DIGIT`(0)로 유실시켰고, `HANGUL_JAMO`
(표41 값 10)를 `HANGUL_SYLLABLE`(8)과 같은 코드로 처리해 두 형식이 파싱 후
구분 불가능했다. #2857(탭 리더 이중/삼중선)과 동일한 버그 유형: 직렬화기
(`src/serializer/hwpx/header.rs` `write_numbering`)는 원본이 없을 때만
`("numFormat", "DIGIT")` 하드코딩 뼈대로 폴백할 뿐 이 "틀린" 코드를 스스로 만들어
내보내지 않으므로, 자체 왕복 테스트로는 드러나지 않고 한컴이 정식 발급한 XML을
읽을 때만 정보가 유실된다.

## 수정 내용

`src/parser/hwpx/header.rs`의 `parse_numbering_format_code`를 표 41 /
`NumberType1` 순서에 맞춰 재작성:

- 누락 6개 리터럴 추가: `CIRCLED_LATIN_CAPITAL`=6, `CIRCLED_LATIN_SMALL`=7,
  `CIRCLED_HANGUL_SYLLABLE`=9, `CIRCLED_HANGUL_JAMO`=11, `HANGUL_PHONETIC`=12,
  `CIRCLED_IDEOGRAPH`=14.
- `HANGUL_JAMO`를 `HANGUL_SYLLABLE`(8)에서 분리해 표41 값대로 10 반환.
- 기존 비표준 별칭(`ARABIC`, `ROMAN_UPPER` 등)은 스펙 외 리터럴이지만 관대한
  파싱을 위해 그대로 유지(값 충돌 없음 확인).

## Red → Green

- 신규 테스트: `parser::hwpx::header::tests::numbering_para_head_circled_latin_capital_is_not_lost_as_digit`
  — `numFormat="CIRCLED_LATIN_CAPITAL"`인 `<hh:paraHead>`를 파싱해
  `number_format == 6`을 검증. 수정 전 코드로는 매치암이 해당 리터럴을 다루지
  않아 `value.parse::<u8>()` 실패 → `unwrap_or(0)`로 0을 반환해 테스트가
  실패(red)함을 코드 검토로 확인. 수정 후 6을 반환해 통과(green).

## 검증

- `cargo build --lib` — 성공.
- `cargo test --lib numbering_para_head_circled_latin_capital_is_not_lost_as_digit` — 통과.
- `cargo test --lib numbering` (관련 기존 테스트 22개 전체) — 전부 통과, 회귀 없음.
- `cargo clippy --all-targets --profile release-test -- -D warnings` — 경고 없음.
- `rustfmt --edition 2021 src/parser/hwpx/header.rs` — 적용.

## 변경 파일

- `src/parser/hwpx/header.rs`
