# Task M100 #2129 구현계획서 — C1d 라인 누적(stacked/percentStacked) + 표식 렌더

- 이슈: #2129
- 브랜치: `local/task2129`
- 작성일: 2026-07-09
- 수행계획서: `mydocs/plans/task_m100_2129.md`

## 구현 개요

C1a Part B(막대 누적)와 대칭. 모델에 `line_grouping`·`line_markers` 필드를 추가하고,
파서의 `b"grouping"` arm을 plot 타입별 분기로 확장 + `b"marker"` arm을 신설한 뒤,
`render_line`(`renderer.rs:575`)에 stacked/percentStacked 분기와 마커 렌더를 추가한다.
축 정책·누적 정규화·percent 라벨은 전부 `render_bars`의 기존 헬퍼
(`category_positive_sum`·`nice_axis`·`render_value_grid` percent 플래그)를 재사용한다 —
새 축 기계장치 없음. `render_bars`·`render_combo`·`render_scatter` 무변경.

---

## 1단계 — 모델 필드 + 파서 (grouping 분기 · marker arm)

**대상**: `src/ooxml_chart/mod.rs`, `src/ooxml_chart/parser.rs`

(a) `mod.rs` — `OoxmlChart.grouping`(line 44) 아래 필드 2개 추가:

```rust
/// 라인(lineChart) plot의 `c:grouping` (standard/stacked/percentStacked).
/// 순수 라인 렌더러(render_line) 전용 — 콤보의 line 시리즈에는 미적용(코퍼스 무해당).
/// 막대 grouping과 별도 필드인 이유: 콤보(bar+line 공존)에서 단일 필드 공유 시
/// XML 문서 순서에 따라 상호 오염. (C1d #2129)
pub line_grouping: BarGrouping,
/// 라인 plot 레벨 `<c:marker val="1"/>` — 표식(마커) 표시 여부. 계열 내부
/// `<c:marker>`(val 없음, symbol/size 래퍼)와 구분됨. (C1d #2129)
pub line_markers: bool,
```

- `BarGrouping` doc(line 69) "line 누적은 미지원(C1d)" → "막대/라인 공용 그룹화 방식
  (`c:grouping`). 막대는 `clustered`, 라인은 `standard`를 Clustered로 흡수." 로 갱신.
- 모듈 doc 지원 범위(line 8-9): lineChart에 "누적/백프로 누적 + 표식(C1d #2129)" 반영.

(b) `parser.rs` — `b"grouping"` arm(line 245-259) 재작성:

```rust
b"grouping" => {
    // bar/bar3D → chart.grouping, line → chart.line_grouping (C1d #2129).
    // 콤보에서 상호 오염 방지 위해 별도 필드에 분기 저장.
    if let Some(val) = attr_val(e, "val") {
        let g = match val.as_str() {
            "stacked" => BarGrouping::Stacked,
            "percentStacked" => BarGrouping::PercentStacked,
            _ => BarGrouping::Clustered,
        };
        match st.cur_plot_type {
            Some(OoxmlChartType::Column | OoxmlChartType::Bar) => chart.grouping = g,
            Some(OoxmlChartType::Line) => chart.line_grouping = g,
            _ => {}
        }
    }
}
```

(c) `parser.rs` — `handle_start`에 `b"marker"` arm 신설:

```rust
b"marker" => {
    // plot 레벨 <c:marker val="0|1"/> (lineChart 직계 자식, Empty 이벤트)만 채택.
    // 계열 내부 <c:marker>는 val 속성이 없는 래퍼(symbol/size)라 자연 배제되고,
    // cur_series 게이트가 이중 방어. scatter는 scatterStyle이 담당하므로 Line 한정.
    if st.cur_plot_type == Some(OoxmlChartType::Line) && st.cur_series.is_none() {
        if let Some(val) = attr_val(e, "val") {
            chart.line_markers = matches!(val.as_str(), "1" | "true");
        }
    }
}
```

