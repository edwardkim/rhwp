# task_m100_1660 Stage 3 완료 보고서 — 렌더러

- 이슈: #1660 (C1b, #1431 Track C)
- 브랜치: `local/task1660`
- 단계: Stage 3 / 4 — SVG 렌더러 (`src/ooxml_chart/renderer.rs`)

## 변경 내용

1. **import**: `ScatterStyle`.
2. **`format_axis_num(v)`** 신규: 정수면 정수, 아니면 `{:.2}` 후 trailing 0 제거 (0.5→"0.5", 1.0→"1", 2.6→"2.6").
   `format_num`(정수 반올림)은 무수정 — 소수 분산형 축 라벨이 손상되지 않게 별도 헬퍼.
3. **`render_value_grid`에 `decimal: bool` 추가**: `decimal`이면 label에 `format_axis_num` 사용.
   기존 4개 호출처(render_bars/render_line/render_combo×2)에 `false` 전달 → bar/line/combo 라벨 출력 무변경.
4. **`scatter_range(iter)`** 신규: `value_range_for`와 달리 min→0 강제 안 함(데이터 bracket), `nice_range`로 눈금 정리.
5. **dispatch `match`에 `Scatter => render_scatter`** arm.
6. **`render_scatter`**: X 범위(모든 `x_values`) + Y 범위(모든 `values`) → 플롯 rect →
   `render_value_grid` ×2(X 하단 수직격자 / Y 좌측 수평격자, decimal=true) → 시리즈별 (x,y) 픽셀 매핑
   (`px+pw*(x-xmin)/xspan`, `py+ph-ph*(y-ymin)/yspan`) → `scatter_style.flags()`로:
   - `show_line && len>=2`: smooth면 `smooth_path`(Catmull-Rom→cubic Bézier), 아니면 `polyline_path`(M/L).
   - `show_markers`: 점마다 `<circle r=3>`.
7. **`polyline_path`/`smooth_path`** 헬퍼. smooth는 끝점 clamp(P₋₁=P₀, Pₙ=Pₙ₋₁), 제어점 `c1=P₁+(P₂−P₀)/6`, `c2=P₂−(P₃−P₁)/6`.
8. **범례**: 추가 코드 0줄 — `render_legend`의 비-Line arm이 rect swatch 사용.

## 엣지 케이스
- x/y 길이 불일치 → `zip` 짧은 쪽 절단. <2점 → 선 skip(마커만). 빈 점 시리즈 → continue. 전 시리즈 빈 데이터 → 조기 return(상위 `<g>`는 출력되어 placeholder 안 뜸). 분모 `(span).max(1e-9)`.

## 테스트 (renderer.rs, 5개 신규)
- `test_render_scatter_marker_only`: `<circle` 有, `<path>` 無, placeholder 無, `hwp-ooxml-chart"` 有.
- `test_render_scatter_line_only`: `<path>` 有, `<circle>` 無, ` C`(베지어) 無.
- `test_render_scatter_line_marker`: `<path>`+`<circle>` 有, ` C` 無(직선).
- `test_render_scatter_smooth`: `<path>`+`<circle>`+` C`(cubic Bézier) 有.
- `test_render_scatter_decimal_axis_labels`: 소수 라벨 "0.5" 有, placeholder 無.

## 검증
```
cargo test -p rhwp ooxml_chart::renderer → 13 passed; 0 failed  (신규 5 + 기존 8 무회귀)
cargo clippy --lib → ooxml_chart 경고 0
```

## 다음 단계
Stage 4 — 통합 테스트(`tests/issue_1431_scatter.rs`, 10파일) + 전체 `cargo test` + `clippy --all-targets` + 실파일 export-svg 시각 검증(`pdf/chart/분산형` 대조).
