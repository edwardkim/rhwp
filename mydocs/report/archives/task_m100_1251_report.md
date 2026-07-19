# Task M100-1251 최종 보고서

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **브랜치**: `task-1251-ole-chart`
- **일자**: 2026-06-03
- **상태**: 구현, 검증, PR 초안 완료

## 1. 요약

`samples/143E433F503322BD33.hwp`의 `BinData #2` OLE chart object는 nested OLE 안에 `OOXMLChartContents`, `OlePres000`, native image가 없고 legacy `Contents` 스트림만 가진다.

이번 작업에서 해당 `Contents`를 최소 차트 IR로 파싱하고, 기본 렌더링 경로는 Rust SVG renderer로 통일했다. native export와 rhwp-studio WASM 모두 같은 `hwp-ole-chart-rust-svg` `RawSvg` fragment를 사용한다.

`yuankunzhang/charming`은 maintainer 지시를 반영해 검토했으나, 통합 검증에서 `deno_core/v8`가 현재 `cdylib` crate-type과 충돌하는 링크 오류를 일으켜 최종 통합본에서는 제외했다. 기본 렌더링 경로는 Rust SVG renderer로 고정했다.

## 2. 주요 산출물

- `src/ole_chart/parser.rs`: legacy HWP chart `Contents` probe/parser
- `src/ole_chart/ir.rs`: renderer-neutral IR JSON/base64 payload
- `src/ole_chart/svg_renderer.rs`: canonical Rust SVG renderer
- `src/renderer/layout/shape_layout.rs`: OLE render priority에 legacy `/Contents` chart 경로 추가
- `tests/issue_1251_ole_chart_contents.rs`: fixture 기반 회귀 테스트
- `mydocs/tech/investigations/issue-1251/hwp_ole_chart_visual_diff_against_hancom_pdf_1251.md`: 정답 PDF 대비 시각 차이 분석
- `mydocs/tech/hwp_ole_chart_renderer_architecture_decision_1251.md`: Rust SVG canonical renderer 결정 기록과 `charming` 검토 결과
- `mydocs/report/task_m100_1251_pr_draft.md`: 공식 문서, 결정 배경, 구현 결정, known gap을 포함한 PR 본문 초안

## 3. 검증

모든 핵심 검증을 통과했다.

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
headless Chrome Studio QA
```

native SVG 결과:

- `hwp-ole-chart hwp-ole-chart-rust-svg` 존재
- `연금 재정 전망`, `적립금` 존재
- `OLE 개체 (BinData #2)`, `OLE 차트 미지원` 없음

rhwp-studio 결과:

- `PageLayerTree`의 `rawSvg` 차트 op 1개 생성
- `data-rhwp-ole-chart-renderer="rust-svg"` 존재
- `.hwp-ole-chart-browser` 0개
- `.rhwp-ole-chart-overlay-layer` 0개
- `charming SSR unavailable on wasm` fallback 문구 없음
- `OLE 개체 (BinData #2)` 없음
- 차트 bbox 내 saturated pixel 11283개 확인
- console warning 1개는 QA 스크립트의 `getImageData` readback 경고이며 앱 렌더 오류가 아님

## 4. 후속 제안

1. 다른 legacy chart fixture를 추가해 `VtDataGrid` 외 object graph와 chart type parsing을 확장한다.
2. Rust SVG renderer를 장기적으로 `Line`/`Rectangle`/`Path`/`TextRun` paint op lowering으로 확장해 `RawSvg` 의존을 줄인다.
3. maintainer가 browser에서도 ECharts/charming runtime을 요구하는 경우 별도 adapter 이슈로 분리한다. 기본 방향은 Rust/WASM + multi-backend 친화 경로로 유지한다.

## 5. 정답 PDF 대비 known visual gap

정답 PDF(`pdf-large/hwpx/143E433F503322BD33.pdf`)와 비교한 결과, 차트 데이터는 안정적으로 추출되지만 시각 차이는 남아 있다.

핵심 원인:

- 현재 `OleChart` IR은 title, categories, series 중심이며 axis/style/layout object graph를 아직 담지 않는다.
- y축 nice scale이 없어 현재 tick은 `0, 425.5, 851, 1276, 1702`로 생성된다. 정답 PDF는 `0, 500, 1000, 1500, 2000`이다.
- series palette, legend 위치, title spacing, plot margin, border, bar gap은 renderer 기본값이다.

이번 PR에서는 이 차이를 휴리스틱으로 보정하지 않는다. 정답 수준의 chart fidelity는 `VtChart`/axis/legend/style object graph parser 확장 후 후속 작업으로 진행한다.

상세 문서:

- `mydocs/tech/investigations/issue-1251/hwp_ole_chart_visual_diff_against_hancom_pdf_1251.md`
- `mydocs/tech/hwp_ole_chart_renderer_architecture_decision_1251.md`
- `mydocs/report/task_m100_1251_pr_draft.md`

## 6. PR 공유 메모

메인테이너에게 공유할 핵심 내용:

1. #1251 fixture의 `BinData #2`는 nested OLE 내부에 `OOXMLChartContents`, `OlePres000`, native image preview가 없고 legacy `Contents` 스트림만 가진다.
2. 이번 PR은 해당 `Contents`를 `VtDataGrid`/`VtChartTitle` 중심으로 파싱해 chart data를 복원하고, generic OLE placeholder 대신 최소 차트를 렌더한다.
3. `charming`은 parser가 아니므로 HWP OLE `/Contents` 해석은 rhwp 내부 parser가 담당한다.
4. `charming` native SSR은 renderer 후보로는 유효하지만, 통합 검증에서는 `deno_core/v8` 의존성이 현재 `cdylib` 링크에서 실패했다.
5. upstream이 `PageLayerTree`, Skia, CanvasKit 등 multi-backend renderer로 확장 중이므로, 기본 경로는 Rust SVG `RawSvg`로 두고 `charming`/ECharts 경로는 후속 adapter 검토로 분리했다.
6. 정답 PDF와의 시각 차이는 알려진 제한이다. 데이터는 맞지만 axis scale, palette, legend/title/layout style object graph는 아직 파싱하지 않는다.
7. pixel-level chart fidelity는 후속 작업으로 분리하는 것이 안전하다. 단기 휴리스틱으로 정답 PDF에 맞추면 #1251 fixture에 과적합될 위험이 있다.

## 7. PR 초안

PR 본문 초안은 `mydocs/report/task_m100_1251_pr_draft.md`에 별도로 정리했다.

초안에는 다음 항목을 포함했다.

1. 참고한 한컴 공식 문서와 Microsoft OLE/CFB 공식 문서
2. #1251 fixture가 legacy OLE `Contents` only chart인 배경
3. `charming`을 parser가 아닌 renderer 후보로만 검토한 이유와 최종 통합에서 제외한 이유
4. Rust SVG `RawSvg` renderer를 canonical path로 선택한 이유
5. 정답 PDF 대비 visual gap과 후속 작업 제안