`<c:marker val="1"/>`는 Empty 이벤트로 `handle_start`를 타므로 `handle_end` 변경 불필요.

(d) 파서 단위 테스트 (기존 `bar_xml_with_grouping` 패턴의 line 변형 헬퍼 추가):

- `test_parse_grouping_line_ignored`(line 710) **반전** → `test_parse_line_grouping_stacked`:
  line stacked → `line_grouping == Stacked` **이면서** `grouping == Clustered` 불변.
- `test_parse_line_grouping_percent_stacked` — percentStacked 매핑.
- `test_parse_combo_grouping_no_cross_contamination` — barChart(stacked) + lineChart(standard)
  공존 XML → `grouping == Stacked` && `line_grouping == Clustered` (역방향도 단언).
- `test_parse_line_marker_flag` — plot 레벨 `<c:marker val="1"/>` → true / `val="0"` ·
  부재 → false.
- `test_parse_series_marker_ignored` — 계열 내부 `<c:marker><c:size val="7"/></c:marker>`
  → `line_markers == false` 유지.

**완료 기준**: `cargo test ooxml_chart` 통과(신규 5건 + 기존 전체). 렌더러 무변경 —
행동 변화 0. 단계별 보고서 `mydocs/working/task_m100_2129_stage1.md` 작성 + 커밋.

## 2단계 — render_line 누적/백프로 기하 + 축

**대상**: `src/ooxml_chart/renderer.rs` (`render_line`, line 575-634)

(a) 함수 서두에 flags + 축 분기 (render_bars line 402-449 미러):

```rust
let stacked = matches!(
    chart.line_grouping,
    BarGrouping::Stacked | BarGrouping::PercentStacked
);
let percent = chart.line_grouping == BarGrouping::PercentStacked;

let (vmin, vmax, vstep) = if percent {
    (0.0, 100.0, 20.0)                       // 정답지: 0%~100% step 20%
} else if stacked {
    let cat_count = max_len;                  // 라인의 카테고리 수 = 최장 시리즈 길이
    let max_sum = (0..cat_count)
        .map(|ci| category_positive_sum(chart, ci))
        .fold(0.0_f64, f64::max);
    nice_axis(0.0, max_sum.max(1.0), VERTICAL_AXIS_TICKS)  // 정답지: 12.3 → 0~15 step 5
} else {
    value_range(chart, VERTICAL_AXIS_TICKS)   // 현행 (개별값)
};
```

주의: `max_len` 계산(현행 line 577-582)을 축 분기보다 앞으로 이동 (`cat_count`로 사용).
`render_value_grid` 호출(line 591-605)의 `percent` 인자(뒤에서 2번째 bool)에 `percent` 전달.

(b) 시리즈 루프를 값공간 누적으로 재작성:

```rust
let step = pw / (max_len - 1).max(1) as f64;
let mut cum = vec![0.0_f64; max_len];        // 카테고리별 누적값 (값공간)
for (si, ser) in chart.series.iter().enumerate() {
    let color = series_color(ser, si);
    let mut points: Vec<(f64, f64)> = Vec::with_capacity(ser.values.len());
    for (i, &v) in ser.values.iter().enumerate() {
        let val = if stacked {
            cum[i] += v.max(0.0);             // 음수 clamp — render_bars와 동일 정책
            if percent {
                let sum = category_positive_sum(chart, i);
                if sum > 0.0 { cum[i] / sum * 100.0 } else { 0.0 }
            } else {
                cum[i]
            }
        } else {
            v
        };
        let t = if vmax > vmin { (val - vmin) / (vmax - vmin) } else { 0.0 };
        points.push((px + step * i as f64, py + ph - ph * t));
    }
    svg.push_str(&format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>\n",
        polyline_path(&points),
        color
    ));
}
```

