---
kind: working
status: active
issue: 5196
---

# HWP3 쪽 테두리 여백 ×4 가 i16 을 넘어도 패닉하지 않는다 (#5196)

작업 브랜치: `fix/5196-hwp3-border-margin-overflow`
대상: `src/parser/hwp3/mod.rs`

## 한 줄

`border_margin: u16` 을 `as i16 * 4` 하면 퍼징 입력에서 debug overflow 로
죽는다. `i32` 로 곱한 뒤 i16 범위에 붙인다.

## 기록

`#5196`, 크래시 `parse_hwp3.rs` / `hwp3/mod.rs:138`.
새 `#[test]` 를 넣지 않고 기존 `test_hwp3_page_border_fill_is_always_page_basis`
에 포화 단언만 더했다 (unit-test-tier 총량 동결).
