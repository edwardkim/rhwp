# Task M100 #1453 구현계획서 — 3D막대·3D원형·ofPie 차트 라우팅

- 이슈: #1453
- 브랜치: `local/task1453`
- 작성일: 2026-06-21
- 수행계획서: `mydocs/plans/task_m100_1453.md`

## 구현 개요

파서 `parse_chart_xml`(`src/ooxml_chart/parser.rs`)의 `handle_start` match에 `bar3DChart`·`pie3DChart`·
`ofPieChart` 세 요소명을 추가하여 기존 plot 타입으로 라우팅한다. 라우팅 코드는 기존
`barChart`/`pieChart` 분기와 동일 구조를 미러한다(중복 최소화 — 같은 4줄 셋업). `handle_end`의
plot-종료 arm에도 세 이름을 추가해 axId 일괄 할당이 동일하게 동작하도록 한다. **렌더러·데이터
모델은 손대지 않는다** — 라우팅된 Column/Bar/Pie 는 기존 `render_bars`/`render_pie` 가 그린다.

---

## 1단계 — 파서 라우팅 + 단위 테스트

**대상**: `src/ooxml_chart/parser.rs`

(a) `handle_start`(`parser.rs:166` match) — `b"pieChart"` 분기(`parser.rs:181`) 뒤에 추가:

```rust
b"bar3DChart" => {
    // 3D 막대 — 2D 근사(C1a). barDir 핸들러가 col/bar를 그대로 채워 후처리가 Column↔Bar 확정.
    chart.chart_type = OoxmlChartType::Column;
    st.cur_plot_type = Some(OoxmlChartType::Column);
    st.cur_plot_ax_ids.clear();
    st.cur_plot_series_start = chart.series.len();
}
b"pie3DChart" | b"ofPieChart" => {
    // 3D 원형 / ofPie — 단일 원형으로 2D 근사(C1a). 보조플롯·입체는 후속(C2).
    chart.chart_type = OoxmlChartType::Pie;
    st.cur_plot_type = Some(OoxmlChartType::Pie);
    st.cur_plot_ax_ids.clear();
    st.cur_plot_series_start = chart.series.len();
}
```

(b) `handle_end`(`parser.rs:344`) plot-종료 arm 확장:

```rust
b"barChart" | b"lineChart" | b"pieChart" | b"bar3DChart" | b"pie3DChart" | b"ofPieChart" => {
    // plot 종료 — 이 plot에 속한 시리즈에 axIds 복사 (기존 로직 그대로)
    ...
}
```

(c) 파서 단위 테스트 추가(`parser.rs` `#[cfg(test)] mod tests`, 기존 `test_parse_pie_chart`
`parser.rs:472` 패턴):
- `test_parse_bar3d_col` — `bar3DChart`+`barDir val="col"` → `chart_type == Column`, values 추출.
- `test_parse_bar3d_bar` — `bar3DChart`+`barDir val="bar"` → `chart_type == Bar`.
- `test_parse_pie3d` — `pie3DChart` → `chart_type == Pie`, values 추출.
- `test_parse_ofpie` — `ofPieChart` → `chart_type == Pie`, values 추출.

**완료 기준**: `cargo test ooxml_chart` 통과 (신규 4건 + 기존 5건). `cargo build` 통과.
단계별 보고서 `mydocs/working/task_m100_1453_stage1.md` 작성 + 커밋.

## 2단계 — 회귀 통합 테스트 + 문서 주석

**대상**: 신규 `tests/issue_1453_chart_3d_ofpie_routing.rs`, `src/ooxml_chart/mod.rs`

- 통합 테스트(패턴: `tests/issue_1156_chart_column_flow.rs` `render_page_svg` 헬퍼 재사용):
  7종 × (hwp, hwpx) = **14파일** 각각 `render_page_svg(0)` 에 대해
  - `assert!(!svg.contains("차트 (미지원)"))` — placeholder 미발생
  - `assert!(svg.contains("hwp-ooxml-chart\""))` — 정상 차트 클래스 존재(fallback `hwp-ooxml-chart-fallback` 아님)
  - 14파일 경로를 `[(rel, kind)]` 테이블로 두고 루프.
- `src/ooxml_chart/mod.rs` 지원 범위 주석(`mod.rs:7-15`) 갱신:
  "지원 범위"에 3D막대·3D원형·ofPie(2D 근사) 추가, "범위 외"에서 3D 차트 제거하고
  "3D 입체감·ofPie 보조플롯·산점도·stock" 으로 재서술.

**완료 기준**: `cargo test --test issue_1453_chart_3d_ofpie_routing` 통과(14파일).
기존 `issue_1156_chart_column_flow`·`issue_1251_ole_chart_contents` 회귀 없음.
단계별 보고서 `task_m100_1453_stage2.md` 작성 + 커밋.

## 3단계 — 14종 시각판정 산출물 + 최종 검증

- `cargo test` 전체 통과 + `cargo build` + `cargo clippy` 무경고(수정 파일 한정).
- 14파일 `rhwp export-svg ... -o output/poc/chart_c1a/` 산출 (PNG 변환 포함).
- `pdf/chart/{종류}-2022.pdf` 정답지와 종류별 대조표 작성(`task_m100_1453_stage3.md`):
  placeholder 0건 / 데이터·기하 정확성 / 알려진 스타일 4갭 / **2D 근사 수용선**(3D→평면, ofPie→단일원형).
