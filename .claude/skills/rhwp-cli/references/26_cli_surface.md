# 기존 CLI 표면만

이 스킬이 호출을 생성하는 이름 목록이다. 여기 없는 이름은 발명이 아니라 매뉴얼 확인 대상이다.
확인 후에도 없으면 호출하지 않는다.

## 핵심 명령

- `export-svg`
- `export-png`
- `export-pdf`
- `export-text`
- `export-markdown`
- `dump-pages`
- `dump`
- `dump-records`
- `diag`
- `info`
- `export-render-tree`
- `ir-diff`
- `thumbnail`
- `convert`
- `hwp5-inventory-diff`
- `hwp5-inventory`
- `hwp5-inventory-diff`
- `hwp5-contract-analyze`
- `hwp5-ctrl-data-trace`
- `hwp5-contract-probe`
- `hwp5-table-probe`
- `hwp5-cell-header-probe`
- `hwp5-mel-personnel-probe`
- `hwp5-borderfill-diagonal-probe`
- `hwp5-first-para-control-probe`
- `hwp5-anchor-trace`
- `hwp5-char-shape-audit`
- `hwp5-roundtrip`

## 명시적으로 이 스킬의 1차 축이 아닌 것

- `edit *` / `batch fill` / `inspect *` / `replay` / `audit` / `lineage`
- `explore` / `digest` / `search` (트리아지 스킬)
- `render-diff` (시각 회귀 스킬). 언급은 하되 여기서 고도화하지 않음
- `test-*` / `gen-*` (내부 개발)

새 rhwp CLI 명령을 이 PR 에서 추가하지 않는다.
