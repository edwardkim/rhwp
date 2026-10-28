# Task M100 #2129 단계별 완료보고서 — 1단계: 모델 필드 + 파서 (grouping 분기 · marker arm)

- 이슈: #2129 (C1d 라인 누적 + 표식)
- 브랜치: `local/task2129`
- 단계: 1/4
- 작성일: 2026-07-09

## 구현 내용

### (a) `src/ooxml_chart/mod.rs` — 모델 확장

- `OoxmlChart`에 필드 2개 추가:
  - `line_grouping: BarGrouping` — 라인 plot의 `c:grouping`. 막대 `grouping`과 **별도
    필드** (콤보에서 XML 문서 순서에 따른 상호 오염 방지).
  - `line_markers: bool` — plot 레벨 `<c:marker val="1"/>` 표식 표시 여부.
- `BarGrouping` doc 갱신: "line 누적은 미지원(C1d)" → 막대/라인 공용 (라인 `standard`
  → Clustered 흡수).
- 모듈 doc 지원 범위에 라인 누적/백프로/표식 반영.

### (b) `src/ooxml_chart/parser.rs` — 파싱

- `b"grouping"` arm 재작성: 매핑 1회 계산 후 `cur_plot_type` 분기 —
  Column/Bar→`chart.grouping`, Line→`chart.line_grouping`, 그 외 무시.
- `b"marker"` arm 신설: 게이트 = `cur_plot_type==Some(Line) && cur_series.is_none()
  && val 속성 존재`. 계열 내부 `<c:marker>`(val 없음, symbol/size 래퍼)는 자연 배제.
  `<c:marker val="1"/>`는 Empty 이벤트 → `handle_start`만으로 충분, `handle_end` 무변경.

### (c) 단위 테스트 6건 (신규 5 + 기존 1 반전)

| 테스트 | 입력 | 기대 |
|--------|------|------|
| `test_parse_line_grouping_stacked` (기존 `test_parse_grouping_line_ignored` 반전) | lineChart + stacked | `line_grouping==Stacked` **이면서** `grouping==Clustered` 불변 |
| `test_parse_line_grouping_percent_stacked` | lineChart + percentStacked | `PercentStacked` |
| `test_parse_line_grouping_standard` | lineChart + standard | `Clustered` 흡수 |
| `test_parse_combo_grouping_no_cross_contamination` | barChart(stacked)+lineChart(standard) 공존 | `grouping==Stacked` && `line_grouping==Clustered` |
| `test_parse_line_marker_flag` | plot 레벨 marker val=1 / 0 / 부재 | true / false / false |
| `test_parse_series_marker_ignored` | 계열 내부 marker 래퍼 + scatterChart 내 marker val=1 | `line_markers==false` 유지 |

테스트 헬퍼 `line_xml(grouping, marker_val)`은 실샘플(누적꺽은선형.hwpx) 구조를 본떠
계열 내부 `<c:marker><c:symbol val="none"/><c:size val="7"/></c:marker>` 래퍼를 항상 포함.

## TDD 절차 준수

RED 확인 (파서 구현 전): 신규 동작 필요 3건이 정확한 사유로 실패 —

```
test_parse_line_grouping_stacked          FAILED (left: Clustered, right: Stacked)
test_parse_line_grouping_percent_stacked  FAILED (left: Clustered, right: PercentStacked)
test_parse_line_marker_flag               FAILED
→ 25 passed; 3 failed
```

GREEN (파서 구현 후):

```
$ cargo test --lib ooxml_chart
test result: ok. 61 passed; 0 failed   (파서 + 렌더러 전체)

$ cargo clippy --all-targets -- -D warnings   → 무경고 (exit 0)
```

## 실샘플 신호 확인 (수행계획서 사실검증 재게시 — HWPX `Chart/chart1.xml` 추출)

| 샘플 | c:grouping | plot 레벨 c:marker val |
|------|-----------|------------------------|
| 꺽은선형 | standard | 0 |
| 표식이있는꺽은선형 | standard | 1 |
| 누적꺽은선형 | stacked | 0 |
| 표식이있는누적꺽은선형 | stacked | 1 (c:symbol 없음 → 기본 사이클) |
| 백프로기준누적꺽은선형 | percentStacked | 0 |

→ 5종 모두 예상 신호 보유. 2단계 렌더 분기 입력 확정.

## 완료 기준 충족

- [x] `cargo test --lib ooxml_chart` 통과 (61/61, 렌더러 무변경 = 행동 변화 0)
- [x] `cargo clippy --all-targets -- -D warnings` 무경고
- [x] 기존 고정 테스트(`test_parse_grouping_line_ignored`) 반전 완료

## 다음 단계

2단계 — `render_line` 누적/백프로 기하 + 축 (stacked=카테고리 합 `nice_axis` /
percent=(0,100,20) + `render_value_grid` percent 플래그).
