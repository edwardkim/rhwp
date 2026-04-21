# RHWP Layered Renderer Architecture

## 1. 목적

이 문서는 현재 rhwp의 멀티 렌더러 구조를 구현 기준으로 설명한다.
특히 다음 내용을 정리한다.

- `PageRenderTree`와 `PageLayerTree`의 역할 차이
- legacy SVG, layer SVG, browser Canvas2D, browser CanvasKit, native Skia의 현재 관계
- CanvasKit compat/default mode의 의미
- 스크린샷 diff 기반 parity 테스트 전략
- 새 백엔드나 새 paint op를 추가할 때 수정해야 하는 지점

상위 수준의 역사적 설계 배경은 [rendering_engine_design.md](./rendering_engine_design.md)를 본다.

## 2. 현재 렌더링 경로 한눈에 보기

rhwp는 더 이상 “하나의 render tree를 각 백엔드가 직접 해석”하는 구조만으로 설명되지 않는다.
현재는 두 단계의 표현을 사용한다.

```text
Document / Section / Paragraph / Control
  -> compose / paginate / layout
  -> PageRenderTree
  -> LayerBuilder
  -> PageLayerTree
  -> backend replay
```

실제 경로는 백엔드마다 다음과 같이 나뉜다.

| 경로 | 입력 | 주요 구현 파일 | 현재 역할 |
|---|---|---|---|
| Legacy SVG | `PageRenderTree` | `src/document_core/queries/rendering.rs`, `src/renderer/svg.rs` | 기존 기준 경로, 구조 비교 baseline |
| Layer SVG | `PageLayerTree` | `src/paint/*`, `src/renderer/svg_layer.rs` | layered replay 검증 경로 |
| Browser Canvas2D | `PageRenderTree` | `src/wasm_api.rs`, `src/renderer/web_canvas.rs`, `rhwp-studio/src/view/page-renderer.ts` | 현재 웹 baseline 렌더러 |
| Browser CanvasKit | `PageLayerTree` | `src/wasm_api.rs`, `rhwp-studio/src/view/canvaskit-renderer.ts` | layered browser backend |
| Native Skia | `PageLayerTree` | `src/renderer/skia/renderer.rs` | layered raster backend |

핵심 포인트는 다음 두 가지다.

1. 웹 baseline인 Canvas2D는 아직 기존 WASM Canvas 렌더링 경로를 사용한다.
2. 새 backend인 layer SVG, CanvasKit, native Skia는 모두 `PageLayerTree`를 소비한다.

즉 현재 구조는 “모든 백엔드가 같은 path를 쓴다”가 아니라, “기존 baseline은 유지하고 새 backend는 layered path로 수렴한다”에 가깝다.

또 하나 중요한 점은, 현재 공통 계약이 완전히 하나로 닫혀 있지는 않다는 것이다.

- `LayerRenderer` trait는 layered SVG처럼 stateful output을 누적하는 backend에 맞춘 좁은 전환기 계약이다.
- native Skia는 아직 `render_png()`처럼 바이트를 직접 돌려주는 명시적 API를 유지한다.

즉 “모든 layered backend가 동일한 Rust trait를 이미 공유한다”기보다는,
“공통 입력 IR은 `PageLayerTree`로 정리했고, 출력 계약은 backend 특성에 맞춰 아직 전환 중”이라고 보는 편이 정확하다.

## 3. 왜 `PageLayerTree`가 필요했는가

`PageRenderTree`는 레이아웃 결과를 표현하기에는 적절하지만, 백엔드 replay용 IR로는 너무 semantic하다.
예를 들어 `Header`, `Footer`, `Table`, `TextLine`, `TextBox`, `Group` 같은 문서적 개념이 포함되어 있고,
backend마다 이를 다시 해석해야 한다.

이 구조는 다음 문제를 만든다.

- 새 raster/vector backend를 추가할 때 semantic container 해석을 다시 구현해야 한다.
- SVG/Canvas/Skia 간 합성 순서와 clip/transform 처리를 동일하게 맞추기 어렵다.
- browser CanvasKit과 native Skia가 같은 입력을 소비하지 못한다.

`PageLayerTree`는 이 문제를 해결하기 위해 도입되었다.

- semantic container를 시각 레이어 단위로 내린다.
- backend가 실제로 필요한 정보만 남긴다.
- clip, group, leaf paint op를 명시적으로 표현한다.
- backend는 layout이 아니라 replay에만 집중한다.

