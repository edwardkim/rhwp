---
kind: working
status: active
issue: 5184
---

# HWP3 빈 셀 vertsize=1000 을 HWPX 왕복에서 지킨다 (#5184)

작업 브랜치: `fix/5184-hwp3-empty-cell-ir`
대상: `src/document_core/commands/document.rs` TAC host LINE_SEG 확대
시험: `tests/cases/issue_5184_hwp3_empty_cell_vertsize.rs`

## 한 줄

HWPX 로드 시 TAC 표 높이로 host `line_height` 를 키우는 보정은 **기본
lh≤100 합성 seg** 에만 적용한다. HWP3 빈 셀의 저장 1000 을 표 높이로
덮으면 `--verify` 가 실패한다.

## 기록

`#5184`, `samples/hwp3-empty-cell.hwp`. char_shapes 차이는 이 패치 전에
이미 닫혀 있었고, 남은 축은 paragraph 5·7 vertsize 다.
