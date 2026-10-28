# task_m100_1660 구현 계획서 — C1b 분산형(scatter) 차트 렌더링

- 이슈: #1660 (C1b, #1431 Track C 하위)
- 브랜치: `local/task1660` (from `local/devel`)
- 마일스톤: M100

## 1. 배경 / 목표

분산형 차트 5종이 `OoxmlChart::parse`에서 `c:xVal`/`c:yVal`를 읽지 못하고(`c:val`만 파싱),
`scatterChart` 요소 미인식으로 `chart_type=Unknown` → 렌더러가 "차트 (미지원)" placeholder로 bail
(`src/ooxml_chart/renderer.rs:64`). 분산형의 **렌더 커버리지**(placeholder 0건) + 합리적 스타일을 달성한다.
픽셀 단위 한컴 parity는 1차 목표 아님(#1251).

대상 5종(각 `.hwp`+`.hwpx` = 10파일, `samples/chart/분산형/`):
표식만있는·직선이있는·직선및표식이있는·곡선이있는·곡선및표식이있는분산형.
정답지 PDF: `pdf/chart/분산형/{stem}-2022.pdf` (한글 2022).

## 2. 설계 결정

- **IR 모델: `OoxmlSeries` 확장**. `x_values: Vec<f64>` 추가, `values`는 Y 유지(비-scatter는 빈 Vec).
  전 파이프라인이 `Vec<OoxmlSeries>` 기반(legend/색상/이름/`is_combo`/시리즈 수집 재사용). 신규 타입 분리는
  현재 소비처 없음 → bubble·stock HLC가 등장하는 C2 시점에 `SeriesData` 합타입으로 재평가.
- **스타일: `c:scatterStyle` 단독으로 4종 구분** (곡선 2종은 동일 XML이라 동일 렌더, 수용):

  | scatterStyle | 선 | 곡선 | 표식 |
  |---|---|---|---|
  | `marker` | – | – | ● |
  | `line` | 직선 | – | – |
  | `lineMarker` | 직선 | – | ● |
  | `smoothMarker`(×2) | 곡선 | ✓ | ● |

  `c:smooth`/`c:marker`/`c:symbol` 파싱 불필요(실측: 곡선 2종 chart XML 바이트 동일).
- **배선 자동**: `shape_layout.rs:1473`가 `OoxmlChart::parse` → `render_svg` 호출 → `chart_type≠Unknown`이면 자동. shape_layout 무수정.

## 3. 단계 (4단계, stage-gated)

수정 파일: `src/ooxml_chart/{mod,parser,renderer}.rs` + 신규 `tests/issue_1431_scatter.rs`.
테스트 패턴은 `tests/issue_1453_chart_3d_ofpie_routing.rs` 미러.

### Stage 1 — 모델 (`mod.rs`)
- `OoxmlChartType::Scatter` + `label()` "분산형".
- `enum ScatterStyle { Marker(default), Line, LineMarker, SmoothMarker }` + `flags() -> (show_line, smooth, show_markers)`.
- `OoxmlSeries.x_values: Vec<f64>`, `OoxmlChart.scatter_style: ScatterStyle`.
- 모듈 doc 범위 갱신. 검증: `cargo build`.

### Stage 2 — 파서 (`parser.rs`)
- `ParseState`에 `in_x_val`/`in_y_val`.
- `handle_start`: `scatterChart`(→Scatter), `xVal`/`yVal`(플래그), `scatterStyle`(→`chart.scatter_style`, 미상=Marker).
- `handle_end` `b"v"`: in_val 뒤 `in_x_val`→x_values, `in_y_val`→values. `xVal`/`yVal` 플래그 clear. plot 종료 매치에 `scatterChart` 추가.
- **핵심 가드**: 축 분류 후처리(parser.rs:109-158)를 `if chart_type != Scatter { … }`로 감싸 has_secondary_axis 오설정 차단.
- 테스트 5: scatterStyle 매핑, x/y 추출, 회귀 `has_secondary_axis==false`. 검증: `cargo test -p rhwp ooxml_chart::parser`.

### Stage 3 — 렌더러 (`renderer.rs`)
- `format_axis_num`(소수 라벨) + `render_value_grid`에 `decimal: bool`(기존 6호출처 `false` → 무회귀). `format_num` 무수정.
- `scatter_range`(min→0 강제 안 함, data-driven nice_range).
- dispatch `match`에 `Scatter => render_scatter`.
- `render_scatter`: X/Y 범위, plot rect, `render_value_grid` ×2(X 하단/Y 좌측, decimal=true), 점 매핑, `flags()`로 line(직선 M/L 또는 곡선 Catmull-Rom→cubic Bézier)/markers(`<circle r=3>`). 엣지(zip min/<2점/빈 시리즈/분모 1e-9).
- 범례: `render_legend` rect swatch 재사용(추가 0줄).
- 테스트 5. 검증: `cargo test -p rhwp ooxml_chart::renderer`(bar/line/combo/percent 라벨 무회귀 포함).

### Stage 4 — 통합 + 시각 검증
- `tests/issue_1431_scatter.rs`: 5 stems × {hwpx,hwp}=10파일, placeholder 0 + `hwp-ooxml-chart"` 有 + fallback 無.
- 전체 `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- 시각: `rhwp export-svg` 5종 → `output/poc/c1b_scatter/`, `pdf/chart/분산형` 대조.

## 4. Out of scope
- `src/ole_chart/`(레거시), `src/model/shape.rs` `ChartType::Scatter`, stock HLC(C2), 스타일 4갭(C1c), 라인 누적(C1d).

## 5. 검증 게이트 (Rust 변경)
`cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings`. touch한 4파일만 `cargo fmt`.

## 6. 워크플로우
- 단계별 소스 + `mydocs/working/task_m100_1660_stage{N}.md` 함께 커밋, 단계마다 승인.
- 컨트리뷰터: 이슈 close/직접 merge 금지, origin push → upstream `devel` PR(`Refs #1431`, `#1660`). orders 미수정.
- 최종 `mydocs/report/task_m100_1660_report.md`.
