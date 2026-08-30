---
kind: working
status: active
issue: 6342
---

# 결재 표를 채운 쪽의 붙임 두 줄은 다음 쪽이다 (#6342)

작업 브랜치: `fix/6342-tac-overflow-page-split`
대상: `src/renderer/typeset.rs`
시험: `tests/cases/issue_6342_tac_overflow_page_split.rs`

## 한 줄

본문의 90% 이상을 채운 TAC 자리차지 표 뒤에 짧은 붙임 두 줄이 오면, 잔여
칸에 한 줄만 끼워 넣지 않고 다음 쪽으로 함께 넘긴다.

## 판별

`36385445` 저장 vpos(68000/70160)는 본문 952.5px 안에 다 담기지 않는다.
첫 줄 28.8px 는 잔여 53px 에 들어가지만 두 줄을 더하면 used=964.3px 로
넘친다. 한글은 둘 다 2쪽에 둔다. `original_hwpx_tac_filled_page_keeps_short_trail`
은 원본 HWPX · TAC · TopAndBottom · 단 기준 4×1 · 표높이≥본문 90% · 40px
미만 두 줄이 혼자 들어가고 둘을 더하면 넘칠 때만 연다. 편람(#3931)·보도자료
상자(#6044)는 이 형상이 아니다.

## 기록

`#6342`, 픽스처 `samples/hwpx/opengov/36385445_…hwpx`.
쪽수 원장 `tests/fixtures/oracle_page_count_baseline.tsv` 는 정답 2 / 기준선 1
이므로 2쪽으로 맞추면 원장도 통과한다.
