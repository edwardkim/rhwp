# task_m100_1660 Stage 2 완료 보고서 — 파서

- 이슈: #1660 (C1b, #1431 Track C)
- 브랜치: `local/task1660`
- 단계: Stage 2 / 4 — XML 파서 (`src/ooxml_chart/parser.rs`)

## 변경 내용

1. **import**: `ScatterStyle` 추가.
2. **`ParseState`**: `in_x_val`, `in_y_val` 플래그 추가.
3. **`handle_start`**:
   - `b"scatterChart"` → `chart_type=Scatter`, `cur_plot_type=Scatter`, plot ax_ids 초기화.
   - `b"scatterStyle"` → `val` 매핑: `line`→Line, `lineMarker`→LineMarker, `smooth`/`smoothMarker`→SmoothMarker, 그 외(`marker`/`none`/미상)→Marker.
   - `b"xVal"`→`in_x_val=true`, `b"yVal"`→`in_y_val=true`.
4. **`handle_end` `b"v"`**: 분기 순서 `in_tx → in_cat → (in_val || in_y_val) → in_x_val`.
   `in_y_val`은 `in_val`과 동일하게 `ser.values`(Y)로, `in_x_val`은 `ser.x_values`로 push.
   시리즈명(`c:tx`)은 `in_tx`가 먼저라 x_values로 새지 않음.
5. **`handle_end`**: `b"xVal"`/`b"yVal"` 플래그 clear. plot 종료 매치에 `b"scatterChart"` 추가(axIds 복사).
6. **핵심 가드**: 축 분류 후처리의 시리즈 axis_group 지정 루프를 `if chart.chart_type != Scatter { … }`로 감쌈.
   분산형은 X·Y 모두 `valAx`(axPos b/l)라 primary/secondary 오분류 → `has_secondary_axis=true` →
   콤보 라우팅으로 새는 것을 차단. scatter는 `axis_group=0`/`has_secondary_axis=false` 기본값 유지.

## 테스트 (parser.rs, 5개 신규)

- `test_parse_scatter_marker`: chart_type=Scatter, 2시리즈, `x_values=[0.7,1.8,2.6]`, `values=[2.7,3.2,0.8]`/`[1,2,4]`, scatter_style=Marker, categories 빔, 시리즈명 보존("Y1 값").
- `test_parse_scatter_style_line` / `_line_marker` / `_smooth_marker`: scatterStyle→ScatterStyle 매핑.
- `test_scatter_no_secondary_axis` (**회귀 가드**): `has_secondary_axis==false`, `is_combo()==false`, 모든 `axis_group==0`.

## 검증

```
cargo test -p rhwp ooxml_chart::parser
→ test result: ok. 18 passed; 0 failed  (신규 5 + 기존 13 무회귀)
```

Scatter는 파싱되나 렌더러 미연결(Stage 3) → 현재 dispatch `_` arm으로 빈 출력. placeholder 해소는 Stage 3.

## 다음 단계

Stage 3 — 렌더러(`renderer.rs`): `render_scatter`, 소수 축 라벨(`format_axis_num`/`render_value_grid` decimal),
`scatter_range`, Catmull-Rom 곡선 + 테스트 5.
