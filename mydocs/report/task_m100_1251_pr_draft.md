# Task M100-1251 PR 초안

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **브랜치**: `task-1251-ole-chart`
- **작성일**: 2026-06-03
- **상태**: 초안

## 1. PR 제목 후보

```text
Render legacy HWP OLE chart Contents with Rust SVG fallback
```

## 2. PR 본문 초안

````markdown
## Summary

This PR adds a first legacy HWP OLE chart `/Contents` rendering path for #1251.

- Detects nested OLE chart data that only has a legacy `Contents` stream.
- Parses the fixture's chart title, categories, series labels, and numeric data into a renderer-neutral `OleChart` IR.
- Renders the chart through the canonical Rust SVG path as a `RawSvg` page layer, so native export and WASM Studio use the same renderer.
- Keeps `yuankunzhang/charming` as an optional native SSR adapter behind the `charming-renderer` feature.

## Background

The sample `samples/143E433F503322BD33.hwp` stores its chart as `BinData #2`.
That nested OLE object does not contain `OOXMLChartContents`, `OlePres000`, or a native image preview.
The only useful chart payload found for this fixture is the legacy `Contents` stream, so the previous renderer fell back to a generic OLE placeholder.

The maintainer asked to investigate `yuankunzhang/charming`.
The conclusion from this work is that `charming` is useful as a chart rendering adapter, but it does not parse Hancom legacy OLE chart data.
The HWP/OLE `/Contents` parser therefore has to live in rhwp.

## Official References

- Hancom HWP/OWPML format download center: https://www.hancom.co.kr/support/downloadCenter/hwpOwpml
- Hancom HWP 5.0 file format revision 1.3: https://cdn.hancom.com/link/docs/%ED%95%9C%EA%B8%80%EB%AC%B8%EC%84%9C%ED%8C%8C%EC%9D%BC%ED%98%95%EC%8B%9D_5.0_revision1.3.pdf
- Hancom chart file format revision 1.2: https://cdn.hancom.com/link/docs/%ED%95%9C%EA%B8%80%EB%AC%B8%EC%84%9C%ED%8C%8C%EC%9D%BC%ED%98%95%EC%8B%9D_%EC%B0%A8%ED%8A%B8_revision1.2.pdf
- Microsoft Compound File Binary File Format, MS-CFB: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b
- Microsoft OLE Data Structures, MS-OLEDS: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-oleds/85583d21-c1cf-4afe-a35f-d6701c5fbb6f
- charming upstream repository: https://github.com/yuankunzhang/charming

## Design Decisions

### Parser ownership

The parser is implemented in rhwp, not in `charming`.
The Hancom chart format stores legacy chart objects such as `VtDataGrid` and `VtChartTitle` inside the OLE `Contents` stream, while `charming` expects an already structured chart model.

### Default renderer

The default renderer is the Rust SVG renderer in `src/ole_chart/svg_renderer.rs`.
This keeps native export and WASM Studio on the same rendering path and fits the upstream direction toward `PageLayerTree`, Skia, CanvasKit, and other backend renderers.

### charming usage

`charming` remains as an optional native SSR adapter:

```toml
charming-renderer = ["dep:charming"]
```

The browser/WASM `charming::WasmRenderer` path was not selected as the default because it requires a DOM element id and a browser global ECharts runtime.
That would make chart rendering a browser-side DOM side effect instead of a renderer-owned page layer.
It would also add a JavaScript ECharts runtime dependency to Studio.

## Implementation Notes

- `src/ole_chart/parser.rs`
  - Probes legacy `ChartOBJ` layout.
  - Detects `VtDataGrid` and `VtChartTitle`.
  - Extracts dense numeric `f64` runs first and only falls back to exact sparse candidates.
- `src/ole_chart/ir.rs`
  - Provides renderer-neutral chart IR serialization helpers.
- `src/ole_chart/svg_renderer.rs`
  - Emits `hwp-ole-chart-rust-svg` SVG fragments for `RawSvg`.
- `src/ole_chart/charming_renderer.rs`
  - Provides optional native `charming` SVG export smoke coverage.
- `src/renderer/layout/shape_layout.rs`
  - Adds `nested OLE Contents -> ole_chart parser -> Rust SVG RawSvg` before preview/native-image/generic-placeholder fallback.

## Validation

Passed locally:

```text
cargo fmt --check
cargo test --test issue_1251_ole_chart_contents -- --nocapture
cargo test --lib ole_chart -- --nocapture
cargo test --features charming-renderer --test issue_1251_ole_chart_contents -- --nocapture
cargo build
cargo check --target wasm32-unknown-unknown --lib
cargo test --test issue_1156_chart_column_flow -- --nocapture
cargo clippy --all-targets -- -D warnings
npm run build
wasm-pack build --target web
target/debug/rhwp export-svg samples/143E433F503322BD33.hwp -o output/poc/task1251/hwp
headless Chrome Studio QA
```

The Studio QA confirmed:

- one chart `RawSvg` layer is generated;
- `data-rhwp-ole-chart-renderer="rust-svg"` is present;
- the previous `charming SSR unavailable on wasm` text is gone;
- the generic `OLE object (BinData #2)` placeholder is gone;
- no browser overlay layer is used.

## Known Limitations

The chart data is recovered, but visual fidelity is not yet identical to the Hancom PDF reference.

Current known gaps:

- y-axis nice scale is not parsed yet, so the current Rust SVG renderer uses data max instead of the Hancom-like 0-2000 / 500-step axis;
- palette, legend location, title spacing, plot margin, border, and bar gap still use renderer defaults;
- broader page layout/text differences also exist outside this chart work.

I intentionally did not add fixture-specific heuristics to match the PDF.
Pixel-level chart fidelity should be handled in follow-up work by parsing more of the legacy chart object graph, especially axis, legend, style, and layout objects.

## Additional Analysis Documents

- Renderer decision record: `mydocs/tech/hwp_ole_chart_renderer_architecture_decision_1251.md`
- Hancom PDF visual diff analysis: `mydocs/tech/hwp_ole_chart_visual_diff_against_hancom_pdf_1251.md`
- Final task report: `mydocs/report/task_m100_1251_report.md`

## Review Notes

The main review point is whether upstream wants to keep the default browser/WASM path renderer-owned as implemented here, or whether it prefers a browser ECharts runtime path through `charming::WasmRenderer`.
Given the current multi-backend renderer direction, this PR chooses the renderer-owned Rust SVG path and keeps `charming` optional.
````

## 3. PR 생성 전 확인 사항

1. 현재 브랜치는 `upstream/devel` 대비 뒤처져 있으므로, PR 생성 전 rebase 또는 merge 최신화가 필요하다.
2. #1251과 무관한 로컬 문서(`task_m100_1142`, `task_m100_1143`, `task_m100_1144`)는 PR에 포함하지 않는다.
3. `AGENTS.md`는 upstream에 없는 로컬 편의 파일이므로 PR에 포함하지 않는다.
4. PR 생성 전 마지막으로 다음 검증을 다시 실행한다.

```text
cargo fmt --check
cargo test --test issue_1251_ole_chart_contents -- --nocapture
cargo test --lib ole_chart -- --nocapture
cargo test --features charming-renderer --test issue_1251_ole_chart_contents -- --nocapture
cargo check --target wasm32-unknown-unknown --lib
cargo clippy --all-targets -- -D warnings
npm run build
```