- 단계별 보고서 `task_m100_1453_stage3.md` 작성 + 커밋 → **작업지시자 시각판정 + 수용선 확정**.

**완료 기준**: 전체 테스트·빌드·clippy 통과 + 시각판정 자료 산출 완료.

---

# Part B — 막대 누적(stacked/percentStacked) 보정 (범위 확장, 4~6단계)

> 3단계 시각판정에서 `render_bars`가 `c:grouping`을 무시하고 항상 grouped로 그려 누적/백프로
> 막대 6종이 왜곡됨을 확인 → 작업지시자 승인(2026-06-21)으로 C1a에 포함. 꺾은선 누적은 후속.

## 4단계 — 파서 grouping 파싱 + 모델 필드

**대상**: `src/ooxml_chart/parser.rs`, `src/ooxml_chart/mod.rs`

- `mod.rs`: `OoxmlChart`에 `grouping: BarGrouping` 필드 추가. 신규 enum
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
  pub enum BarGrouping { #[default] Clustered, Stacked, PercentStacked }
  ```
  (`standard`는 Clustered로 흡수.)
- `parser.rs` `handle_start`에 `b"grouping"` 추가: `val` 속성을 BarGrouping으로 매핑
  (`stacked`→Stacked, `percentStacked`→PercentStacked, 그 외→Clustered). bar/bar3D plot 내에서만 유효.
- 단위 테스트: `c:grouping=stacked`·`percentStacked`·`clustered` → 필드 매핑 검증.

**완료 기준**: `cargo test ooxml_chart` 통과. 빌드 통과.

## 5단계 — render_bars 누적/백프로 렌더 + 값축

**대상**: `src/ooxml_chart/renderer.rs`

- `render_bars` 분기:
  - **Clustered**(현행): 그대로 side-by-side.
  - **Stacked**: 카테고리별로 시리즈 값을 누적 — 같은 x(카테고리 1열)에 아래에서 위로 쌓음.
    값축 max = 카테고리별 시리즈 합의 최대.
  - **PercentStacked**: 카테고리 합을 100%로 정규화 — 각 시리즈는 비율만큼 차지, 누적이 플롯 전체 높이.
    값축 0~100%(`%` 라벨).
- `value_range` / `render_value_grid`를 grouping-aware로 (stacked max, percent 라벨). 가로(`horizontal`)도 동일 원리.
- Pie/Line 경로 무영향.

**완료 기준**: 6종 export-svg에서 stacked는 누적 막대, percent는 100% 꽉 찬 막대로 렌더.

## 6단계 — 누적 회귀 테스트 + 시각판정 + 최종 검증

- `tests/issue_1453_chart_3d_ofpie_routing.rs`(또는 신규)에 누적 기하 가드 추가:
  - stacked 수직 막대: 한 카테고리 내 시리즈 rect가 **같은 x 공유**(grouped와 구분).
  - percentStacked: 각 카테고리 누적 top이 플롯 상단 도달(전체 높이).
  - 대상 6종(3차원누적세로/가로 + 누적세로/가로 + 백프로기준누적세로/가로) hwpx.
- `output/poc/chart_c1a/` 6종 재산출 → `pdf/chart/` 대조표 갱신.
- `cargo test` 전체 + `clippy` 무경고.

**완료 기준**: 누적 가드 통과 + 전체 스위트 통과 + 시각판정 자료 산출.

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `src/ooxml_chart/parser.rs` | (A) `handle_start` 요소명 3개 + `handle_end` arm + 단위 4건 / (B) `grouping` 파싱 + 단위 |
| `src/ooxml_chart/mod.rs` | (A) 범위 주석 / (B) `BarGrouping` enum + `OoxmlChart.grouping` 필드 |
| `src/ooxml_chart/renderer.rs` | (B) `render_bars` 누적/백프로 + 값축 grouping-aware |
| `tests/issue_1453_chart_3d_ofpie_routing.rs` | (A) 14파일 가드 / (B) 누적 기하 가드 6종 |
| `mydocs/working/task_m100_1453_stage{1..6}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_1453_report.md` | 최종 보고서 |
| `output/poc/chart_c1a/` | 시각판정 산출물 (gitignore) |

## 위험 / 주의

- **3D 누적막대**: 라우팅 후 기존 `render_bars` 가 (true stack 아닌) grouped 로 그릴 수 있음 —
  이는 2D 표본에도 존재하는 기존 스타일 갭이며 C1a 범위 밖(데이터·기하 정확성은 유지). 시각판정에서 수용선 확인.
- **ofPie 단일 원형 근사**: 보조플롯(원형대원형의 2차 원, 원형대막대의 막대)은 미표현. 의도된 C1a 근사.
- **barDir 후처리 의존**: `bar3DChart` 가 `barDir` 자식을 항상 가진다는 전제(7종 샘플로 확인). 누락 시 기본 Column.
- 기능 변경만 포함 — 포맷(`cargo fmt --all`) 전체 적용 금지(수정 파일 범위만).
