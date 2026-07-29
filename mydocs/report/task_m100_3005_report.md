# task_m100_3005 처리 결과 보고

## 이슈

[#3005 HWPX 쪽번호(pageNum) 원문자(CIRCLED_DIGIT) formatType이 CIRCLE_DIGIT 오탈자로
파싱·직렬화되어 소실](https://github.com/edwardkim/rhwp/issues/3005)

## 배경

같은 클래스의 오탈자 버그가 이슈 #2957(PR #2964, 처리 시점 기준 아직 미병합)에서
`<hp:autoNumFormat type="...">`에 대해 이미 지적됐다. 그런데 #2957의 이슈 본문은
"`<hp:pageNum formatType="...">`는 별개 요소·속성이므로 그쪽의 `CIRCLE_DIGIT` 표기는
그대로 유지해야 한다"고 명시적으로 결론지었다. 이번 작업은 그 결론을 스키마와 직접
대조해 검증하는 것이 목표였다.

`mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml` 193~194행에서 `<hp:pageNum>`의
`formatType` 속성은 `type="hc:NumberType1"`로 선언돼 있다. `mydocs/manual/OWPML SCHEMA/Core
XML schema.xml`을 보면 `NumberType1`(5~83행)과 `NumberType2`(84행~)가 각각 정의돼
있는데, 두 enum 모두 원문자 값을 `CIRCLED_DIGIT`(12행·91행)로 동일하게 표기한다.
즉 `pageNum`이 `autoNumFormat`과 별개 속성이라는 #2957의 전제는 맞지만, 값 자체는
동일한 스펙 표기(`CIRCLED_DIGIT`)를 따라야 한다는 점에서 "그대로 유지" 판단은
스키마 미대조 상태의 오판이었다 — `page_num_format_to_str()`/`parse_page_num_attrs()`가
쓰던 `CIRCLE_DIGIT` 자체가 별도의 동일 클래스 오탈자였다.

## 근본 원인

- `src/serializer/hwpx/section.rs`의 `page_num_format_to_str()`에서
  `1 => "CIRCLE_DIGIT"`로 방출.
- `src/parser/hwpx/section.rs`의 `parse_page_num_attrs()`에서 `formatType` 매치에
  `"CIRCLE_DIGIT" => 1`만 있고 스펙 문자열 `"CIRCLED_DIGIT"`는 인식하지 못해
  `_ => 0`(DIGIT)으로 떨어짐.

## 변경 내용

- `src/serializer/hwpx/section.rs`: `page_num_format_to_str()`의 `1 => "CIRCLE_DIGIT"`를
  `1 => "CIRCLED_DIGIT"`로 수정(스펙 철자).
- `src/parser/hwpx/section.rs`: `parse_page_num_attrs()`의 `formatType` 매치에
  `"CIRCLED_DIGIT" | "CIRCLE_DIGIT" => 1`로 두 표기 모두 인식하도록 추가(과거 오탈자로
  저장된 한컴 실물 파일과의 하위 호환 유지).
- `src/serializer/hwpx/section.rs`: 단위 테스트
  `page_num_circled_digit_format_reflects_spec_spelling` 추가 — `PageNumberPos.format = 1`
  (원문자)일 때 `render_page_num()`이 `formatType="CIRCLED_DIGIT"`를 방출하고
  `formatType="CIRCLE_DIGIT"`는 방출하지 않음을 검증.

## 검증 (red → green)

수정 전 `page_num_format_to_str()`를 `"CIRCLE_DIGIT"`로 되돌린 상태에서
`cargo test --lib page_num_circled_digit_format_reflects_spec_spelling`을 실행하면
`formatType="CIRCLE_DIGIT"` 출력에 대해 `CIRCLED_DIGIT` 단언이 실패함을 확인(red).
수정을 되돌린 뒤(green) 동일 테스트가 통과함을 재확인했다.

```
test serializer::hwpx::section::tests::page_num_circled_digit_format_reflects_spec_spelling ... ok
```

`cargo check --lib`도 통과했다.

## 범위 밖

이슈 #2957/PR #2964가 다루는 `<hp:autoNumFormat type="...">`(인라인 자동번호) 경로는
이번 작업에서 건드리지 않았다. `<hp:pageNum formatType="...">`만 별도 이슈(#3005)로
분리해 수정했다.
