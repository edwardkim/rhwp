# Task M100-1251 구현 계획서

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **수행 계획서**: `mydocs/plans/task_m100_1251.md`
- **브랜치**: `task-1251-ole-chart`
- **작성일**: 2026-06-03
- **상태**: Stage 8 PR 초안 완료

## 1. 구현 원칙

1. `charming`은 HWP OLE `/Contents` 파서를 대체하지 않는다. 먼저 `/Contents`를 rhwp 내부 차트 IR로 파싱한다.
2. 기본 렌더링 경로는 Rust `OleChart` SVG renderer로 통일한다. native `export-svg`와 WASM Studio는 같은 SVG fragment를 `RawSvg`로 소비한다.
3. `charming`은 native 고품질 export/비교용 optional adapter로 유지한다. 기본 빌드에서는 `charming-renderer` feature를 켜지 않으면 컴파일하지 않는다.
4. WASM core는 DOM id 기반 `WasmRenderer`를 직접 호출하지 않는다. Stage 5의 Studio 전용 TS chart overlay도 canonical 경로에서 제거한다.
5. 기존 `src/ooxml_chart`의 OOXML 차트 파서/자체 SVG 렌더러는 건드리지 않는다. 새 경로는 OLE `/Contents` 전용으로 좁힌다.
6. `/Contents` 파싱이 실패해도 기존 generic placeholder로 묻히지 않게 실패 사유를 안정적인 문자열로 노출한다.

## 2. 대상 파일

### 신규 후보

| 파일 | 목적 |
|---|---|
| `src/ole_chart/mod.rs` | OLE `/Contents` 기반 차트 IR과 public API |
| `src/ole_chart/parser.rs` | `Contents` 스트림 최소 파서 |
| `src/ole_chart/ir.rs` | renderer-neutral IR JSON/base64 payload |
| `src/ole_chart/svg_renderer.rs` | canonical Rust `OleChart` SVG renderer |
| `src/ole_chart/charming_renderer.rs` | optional `OleChart` → `charming::Chart` → SVG 문자열 변환 |
| `tests/issue_1251_ole_chart_contents.rs` | fixture 기반 회귀 테스트 |
| `mydocs/working/task_m100_1251_stage{N}.md` | 단계별 완료 보고 |

### 수정 후보

| 파일 | 변경 |
|---|---|
| `Cargo.toml` | `charming` 네이티브 target 전용 dependency 후보 추가 |
| `Cargo.lock` | `charming` 도입 시 갱신 |
| `src/lib.rs` | `pub mod ole_chart;` 추가 |
| `src/parser/ole_container.rs` | `/Contents` 스트림 전달/진단 helper 보강 |
| `src/renderer/layout/shape_layout.rs` | OLE render 우선순위에 `/Contents` 차트 경로 추가 |
| `rhwp-studio/src/view/page-renderer.ts` | Stage 5 임시 OLE chart DOM overlay 합성 제거 |

## 3. Dependency 계획

최종 적용:

```toml
[features]
default = ["console_error_panic_hook"]
charming-renderer = ["dep:charming"]

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
charming = { version = "0.6", optional = true, default-features = false, features = ["ssr"] }
```

근거:

- `charming 0.6.0`의 `ImageRenderer`는 `ssr` feature에서만 사용 가능하며 SVG 문자열을 반환한다.
- `ssr` feature는 `html`, `deno_core`, `serde_v8`를 포함한다.
- `wasm` feature는 DOM id 렌더 API이며 `RawSvg` 문자열 렌더 경로가 아니다.
- 현재 로컬 `rustc`는 `1.93.1`이므로 `charming`의 `rust-version=1.85` 자체는 로컬 빌드에는 문제가 없다.

검증 후 결정:

- `charming` dependency는 기본 native 빌드에서도 항상 컴파일하지 않고 `charming-renderer` feature 뒤로 분리했다.
- WASM 기본 빌드는 `charming`에 의존하지 않는다.

## 4. 렌더 우선순위

현재 OLE 경로:

```text
HWPX ooxml_chart 직접 XML
→ nested OLE OOXMLChartContents
→ OlePres000 EMF preview
→ native image
→ generic OLE placeholder
```

변경 후보:

```text
HWPX ooxml_chart 직접 XML
→ nested OLE OOXMLChartContents
→ nested OLE Contents + ole_chart parser + Rust SVG RawSvg
→ OlePres000 EMF preview
→ native image
→ explicit OLE chart fallback
→ generic OLE placeholder
```

주의:

- `/Contents`가 존재한다고 항상 차트로 단정하지 않는다.
- `SHAPE_COMPONENT_OLE`의 `bin_data_id=2`와 fixture 특성을 우선 회귀 대상으로 삼고, 일반 OLE에는 기존 fallback을 유지한다.

## 5. Stage별 구현

### Stage 1 — `/Contents` 스트림 진단과 최소 파서 골격

목표:

