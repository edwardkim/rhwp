---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 34 — #3637 p26 visible page-owner 보정

## 고정된 실패 신호

Stage 33의 `visible-text-excess-candidates.tsv`는 HWP 2020 PDF p26에서
`reference_only=0`, `visible_svg_only=66`을 기록했다. 즉 PDF p26의 text는 rhwp에 남아
있지만 다음 source owner의 text가 p26에 추가로 보인다. 이 신호는 raw SVG에 남는 off-page
table descendants 415자를 제외한 결과다.

## 보정 제약

- fixture 전체는 HWP 2020 PDF와 31쪽을 유지해야 한다.
- p26 cut을 전역/무조건으로 한 unit 줄이는 실험은 전체 30쪽으로 변해 반증됐다.
- `overflow_cell_baseline=19`은 계속 lower-bound clipping gate로만 유지한다.
- 보정은 `pi=197`의 1×1 RowBreak host에 mixed nested-tail physical flow를 더한 경우와,
  그 continuation의 source-unit 재개가 불일치하는지를 먼저 증명한 뒤에만 적용한다.

## 구현

`TableLayouter::mixed_nested_split_from_cut`에서 다음 조건을 모두 만족할 때만 terminal
1×1 nested table의 continuation origin을 첫 visible child line만큼 전진시킨다.

- HWPX stored-layout profile
- 앞 fragment가 source offset을 가진 terminal slice
- 첫 visible source가 nested table 이전이 아닌 경우
- 해당 paragraph가 정확히 1×1 nested table control을 포함하는 경우

따라서 일반 HWP 및 다른 nested-table continuation에는 이 보정을 적용하지 않는다.
`typeset.rs`의 mixed nested owner guard는 페이지 cut 예산에서 이미 paint된 tail을 제외해
source owner를 새 페이지에 중복 배정하지 않도록 한다.

## 결과와 증적

- fixture render tree와 HWP 2020 PDF는 모두 31쪽이다.
- p26에는 마지막 `시간당 근로임금은 2024년 7월 기준` 행만 남고, p27은
  `사업체노동력조사(고용노동부)의`부터 시작한다.
- 24–30쪽 text-only + layout ledger의 visible text-excess, owner-shift,
  owner-sequence 원장은 p26·p27 후보를 기록하지 않았다.
- 페이지 직접 비교: [p26 owner 비교](../pr/assets/task_m100_3820_stage34_issue3637_p26_owner_repair/review-p26-owner-compare.png),
  [p27 owner 비교](../pr/assets/task_m100_3820_stage34_issue3637_p26_owner_repair/review-p27-owner-compare.png)

## 검증

```text
CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \\
  cargo test --profile release-test --test issue_3637_nested_table_starts_inside_parent_cell -- --nocapture
# 1 passed; 0 failed

RHWP_BIN=target/task-3820-3821-fidelity/release-test/rhwp \\
  python tools/fidelity_compare/fidelity_compare.py 24 30 \\
  --source samples/issue3637/regulatory_impact_nested_table_escape.hwpx \\
  --reference-pdf pdf/issue3637/regulatory_impact_nested_table_escape-current-2020.pdf \\
  --label issue3637_stage34_2020 --text-only --layout-ledger
# PDF/render tree page count: 31/31; p26·p27 visible-text-excess 후보 없음
```

다음 stage에서는 이 HWPX 전용 owner 보정을 일반 HWP nested-table fixture에 확대하지 않고,
별도 재현·PDF 기준으로 검증한다.