`src/paint/mod.rs`가 이 계층의 진입점이고, `src/paint/builder.rs`의 `LayerBuilder`가
`PageRenderTree -> PageLayerTree` 변환을 담당한다.

## 4. `PageRenderTree`와 `PageLayerTree`의 역할

| 항목 | `PageRenderTree` | `PageLayerTree` |
|---|---|---|
| 주 목적 | 레이아웃 결과 표현 | backend replay 입력 |
| 포함 정보 | 문서 의미 + 레이아웃 결과 | 시각 합성 정보 |
| 노드 성격 | semantic container + leaf | group / clip / leaf op |
| 대표 타입 | `RenderNodeType::Table`, `TextLine`, `Group` | `LayerNodeKind::Group`, `ClipRect`, `Leaf` |
| 소비자 | legacy SVG, 디버그/쿼리 계층 | layer SVG, native Skia, CanvasKit |
| backend 재해석 필요성 | 높음 | 낮음 |

실무적으로는 아래처럼 생각하면 된다.

- `PageRenderTree`는 “문서가 어떻게 조판되었는가”를 설명한다.
- `PageLayerTree`는 “그 결과를 어떤 순서로 어떻게 그릴 것인가”를 설명한다.

## 5. Layered path의 핵심 규칙

### 5.1 layout은 한 번만 한다

backend가 텍스트 줄바꿈, 문단 조판, 표 배치, 도형 내부 레이아웃을 다시 계산하면 안 된다.
그 책임은 `compose / paginate / layout` 단계에 있다.

### 5.2 backend는 replay만 한다

backend는 다음만 담당해야 한다.

- paint op 순서 재생
- clip 적용
- transform 적용
- raster/vector 출력
- browser-specific fallback 또는 compat 처리

### 5.3 semantic 정보는 필요한 만큼만 남긴다

`GroupKind`, `ClipKind`, `CacheHint` 같은 메타데이터는 남기되,
문서 의미를 backend가 다시 해석해야 할 정도로 semantic 정보를 넣지는 않는다.

### 5.4 shape 자식은 보존되어야 한다

shape 자체가 leaf paint op를 가지더라도, 그 안의 이미지 채우기, 글상자 텍스트, 그룹 내부 자식은
layer tree에서 사라지면 안 된다.

이 점은 layered path에서 중요한 invariant다.
최근 `group-drawing-02` parity 이슈도 shape leaf를 내리면서 자식을 버리면 안 된다는 점을 다시 확인한 사례다.

## 6. 백엔드별 현재 동작

### 6.1 Legacy SVG

- 진입점: `DocumentCore::render_page_svg_legacy_native()`
- 구현: `src/renderer/svg.rs`
- 입력: `PageRenderTree`

이 경로는 완전히 제거된 것이 아니라, layered path와의 구조 비교 기준으로 계속 남아 있다.
`RHWP_RENDER_PATH=layer-svg`를 지정하지 않으면 기본적으로 이 경로가 SVG 내보내기에 사용된다.

### 6.2 Layer SVG

- 진입점: `DocumentCore::render_page_svg_layer_native()`
- 구현: `src/renderer/svg_layer.rs`
- 입력: `PageLayerTree`

`SvgLayerRenderer`는 layer tree를 다시 temporary render tree 형태로 조립해
기존 SVG leaf 로직을 재사용한다.
즉 layer tree 기반이지만, SVG 출력 품질을 맞추기 위해 기존 SVG renderer를 완전히 버리지는 않았다.

### 6.3 Browser Canvas2D

- 진입점: `PageRenderer.renderPage()`에서 `this.wasm.renderPageToCanvas(...)`
- 구현: `src/renderer/web_canvas.rs`
- 입력: `PageRenderTree`

현재 브라우저에서의 baseline이다.
CanvasKit parity 테스트도 이 경로의 스크린샷을 기준으로 비교한다.

중요한 점은 Canvas2D가 아직 layered path로 전환되지 않았다는 것이다.
즉 CanvasKit parity는 “같은 tree를 두 backend가 replay하는 exact sibling test”가 아니라,
“새 layered backend가 기존 baseline과 얼마나 가깝게 보이는가”를 검증하는 테스트다.

### 6.4 Browser CanvasKit

