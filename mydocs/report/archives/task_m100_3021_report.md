# 완료 보고서 — Task M100-3021

- 이슈: #3021
- 제목: HWP3 doc_info.footnote_bracket(각주 옵션) 파싱만 되고 미주 모양에 배선되지 않음
- 작성일: 2026-07-22
- 브랜치: `task/m100-3021-hwp3-footnote-bracket`

## 1. 완료 내용

#3006 (`Hwp3DocInfo.hide_empty_line` 배선 누락) 처리 시 사용한 방법 — doc_info 인접
플래그를 `impl Hwp3DocInfo`와 `SectionDef` 조립부(`src/parser/hwp3/mod.rs`)에서
전수 대조 — 를 이어서 적용해 offset 110 `footnote_bracket`(각주 옵션)에서 동일한 유형의
문제를 찾았다.

`Hwp3DocInfo::read`(`src/parser/hwp3/records.rs`)는 스펙(`mydocs/tech/한글문서파일구조3.0.md:244`,
`110 | echar | ')' = 각주 번호에 ')'를 붙임, 0=안 붙임 | 각주 옵션`)대로 `footnote_bracket: u8`을
정확히 파싱하지만, `hwp3_default_endnote_shape()`은 공통 IR `FootnoteShape.suffix_char`을
항상 `')'`로 하드코딩해 이 값을 사용하지 않았다. HWP5(`src/parser/doc_info.rs`)와
HWPX(`src/parser/hwpx/section.rs:901,908`)는 각각 원본 값으로 `suffix_char`을 채우므로
HWP3만 이 옵션을 무시하는 genuine gap이었다. 사용자가 한/글에서 "각주 번호 뒤 ')' 안 붙임"으로
설정한 HWP3 문서도 미주 번호 뒤에 항상 ')' 가 렌더링되는 시각 차이가 있었다.

## 2. 주요 변경

- `src/parser/hwp3/mod.rs`
  - `hwp3_default_endnote_shape()` → `hwp3_default_endnote_shape(bracket: bool)`:
    `suffix_char`을 `bracket`에 따라 `')'` 또는 `'\0'`(안 붙임)으로 결정
  - `fixup_hwp3_notes()`에 `footnote_bracket: bool` 매개변수 추가, 호출부에
    `doc_info.footnote_bracket != 0` 전달
  - 단위 테스트 `issue_hwp3_endnote_suffix_char_wires_footnote_bracket_flag` 추가
    (bracket=true → `')'`, bracket=false → `'\0'` 확인)

## 3. 검증 결과

통과:

- `cargo check --lib`
- `cargo test --lib hwp3::` (14 passed, 신규 테스트 포함)
- `rustfmt --edition 2021 src/parser/hwp3/mod.rs`

## 4. 범위 밖 후속 과제

- `hwp3_default_endnote_shape`의 `start_number: 1` 도 `doc_info.footnote_start_number`를
  직접 반영하지 않고 하드코딩되어 있다(실제 번호 매기기 시작값은
  `doc.doc_properties.footnote_start_num` 경유로 별도 적용되어 렌더 결과에는 영향 없음).
  `FootnoteShape.start_number`는 메타데이터 필드라 별도 검증이 필요해 이번 타스크
  범위에서는 제외했다.
- 각주(footnote, `SectionDef.footnote_shape`) 자체는 HWP3 파서에서 전혀 조립되지 않아
  항상 `Default`값이다. 이는 offset 하나짜리 배선 누락보다 훨씬 큰 범위의 기능 격차라
  별도 타스크로 분리해야 한다.
