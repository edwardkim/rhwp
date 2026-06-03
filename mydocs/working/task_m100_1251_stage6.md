# Task M100-1251 Stage 6 완료 보고

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **브랜치**: `task-1251-ole-chart`
- **일자**: 2026-06-03
- **상태**: 완료

## 1. 목표

Stage 5의 Studio TypeScript chart overlay를 장기 경로로 유지하지 않고, Rust/WASM 프로젝트 철학과 PageLayerTree/Skia/CanvasKit 확장 방향에 맞춰 OLE chart 기본 렌더링을 Rust SVG renderer로 통일한다.

## 2. 구현 내용

1. `src/ole_chart/ir.rs`를 추가해 renderer-neutral IR JSON/base64 export만 담당하게 했다.
2. `src/ole_chart/svg_renderer.rs`를 추가해 `OleChart`를 SVG body, standalone SVG, positioned RawSvg fragment로 렌더한다.
3. `src/renderer/layout/shape_layout.rs`의 legacy OLE `/Contents` 성공 경로에서 native/WASM 분기를 제거하고 `render_ole_chart_svg_fragment`를 공통 호출한다.
4. `rhwp-studio/src/view/ole-chart-renderer.ts`를 제거하고 `page-renderer.ts`의 `.rhwp-ole-chart-overlay-layer` 합성 경로를 삭제했다.
5. `charming` dependency를 `charming-renderer` feature 뒤 optional adapter로 분리했다.

## 3. 검증 결과

통과:

```text
cargo fmt --check
cargo test --test issue_1251_ole_chart_contents -- --nocapture
cargo test --lib ole_chart -- --nocapture
cargo test --features charming-renderer --test issue_1251_ole_chart_contents -- --nocapture
cargo check --target wasm32-unknown-unknown --lib
cargo build
cargo clippy --all-targets -- -D warnings
cargo test --test issue_1156_chart_column_flow -- --nocapture
npm run build
wasm-pack build --target web
target/debug/rhwp export-svg samples/143E433F503322BD33.hwp -o output/poc/task1251/hwp
headless Chrome Studio QA
```

Studio QA 결과:

```text
pageCount: 1
chartRawSvgCount: 1
hasRustSvgMarker: true
hasIrPayloadAttr: false
hasTitleInLayerTree: true
hasSeriesInLayerTree: true
browserOverlayCount: 0
overlayLayerCount: 0
hasUnavailableText: false
hasGenericOlePlaceholder: false
saturatedChartPixels: 11283
```

스크린샷:

```text
/private/tmp/rhwp-studio-issue-1251-rust-svg.png
```

## 4. 잔여 리스크

- 현재 canonical renderer는 SVG fragment 기반이다. PageLayerTree/Skia/CanvasKit 친화성을 더 높이려면 후속 단계에서 chart visual model을 `Line`/`Rectangle`/`Path`/`TextRun` paint op로 낮추는 작업이 필요하다.
- `charming` adapter는 optional feature로 컴파일 검증했지만 기본 렌더 경로에서는 사용하지 않는다. PR 설명에서 maintainer 지시를 “기본 renderer”가 아니라 “optional adapter”로 해석한 이유를 명확히 적어야 한다.
- legacy `/Contents` parser는 #1251 fixture에 필요한 `VtDataGrid`/`VtChartTitle` 중심 최소 parser다. 다른 legacy chart object graph는 추가 fixture로 확장해야 한다.
