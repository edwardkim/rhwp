# task/m100-2947: HWP5→HWPX 저장 시 문단 번호 형식(number_format) DIGIT 고정 하드코딩 수정

## 이슈

edwardkim/rhwp#2947

## 증상

HWP5(.hwp) → HWPX 저장 경로에서 문단 번호(numbering) 정의의 번호 형식이 항상
`numFormat="DIGIT"` 로 방출되어, 로마자(ROMAN_CAPITAL/SMALL), 원문자
(CIRCLED_DIGIT), 한글(HANGUL_SYLLABLE/NUMBER), 한자(HANJA_NUMBER) 등으로
설정된 문단 번호가 전부 아라비아 숫자로 뒤바뀐다.

## 근본 원인

- `src/model/style.rs:333` `NumberingHead.number_format: u8` 필드는 HWP5
  바이너리 파서가 표 43 규격에 따라 정확히 채워 넣는다.
- `src/serializer/hwpx/header.rs`의 `write_numbering()` 은 HWPX 원본
  `raw_para_heads` splice 경로가 없는 경우(= 순수 HWP5 경유 등) 10개 레벨
  전부에 대해 `hh:paraHead` 를 새로 구성하는 폴백 스켈레톤을 쓰는데, 이때
  `numFormat` 속성을 `h.number_format` 값과 무관하게 `"DIGIT"` 상수로
  하드코딩하고 있었다.
- 파서 쪽에는 이미 HWPX `numFormat` 문자열 → 코드 변환 함수
  `parse_numbering_format_code()` (`src/parser/hwpx/header.rs:1946`)가
  존재하므로, 역방향(코드 → 문자열) 매핑 함수만 추가하면 되는 순수 직렬화
  버그였다.

이 버그는 기존에 확립된 "IR에는 파싱이 올바른데 emit 시 하드코딩되어 유실"
패턴(lock #2931, dropcapstyle, groupLevel #2907, fieldid #2929, textDirection,
outline/shadow enum, tab leader 리터럴, pageBorderFill@type 등)과 동일 클래스다.

## 수정 내용

`src/serializer/hwpx/header.rs`

1. `numbering_format_str(code: u8) -> &'static str` 함수 추가. 파서의
   `parse_numbering_format_code()` 매핑을 역으로 뒤집은 것이다
   (1→CIRCLED_DIGIT, 2→ROMAN_CAPITAL, 3→ROMAN_SMALL, 4→LATIN_CAPITAL,
   5→LATIN_SMALL, 8→HANGUL_SYLLABLE, 12→HANGUL_NUMBER, 13→HANJA_NUMBER,
   그 외→DIGIT).
2. `write_numbering()` 의 폴백 스켈레톤 루프에서 `("numFormat", "DIGIT")`
   를 `("numFormat", numbering_format_str(h.number_format))` 로 교체.

`hh:bullets/hh:bullet` 쪽의 `numFormat="DIGIT"` (파서가 애초에 무시하는
뼈대용 상수)는 이번 버그와 무관하므로 그대로 두었다.

## 테스트 (Red → Green)

`write_numbering_skeleton_uses_number_format_not_hardcoded_digit`
(`src/serializer/hwpx/header.rs`):

- `Numbering::default()` 에 `heads[0].number_format = 2` (ROMAN_CAPITAL) 를
  설정하고 `write_numbering()` 을 호출, 결과 XML에
  `numFormat="ROMAN_CAPITAL"` 이 포함되는지 확인.
- 수정 전(하드코딩 `"DIGIT"` 상태)에서 동일 테스트를 수동으로 재현해
  `numFormat="DIGIT"` 로 방출되며 실패(RED)함을 확인했고, 수정 후 통과(GREEN)
  함을 확인했다.

```
running 1 test
test serializer::hwpx::header::tests::write_numbering_skeleton_uses_number_format_not_hardcoded_digit ... ok
```

## 검증

- `cargo check --lib` 통과.
- `cargo test --lib write_numbering` 3개 테스트(기존 splice/skeleton 테스트
  포함) 모두 통과.
- `rustfmt --edition 2021 src/serializer/hwpx/header.rs` 적용.

## 범위

`src/serializer/hwpx/header.rs` 1개 파일. 파서 쪽은 이미 올바르게 동작하므로
변경 없음.