기존 인라인 path 문자열 조립을 `points` 수집 + `polyline_path`(line 712) 재사용으로 정리
(3단계 마커가 같은 좌표를 씀). 비누적(`stacked == false`) 경로의 출력은 현행과 동일해야 함.

(c) 렌더러 기하 단위 테스트 — `line_chart(grouping)` 헬퍼 신설(`bars_chart` line 1295 미러,
3계열×4카테고리·카테고리 최대 합 12.3: 계열1 `[4.3,2.5,3.5,4.5]` / 계열2 `[2.4,4.4,1.8,2.8]` /
계열3 `[2.0,2.0,3.0,5.0]`):

- `test_line_stacked_axis_from_category_sum` — 라벨 `>15<` 존재, `>13<` 부재
  (개별 최대 기반이면 나올 값), `>14<` 부재.
- `test_line_stacked_series_order` — 계열2 path의 첫 M점 y < 계열1 (위에 쌓임).
- `test_line_percent_axis_labels` — `100%`·`20%` 존재.
- `test_line_percent_top_series_flat` — 최상위 계열 path의 y 좌표 전부 동일(플롯 상단).
- `test_line_percent_zero_sum_category_no_nan` — 합 0 카테고리 포함 데이터 → SVG에
  `NaN` 부재.
- `test_line_clustered_unchanged` — Clustered(기본)에서 계열별 y가 개별값 기반(현행 유지).

**완료 기준**: `cargo test ooxml_chart` 통과(막대/scatter/콤보 회귀 포함).
단계별 보고서 `task_m100_2129_stage2.md` 작성 + 커밋.

## 3단계 — 표식(마커) 렌더

**대상**: `src/ooxml_chart/renderer.rs`

(a) 헬퍼 신설 (`polyline_path` 부근):

```rust
/// 라인 차트 표식. 계열 인덱스별 한컴 기본 사이클 ◆■▲(+원 폴백) — 정답지 PDF 실측.
/// 크기 상수는 근사값, 시각판정에서 조정 여지. (C1d #2129)
fn push_line_marker(svg: &mut String, si: usize, cx: f64, cy: f64, color: &str) {
    let d = match si % 4 {
        0 => { // ◆ 다이아몬드
            let r = 3.5;
            format!("M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} Z",
                cx, cy - r, cx + r, cy, cx, cy + r, cx - r, cy)
        }
        1 => { // ■ 정사각형
            let h = 3.0;
            format!("M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} Z",
                cx - h, cy - h, cx + h, cy - h, cx + h, cy + h, cx - h, cy + h)
        }
        2 => { // ▲ 삼각형
            let r = 3.5;
            format!("M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} Z",
                cx, cy - r, cx + r, cy + r * 0.8, cx - r, cy + r * 0.8)
        }
        _ => { // 원 폴백 (계열 4+ — 코퍼스 밖, scatter 마커와 동일 반경)
            format!("M{:.2},{:.2} a3,3 0 1,0 6,0 a3,3 0 1,0 -6,0", cx - 3.0, cy)
        }
    };
    svg.push_str(&format!(
        "<path class=\"hwp-chart-marker\" d=\"{}\" fill=\"{}\" stroke=\"#ffffff\" stroke-width=\"1\"/>\n",
        d, color
    ));
}
```

(b) `render_line` 시리즈 루프 말미(path 출력 후)에 배선:

```rust
if chart.line_markers {
    for &(mx, my) in &points {
        push_line_marker(svg, si, mx, my, &color);
    }
}
```

(c) 마커 단위 테스트:

- `test_line_markers_rendered` — `line_markers = true` → `hwp-chart-marker` 출현 횟수
  == 계열 수 × 점 수 (3×4=12).
- `test_line_marker_shape_cycle` — 계열 1~3의 마커 `d` 문자열이 서로 다름(◆■▲ 사이클).
- `test_line_no_markers_by_default` — 기본값 → `hwp-chart-marker` 부재 (꺽은선형 무회귀).

