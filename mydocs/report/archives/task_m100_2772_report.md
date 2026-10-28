# Task #2772 — HWP3 footnote_line_margin → separator_margin_top 배선

## 이슈
- edwardkim/rhwp#3046

## 배경

`Hwp3DocInfo`의 `impl` 블록에는 `read()` 외 별도 accessor 메서드가 없어, 필드는 전부 `pub` 필드로
직접 참조된다. `src/parser/hwp3/mod.rs`에서 각 필드의 실사용 여부를 확인한 결과
`footnote_line_margin`(오프셋 104, "각주 분리선과 본문 사이의 간격", `한글문서파일구조3.0.md` 10.11절)이
파싱만 되고 어디에도 배선되지 않은 상태였다.

`footnote_bracket`(#3023), `footnote_line_width`(#3027), `footnote_between_margin`(#3036)는 이미
별도 PR로 진행 중이라 중복을 피하고 `footnote_line_margin`을 선택했다.

## 원인

HWP3 각주/미주 모양은 `hwp3_default_endnote_shape()`가 만드는데, 이 함수는 `doc_info`를 전혀
받지 않고 `separator_margin_top`(분리선 위 여백)을 하드코딩된 864로 채우고 있었다.

## 수정

- `hwp3_default_endnote_shape(doc_info: &Hwp3DocInfo)`로 시그니처 변경.
- `doc_info.footnote_line_margin`이 0이 아니면 `× 4`(HWP3 hunit → HWPUNIT, 기존 margin 필드들과 동일한
  변환 규칙)해 `separator_margin_top`으로 배선. 0이면 기존 기본값 864 유지(회귀 방지).
- 호출부 `fixup_hwp3_notes(doc, doc_info)`로 `doc_info`를 관통 전달.

## 검증

- `cargo check --lib` 통과.
- `cargo test --lib task2772_hwp3_default_endnote_shape` 1개 통과.

## 남은 doc_info 필드

`footnote_bracket`, `footnote_line_width`, `footnote_between_margin`은 각각 오픈 PR(#3023/#3027/#3036)에서
다루는 중이며, `footnote_text_margin`은 유사 후속 작업 후보로 남아 있다. 나머지 필드
(cursor_para/cursor_pos, paper_kind, doc_protected, reserved1, link_page_number, link_footnote_number,
link_print_file, description, encrypted, footnote_reserved, move_frame, sub_revision)는 파서 내부 상태이거나
IR로 옮길 대응 필드가 없어 미배선이 타당하다.
