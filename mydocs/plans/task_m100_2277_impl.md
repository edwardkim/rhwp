# Task M100 #2277 구현계획서 — C2a stock HLC 렌더 + 2D fidelity 정합

- 이슈: #2277
- 브랜치: `local/task2277`
- 작성일: 2026-07-15
- 수행계획서: `mydocs/plans/task_m100_2277.md`

## 구현 개요

5단계. ① 마커 경로 생성을 `marker_path` 헬퍼로 추출하며 사이클 4번째를 원→×로 교체하고
scatter 마커를 사이클화한다(이후 단계의 공용 인프라). ② 파서에 `stockChart` 계열 arm과
`marker_symbol`을 추가하고 `render_stock`(고저선/캔들/종가 마커/전용 +1 step 축)을 신설한다.
③ 28 PDF 전수 실측표로 범례 순서 규칙을 확정한 뒤 `legend_items`에 단일 결정 함수로
역순을 적용하고, 묶은가로의 슬롯 내 배치 반전을 같은 커밋에 묶는다. ④ 범례 스와치를
`SwatchKind`로 일반화해 글리프(—◆—)/빈 스와치를 지원한다. ⑤ 특이케이스 0.5축 게이트와
`line3DChart` 방어 라우팅을 넣고 코퍼스 28종 시각판정 산출물을 만든다.

축·격자·카테고리 라벨은 전부 기존 헬퍼(`nice_axis_no_headroom`·`render_value_grid`·
`render_category_labels`) 재사용 — 새 축 기계장치 없음. `render_combo`·`render_pie` 무변경.

---

## 1단계 — 마커 인프라 (× 교체) + scatter 마커 사이클

**대상**: `src/ooxml_chart/renderer.rs` (`push_line_marker` 749-803, `render_scatter` 735-742)

(a) `push_line_marker`에서 순수 경로 생성을 추출:

```rust
/// 마커 경로. 계열 인덱스 사이클 ◆■▲× (한컴 기본 — 정답지 실측: 라인 ◆■▲(C1d),
/// OHLC 종가 ×, scatter ◆■). 반환 (d, stroke 기반 여부) — ×는 채움 없는 열린 경로라
/// stroke=계열색으로 그린다. r=명목 반경 (■/×는 r-0.5 하프폭 — 기존 3.5/3.0 비율 유지).
fn marker_path(si: usize, cx: f64, cy: f64, r: f64) -> (String, bool) {
    let h = r - 0.5;
    match si % 4 {
        0 => (/* ◆ 기존 식, r */, false),
        1 => (/* ■ 기존 식, h */, false),
        2 => (/* ▲ 기존 식, r */, false),
        _ => ( // × — C2a: 원 폴백 교체 (OHLC 종가 정답지 실측)
            format!("M{:.2},{:.2} L{:.2},{:.2} M{:.2},{:.2} L{:.2},{:.2}",
                cx - h, cy - h, cx + h, cy + h, cx + h, cy - h, cx - h, cy + h),
            true,
        ),
    }
}

/// 마커 1개 방출. stroke 기반(×)은 fill=none/stroke=색, 채움형은 fill=색/stroke=흰색.
fn push_marker(svg: &mut String, class: &str, si: usize, cx: f64, cy: f64, r: f64, color: &str)
```

- `push_line_marker`는 `push_marker(svg, "hwp-chart-marker", si, cx, cy, 3.5, color)` 위임으로
  축소 (기존 ◆■▲ 출력 바이트 동일 — ◆▲ r=3.5, ■ h=3.0 유지).
- `render_scatter`(735-742): `<circle r=3>` → `push_marker(..., si, xp, yp, 4.5, &color)` —
  scatter 마커는 실측상 라인보다 큼(r≈4.5 근사, 시각판정에서 조정).

(b) 테스트:

- 렌더러 단위: `test_marker_cycle_fourth_is_x` — si=3의 d에 `M`+`L` 2쌍(열린 경로)·
  `stroke=`계열색 / `test_scatter_markers_use_cycle` — scatter 픽스처(2계열×3점)에서
  `hwp-chart-marker` 6개·`<circle` 0개·계열1/2 d 상이.
- 기존 반전: 렌더러 단위 중 원 폴백을 단언하는 케이스가 있으면 × 단언으로 교체.
  `tests/issue_1431_scatter.rs`가 circle을 핀하고 있으면 `hwp-chart-marker` 카운트로 갱신
  (RED 선행 확인). issue_2129 마커 12개는 3계열이라 무접점 — 실행으로 확인.