- 진입점: `PageRenderer.renderPage()`에서 `this.wasm.getPageLayerTree(...)`
- 구현: `rhwp-studio/src/view/canvaskit-renderer.ts`
- 입력: `PageLayerTree`

CanvasKit은 browser-side replay renderer다.
Rust core가 layout과 layer tree export를 담당하고, TypeScript가 CanvasKit API 호출로 이를 그린다.

이 경로를 둔 이유는 다음과 같다.

- native Skia와 유사한 2D drawing model 확보
- 브라우저에서 Skia 계열 renderer 실험
- 추후 native/backend 확장성 확보

### 6.5 Native Skia

- 진입점: `DocumentCore::render_page_png_native()`
- 구현: `src/renderer/skia/renderer.rs`
- feature: `native-skia`
- 입력: `PageLayerTree`

native Skia는 non-wasm 타깃에서 layered raster backend 역할을 한다.
현재는 테스트/검증용 경로가 중심이며, 별도의 일반 사용자용 `export-png` CLI는 아직 없다.

### 6.6 RenderProfile 기본값

`RenderProfile`은 layered 출력 경로가 어떤 품질/캐시 힌트를 기본으로 택할지 나타내는 enum이다.
아직 모든 profile이 큰 동작 차이를 만드는 것은 아니지만, 호출 경로에는 기본값이 명시되어 있다.

| 경로 | 기본 profile |
|---|---|
| browser layer tree (`getPageLayerTree`) | `Screen` |
| layer SVG export | `Print` |
| native Skia PNG | `HighQuality` |

추가로 `RHWP_RENDER_PROFILE` 환경 변수로 `screen`, `print`, `high-quality`, `fast-preview`를 지정해
기본값을 덮어쓸 수 있다.

현재 `FastPreview`는 page background 쪽 cache hint만 다르게 적용하며,
더 적극적인 preview simplification을 위한 예약 성격이 강하다.

## 7. CanvasKit render mode

CanvasKit에는 현재 두 가지 모드가 있다.

| 모드 | 의미 | 기본값 |
|---|---|---|
| `default` | CanvasKit 기본 동작 우선 | 아님 |
| `compat` | Canvas2D와의 시각적 유사도 우선 | 기본 |

`rhwp-studio/src/view/render-backend.ts`에서 query param과 localStorage를 통해 이 값을 결정한다.

`compat`가 기본인 이유는 다음과 같다.

- 브라우저 baseline은 아직 Canvas2D다.
- 텍스트 rasterization, font fallback, glyph positioning에서 CanvasKit과 Canvas2D는 그대로는 많이 다를 수 있다.
- 사용자가 backend를 바꿨을 때 “렌더링이 달라 보인다”는 인상을 최소화해야 한다.

현재 compat mode는 특히 텍스트 계열에서 Canvas2D overlay/fallback을 사용해
CanvasKit의 순수 raster 차이를 흡수한다.
이 로직은 browser-specific compatibility layer이며, Rust core의 layout 자체를 바꾸는 것은 아니다.

## 8. Parity와 diff 전략

렌더러 parity는 “무조건 exact diff 0”만으로 관리하지 않는다.
backend 특성상 anti-aliasing, subpixel coverage, font rasterization 차이가 생길 수 있기 때문이다.

현재 검증 전략은 다음과 같다.

### 8.1 Legacy SVG vs Layer SVG

- 성격: 구조 전환 검증
- 기준: exact match 중심
- 산출물: `output/layer-svg-diff/`

### 8.2 Layer SVG vs Native Skia PNG

- 성격: layered raster backend 검증
- 기준: tolerant diff pixel budget 사용
- 산출물: `output/skia-diff/`

### 8.3 Browser Canvas2D vs CanvasKit

- 성격: browser parity 검증
- 테스트 파일: `rhwp-studio/e2e/canvaskit-render.test.mjs`
- 기준:
  - exact diff는 항상 기록
  - tolerant diff는 채널 차이 `8` 이하 픽셀을 무시
  - 최종 pass/fail은 tolerant diff ratio `0.25%` 이하
- 산출물:
  - `output/e2e/`
  - `rhwp-studio/e2e/screenshots/`

이 전략의 의도는 다음과 같다.

- exact diff는 계속 남겨서 변화량을 추적한다.
- 하지만 통과 기준은 renderer 엔진 차이만 허용하는 tolerant 값으로 잡는다.
- 즉 “눈에 띄는 구조 차이”와 “작은 raster 차이”를 구분한다.

