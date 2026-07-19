---
kind: decision
status: active
canonical: mydocs/tech/hwp_ole_chart_renderer_architecture_decision_1251.md
last_verified: 2026-07-17
---

# Task M100-1251 렌더러 선택 결정 기록

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **작성일**: 2026-06-03
- **결정**: Rust SVG renderer를 canonical path로 두고, `charming`은 검토 결과 후속 adapter 후보로 분리한다.

## 1. 배경

maintainer는 `yuankunzhang/charming` 라이브러리 활용을 지시했다. 조사 결과 `charming`은 Rust에서 Apache ECharts option을 구성하고 렌더러에 넘기는 라이브러리다.

렌더러는 크게 두 종류다.

1. native SSR `ImageRenderer`
   - `ssr` feature 필요
   - Deno/V8 기반으로 ECharts JS를 실행해 SVG 문자열을 반환
   - CLI `export-svg`나 native 검증에는 적합

2. WASM `WasmRenderer`
   - `wasm` feature 필요
   - SVG 문자열 반환 API가 아니라 DOM element id에 ECharts instance를 붙임
   - 브라우저에 global `echarts` JS가 필요
   - rhwp의 `PageLayerTree`/Skia/CanvasKit replay 흐름과 직접 맞지 않음

## 2. 검토한 선택지

### 선택지 A: charming WASM renderer를 Studio에 연결

구조:

```text
OleChart IR
→ charming::Chart
→ WasmRenderer.render(dom_id, chart)
→ browser ECharts DOM/canvas/svg side effect
```

장점:

- maintainer의 “charming 활용”을 가장 직접적으로 충족한다.
- 복잡한 차트 feature를 ECharts에 맡길 수 있다.

단점:

- browser global `echarts` JS dependency가 추가된다.
- DOM lifecycle, resize, z-order, clipping, overlay 관리를 Studio가 알아야 한다.
- native Skia/PDF/CanvasKit downstream에서 같은 경로를 재사용하기 어렵다.
- `PageLayerTree` 밖의 side effect가 되어 multi-backend renderer 방향과 충돌한다.

### 선택지 B: Rust SVG renderer를 canonical path로 사용

구조:

```text
OleChart IR
→ Rust SVG renderer
→ RawSvg PaintOp
→ native SVG / WASM Canvas / Skia / CanvasKit 경로가 소비
```

장점:

- Rust/WASM 공유 코드라는 프로젝트 철학에 맞다.
- downstream renderer는 DOM/ECharts를 몰라도 된다.
- `PageLayerTree` 안의 paint op로 남아 z-order/clipping/profile 정책을 통합하기 쉽다.
- 향후 `Line`/`Rectangle`/`Path`/`TextRun` lowering으로 발전시킬 수 있다.

단점:

- ECharts 수준의 복잡한 chart feature를 직접 구현해야 한다.
- 초기 시각 품질은 한컴 chart engine과 차이가 난다.

## 3. 최종 결정

기본 경로는 선택지 B로 결정했다.

```text
HWP OLE Contents
→ Rust parser
→ OleChart IR
→ Rust SVG renderer
→ RawSvg
→ native/WASM 공통 소비
```

`charming`은 다음 용도로 검토했다.

- native 고품질 export/비교용 adapter
- maintainer 지시를 반영한 Rust chart adapter 검증 경로
- 후속 논의에서 ECharts backend가 필요한 downstream을 위한 optional feature

다만 통합 검증에서 `charming`의 `ssr` feature가 끌어오는 `deno_core/v8`가 현재 `cdylib` crate-type 링크와 충돌했다.

```text
rust-lld: error: relocation R_X86_64_TPOFF32 against v8::internal::g_current_isolate_
cannot be used with -shared
```

따라서 PR #1268 통합본에서는 `charming-renderer` feature와 `charming` dependency를 제외하고, Rust SVG renderer만 canonical path로 유지한다.

## 4. PR에서 설명할 점

- `charming`은 parser가 아니므로 HWP OLE `/Contents` 해석은 rhwp가 직접 수행해야 한다.
- `charming` native SSR은 renderer 후보로는 유효하지만 현재 crate-type과의 링크 호환성 검증이 더 필요하다.
- rhwp upstream은 `PageLayerTree`, Skia, CanvasKit 등 multi-backend renderer로 확장 중이므로 chart도 renderer-neutral path에 올리는 것이 장기적으로 낫다.
- 따라서 `charming`을 기본 renderer로 강제하지 않고 후속 adapter 이슈로 분리한다.

## 5. 후속 판단 지점

maintainer가 “browser Studio에서도 반드시 charming/ECharts runtime으로 렌더하라”고 명시하면 선택지 A를 재검토해야 한다. 다만 그 경우에도 다음 비용을 PR에 명확히 적어야 한다.

- `echarts` JS asset 또는 npm dependency 추가
- WASM/JS bridge 및 DOM lifecycle 관리
- Skia/PDF/CanvasKit backend와의 재사용성 저하
- PageLayerTree 밖 overlay side effect 증가