**완료 기준**: `cargo test` 전체 통과. 단계 보고서 `task_m100_2277_stage1.md` + 커밋.

## 2단계 — stock 2종 (파서 + render_stock)

**대상**: `src/ooxml_chart/{mod,parser,renderer}.rs`, 신규 `tests/issue_2277_stock.rs`

(a) `mod.rs`:

```rust
pub enum OoxmlChartType { …, Stock, … }   // label() → "주식형"
pub struct OoxmlChart {
    …
    /// stock plot의 <c:hiLowLines/> 존재 (고저선). HLC/OHLC 공통. (C2a #2277)
    pub has_hi_low_lines: bool,
    /// stock plot의 <c:upDownBars> 존재 (시가↔종가 캔들). OHLC만. (C2a #2277)
    pub has_up_down_bars: bool,
    /// <c:upDownBars><c:gapWidth val> (캔들 폭 = cat_span/(1+gap/100)). 기본 150.
    pub up_down_gap_width: Option<f64>,
}
pub struct OoxmlSeries {
    …
    /// 계열 내부 <c:marker> 상태 — stock 종가 판별용. (C2a #2277)
    /// NotSpecified=요소 없음 / None=<c:symbol val="none"> / Auto=래퍼만(symbol 부재)
    /// / Named=명시 심볼.
    pub marker_symbol: SeriesMarker,
}
pub enum SeriesMarker { #[default] NotSpecified, None, Auto, Named(String) }
```

- 모듈 doc(mod.rs:19-20) 범위에서 stock 제거, 지원 목록에 stockChart 추가.

(b) `parser.rs`:

- `handle_start`: `b"stockChart"` arm(기존 plot arm 미러 — `chart_type=Stock`,
  `cur_plot_type=Some(Stock)`, ax_ids clear, series_start). `b"hiLowLines"`/`b"upDownBars"` →
  플래그 set. `b"gapWidth"` → `cur_plot_type==Some(Stock)` 게이트로 `up_down_gap_width` 저장
  (barChart의 동명 요소와 격리).
- `b"marker"` arm(261-271) 확장 — 기존 plot 레벨 게이트는 그대로 두고 분기 추가:

```rust
if st.cur_series.is_some() {
    // 계열 내부 <c:marker> 래퍼 — symbol 자식이 없으면 auto (stock 종가 실측).
    if let Some(ser) = st.cur_series.as_mut() {
        if ser.marker_symbol == SeriesMarker::NotSpecified {
            ser.marker_symbol = SeriesMarker::Auto;
        }
    }
} else if st.cur_plot_type == Some(OoxmlChartType::Line) { /* 기존 line_markers */ }
```

- `b"symbol"` arm 신설: `cur_series` 게이트, `val=="none"`→`None`, 그 외→`Named(val)`.
- `handle_end` plot arm(448-449) 리스트에 `b"stockChart"` 추가.

(c) `renderer.rs` — 라우팅 match(189-201)에 `Stock => render_stock(...)` + 신설:

```rust
/// stock (주식형). 계열 역할 = XML 순서 규약: 3계열=고/저/종, 4계열=시/고/저/종
/// (코퍼스 c:order 실측). 그 외 계열 수는 render_line 폴백.
fn render_stock(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64) {
    let (hi, lo, close, open) = match chart.series.len() {
        3 => (0, 1, 2, None),
        4 => (1, 2, 3, Some(0)),
        _ => return render_line(svg, chart, px, py, pw, ph),
    };
    // 축: 전용 무조건 +1 step 헤드룸 (정답지: max 59 → 0~80 step 20.
    // nice_axis(경계에서만 +1)로는 0~60 — 3D 누적세로의 "+1 step" 패턴과 동형)
    let (mn, mx) = raw_value_bounds(chart.series.iter());
    let (vmin, mx0, vstep) = nice_axis_no_headroom(mn.min(0.0), mx, VERTICAL_AXIS_TICKS);
    let vmax = mx0 + vstep;
    // 배경 rect + render_value_grid(…, percent=false) — render_line과 동일 배선
    // 카테고리 루프: x = px + cat_span*(ci+0.5)
    //   고저선: <line class="hwp-stock-hilow" stroke="#000" stroke-width="1"> y(hi)→y(lo)
    //   캔들(has_up_down_bars && open 존재): rect y(open)↔y(close),
    //     w = cat_span/(1.0 + gap/100.0) (gap 기본 150 → 슬롯 40%),
    //     close < open → fill 진회색(#404040 근사, 시각판정 확정) /
    //     close >= open → fill #fff + stroke #000. class="hwp-stock-candle"
    //   종가 마커: chart.series[close].marker_symbol != SeriesMarker::None인 경우
    //     push_marker(svg, "hwp-chart-marker", close, x, y(close), 3.5, &series_color(...))
    //     — si=2→▲ 회색(팔레트3), si=3→× 노랑(팔레트4) 자동 정합
    // render_category_labels(..., false)
}
```

