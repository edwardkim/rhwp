# Task #2957 처리 결과 — HWPX 인라인 autoNum 원문자(CIRCLED_DIGIT) 서식 파싱·직렬화 오탈자 수정

## 이슈

- https://github.com/edwardkim/rhwp/issues/2957
- 인라인 자동번호 컨트롤 `<hp:ctrl><hp:autoNum>...</hp:autoNum></hp:ctrl>`의 자식
  `<hp:autoNumFormat type="...">`가 원 문자(circled digit, ①②③…) 형식일 때, 파서와
  직렬화기가 실제 한컴 스펙 표기 `CIRCLED_DIGIT` 대신 잘못된 철자 `CIRCLE_DIGIT`(D 없음)를
  사용해 왕복 시 값이 소실되는 문제.

## 근거

같은 파일 `src/serializer/hwpx/section.rs`의 각주/미주 모양(`footNotePr`/`endNotePr`)
`<hp:autoNumFormat type="...">` 방출 경로(`NumberFormat::CircledDigit => "CIRCLED_DIGIT"`,
약 244줄)는 #2742에서 실측 코퍼스로 검증된 올바른 철자를 쓰고 있다. 반면 인라인 `<hp:autoNum>`
경로의 `render_autonum()`은 `<hp:pageNum formatType="...">` 전용 매핑 함수
`page_num_format_to_str()`를 그대로 재사용해서 `1 => "CIRCLE_DIGIT"`(오탈자)를 방출했다.
`<hp:pageNum formatType>`은 별개 요소이므로 그쪽의 `CIRCLE_DIGIT` 표기 자체는 정상이며
변경하지 않았다 — `<hp:autoNumFormat type>` 쪽에서만 재사용한 것이 원인이었다.

파서 `src/parser/hwpx/section.rs`의 `parse_ctrl_autonum()`도 동일한 원인으로 `"CIRCLE_DIGIT"
=> 1`만 인식하고 실제 스펙 표기 `"CIRCLED_DIGIT"`를 만나면 `_ => 0`(DIGIT)으로 떨어뜨려
원 문자 설정을 소실했다.

## 변경

1. `src/serializer/hwpx/section.rs` `render_autonum()`: `<hp:autoNumFormat type="...">` 방출 시
   `format == 1`이면 `"CIRCLED_DIGIT"`을 직접 쓰고, 그 외에는 기존
   `page_num_format_to_str()` 결과를 그대로 사용(다른 형식·`<hp:pageNum>` 쪽은 변경 없음).
2. `src/parser/hwpx/section.rs` `parse_ctrl_autonum()`: `type` 매치에
   `"CIRCLE_DIGIT" | "CIRCLED_DIGIT" => 1`로 두 철자 모두 인식하도록 추가(구값 호환 유지).
   `parse_page_num_attrs()`(pageNum 전용)는 손대지 않았다.

diff 규모: 실질 로직 변경 7줄(파서 3줄 + 직렬화기 4줄) + 테스트 1개.

## 검증 (red → green)

- `cargo check --lib`: 통과.
- 신규 단위 테스트 `parser::hwpx::section::tests::task2957_autonum_format_circled_digit_parses_as_1`
  (`src/parser/hwpx/section.rs`): `<hp:autoNumFormat type="CIRCLED_DIGIT" .../>`를 포함한
  `<hp:autoNum>`을 파싱해 `AutoNumber.format == 1`을 단언.
  - **Red**: 파서 수정 전(= `"CIRCLE_DIGIT" => 1`만 인식)으로 되돌려 실행하면
    `assertion left == right failed: left: 0, right: 1`로 실패 확인.
  - **Green**: 수정 후 `cargo test --lib task2957` → `test result: ok. 1 passed`.
- `rustfmt --edition 2021`을 두 변경 파일에 적용.

## 범위 밖

이 worktree(`rhwp-wt-s`)에는 이전 세션의 미완료 변경(PR #2927 대응 CData 수정, `hwp3/*`
관련 파일 등)이 그대로 남아 있다. 이번 작업은 위 2개 파일 + 본 보고서만 커밋하며, 그 외
사전 존재 변경분은 건드리지 않았다.
