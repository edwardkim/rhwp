---
kind: working
status: done
canonical: mydocs/working/task_m100_5652_stage3.md
last_verified: 2026-08-23
---

# #5652 Stage 3 — 코어: `structure` 의도 분기 · 위치 기반 plan · 종류별 가드 · self-check

- **계획서**: [`mydocs/plans/task_m100_5652.md`](../../plans/archives/task_m100_5652.md) §3-4
- **브랜치**: `task5652` (`upstream/devel` `bf30bd792` 기준)

## 1. 무엇을 만들었나 (`src/document_core/commands/object_ops/chart.rs`)

| 항목 | 내용 |
|---|---|
| `ChartEdits.structure: bool` | `#[serde(default)]` — 없으면 B1 그대로 |
| `validate()` → `validate_values()` / `validate_structure()` | B1 네 거부(`seriesCountMismatch`·`valueCountMismatch`·`seriesNameMismatch`·`categoryMismatch` ×2곳)는 그대로 서고 메시지만 `structure:true` 안내 추가 |
| `validate_structure()` 순서 | 행렬 모양(`lastSeriesDeleteRefused`·`rowCountMismatch`·`lastPointDeleteRefused`) → 종류별 가드(`pieSeriesCountFixed`·`stockSeriesCountFixed`·`multiLevelLabelsUnsupported`) → 라벨 규칙(`labelsRequired`/`scatterXYMismatch`·`labelCountMismatch`·`sharedCategoryRequired/sharedXRequired`·`notANumber`·`unsafeText`·`valueNotPatchable`) → 기존 계열 값·이름(`notANumber`·`valueNotPatchable`·`unsafeText`·`seriesNameNotPatchable`·`pointsNotInsertable`) → 신설 계열(`seriesNotClonable`·`seriesNameRequired`·`seriesNameNotPatchable`·`unsafeText`·`notANumber`) |
| `plan_edits()` → `Vec<ChartEdit>` | 겹치는 칸 치환, 이름 변경 `SeriesName`, 행 증감 `AppendPoints`/`TruncatePoints`(값 + 라벨 보유 계열의 라벨), 라벨 텍스트 변경은 라벨 보유 **전 계열 동기**, 계열 증감 `AppendSeries`(labels = 목표 라벨)/`TruncateSeries`. `changed[]` 에 `op` 항목(`renameSeries`·`appendPoints`·`truncatePoints`·`relabel`·`appendSeries`·`truncateSeries`), B1 값 항목 형상 유지 |
| self-check | 패치 직후 ①② 각각 `scan_chart_values` + `rescan_matches`(계열 수·값·이름·라벨 == 목표), 둘 다 있으면 `same_chart_data`. 어긋나면 `selfCheckFailed` 로 한 바이트도 쓰지 않는다 |

`apply_value_edits` → `apply_chart_edits` 교체 외에 ①② 원자 대입·dry-run·무변경 조기 반환·캐시 무효화는 B1 그대로.

## 2. 결정과 근거

- **의도 분기는 `seriesCountMismatch` 조기 return 앞** — 계열 수가 다르면 B1 은 대조를 끊는다.
  `structure` 를 그 앞에서 가르지 않으면 열 편집이 엉뚱한 계열을 짝짓는다(탐색 리스크 1).
- **행렬은 직사각형** — `rowCountMismatch` 를 가장 먼저 본다. CSV 동형이 성립하는 조건이고,
  신설 계열의 최종 행 수를 계열 0 의 값 개수로 정한다.
- **라벨 규칙** — 라벨 보유 계열의 행 수가 바뀌면 `labels` 필수(`labelsRequired`; 분산형은
  `scatterXYMismatch` — xVal/yVal 동기 가드와 같은 사유). 라벨 없는 차트(`c:cat` 부재)는
  labels 없이도 행 증감 가능.