- stock 계열은 `<a:ln><a:noFill/>`이라 파서 color=None → 팔레트 폴백이 ▲회색/×노랑을
  자동 결정 (신규 색 상수 불필요).

(d) 테스트:

- 파서 단위: stockChart 3계열 XML → `chart_type=Stock`+`has_hi_low_lines`+시/고/저
  `marker_symbol==None`·종가 `Auto` / OHLC XML → `has_up_down_bars`+gapWidth /
  기존 line plot 레벨 marker 무간섭(`test_parse_line_marker_flag` 무회귀).
- 렌더러 단위: `stock_chart(n)` 픽스처 신설(`bars_chart` 미러, 고 55~59) —
  축 `>80<` 존재·`>60<` 라벨만으로 끝나지 않음 / hilow 카운트=카테고리 수 /
  3계열 캔들 0개·4계열 4개 / 종가 마커만 출현(고/저/시 무마커).
- 통합 `tests/issue_2277_stock.rs` (`render_page0_svg` + stem×{hwpx,hwp} 4파일):
  placeholder `"차트 (미지원)"` 부재 + `hwp-ooxml-chart` / `>80<` / `hwp-stock-hilow` 4개 /
  OHLC `hwp-stock-candle` 4개·HLC 0개 / `hwp-chart-marker` 4개.

**완료 기준**: 코퍼스 placeholder 0건 도달(28/28 렌더). `cargo test` 전체 통과.
단계 보고서 `task_m100_2277_stage2.md` + 커밋.

## 3단계 — 범례 순서 규칙 + 묶은가로 슬롯 반전

**대상**: `src/ooxml_chart/renderer.rs` (`legend_items` 1147-1184, `render_bars` 539-546),
신규 `tests/issue_2277_legend_order.rs`

(a) **선행: 28 PDF 전수 실측표** — `pdf/chart/**/*-2022.pdf` 전체의 범례 순서(+플롯 내
계열 배치)를 판독해 stage3 보고서에 표로 수록. 특히 백프로 3종·3D 4종의 역순 귀속 확정
(예측: 세로 백프로=역순, 가로 백프로=정순, 3D는 2D와 동일 규칙 — 빗나가면 아래 결정
함수의 규칙 표만 수정).

(b) 단일 결정 함수 + `legend_items` 적용:

```rust
/// 범례 항목 역순 여부 — 정답지 28종 전수 실측 규칙 (stage3 보고서 표).
/// 계열이 화면상 세로로 배열되는 차트는 우측 세로 범례를 시각 상→하 순서와
/// 일치시키기 위해 역순: 세로 값축 누적(막대·라인) / 가로막대 묶음.
/// pie(카테고리 범례)/scatter/stock/콤보/이중축 = 정순 고정.
fn legend_order_reversed(chart: &OoxmlChart) -> bool {
    if chart.is_combo() || chart.has_secondary_axis {
        return false;
    }
    match chart.chart_type {
        OoxmlChartType::Column => matches!(chart.grouping, Stacked | PercentStacked),
        OoxmlChartType::Bar => chart.grouping == BarGrouping::Clustered,
        OoxmlChartType::Line => matches!(chart.line_grouping, Stacked | PercentStacked),
        _ => false,
    }
}
```

`legend_items` 비-pie 분기 말미에 `if legend_order_reversed(chart) { items.reverse(); }`
(색 매핑 후 반전 — palette(si)는 원 인덱스 유지).

(c) 묶은가로 슬롯 내 배치 반전 — `render_bars` 비누적 horizontal(541):
`bar_w * si` → `bar_w * (ser_count - 1 - si) as f64` (정답지: 계열1이 슬롯 맨 아래).
누적·세로 경로 무변경.

(d) 테스트:

- 렌더러 단위: 결정 함수 진리표(Column stacked/percent=true·clustered=false, Bar
  clustered=true·stacked/percent=false, Line stacked=true, Pie/Scatter/Stock/콤보=false) +
  `test_hbar_clustered_slot_order` — 묶은가로 픽스처에서 계열1 rect y > 계열3 rect y.
