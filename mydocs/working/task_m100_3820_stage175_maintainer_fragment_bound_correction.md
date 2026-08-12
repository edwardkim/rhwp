# Task M100 / #3820 Stage 175 - maintainer fragment-bound correction

## 검토 지적과 보정

Stage 174의 `source_first_fragment_overflow_allowance`는 saved frame bottom을
`base_available`(전체 body 높이)와 비교했다. 그러나 RowBreak first fragment의
실제 row-scan 예산은 기존 각주, zone, caption, host offset, outer spacing,
painted footer guard를 차감한 `page_avail`이다.

따라서 source frame이 전체 body 안에 있더라도 현재 fragment의 각주/zone 예약
lane에 닿을 수 있었다. `table_footnotes.is_empty()`는 해당 표에 새 각주가 없다는
뜻일 뿐, 앞서 배치된 page footnote 또는 zone 제한이 없다는 보장은 아니다.

## 구현 변경

[`src/renderer/typeset.rs`](../../src/renderer/typeset.rs)는 다음 값을 first
fragment source-frame의 유일한 비교 상한으로 사용한다.

```rust
let source_first_fragment_scan_bottom = st.current_height + page_avail;
```

`page_avail`은 이미 모든 fragment-local 제한을 반영한 row budget이다. saved
physical bottom이 이 상한 안에 있을 때만, source frame 아래부터 이 상한까지의
남은 실제 px을 overflow allowance로 쓴다. 따라서 Stage 174의 p4 cut은 유지하되,
전체 body 아래에 남은 공간을 footnote/zone 예약을 넘어 재사용하지 않는다.

## 강화한 회귀 계약

[`tests/issue_3820_rowbreak_rowspan_band.rs`](../../tests/issue_3820_rowbreak_rowspan_band.rs)의
p4 검사는 이제 다음을 함께 확인한다.

- p4가 source-owned `제32조(보호구의 지급 등)` 및 `이륜자동차` body를 가진다.
- p4 outer RowBreak fragment bottom이 PDF footer band(1040--1052px)에 남는다.
- p5가 `안전모를 착용하도록 지시`로 시작하는 saved tail을 가지고, p4-owned
  Article 32 opening을 되풀이하지 않는다.

실행 결과:

```text
issue_3820_rowbreak_rowspan_band: 4 passed; 0 failed
```

이 계약은 header-only 조기 이월과, 허용치를 과도하게 넓혀 row/table 전체를 p4에
흡수하는 양쪽 회귀를 모두 막는다.

## 전수 owner 확인

Stage 175의 `fidelity_compare --text-only --export-all-svg --layout-ledger` 전수 실행은
기준 PDF, rhwp SVG, rhwp render tree가 모두 82쪽임을 다시 확인했다.

- p4 -> p5의 `rhwp_later_than_reference` table-fragment owner 후보는 없다.
- p33 -> p36의 page-boundary/text-owner/text-sequence 후보도 없다.
- 남은 후보는 이번 first-fragment 경계와 무관한 p6->7, p38->39, p55->56,
  p70->71의 기존 visual-review 후보뿐이다.

산출물은 `output/task-3820-stage175-76076-full-owner`에 남겼다.
