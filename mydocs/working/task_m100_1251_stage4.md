# Task M100-1251 Stage 4 — renderer 통합

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **브랜치**: `task-1251-ole-chart`
- **상태**: 구현 및 검증 완료

## 1. 구현 내용

`src/ole_chart/charming_renderer.rs`

- `render_ole_chart_svg(&OleChart, width, height)` 추가
- parsed `OleChart`를 `charming::Chart`로 변환
- 현재 parser가 `chart_type=Unknown`을 반환하는 #1251 fixture는 기본 column/bar 계열로 렌더
- `Line` type은 line series로 분기할 수 있도록 유지

`src/renderer/layout/shape_layout.rs`

- OLE CFB 렌더 우선순위에 legacy `Contents` chart 경로 추가
- 우선순위:
  1. HWPX direct `ooxml_chart`
  2. nested OLE `OOXMLChartContents`
  3. nested OLE legacy `Contents` + `ole_chart` parser + `charming` SSR SVG
  4. `OlePres000` EMF preview
  5. native image
  6. generic OLE placeholder
- legacy `Contents` parsing/rendering 실패 시 generic placeholder로 묻지 않고 `OLE 차트 미지원:` 라벨을 노출
- native SSR 결과는 `hwp-ole-chart hwp-ole-chart-charming` class를 가진 RawSvg fragment로 삽입
- wasm target에서는 parsing 성공 시 `charming SSR unavailable on wasm` fallback 라벨을 사용

`tests/issue_1251_ole_chart_contents.rs`

- parsed fixture chart가 `charming` SVG로 렌더되는지 테스트 추가
- `render_page_svg_native(0)` 결과가 `hwp-ole-chart`를 포함하고 generic `OLE 개체 (BinData #2)` placeholder를 포함하지 않는지 회귀 테스트 추가

## 2. 검증 결과

통과:

```text
cargo fmt --check
cargo test --test issue_1251_ole_chart_contents -- --nocapture
cargo test --lib ole_chart -- --nocapture
cargo build
cargo check --target wasm32-unknown-unknown --lib
cargo test --test issue_1156_chart_column_flow -- --nocapture
target/debug/rhwp export-svg samples/143E433F503322BD33.hwp -o output/poc/task1251/hwp
```

출력 확인:

- `output/poc/task1251/hwp/143E433F503322BD33.svg`
- 포함 marker:
  - `hwp-ole-chart hwp-ole-chart-charming`
  - `연금 재정 전망`
  - `적립금`
- 미포함 marker:
  - `OLE 개체 (BinData #2)`
  - `OLE 차트 미지원`

## 3. 남은 범위

- Stage 5에서 전체 검증/보고서 작성
- `ChartType` property의 byte-level 위치가 확정되면 `OleChartType::Unknown`을 실제 type으로 승격
- `charming` dependency가 CI에서 과도하면 feature gate나 renderer 분리 여부 재평가
