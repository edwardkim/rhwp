# Task m100-3032 처리 결과

## 이슈

edwardkim/rhwp#3032 — `doc_info.footnote_between_margin`(각주 사이 간격) 미주 모양
`raw_unknown` 미배선

## 원인

HWP3 `doc_info` 오프셋 108 "각주와 각주 사이의 간격"(`footnote_between_margin`)이
`Hwp3DocInfo::read`에서 파싱만 되고, `src/parser/hwp3/mod.rs`의
`hwp3_default_endnote_shape()`는 `FootnoteShape.raw_unknown`("주석 사이" 값)을 항상
기본값 0으로 남겼다. `#3006`(hide_empty_line) · `#3021`(footnote_bracket) ·
`#3017`(footnote_line_width→separator_length)와 같은 클래스의 "파싱만 되고 IR에
배선되지 않은 필드" 누락이다.

## 수정

- `hwp3_default_endnote_shape()`가 `between_margin: u16` 인자를 받아
  `raw_unknown` 필드에 배선하도록 변경.
- `fixup_hwp3_notes()`가 `footnote_between_margin: u16` 인자를 받아 호출부
  (`parse_hwp3`)에서 `doc_info.footnote_between_margin`을 전달.

파일: `src/parser/hwp3/mod.rs` (16 lines changed).

## 검증

- 신규 단위 테스트 `issue_hwp3_endnote_raw_unknown_wires_footnote_between_margin`
  추가: `hwp3_default_endnote_shape(720).raw_unknown == 720` 확인 (수정 전 실패,
  수정 후 통과).
- `cargo check --lib` 통과.
- `cargo test --lib issue_hwp3_endnote_raw_unknown_wires_footnote_between_margin`
  통과.
- `rustfmt --edition 2021 src/parser/hwp3/mod.rs` 적용 완료.

## 범위

`src/parser/hwp3/` 내부로 국한. 렌더러·문서 코어에 HWP3 전용 분기를 추가하지
않았다.
