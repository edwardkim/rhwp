# Task M100 #2278 구현계획서 — C2b 3D 입체·ofPie 보조플롯 렌더

- 이슈: #2278
- 브랜치: `local/task2278`
- 작성일: 2026-07-16
- 수행계획서: `mydocs/plans/task_m100_2278.md`

## 구현 개요

3단계. ① 정답지 4종 정밀 판독 후 `shade`/`push_bar_3d` 헬퍼를 신설하고 `render_bars`의
rect 방출 4개소를 `is_3d` 분기로 감싸 3D 막대 4조합(묶음/누적 × 세로/가로)에 사선 압출을
입힌다 — 축 계산 무접촉(#1882). ② `render_pie_3d`를 신설해 3차원원형을 타원 슬라이스 +
하반부 측벽 밴드로 렌더한다(`render_pie` 무변경). ③ 정답지에서 팔레트 5번을 픽셀 실측해
교정하고, `OfPieInfo` 모델 + 파서 arm 4개 + `render_of_pie`(주 원형 + 결합 슬라이스 +
보조 플롯 + serLines)를 신설한다 — `chart_type=Pie` 유지(#1453).

축·격자·라벨·범례는 전부 기존 헬퍼(`nice_axis*`·`render_value_grid`·
`render_category_labels`·`legend_items`·`push_legend_swatch`) 재사용 — 새 축·범례
기계장치 없음. 2D 경로(`render_pie`·기존 rect 방출·`render_combo`)는 출력 바이트 불변.

---

## 1단계 — 3D 막대 압출 (`shade` + `push_bar_3d`)

**대상**: `src/ooxml_chart/renderer.rs` (`color_hex` 뒤 헬퍼 신설, `render_bars`
rect 방출 4개소 :525-528/:533-536/:560-563/:569-572), 신규
`tests/issue_2278_chart_3d_ofpie.rs`

(a) **정밀 판독** — `pdf/chart/{세로막대형,가로막대형}/3차원{묶은,누적}*-2022.pdf` 4장:
압출 각도(45° 가정)·top/side 음영 정도·누적의 세그먼트별 side 노출(예상: 각자 색)·
top face의 플롯 상단 침범 여부를 확정, stage1 보고서에 실측 기록. 계수는 근사 시작 +
시각판정 보정.

(b) `shade` 헬퍼 + 음영 상수:

```rust
/// RGB 음영 — factor>0 은 흰색 방향 lighten, factor<0 은 검정 방향 darken.
/// 상위 바이트(0xFF------)는 보존. (C2b #2278)
fn shade(rgb: u32, factor: f64) -> u32 {
    let f = factor.clamp(-1.0, 1.0);
    let ch = |c: u32| -> u32 {
        let c = c as f64;
        let v = if f >= 0.0 { c + (255.0 - c) * f } else { c * (1.0 + f) };
        v.round().clamp(0.0, 255.0) as u32
    };
    (rgb & 0xFF00_0000) | (ch((rgb >> 16) & 0xFF) << 16) | (ch((rgb >> 8) & 0xFF) << 8) | ch(rgb & 0xFF)
}

/// 3D 막대 면 음영 계수 (정답지 근사 — 시각판정 보정)
const BAR3D_TOP_SHADE: f64 = 0.25;
const BAR3D_SIDE_SHADE: f64 = -0.25;
```

(c) `push_bar_3d` — 우상 45° 압출(+d, −d)에서 보이는 면은 세로/가로 막대 공통
"top 평행사변형(밝게) + right 평행사변형(어둡게) + front rect(원색)" → 방향 플래그 없이
단일 헬퍼로 4조합 커버:

```rust
/// 3D 막대 1개(또는 누적 세그먼트 1개) — 사선 압출 3면. 압출 벡터 = (+depth, -depth).
/// w/h ≤ 0(0값 세그먼트)이면 무방출 — 누적에서 이웃 캡 재도색 방지. (C2b #2278)
fn push_bar_3d(svg: &mut String, x: f64, y: f64, w: f64, h: f64, depth: f64, color: u32) {
    if w <= 0.0 || h <= 0.0 { return; }
    let d = depth;
    // top (lighter): (x,y) (x+d,y-d) (x+w+d,y-d) (x+w,y)
    svg.push_str(&format!(
        "<polygon class=\"hwp-bar3d-top\" points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"{}\"/>\n",
        x, y, x + d, y - d, x + w + d, y - d, x + w, y,
        color_hex(shade(color, BAR3D_TOP_SHADE))));
    // right (darker): (x+w,y) (x+w+d,y-d) (x+w+d,y+h-d) (x+w,y+h)
    svg.push_str(&format!(
        "<polygon class=\"hwp-bar3d-side\" points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"{}\"/>\n",
        x + w, y, x + w + d, y - d, x + w + d, y + h - d, x + w, y + h,
        color_hex(shade(color, BAR3D_SIDE_SHADE))));
    // front (원색, 무클래스 — 2D와 형태 통일)
    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
        x, y, w, h, color_hex(color)));
}
```

**누적 은면 처리(분기 불필요)**: 각 세그먼트가 3면을 모두 그리되 기존 페인트 순서가
자연 은면 제거를 수행한다 — 세로 누적(아래→위)은 세그먼트 i의 top 캡을 i+1의
front+side가 정확히 덮어 최상단 캡만 노출·side는 각자 색, 가로 누적(왼→오른쪽)은
대칭으로 최우측 end 캡만 노출·top 길이면은 각자 색. 0값 세그먼트는 조기 반환이
캡 재도색을 차단.

**통합 지점** — stacked/clustered 분기 직전에 depth 클로저 1개, rect 방출 4개소를
`if chart.is_3d { push_bar_3d(...) } else { 기존 format! }`로 분기 (축/grid/라벨
블록 :427-462 무접촉):

```rust
// 3D 압출 깊이 — 막대 두께 기반 고정 근사 (c:view3D/gapDepth 미파싱, 이슈 확정식)
let depth3d = |thickness: f64| (thickness * 0.45).clamp(3.0, 9.0);
```

| 방출 지점 | 3D 분기 호출 |
|---|---|
| stacked-가로 :525-528 | `push_bar_3d(svg, base + acc, cell, seg.max(0.0), bar_span_total, depth3d(bar_span_total), rgb)` |
| stacked-세로 :533-536 | `push_bar_3d(svg, cell, by, bar_span_total, seg.max(0.0), depth3d(bar_span_total), rgb)` |
| clustered-가로 :560-563 | `push_bar_3d(svg, px, cy, bw.max(0.0), bar_w * 0.95, depth3d(bar_w), rgb)` |
| clustered-세로 :569-572 | `push_bar_3d(svg, cx, by, bar_w * 0.95, bh.max(0.0), depth3d(bar_w), rgb)` |

색: 3D 분기 안에서만 `let rgb = ser.color.unwrap_or_else(|| palette(si));` (u32).
기존 `let color = series_color(ser, si)` (String)는 2D 분기 전용으로 유지 —
2D 출력 바이트 불변.

(d) 테스트:

- 렌더러 단위: `test_shade_lighten_darken`(0x6183D7 ±0.25 채널 수치·±1.0 클램프·
  상위 바이트 보존) / `test_bar3d_clustered_faces_both_orientations`(`bars_chart` 픽스처
  + is_3d, Column·Bar 각 `hwp-bar3d-top`==`hwp-bar3d-side`==cat×ser, is_3d=false→0) /
  `test_bar3d_stacked_all_segments_extrude`(누적 — 면 카운트 + 방출 순서가 세그먼트
  순서와 일치) / `test_bar3d_zero_segment_skipped`(값 0 계열 → 면 카운트 감소) /
  `test_bar3d_depth_clamp`(좁은/넓은 플롯에서 top 폴리곤 x-delta 3.0/9.0).
- 기존 앵커 무수정 통과 확인: `test_axis_3d_clustered_no_headroom`/
  `test_axis_3d_stacked_vertical_extra_headroom`/`test_legend_order_3d_same_as_2d`.
- 통합 `tests/issue_2278_chart_3d_ofpie.rs` 신설(`render_page0_svg` 로컬 헬퍼 —
  기존 테스트 패턴 복제): 3D 막대 4종 × {hwpx,hwp} — `hwp-bar3d-top` 카운트 ==
  `hwp-bar3d-side` > 0 + #1882 축 라벨 문구 재확인(묶은 `>5<` 유·`>6<` 무, 누적세로
  `>20<`, 누적가로 `>14<`). 페이지 전역에 도형/WMF polygon이 있으므로 계수는 반드시
  클래스 기준.
- 기존 통합 재실행: issue_1453 / issue_1882 / issue_2277 3종.

**완료 기준**: `cargo test` 전체 통과 + clippy 무경고. `export-svg` 3D 막대 4종 →
`output/poc/chart_c2b/`. 단계 보고서 `task_m100_2278_stage1.md`(실측 기록 포함) + 커밋.

## 2단계 — 3D 원형 (타원 + 하반부 측벽)

**대상**: `src/ooxml_chart/renderer.rs` (`render_pie` 아래 `render_pie_3d` 신설,
`render_chart_svg` Pie 분기 :169-185)

(a) **정밀 판독** — `pdf/chart/원형/3차원원형-2022.pdf`: ry/rx 비율(≈0.55)·측벽
높이·측벽 음영·벽의 흰 테두리 유무·중심 세로 위치 확정, stage2 보고서에 기록.

(b) `render_pie_3d`:

```rust
/// 3D 원형 — 타원(top) 슬라이스 + 하반부 측벽 밴드. 벽 전체를 먼저, top을 나중에
/// 그린다. rotX/perspective 미파싱 — ry = rx*0.55 고정 근사(정답지 판독). (C2b #2278)
fn render_pie_3d(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64)
```

- 기하: `rx = (pw.min(ph) / 2.0) * 0.9; ry = rx * PIE3D_RY_RATIO(0.55);
  wall_h = ry * PIE3D_WALL_RATIO(0.35, 보정)`; `cx = px + pw/2`,
  `cy = py + (ph - wall_h) / 2.0` (벽 포함 세로 중앙).
- 각도 규약은 2D와 동일: start `-FRAC_PI_2`, 시계방향 누적, 점 =
  `(cx + rx·cosθ, cy + ry·sinθ)`. SVG y-down에서 **하반부(벽 노출) = θ ∈ (0, π)**.
- **1차 루프(벽 먼저)**: 각 슬라이스 `[s, e]`를 `a = s.max(0.0); b = e.min(PI);`로
  클립, `b - a > 1e-6`일 때만 방출:

```rust
let (xa, ya) = (cx + rx * a.cos(), cy + ry * a.sin());
let (xb, yb) = (cx + rx * b.cos(), cy + ry * b.sin());
svg.push_str(&format!(
    "<path class=\"hwp-pie3d-wall\" d=\"M{:.2},{:.2} A{:.2},{:.2} 0 0 1 {:.2},{:.2} \
     L{:.2},{:.2} A{:.2},{:.2} 0 0 0 {:.2},{:.2} Z\" fill=\"{}\" stroke=\"#ffffff\" stroke-width=\"1\"/>\n",
    xa, ya, rx, ry, xb, yb, xb, yb + wall_h, rx, ry, xa, ya + wall_h,
    color_hex(shade(rgb, BAR3D_SIDE_SHADE))));
```

  (외호 sweep=1 정방향 → 아래로 wall_h → 복귀호 sweep=0 역방향 → Z. 클립 결과
  호 길이 ≤ π라 large-arc 항상 0. 각도 도메인 [−π/2, 3π/2]가 연속이라 0/π를 걸치는
  슬라이스도 max/min 클립 한 번으로 정확 — 랩어라운드 없음. 우상(−π/2,0)·좌상(π,3π/2)
  전용 슬라이스는 a ≥ b → 벽 없음.)
- **2차 루프(top)**: 기존 `render_pie` 슬라이스 로직 복제에 원호 `A r,r` →
  타원호 `A rx,ry` + 클래스 `hwp-pie3d-top`만 변경. 색·흰 테두리 규칙 동일
  (`first.color.unwrap_or_else(|| palette(i))`).

(c) 훅 — `render_chart_svg` Pie 단독 경로(:169-185):

```rust
if chart.chart_type == OoxmlChartType::Pie {
    if chart.is_3d {
        render_pie_3d(&mut svg, chart, plot_x, plot_y, plot_w, plot_h);
    } else {
        render_pie(&mut svg, chart, plot_x, plot_y, plot_w, plot_h);
    }
    // 범례 호출 현행 유지 (3단계에서 of_pie 분기가 이 앞에 추가됨)
```

`render_pie`(2D)는 무변경 → 2D 원형(2차원원형·쪼개진원형 등) 바이트 불변.

(d) 테스트:

- 렌더러 단위: `test_pie3d_ellipse_ratio`(top path의 `A{rx},{ry}` 파싱 →
  ry/rx ≈ 0.55) / `test_pie3d_wall_lower_half_only`(값 [25,25,50]: 슬라이스1 우상 →
  벽 없음, 2·3만 벽 → `hwp-pie3d-wall` 2개) / `test_pie3d_wall_clipped_at_boundaries`
  (첫 벽 시작 x ≈ cx+rx (θ=0 클립), 마지막 벽 끝 x ≈ cx−rx (θ=π 클립)) /
  `test_pie3d_walls_before_tops`(`svg.rfind("hwp-pie3d-wall") < svg.find("hwp-pie3d-top")`) /
  `test_pie_2d_no_walls`(is_3d=false → `hwp-pie3d` 부재 — 2D 가드).
- 통합(stage2 파트 추가): 3차원원형 × {hwpx,hwp} — `hwp-pie3d-wall` ≥ 1,
  `hwp-pie3d-top` == 4(코퍼스 4슬라이스), placeholder 부재.
- 기존 재실행: issue_1453(3차원원형 stem), issue_1882 자동제목(원형 5종 "판매" —
  dispatch 앞단이라 무접점 확인).

**완료 기준**: `cargo test` 전체 통과 + clippy 무경고. `export-svg` 3차원원형 →
`output/poc/chart_c2b/`. 단계 보고서 `task_m100_2278_stage2.md` + 커밋.

## 3단계 — ofPie 보조플롯 + 팔레트 #5 교정

**대상**: `src/ooxml_chart/{mod,parser,renderer}.rs`, 통합 테스트 확장

(a) **팔레트 실측** — `pdf/chart/원형/원형대원형-2022.pdf` 주 원형의 결합 슬라이스
("기타" 조각, 초록 계열) 픽셀 실측 → `DEFAULT_PALETTE[4]`(:21) 교체. 강등되는 하늘
0xFF5B9BD5은 [5]로 스왑(기존 [5] 유추 초록 0xFF70AD47 자리 — 실측 초록과 중복 방지),
[6][7] 유지. 주석 `하늘 (유추)` → `초록계 (실측 — ofPie 결합 슬라이스)` + 헤더 주석
갱신. `test_default_palette_hancom_order`는 앞 3색만 검사 → 무수정 통과 확인,
신규 핀 `test_palette_index4_measured`(실측 hex 고정) 추가.

(b) 모델 + 파서:

`mod.rs` — `OoxmlChart`에 필드 추가(:70 `up_down_gap_width` 뒤) + 타입 신설.
모듈 doc(:12-13, :22)에서 ofPie를 2D 근사 → 보조플롯 지원으로, 3D 입체감을 범위 외 →
지원으로 갱신:

```rust
pub struct OoxmlChart {
    …
    /// ofPie(원형대원형/원형대가로막대형) 보조플롯 정보. chart_type은 Pie를 유지하고
    /// (#1453 라우팅 앵커) 이 필드 유무로 render_of_pie를 분기. (C2b #2278)
    pub of_pie: Option<OfPieInfo>,
}

/// `c:ofPieChart` 보조플롯 파라미터 (C2b #2278)
#[derive(Debug, Clone, PartialEq)]
pub struct OfPieInfo {
    /// `c:ofPieType val` — pie=원형대원형(보조 원), bar=원형대가로막대형(누적 막대)
    pub of_pie_type: OfPieType,
    /// `c:splitPos val` — 보조 플롯으로 보낼 마지막 카테고리 수. 코퍼스 부재 →
    /// None → 기본 2. (스키마상 double — f64 파싱, 사용 시 반올림·클램프)
    pub split_pos: Option<f64>,
    /// `c:secondPieSize val` (% — 스키마 기본 75) — 보조 원 크기 / 주 원 대비
    pub second_pie_size: f64,
    /// `c:serLines` 존재 — 결합 슬라이스→보조 플롯 연결선 2줄
    pub has_ser_lines: bool,
}

impl Default for OfPieInfo {
    fn default() -> Self {
        Self { of_pie_type: OfPieType::Pie, split_pos: None, second_pie_size: 75.0, has_ser_lines: false }
    }
}

/// ofPie 보조 플롯 종류 (C2b #2278)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OfPieType {
    #[default]
    Pie,
    Bar,
}
```

`parser.rs` — `ofPieChart` arm(:213-220)에 `chart.of_pie = Some(OfPieInfo::default());`
1줄 추가(chart_type=Pie 유지), 신규 arm 4개(`scatterStyle` arm 근처):

```rust
b"ofPieType" => {
    if let (Some(of), Some(val)) = (chart.of_pie.as_mut(), attr_val(e, "val")) {
        if val == "bar" { of.of_pie_type = OfPieType::Bar; }
    }
}
b"splitPos" => { /* of.split_pos = attr_val 파싱 f64 */ }
b"secondPieSize" => { /* of.second_pie_size = attr_val 파싱 f64 */ }
b"serLines" => {
    // barChart(누적 계열선)에도 오는 요소 — Pie plot + of_pie 이중 게이트
    // (hiLowLines의 Stock 게이트 선례)
    if st.cur_plot_type == Some(OoxmlChartType::Pie) {
        if let Some(of) = chart.of_pie.as_mut() { of.has_ser_lines = true; }
    }
}
```

`splitType` arm은 추가하지 않음(범위 외 — pos 외 값이 와도 기본 last-k 동작).

(c) `render_of_pie` + 훅:

```rust
/// ofPie — 주 원(앞 n−k 카테고리 + 결합 슬라이스) + 보조 플롯(pie|bar) + serLines.
/// k = split_pos(반올림, 1..=n−1 클램프) 없으면 2. n < 3 → 일반 원형 폴백. (C2b #2278)
fn render_of_pie(svg: &mut String, chart: &OoxmlChart, of: &OfPieInfo,
                 px: f64, py: f64, pw: f64, ph: f64)
```

- 분할: `let n = first.values.len();` n < 3 → `render_pie` 위임 후 return.
  `let k = of.split_pos.map(|v| (v.round() as usize).clamp(1, n - 1)).unwrap_or(2).min(n - 1);`
  `let combined: f64 = first.values[n - k..].iter().sum();`
  (코퍼스: [10, 3.5, 1.5, 1.2] → 주 원 [10, 3.5] + 결합 2.7)
- 레이아웃(정답지 근사 초기값 — 시각판정 보정):
  `cx1 = px + pw * 0.30; cy = py + ph / 2.0; r1 = ((pw * 0.55).min(ph) / 2.0) * 0.85;
  cx2 = px + pw * 0.78; r2 = r1 * of.second_pie_size / 100.0;`
- **주 원**(슬라이스 n−k+1개): 값 시퀀스 = `values[..n-k]` + `[combined]`.
  색 = 앞 조각 `palette(ci)`(카테고리 원 인덱스 — 범례와 일치), **결합 슬라이스 =
  `palette(n)`**(코퍼스 n=4 → 교정된 [4] 초록계; 정밀 판독으로 인덱스 확인).
  **회전**: 결합 슬라이스 중앙이 보조 플롯을 향하도록(θ=0, 3시 방향) start를
  `-sweep_c / 2.0 - (total - combined) / total * TAU`로 설정(정밀 판독으로 확인) —
  이후 2D 슬라이스 루프와 동일 방출(원호 `A r1,r1`, 흰 테두리).
- **보조 플롯**:
  - `OfPieType::Pie`: 중심 (cx2, cy)·반지름 r2 원, 슬라이스 k개(값 `values[n-k..]`,
    색 `palette(n - k + j)` — 코퍼스: 회색[2]·노랑[3], 범례와 일치), start −π/2 표준.
  - `OfPieType::Bar`: 세로 누적 막대 — `bar_h = 2.0 * r2; bar_w = bar_h * 0.45;
    bx = cx2 - bar_w / 2.0; top = cy - bar_h / 2.0`. 세그먼트 높이 =
    `v / combined * bar_h`, **첫 분할 카테고리(n−k)가 맨 위** — j 순서대로 위→아래
    누적 rect(fill = `palette(n - k + j)`).
- **serLines**(`of.has_ser_lines`일 때만): 결합 슬라이스 양 모서리
  `(cx1 + r1·cos(∓sweep_c/2), cy + r1·sin(∓sweep_c/2))` → 보조 플롯 상/하단
  (pie: `(cx2, cy ∓ r2)` / bar: `(bx, top)`·`(bx, top + bar_h)`):
  `<line class="hwp-ofpie-serline" … stroke="#a6a6a6" stroke-width="1"/>` × 2.
- **범례 무변경**: `legend_items` Pie 분기(:1392)가 카테고리 n개 정순 + `palette(i)` —
  "결합 슬라이스 범례 제외·카테고리 정순" 요구를 현행이 충족. 플롯 색 매핑을 이에
  일치시킴(위 색 규칙).
- 훅 — 2단계에서 확장한 Pie 단독 경로 선두에:

```rust
if let Some(of) = &chart.of_pie {
    render_of_pie(&mut svg, chart, of, plot_x, plot_y, plot_w, plot_h);
} else if chart.is_3d { … } else { … }
```

(d) 테스트:

- 파서 단위: `test_parse_ofpie_info`(기존 ofPie XML + secondPieSize/serLines →
  of_pie Some{Pie, None, 75.0, true} + **chart_type==Pie 재확인**) /
  `test_parse_ofpie_bar_type`(val="bar") / `test_parse_ofpie_split_pos`(val="3" →
  Some(3.0)) / `test_parse_serlines_not_leaked_to_barchart`(barChart + serLines →
  of_pie None). 기존 `test_parse_ofpie` 무회귀.
- 렌더러 단위(모델 직구성: 값 [10.0, 3.5, 1.5, 1.2], 카테고리 4, of_pie Some):
  `test_ofpie_pie_secondary_and_serlines`(주 원 path 3 + 보조 path 2 +
  `hwp-ofpie-serline` 2, has_ser_lines=false → 0) /
  `test_ofpie_combined_slice_uses_palette4`(결합 fill == `color_hex(palette(4))` —
  hex 하드코딩 대신 palette 참조) / `test_ofpie_bar_secondary_first_split_cat_on_top`
  (Bar형 보조 rect 2개 — palette(2)색 y < palette(3)색 y) /
  `test_ofpie_split_pos_respected`(split_pos 3.0 → 주 2 path·보조 3) /
  `test_ofpie_legend_categories_in_order_no_combined`(범례 스와치 4개 정순·범례 조각에
  palette(4) hex 부재) / `test_ofpie_two_values_plain_pie_fallback`(n=2 → serline 0) /
  `test_palette_index4_measured`.
- 통합(stage3 파트 추가): 원형대원형·원형대가로막대형 × {hwpx,hwp} —
  `hwp-ofpie-serline` 2개·placeholder 부재·결합 슬라이스 hex 존재·가로막대형은
  보조 rect 존재.
- 기존 재실행: issue_1453 전체(ofPie 2종 포함), issue_1882 자동제목(원형대원형 "판매").

**완료 기준**: `cargo test` 전체 통과 + clippy 무경고. `export-svg` ofPie 2종 →
`output/poc/chart_c2b/` + 3D 5종 재산출 → 코퍼스 대조표 → **작업지시자 시각판정**.
단계 보고서 `task_m100_2278_stage3.md`(팔레트 실측 기록 포함) + 커밋.

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `src/ooxml_chart/mod.rs` | `OfPieInfo`/`OfPieType` 신설, `OoxmlChart.of_pie` 필드, 모듈 doc 갱신 (3단계) |
| `src/ooxml_chart/parser.rs` | ofPieChart arm 1줄 + ofPieType/splitPos/secondPieSize/serLines arm 4개 (3단계) |
| `src/ooxml_chart/renderer.rs` | `shade`/`push_bar_3d` + rect 4개소 분기 (1단계), `render_pie_3d` + Pie 분기 (2단계), `render_of_pie` + 팔레트 [4]↔[5] 스왑 (3단계), 단위 테스트 |
| `tests/issue_2278_chart_3d_ofpie.rs` | 신규 통합 가드 — 단계별 파트 추가 (1~3단계) |
| `mydocs/working/task_m100_2278_stage{1..3}.md` | 단계별 보고서 (실측 기록 포함) |
| `mydocs/report/task_m100_2278_report.md` | 최종 보고서 |
| `output/poc/chart_c2b/` | 시각판정 산출물 (gitignore) |

## 위험 / 주의

- **#1882 축 앵커**: 축/range 블록(:427-462)·`render_value_grid`·
  `render_category_labels` 호출 무접촉 — 변경은 rect 방출 4개소의 조건 분기뿐.
  단계마다 issue_1882 + `test_axis_3d_*` 재실행.
- **#1453 라우팅 앵커**: `chart_type=Pie` 불변, `of_pie` 필드로만 분기 — 파서 단위로
  명시 재확인.
- **count 앵커 오염**: 신규 요소 전부 고유 클래스(`hwp-bar3d-top/side`,
  `hwp-pie3d-wall/top`, `hwp-ofpie-serline`) — 기존 계수 대상(`hwp-chart-marker`·
  `hwp-legend-glyph`·`hwp-stock-*`·범례 10×10 rect)과 불교차. front rect는 무클래스
  (2D와 형태 통일 — 기존 rect 검사와 간섭 여부는 1단계 RED에서 확인).
- **serLines 누출**: barChart의 동명 요소 — Pie plot + of_pie 이중 게이트
  (gapWidth의 Stock 게이트 선례). 파서 단위로 비누출 핀.
- **팔레트 스왑 파급**: 코퍼스에 5시리즈+ 무지정색 차트 없음(주석 실측 기록) —
  `test_default_palette_hancom_order` 무수정 통과로 검증, [4] 핀 테스트 신설.
- **top face 상단 침범**: 3D 묶은막대는 무헤드룸 축이라 max값 막대의 top이 플롯
  상단을 넘을 수 있음 — depth ≤ 9px vs 제목 하단 여백 4px+. 시각판정에서 거슬리면
  depth 상한만 축소(축 계산 무관).
- **누적 페인트 순서 의존**: 은면 제거가 "아래→위/왼→오른쪽" 방출 순서에 의존 —
  루프 순서 변경 금지 주석 명기. 방출 순서 단위 테스트로 핀.
- **ofPie 회전·결합 색 인덱스**: Excel/한컴 관행 기반 초기값 — 3단계 (a) 정밀 판독으로
  확정 후 구현(빗나가면 상수/인덱스만 수정, 구조 불변).
- **`#[allow]`**: `push_bar_3d` 7인자·`render_of_pie` 8인자 — clippy 기본 임계(>7)는
  `render_of_pie`만 해당, 기존 :1209 선례(`#[allow(clippy::too_many_arguments)]`) 적용.
- 기능 변경만 — `cargo fmt --all` 금지(수정 파일 범위만), 저장 경로 무접점.