- `src/ole_chart/parser.rs`에 `parse_ole_chart_contents(bytes: &[u8]) -> Result<OleChart, OleChartParseError>`를 추가한다.
- 첫 단계에서는 헤더, magic 후보, record boundary, 문자열/숫자 후보를 안전하게 스캔한다.
- 파싱 실패 시 오류 enum을 안정화한다.

완료 조건:

- `BinData #2`가 `/Contents` only OLE임을 테스트로 확인
- 파싱 실패도 `UnsupportedContentsLayout`처럼 안정적인 오류로 떨어짐
- `mydocs/working/task_m100_1251_stage1.md` 작성

### Stage 2 — `charming` 네이티브 SSR 스파이크

목표:

- `Cargo.toml`에 target-specific `charming` 후보를 추가한다.
- 임의의 최소 `OleChart` 값을 `charming::Chart`로 변환하고 `ImageRenderer::render`가 SVG 문자열을 반환하는지 확인한다.
- 빌드 영향과 WASM 영향 가능성을 기록한다.

완료 조건:

- `cargo build` 성공
- `cargo test --test issue_1251_ole_chart_contents -- --nocapture`에서 sample chart SVG 생성 smoke test 통과
- WASM 빌드 필요 시 `docker compose --env-file .env.docker run --rm wasm` 결과 기록
- `mydocs/working/task_m100_1251_stage2.md` 작성

### Stage 3 — fixture 최소 데이터 추출

목표:

- `/Contents`에서 이 fixture에 필요한 최소 차트 데이터를 추출한다.
- 추출 가능 후보는 다음 순서로 확정한다.
  1. 차트 종류
  2. 카테고리 라벨
  3. 시리즈 값
  4. 시리즈 이름/범례
  5. 제목
- 불명확한 필드는 파싱하지 않고 `None` 또는 기본값으로 둔다.

완료 조건:

- fixture에서 `OleChart`가 생성되거나, 불가능 사유가 오류 enum으로 고정됨
- 파싱 성공 시 최소 bar/line 렌더용 값 목록이 비어 있지 않음
- `mydocs/working/task_m100_1251_stage3.md` 작성

### Stage 3.5 — 한컴 차트 사양 기반 legacy Contents parser 보강

목표:

- `한글문서파일형식_차트_revision1.2.pdf`의 `ChartOBJ` 기본 구조를 현재 `/Contents` parser에 반영한다.
- `Contents` 첫 16바이트 probe에서 `ChartOBJ` 시작 오프셋을 기록한다.
- `VtDataGrid`, `VtChartTitle` marker 존재 여부를 probe에 고정한다.
- grid 값 추출은 무조건적인 `f64` 후보 누적이 아니라 dense run 우선, 실패 시 기대 개수와 정확히 일치하는 sparse 후보만 허용한다.

완료 조건:

- #1251 fixture에서 기존 추출 결과가 유지됨
- synthetic unit test로 marker probe와 dense/sparse value extraction 정책을 고정함
- `mydocs/working/task_m100_1251_stage3_5.md` 작성

### Stage 4 — renderer 통합

목표:

- `shape_layout.rs`에서 `container.raw_contents`가 존재할 때 `ole_chart` 경로를 호출한다.
- Stage 4 당시에는 native `charming` SVG와 WASM browser IR overlay를 연결했다.
- Stage 6 이후 최종 경로는 native/WASM 모두 Rust SVG `RawSvg` fragment를 삽입한다.
- 실패 경로에서는 안정적인 fallback label을 생성한다.

fallback label 후보:

```text
OLE 차트 미지원: Contents 파싱 실패 (BinData #2)
```

완료 조건:

- 성공 경로면 `export-svg` 결과에서 `OLE 개체 (BinData #2)`가 사라짐
- Studio 성공 경로면 `charming SSR unavailable on wasm` 문구가 사라지고 `PageLayerTree`의 `rawSvg`에 `hwp-ole-chart-rust-svg`가 존재함
- 실패 경로면 위 fallback label 중 하나가 SVG에 존재함
- 기존 EMF/native image fallback 경로 회귀 없음
- `mydocs/working/task_m100_1251_stage4.md` 작성

### Stage 5 — 검증과 보고

목표:

- 신규 fixture 테스트와 기존 레이아웃 테스트를 실행한다.
- `charming` 도입이 clippy baseline을 깨지 않는지 확인한다.
- 최종 보고서를 작성한다.

검증 명령:

```text
cargo fmt --check
cargo test --test issue_1251_ole_chart_contents -- --nocapture
cargo test --lib ole_chart -- --nocapture
cargo test --features charming-renderer --test issue_1251_ole_chart_contents -- --nocapture
cargo test --test issue_1156_chart_column_flow -- --nocapture
cargo build
cargo check --target wasm32-unknown-unknown --lib
cargo clippy --all-targets -- -D warnings
npm run build
wasm-pack build --target web
target/debug/rhwp export-svg samples/143E433F503322BD33.hwp -o output/poc/task1251/hwp
node /private/tmp/rhwp_issue_1251_qa.mjs
```

