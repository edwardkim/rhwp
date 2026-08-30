---
kind: working
status: active
issue: 6448
---

# leftover CELL/TAC 표를 다음 쪽에 통째 이월하지 않는다 (#6448)

브랜치: `fix/6448-table-leftover-defer`

## 한 줄

HWPX `pageBreak="CELL"`(모델 RowBreak) + `treatAsChar` 표는 leftover 에 선언
높이가 들어가면 통째 둔다.

## 원인

선언-fit 게이트가 `treatAsChar` 와 모델 CellBreak 를 제외한다. HWPX CELL 은
파서가 RowBreak 로 올리므로 leftover 에서 측정 팽창이 나면 표를 다음 쪽으로
민다. 156760012 는 잔여 205.7pt, 표 115.4pt 인데 4쪽으로 통째 이월되어 11쪽 vs
한글 10쪽.

## 범위

- `src/renderer/typeset.rs` — leftover 선언-fit
- `samples/issue6448/tac_cell_leftover_fits.hwpx`
- `tests/cases/issue_6448_tac_cell_leftover_fits.rs`