## 9. 테스트가 보호해야 하는 것

현재 parity 테스트는 단순 픽셀 비교가 아니라 layered architecture의 invariant를 지키는 장치다.

특히 다음 문제를 잡아내야 한다.

- shape leaf를 만들면서 shape 자식을 누락하는 경우
- group/clip 계층이 flatten되면서 draw order가 바뀌는 경우
- CanvasKit에서 텍스트 fallback이 빠져 한글/수식이 깨지는 경우
- equation, crop, field, group drawing처럼 backend 차이가 잘 드러나는 샘플 회귀

즉 스크린샷 테스트는 “예쁘게 보이는지” 이상의 의미를 가진다.
현재 layered path가 기존 baseline을 어느 정도 유지하는지 보여주는 계약 테스트다.

## 10. 새 backend를 추가할 때의 작업 순서

새 backend를 붙일 때는 보통 아래 순서를 따른다.

1. `PageLayerTree`를 입력으로 받을지 먼저 결정한다.
2. `LayerNodeKind::Group / ClipRect / Leaf` replay 전략을 정의한다.
3. 필요한 `PaintOp`를 backend별 draw call로 매핑한다.
4. browser backend면 `paint/json.rs`와 TS 타입까지 같이 맞춘다.
5. 기존 baseline과의 parity 테스트를 추가한다.

수정 지점은 대체로 다음 파일들이다.

| 목적 | 주요 파일 |
|---|---|
| layer tree 생성 | `src/paint/builder.rs` |
| layer tree JSON export | `src/paint/json.rs` |
| layered SVG replay | `src/renderer/svg_layer.rs` |
| native Skia replay | `src/renderer/skia/renderer.rs` |
| wasm export | `src/document_core/queries/rendering.rs`, `src/wasm_api.rs` |
| browser CanvasKit replay | `rhwp-studio/src/view/canvaskit-renderer.ts` |
| browser parity test | `rhwp-studio/e2e/canvaskit-render.test.mjs` |

## 11. 새 paint op를 추가할 때의 체크리스트

새 `PaintOp`를 추가하면 보통 아래를 같이 확인해야 한다.

1. `src/paint/paint_op.rs`에 타입 추가
2. `src/paint/builder.rs`에서 해당 render node를 layer op로 변환
3. `src/paint/json.rs` 직렬화 추가
4. `src/renderer/svg_layer.rs` replay 경로 반영
5. `src/renderer/skia/renderer.rs` replay 경로 반영
6. browser에서 쓰면 TS layer type + CanvasKit replay 반영
7. parity test 샘플 추가

이 중 하나라도 빠지면 “특정 backend에서만 안 보임” 같은 비대칭 문제가 쉽게 생긴다.

## 12. 현재 구조를 해석할 때 주의할 점

### 12.1 Canvas2D와 CanvasKit은 아직 완전히 대칭이 아니다

Canvas2D는 legacy browser path이고, CanvasKit은 layered browser path다.
둘은 같은 renderer 구현체가 아니다.

### 12.2 Layered path가 semantic tree를 완전히 대체한 것은 아니다

legacy SVG와 여러 query/debug 기능은 여전히 `PageRenderTree`에 기대고 있다.
따라서 현재는 “semantic tree 제거”가 아니라 “layered replay를 병행 도입”한 단계다.

### 12.3 compat 코드는 backend 불일치를 숨기는 완충층이다

compat mode는 아키텍처 오염이라기보다,
현재 baseline을 보존하면서 새 backend를 도입하기 위한 전환 비용으로 보는 편이 맞다.

다만 이 코드는 가능한 한 `rhwp-studio` 쪽 browser layer에만 머물러야 하고,
Rust layout core로 역류하면 안 된다.

## 13. 요약

현재 rhwp의 layered renderer 구조는 다음으로 요약할 수 있다.

- layout 결과는 `PageRenderTree`로 만들어진다.
- backend replay용 시각 IR은 `PageLayerTree`다.
- layer SVG, native Skia, CanvasKit은 `PageLayerTree`를 공유한다.
- browser Canvas2D는 아직 baseline으로 유지된다.
- 따라서 현재 parity 테스트는 “새 layered backend가 기존 baseline과 얼마나 가까운가”를 검증한다.
- 새 backend를 추가할 때 가장 중요한 원칙은 “layout을 다시 하지 말고 layer tree를 replay하라”이다.
