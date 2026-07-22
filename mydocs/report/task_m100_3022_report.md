# Task m100-3022: hc:ArrowSize 직렬화 *_BIG → *_LARGE 정본 리터럴 정정

## 이슈

edwardkim/rhwp#3022 — `hc:ArrowSize` (Core XML schema.xml:407, `ParaList XML schema.xml`의
`headSz`/`tailSz` 두 속성이 참조) 직렬화 시 `src/serializer/hwpx/shape.rs::arrow_size_str()`
가 스펙에 없는 `*_BIG` 표기(SMALL_BIG/MEDIUM_BIG/BIG_SMALL/BIG_MEDIUM/BIG_BIG)를 방출하는
문제. 이전에 정리한 `hc:NumberType1` 공유 enum 불일치(#2957/#2964, #3005/#3007, #3011/#3015)와
같은 방법으로 다른 OWPML 공유 enum(`hc:LineType2`, `hc:AlignStyleType`, `hc:ArrowSize` 등)을
훑던 중 발견했다.

## 원인

- 스펙 정본 9개 열거값: `SMALL_SMALL, SMALL_MEDIUM, SMALL_LARGE, MEDIUM_SMALL, MEDIUM_MEDIUM,
  MEDIUM_LARGE, LARGE_SMALL, LARGE_MEDIUM, LARGE_LARGE`.
- `src/serializer/hwpx/shape.rs` 의 `arrow_size_str()` 은 `LARGE` 자리를 전부 `BIG` 으로
  방출했다(5개 값 오표기).
- `src/parser/hwpx/section.rs` 의 `parse_line_shape_attr` 파서는 이미 `"SMALL_BIG" |
  "SMALL_LARGE"` 식으로 두 표기를 모두 관용 수용하고 있어, rhwp 자체 라운드트립(쓰고 다시
  읽기)에서는 IR diff 게이트에 차이가 드러나지 않았다. 이는 #1402 가 지적한 "파서 관용 수용에
  가려진 비실재 토큰" 패턴과 동일하다. 실제 한컴 오피스 등 스펙 준수 도구가 rhwp 출력을 열면
  `headSz="SMALL_BIG"` 는 정의되지 않은 값이라 무시될 수 있다.

## 수정 내용

`src/serializer/hwpx/shape.rs`:
- `arrow_size_str()` 의 `2/5/6/7/8` 분기 방출값을 `SMALL_LARGE/MEDIUM_LARGE/LARGE_SMALL/
  LARGE_MEDIUM/LARGE_LARGE` 로 정정.
- 파서의 `*_BIG` 관용 수용은 기존 실물 파일 하위호환을 위해 그대로 유지(수정 범위 밖).

## 테스트 (Red → Green)

`task3022_arrow_size_uses_spec_large_literal`:
- 수정 전: `arrow_size_str(2)` 등이 `"SMALL_BIG"` 등을 반환해 `assert_eq!(.., "SMALL_LARGE")`
  가 실패(RED).
- 수정 후: 5개 코드 모두 스펙 리터럴을 반환해 통과(GREEN).

## 검증

```
cargo test --lib task3022_arrow_size_uses_spec_large_literal
```
→ `test result: ok. 1 passed; 0 failed`

```
cargo check --lib
```
→ 정상 통과 (신규 에러 없음)

`rustfmt --edition 2021 src/serializer/hwpx/shape.rs` 적용 완료.

## 변경 파일

- `src/serializer/hwpx/shape.rs` (fix + test)
- `mydocs/report/task_m100_3022_report.md` (본 문서)
