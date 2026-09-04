# task_m100_1660 Stage 1 완료 보고서 — 모델 확장

- 이슈: #1660 (C1b, #1431 Track C)
- 브랜치: `local/task1660`
- 단계: Stage 1 / 4 — 데이터 모델 (`src/ooxml_chart/mod.rs`)

## 변경 내용

1. **`OoxmlChartType::Scatter` 추가** (Pie와 Unknown 사이) + `label()` → `"분산형"`.
2. **`ScatterStyle` enum 신규**: `Marker`(default) / `Line` / `LineMarker` / `SmoothMarker`,
   `flags(&self) -> (show_line, smooth, show_markers)`:
   - Marker `(false,false,true)`, Line `(true,false,false)`, LineMarker `(true,false,true)`, SmoothMarker `(true,true,true)`.
3. **`OoxmlSeries.x_values: Vec<f64>` 추가** — 분산형 `c:xVal` 전용, 그 외 차트는 빈 Vec. `values`는 Y(`c:yVal`) 유지.
4. **`OoxmlChart.scatter_style: ScatterStyle` 추가** — scatter 렌더러 전용.
5. 모듈 doc: 산점도를 "범위 외" → "지원 범위"로 이동.

## 설계 근거 (IR 모델: 확장 vs 신규 타입)

`OoxmlSeries` **확장** 선택. 전 파이프라인이 `Vec<OoxmlSeries>` 기반(legend/색상/이름/`is_combo`/시리즈 수집)이라
`x_values` 한 필드 추가로 전부 재사용. 비-scatter 시리즈의 빈 Vec은 24B/시리즈(heap 미할당)로 무시 가능.
신규 `ScatterSeries` 타입은 `chart.series` 컬렉션 타입을 깨고 legend/색상/이름 로직을 중복시킴.
bubble·stock HLC가 실제 등장하는 **C2 시점**에 `SeriesData` 합타입으로 재평가(그때 마이그레이션은 기계적).

## 검증

```
cargo build → Finished (경고 0)
```

`OoxmlChartType::Scatter` 추가 후에도 기존 매치(`renderer.rs`의 `_ => {}`, `label()`)가 모두 컴파일.
Scatter는 아직 파서/렌더러 미연결(Stage 2~3) → 현재는 렌더 시 `_` arm으로 빈 출력(placeholder는 Stage 3에서 해소).

## 다음 단계

Stage 2 — 파서(`parser.rs`): `scatterChart`/`xVal`/`yVal`/`scatterStyle` 인식 + 축 분류 가드 + 테스트 5.