**완료 기준**: `cargo test ooxml_chart` 통과. 단계별 보고서 `task_m100_2129_stage3.md`
작성 + 커밋.

## 4단계 — 통합 테스트 + 시각판정 산출물 + 최종 검증

**대상**: 신규 `tests/issue_2129_line_stacked.rs`

- 통합 테스트 (패턴: `tests/issue_1882_chart_style_gaps.rs`의 `render_page0_svg` +
  `for_both_exts` — `samples/chart/라인/` 5종 stem × hwp/hwpx = **10파일**):
  - 누적꺽은선형 → `>15<` 존재 + `>14<` 부재 + `hwp-chart-marker` 부재 +
    `"차트 (미지원)"` 부재.
  - 백프로기준누적꺽은선형 → `100%` 존재 + 마커 부재.
  - 표식이있는누적꺽은선형 → `hwp-chart-marker` 존재 + `>15<` 존재.
  - 표식이있는꺽은선형 → 마커 존재 + 개별값 축(`>6<` 존재, `>15<` 부재).
  - 꺽은선형 → 마커 부재 + `>6<` 존재 (무회귀 핀).
- 전체 게이트: `cargo test` 전체 + `cargo clippy --all-targets -- -D warnings` 무경고 +
  포맷은 수정 파일만.
- 시각판정 산출물: 라인 5종 hwpx+hwp `rhwp export-svg -o output/poc/chart_c1d/` (+PNG) →
  `pdf/chart/라인/{stem}-2022.pdf` 정답지 대조표를 stage4 보고서에 작성
  (누적 기하 / 축 라벨 / 마커 유무·형상 / 알려진 갭: 범례 순서 역전은 C2 기이관 명기).
- 단계별 보고서 `task_m100_2129_stage4.md` 작성 + 커밋 → **작업지시자 시각판정**.

**완료 기준**: 통합 10파일 가드 통과 + 전체 스위트·clippy 통과 + 시각판정 자료 산출.

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `src/ooxml_chart/mod.rs` | `line_grouping`·`line_markers` 필드 + doc 갱신 (1단계) |
| `src/ooxml_chart/parser.rs` | `grouping` arm 분기 + `marker` arm + 단위 5건 (1단계) |
| `src/ooxml_chart/renderer.rs` | `render_line` 누적/백프로/마커 + `push_line_marker` + 단위 9건 (2·3단계) |
| `tests/issue_2129_line_stacked.rs` | 10파일 통합 가드 (4단계) |
| `mydocs/working/task_m100_2129_stage{1..4}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_2129_report.md` | 최종 보고서 |
| `output/poc/chart_c1d/` | 시각판정 산출물 (gitignore) |

## 위험 / 주의

- **콤보 marker 플래그**: 콤보 차트의 lineChart에 `<c:marker val="1"/>`가 있으면
  `line_markers`가 true로 설정되지만 `render_combo`는 이 필드를 읽지 않음 — 무해.
  주석으로 명기.
- **percent 합 0 카테고리**: `cum/0` NaN 방지 가드(합≤0→0.0). 막대의 `denom=1.0` 가드와
  동등한 출력.
- **음수값**: `.max(0.0)` clamp — Excel 실음수 누적과 다르나 코퍼스 무해당. 문서화만.
- **길이 다른 시리즈**: `cum`은 `max_len` 크기, 각 시리즈는 자기 점만 그림.
  `category_positive_sum`은 `unwrap_or(0.0)`로 이미 방어.
- **비누적 경로 무회귀**: 2단계의 path 조립 정리(`polyline_path` 재사용) 후에도 Clustered
  출력이 바이트 동일해야 함(포맷 `{:.2}` 동일). `test_line_clustered_unchanged`로 핀.
- **마커 상수는 PDF 근사** — 4단계 대조표에서 크기/형상 확인 후 필요 시 조정.
- 기능 변경만 포함 — 포맷(`cargo fmt --all`) 전체 적용 금지(수정 파일 범위만).
