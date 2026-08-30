---
kind: working
status: active
issue: 6448
---

# HWPX CELL/TAC 표의 누락된 host band를 물리 높이로 보정한다 (#6448)

브랜치: `fix/6448-table-leftover-defer`

## 한 줄

HWPX `pageBreak="CELL"`(모델 RowBreak) + `treatAsChar` 표가 단일 host
`LINE_SEG`에 표 높이를 저장하지 않았으면, 그 짧은 줄을 표 band로 쓰지 않고 측정
표 높이와 trailing line spacing으로 쪽 흐름을 계산한다.

## 원인

`tac_cell_leftover_fits.hwpx`의 3행 표는 선언 높이 68000HU(약 906.7px)인데,
빈 host 문단에는 1000HU(약 13.3px) `LINE_SEG` 하나만 있다. 이 저장 줄을 표 band로
신뢰하면 rhwp는 HEAD·표·TAIL을 한 쪽 또는 두 쪽에 과밀 배치한다.

Hancom 2020 변환 기준(`pdf/tac_cell_leftover_fits-2020.pdf`, 3쪽)은 HEAD를 1쪽,
표를 2쪽, `AFTER TABLE`을 3쪽에 둔다. 따라서 특례는 HWPX 저장 조판, 단일 빈 host,
control 하나, RowBreak/TAC `3행 x 1열`·3셀·`repeatHeader=0` 표, 단일 비합성
`LINE_SEG`가 측정 표보다 짧은 경우로 제한한다.

## 범위

- `src/renderer/typeset.rs` — 누락된 RowBreak/TAC host band 보정 및 다음 일반 문단의
  1회 엄격 fit
- `samples/issue6448/tac_cell_leftover_fits.hwpx`
- `tests/cases/issue_6448_tac_cell_leftover_fits.rs`