- 통합 `tests/issue_2277_legend_order.rs` (역순 대표: 누적세로·묶은가로·누적라인·백프로세로 /
  정순 대표: 묶은세로·누적가로·원형·분산형, × hwp/hwpx): 범례 `<g class="hwp-chart-legend">`
  조각에서 `>계열 3<`(또는 해당 라벨)의 y가 `>계열 1<`보다 작음/큼 비교.
- 기존 무회귀: `issue_1882::chart_legend_on_right`(묶은세로 — 정순 유지·x좌표만 검사) 통과.

**완료 기준**: 전수 실측표 + 규칙 구현 일치. `cargo test` 전체 통과.
단계 보고서 `task_m100_2277_stage3.md`(실측표 포함) + 커밋.

## 4단계 — 범례 스와치 글리프 (SwatchKind)

**대상**: `src/ooxml_chart/renderer.rs` (`legend_items`, `push_legend_swatch` 1187-1201)

(a) `legend_items` 반환 `(String, u32, OoxmlChartType)` → `(String, u32, SwatchKind)`:

```rust
/// 범례 스와치 형태 (정답지 실측 — C2a #2277)
enum SwatchKind {
    Square,          // 막대/원형: 10×10 색 사각형 (현행 유지 — issue_1882 필터 보호)
    LineOnly,        // 무표식 라인: 14px 색 선 (현행 유지)
    LineGlyph(usize),// 표식 라인: 선 + 중앙 마커 글리프 (—◆—)
    GlyphOnly(usize),// scatter 표식만·stock 종가: 마커 글리프만
    Blank,           // stock 시/고/저: 스와치 없음 (텍스트 오프셋은 유지)
}
```

배정: Pie→Square(카테고리) / Column·Bar→Square / Line→`line_markers ? LineGlyph(si) :
LineOnly` / Scatter→flags로 (선∧표식)=LineGlyph·표식만=GlyphOnly·선만=LineOnly /
Stock→`marker_symbol==None ? Blank : GlyphOnly(si)` / 콤보(시리즈별)→Line=LineOnly·
그 외 Square (현행 보존).

(b) `push_legend_swatch(svg, ix, cy, color, kind)` 재작성 — Square/LineOnly 출력 바이트
현행 동일. LineGlyph=기존 14px 선 + `push_marker(svg, "hwp-legend-glyph", si, ix+7.0, cy,
3.0, …)`, GlyphOnly=글리프만, Blank=무출력. 호출부 2곳(render_legend/render_legend_right)과
`max_chars` 클로저 패턴 갱신. 텍스트 x=+18 불변.

(c) 테스트:

- 렌더러 단위: 표식 라인 픽스처 → 범례에 `hwp-legend-glyph` 계열 수만큼+선 유지 /
  scatter 표식만 → 글리프만(선 부재) / stock → Blank(시/고/저 스와치 rect·line 부재) +
  종가 글리프 1개 / 막대 → `width="10" height="10"` 불변.
- 통합: `issue_2277_stock.rs`에 범례 단언 보강(시/고/저 스와치 부재 — 1882의 fragment
  split 기법). issue_2129 무마커 stems가 `hwp-chart-marker` 부재 단언 유지 — 범례 글리프가
  별도 클래스라 무오염(무회귀 실행 확인).

**완료 기준**: `cargo test` 전체 통과. 단계 보고서 `task_m100_2277_stage4.md` + 커밋.

## 5단계 — 0.5축 게이트 + line3DChart 라우팅 + 종합 회귀·시각판정

**대상**: `src/ooxml_chart/{parser,renderer}.rs`, `output/poc/chart_c2a/`

(a) 0.5축 — `render_bars` 축 분기의 비누적·비3D else(447-449):

```rust
} else {
    let (mn, mx, st) = value_range(chart, ticks);
    // 특이케이스(C1c v2 실측): 가로막대 1카테고리 미니차트는 축 범위 유지·step 절반
    // (4.3 → 0~5 step 0.5, 라벨 11개). 단일 샘플 근거 — 가로·1카테고리로 좁게 게이트
    // (코퍼스 나머지 27종 전부 4카테고리 → 회귀 반경 0). 세로 1카테고리는 미실측.
    if horizontal && cat_count == 1 { (mn, mx, st / 2.0) } else { (mn, mx, st) }
};
```