Docker compose는 로컬 Docker daemon 미실행으로 사용할 수 없어, 로컬 `wasm-pack build --target web`로 `pkg/`를 재생성했다.

완료 산출물:

- `mydocs/working/task_m100_1251_stage5.md`
- `mydocs/report/task_m100_1251_report.md`

### Stage 6 — Rust SVG canonical renderer 전환

목표:

- Stage 5의 Studio TypeScript overlay를 제거한다.
- native와 WASM 모두 `src/ole_chart/svg_renderer.rs`의 Rust SVG renderer를 사용한다.
- `charming`은 `charming-renderer` feature 뒤 optional adapter로 분리한다.
- PageLayerTree/Skia/CanvasKit 확장 방향에 맞춰 차트가 DOM side effect가 아니라 `RawSvg` paint op로 남게 한다.

완료 조건:

- 기본 `cargo test --test issue_1251_ole_chart_contents`에서 Rust SVG fragment/standalone SVG 경로가 통과함
- `cargo test --features charming-renderer --test issue_1251_ole_chart_contents`에서 charming adapter smoke가 통과함
- `cargo check --target wasm32-unknown-unknown --lib`, `wasm-pack build --target web`, `npm run build` 통과
- Studio QA에서 `.rhwp-ole-chart-overlay-layer`와 `.hwp-ole-chart-browser`가 0개이며, `PageLayerTree`의 `rawSvg`에 `hwp-ole-chart-rust-svg`가 존재함

### Stage 7 — 정답 PDF 대비 시각 차이 분석과 결정 문서화

목표:

- `pdf-large/hwpx/143E433F503322BD33.pdf`를 정답지로 삼아 현재 Rust SVG chart와의 시각 차이를 분석한다.
- 이번 PR에서 pixel-level 정합성 보강까지 진행할지, 또는 known gap을 문서화하고 후속 이슈로 분리할지 결정한다.
- maintainer에게 공유할 renderer 선택 이유와 `charming` 사용 범위를 별도 기술 문서로 남긴다.

완료 조건:

- `mydocs/tech/hwp_ole_chart_visual_diff_against_hancom_pdf_1251.md` 작성
- `mydocs/tech/hwp_ole_chart_renderer_architecture_decision_1251.md` 작성
- `mydocs/working/task_m100_1251_stage7.md` 작성
- 최종 보고서에 known visual gap과 후속 작업 후보 반영

### Stage 8 — PR 초안 작성과 공유 자료 정리

목표:

- PR 본문에 포함할 공식 문서, 결정 배경, 결정한 것과 이유, 분석 문서 링크를 정리한다.
- `charming`을 기본 WASM renderer로 쓰지 않은 이유와 Rust SVG canonical renderer를 선택한 이유를 reviewer가 바로 확인할 수 있게 한다.
- PR 생성 전 rebase 필요성과 staging 제외 대상을 기록한다.

완료 조건:

- `mydocs/report/task_m100_1251_pr_draft.md` 작성
- `mydocs/working/task_m100_1251_stage8.md` 작성
- 최종 보고서에 PR 초안 산출물 반영

## 6. 테스트 설계

`tests/issue_1251_ole_chart_contents.rs`

1. `fixture_has_bin_data_2_ole_contents_only`
   - `parse_document` 후 `bin_data_content`에서 id `2`, extension `OLE` 확인
   - `parse_ole_container` 결과 `raw_contents.is_some()`
   - `ooxml_chart`, `preview_emf`, `native_image`가 없음

2. `ole_chart_contents_parse_result_is_stable`
   - `parse_ole_chart_contents(raw_contents)` 호출
   - 성공이면 `series`/`categories`가 기대 최소 조건을 만족
   - 실패이면 오류 code 문자열이 기대값과 일치

3. `issue_1251_svg_does_not_use_ambiguous_placeholder`
   - `render_page_svg(0)` 호출
   - `OLE 개체 (BinData #2)` generic placeholder가 남지 않는지 확인
   - 성공 렌더면 `hwp-ole-chart` 또는 `echarts` SVG marker 확인
   - fallback이면 `OLE 차트 미지원:` 사유 marker 확인

## 7. 중단 조건

다음 중 하나가 발생하면 구현을 멈추고 작업지시자 승인을 다시 받는다.

- `charming` dependency가 WASM 빌드를 깨는 경우
- `deno_core` 도입으로 CI 빌드 시간이 과도하게 증가하는 경우
- `/Contents` 포맷에서 차트 데이터 구조를 안정적으로 특정할 수 없는 경우
- fallback-only 처리만 가능한데 이슈 완료로 인정할지 판단이 필요한 경우

## 8. 승인 요청 범위

승인 후 Stage 1부터 진행한다. Stage 1은 소스 변경을 포함한다.

승인 대상:

1. `src/ole_chart/*` 신규 모듈 추가
2. fixture 기반 테스트 추가
3. Stage 2에서 `charming` 네이티브 target 전용 dependency 추가 및 검증
4. 실패 시 명시적 fallback으로 회귀 테스트를 고정하는 방향