- **이름 None≡""** — B1 의 동일시를 유지한다. `c:tx` 없는 계열에 빈 이름은 무편집, 빈 아닌
  이름은 `seriesNameNotPatchable`(넣을 자리가 없다 — 구조 신설은 범위 밖).
- **종류 가드는 계열 수 변경에만** — 원형/주식형도 행 증감·값·이름·라벨은 된다. 다층
  카테고리는 행 수 변화·라벨 지정만 거부하고 값 편집은 B1 처럼 허용.
- **self-check 는 B1 경로에도** — 비용은 재스캔 두 번이고, "썼는데 못 읽는 파일"을 코드로 막는다.

## 3. 판정 (`tests/issue_4100_chart_data_edit.rs` Stage 9)

| 테스트 | 고정 내용 |
|---|---|
| `structure_flag_off_keeps_every_b1_refusal` (+ 기존 `every_refusal_writes_nothing`) | 플래그 없으면 B1 거부, 메시지에 `structure` 안내 |
| `structure_row_append_writes_both_representations_and_rereads` | hwpx ①② / hwp ② 기록, 재개방 행 5·ptCount 5·idx 0..4, `c:f`·`hncChartStyle`·③④ 불변, ①==② |
| `structure_middle_row_delete_equals_spike_surgery` | 「항목 2」 삭제 ①② == `b2_remove_point` ×2 |
| `structure_series_append_and_truncate_rename_relabel` | 4 연산 각각 스파이크 수술과 바이트 동일 + `changed[].op` |
| `structure_dry_run_reports_ops_and_writes_nothing` | dry-run 봉투 + 바이트 불변 |
| `structure_matrix_rules_are_enforced` | `rowCountMismatch`·`labelsRequired`·`labelCountMismatch`·`notANumber`·`unsafeText`·`seriesNameRequired` 전건 0바이트 |
| `structure_edit_refuses_when_block_not_resizable` | ptCount 없는 리터럴 블록 → `pointsNotInsertable` |
| `pie_series_count_is_fixed` (3종 × 2포맷) · `stock_series_count_is_fixed` (OHLC 4→3 ×2포맷, HLC 3→4) · `last_point_and_last_series_cannot_be_deleted` (특이케이스) · `scatter_rows_require_synchronized_x` (거부 2 + 동기 통과 == 스파이크) · `multi_level_labels_refuse_structure_edits` (합성 주입, 값 편집은 통과) · `guards_fire_in_dry_run_too` | 종류별 가드 fail-closed, `invalid[]`, 바이트 불변 |

### 게이트 실측 (2026-08-23)

| 게이트 | 결과 |
|---|---|
| `issue_4100_chart_data_edit` | 53 passed / 2 ignored (+13) |
| `cargo test -p rhwp-ooxml-chart --lib` 165 · `ooxml_chart_structure_contract` 33 · `issue_4099` 10 · `issue_4098` 9 · `issue_4055` 9 · `issue_4694` 5 · `issue_3546` 2 · `issue_3547` 3 · `chart_csv_contract` 17 · `set_chart_data_contract` 4 · `charts_contract` 3 | 전건 passed |
| fmt `--check` · suite-manifest `--prepare`→`--check` · unit-tiers `--base-ref upstream/devel`(4225 불변) · clippy `-D warnings` | 통과 |

## 4. 확인된 것

- 코어 경로로 만든 ①② 가 스파이크 수술(한컴 판정 완료 바이트)과 동일하다 — S5 재판정은 "엔진이
  같은 바이트를 만든다"의 확인이 된다.
- `selfCheckFailed` 경로는 합성으로 유발하기 어렵다(패처와 스캐너가 한 좌표계) — 코드 리뷰와
  B1·B2 전 경로의 재스캔 통과로 간접 검증. 방어 계층으로 둔다.

## 5. 다음

S4 — CLI `csv-to-chart --structure`, `edit set-chart-data --dry-run` 코어 경유, MCP 스키마·프로필·문서·codex.