주의: `cat_count` 계산(408-415)이 축 분기(429-449)보다 뒤에 있으면 앞으로 이동.
소수 라벨은 `render_value_grid`의 비정수 step 가드가 처리(scatter 0.5와 동일 경로).

(b) line3DChart — `handle_start`에 arm 추가(bar3D 미러, lineChart의 Unknown 가드 유지):

```rust
b"line3DChart" => {
    // 코퍼스 27종에 없음 — 방어적 라우팅(placeholder 방지, C1a bar3D/pie3D 선례).
    // 입체 표현은 C2b, 여기서는 2D 라인 근사 + is_3d(축 정책)만.
    if chart.chart_type == OoxmlChartType::Unknown { chart.chart_type = OoxmlChartType::Line; }
    chart.is_3d = true;
    st.cur_plot_type = Some(OoxmlChartType::Line);
    st.cur_plot_ax_ids.clear();
    st.cur_plot_series_start = chart.series.len();
}
```

handle_end plot arm 리스트에도 추가. 파서 단위: 인라인 XML → Line+is_3d+series 파싱.

(c) 종합 회귀·시각판정:

- `cargo test` 전수 + `cargo clippy --all-targets -- -D warnings` + fmt 수정 파일만.
- 특이케이스 통합 단언(`>0.5<`·`>4.5<` 라벨)을 issue_2277 테스트에 추가
  (issue_1882의 특이케이스 단언과 충돌 여부 선확인 — step 1 라벨을 핀하고 있으면 갱신).
- 코퍼스 28종 hwp+hwpx `export-svg -o output/poc/chart_c2a/` → 정답지 대조표
  (stock 2종/scatter 5종/표식 라인 2종/범례 역순 4종/특이케이스 중점, 나머지 무회귀 스윕)
  → stage5 보고서 → **작업지시자 시각판정**.

**완료 기준**: placeholder 0건 + 전체 게이트 통과 + 시각판정 자료 산출.
단계 보고서 `task_m100_2277_stage5.md` + 커밋.

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `src/ooxml_chart/mod.rs` | `Stock` variant, stock 플래그 3개, `SeriesMarker`+필드, doc (2단계) |
| `src/ooxml_chart/parser.rs` | stockChart/hiLowLines/upDownBars/gapWidth/symbol/line3DChart arm, marker 게이트 확장 (2·5단계) |
| `src/ooxml_chart/renderer.rs` | `marker_path`/`push_marker` 추출+×, scatter 마커, `render_stock`, `legend_order_reversed`+반전, 슬롯 반전, `SwatchKind`, 0.5축 게이트 (1~5단계) |
| `tests/issue_2277_stock.rs` | stock 통합 가드 (2·4단계) |
| `tests/issue_2277_legend_order.rs` | 범례 순서 통합 가드 (3단계) |
| `tests/issue_1431_scatter.rs` | 마커 사이클 단언 보강/핀 갱신 (1단계, 필요 시) |
| `mydocs/working/task_m100_2277_stage{1..5}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_2277_report.md` | 최종 보고서 |
| `output/poc/chart_c2a/` | 시각판정 산출물 (gitignore) |

## 위험 / 주의

- **× 교체의 숨은 핀**: 기존 단위/통합 테스트가 원 폴백(`a3,3`)이나 scatter `<circle`을
  핀하고 있으면 1단계 RED에서 드러남 — 단언을 사이클 기준으로 갱신(행동 변화가 의도임을
  보고서에 명기).
- **범례 반전 ↔ 플롯 불일치 창**: 3단계에서 범례 반전과 슬롯 반전을 반드시 같은 커밋에.
- **규칙 예측 빗나감**: 백프로·3D 귀속이 실측과 다르면 `legend_order_reversed` match 표만
  수정 (호출부 불변 — 격리 설계).
- **stock 폴백**: 계열 수 3/4 외(예: 손상 문서)는 `render_line` 폴백 — placeholder 재발
  방지 우선.
- **캔들 동률**: `close == open` → 상승(흰 박스) 처리 고정 + 주석 (미실측, 근사 명기).
- **하락 채움색**: #404040 근사 시작, stage5 대조표에서 픽셀 실측로 확정.
- **Blank 스와치 정렬**: 텍스트 x 오프셋(+18) 유지 — 정답지에서 시/고/저/종 라벨 좌정렬.
- **gapWidth 격리**: barChart에도 `c:gapWidth`가 있으므로 Stock plot 게이트 필수.
- 기능 변경만 — `cargo fmt --all` 금지(수정 파일 범위만), 저장 경로 무접점.
