---
kind: report
status: active
last_verified: 2026-07-23
---

# 처리 결과 — 파이 차트 범례 공간 미예약 (Issue: #3186)

## 증상

원형(파이) 차트는 카테고리 기반 범례를 사용하므로 시리즈 이름(`c:tx`)이 비어 있어도
`render_chart_svg`의 파이 분기가 `render_legend`/`render_legend_right`를 무조건
호출해 범례가 항상 그려진다. 그런데 범례를 위한 레이아웃 공간(`legend_h`/`legend_w`)은
`legend_visible = chart.series.iter().any(|s| !s.name.is_empty())`, 즉 시리즈 이름
존재 여부로만 계산되고 있었다.

결과: 시리즈 이름이 없는 파이 차트는 범례는 그려지는데 플롯 영역이 그 공간을 빼지
않고 계산되어 파이가 부당하게 커지고 범례와 겹친다.

## 재현 (red)

`src/ooxml_chart/renderer.rs`에 테스트
`test_pie_legend_reserves_space_regardless_of_series_name` 추가. 카테고리 3개,
값 3개, 시리즈 이름만 다른(`"판매"` vs `""`) 두 파이 차트를 400x300으로 렌더링해
파이 반지름(SVG path의 `A r,r`)을 비교:

- 이름 있음: r = 111.6 (범례 공간 22px 예약)
- 이름 없음: r = 121.5 (범례 공간 미예약 — 버그)

두 경우 모두 `hwp-chart-legend`가 렌더 결과에 포함되므로(카테고리 기반) 반지름은
같아야 정상. 수정 전 FAIL 확인.

## 원인

`render_chart_svg`(`src/ooxml_chart/renderer.rs`)의 `legend_visible` 계산이 모든
차트 종류에 대해 "시리즈 이름 존재 여부"만 사용했다. 그러나 파이 차트의 범례는
`legend_items()`에서 카테고리 라벨을 사용하며(`chart.categories`), 시리즈 이름과
무관하게 항상 렌더링 분기를 탄다(파이 전용 분기는 `legend_visible`을 참조하지 않고
무조건 `render_legend`/`render_legend_right`를 호출).

## 수정

`legend_visible` 계산에 파이 차트 종류를 추가로 포함:

```rust
let legend_visible =
    chart.chart_type == OoxmlChartType::Pie || chart.series.iter().any(|s| !s.name.is_empty());
```

`render_chart_svg` 진입 시 `chart.series.is_empty()`면 이미 fallback으로 반환되므로
파이 차트는 항상 `series`가 1개 이상 존재 — 위 변경은 안전하다.

## 검증

- 신규 테스트 green 확인.
- `cargo test --lib ooxml_chart` 138개 전부 pass (기존 회귀 없음).
- `cargo fmt --check`: 대상 파일 diff 없음(CRLF 관련 노이즈만, 기존에도 존재).
- `cargo clippy --lib -- -D warnings`: 경고 없음.
- `cargo test --release --lib`: 실행 완료 확인(별도 로그).

## 영향 범위

`src/ooxml_chart/renderer.rs` 한 줄 조건 추가 + 테스트 1건. 파이 차트가 아닌
경우(막대/선/분산형/stock/콤보) 동작 변화 없음.
