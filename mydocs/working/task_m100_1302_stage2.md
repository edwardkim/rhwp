# Stage 2 완료보고서 — Task #1302 수정 + 회귀 테스트

- **이슈**: #1302 / 브랜치 `local/task1302` (base `stream/devel` 9d3aa212)

## 변경

### `src/renderer/height_cursor.rs` — `vpos_adjust`
`compact_endnote_page_tail_backtrack` 조건에 게이트 추가:
```rust
let curr_is_equation_only_tail = paragraphs.get(item_para)
    .map(para_is_treat_as_char_equation_only).unwrap_or(false);
let curr_first_full_advance = !curr_is_equation_only_tail
    && matches!(curr_first_vpos, Some(v) if v - seg.vertical_pos >= seg.line_height + seg.line_spacing);
// ... && !curr_first_full_advance && ...
```
- curr 첫 줄 stored vpos 가 **정상 한 줄 전진(lh+ls) 이상**을 인코딩하는 **breakable 텍스트**
  연속 문단이면 page-path tail backtrack 비활성 → y_offset(trailing 포함 정답) 유지.
- **수식-only tail(#1274)은 제외** — atomic 이라 frame-fit backtrack 필요(종전 유지).

### `tests/issue_1139_inline_picture_duplicate.rs`
회귀 핀 추가: `issue_1302_2022_nov_page18_multiline_endnote_continuation_keeps_line_spacing`
- 18쪽 pi=852 끝줄 → pi=853 첫줄 gap 이 16~22px(정상 한 줄) 범위인지 검증.
- 수정 전 gap=14(FAIL), 수정 후 gap=18(PASS) 확인.

## 빌드/단위 검증

- `cargo build` OK.
- 신규 테스트 PASS, 수정 전 FAIL 확인(진짜 핀).
- 1차 시도(게이트가 수식-only 미제외)에서 `issue_1274`(3-10월 11쪽 수식 tail) 회귀 →
  수식-only 제외로 해소. 교훈: tail backtrack 은 atomic 수식과 breakable 텍스트를 구분해야 함.

## 다음

Stage 3: SVG↔PDF 측정, 전체 `cargo test`, 회귀 샘플 diff.
