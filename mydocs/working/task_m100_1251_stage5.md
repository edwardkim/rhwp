# Task M100-1251 Stage 5 완료 보고

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **브랜치**: `task-1251-ole-chart`
- **일자**: 2026-06-03
- **상태**: 완료

> 후속 Stage 6에서 이 단계의 Studio TypeScript overlay 경로는 제거되었다. 현재 canonical 경로는 Rust SVG renderer가 native/WASM 모두에 같은 `RawSvg` fragment를 제공하는 구조다.

## 1. 목표

Stage 4까지 구현한 native `charming` SSR 경로를 유지하면서, rhwp-studio WASM 경로에서 보이던 `charming SSR unavailable on wasm` fallback을 제거한다.

장기 방향은 다음 구조로 고정했다.

```text
OLE Contents
→ OleChart renderer-neutral IR
→ native charming SSR adapter
→ Studio browser SVG overlay adapter
→ downstream renderer adapter 후보
```

## 2. 구현 내용

1. `OleChart`, `OleChartSeries`, `OleChartType`에 `serde::Serialize`를 추가했다.
2. `src/ole_chart/browser_ir.rs`를 추가해 다음 API를 제공했다.
   - `ole_chart_ir_json`
   - `ole_chart_ir_base64`
   - `render_ole_chart_ir_svg_fragment`
3. WASM target의 OLE `/Contents` 성공 경로를 placeholder에서 IR SVG fragment로 변경했다.
4. `rhwp-studio/src/view/ole-chart-renderer.ts`를 추가해 `data-rhwp-ole-chart-ir` payload를 DOM overlay SVG로 렌더한다.
5. `rhwp-studio/src/view/page-renderer.ts`에서 page layer tree의 `rawSvg` op를 읽어 `.rhwp-ole-chart-overlay-layer`를 합성한다.

## 3. 검증 결과

통과:

```text
cargo fmt --check
cargo test --test issue_1251_ole_chart_contents -- --nocapture
cargo test --lib ole_chart -- --nocapture
cargo build
cargo check --target wasm32-unknown-unknown --lib
cargo test --test issue_1156_chart_column_flow -- --nocapture
cargo clippy --all-targets -- -D warnings
npm run build
wasm-pack build --target web
target/debug/rhwp export-svg samples/143E433F503322BD33.hwp -o output/poc/task1251/hwp
node /private/tmp/rhwp_issue_1251_qa.mjs
```

rhwp-studio QA 결과:

```text
pageCount: 1
chartCount: 1
overlayLayerCount: 1
hasUnavailableText: false
hasGenericOlePlaceholder: false
hasTitle: true
hasSeries: true
consoleMessages: []
```

스크린샷:

```text
/private/tmp/rhwp-studio-issue-1251-chart.png
```

## 4. 잔여 리스크

- Studio adapter는 현재 외부 의존성 없는 SVG renderer다. ECharts/charming browser renderer로 교체하려면 `OleChartIrPayload`를 그대로 소비하는 adapter만 교체하면 된다.
- legacy `/Contents` parser는 #1251 fixture에 필요한 `VtDataGrid`/`VtChartTitle` 중심 최소 parser다. 다른 legacy chart object graph는 추가 fixture로 확장해야 한다.
- issue close 및 PR 생성은 작업지시자 승인 후 진행한다.
