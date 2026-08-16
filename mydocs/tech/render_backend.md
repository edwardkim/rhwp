---
kind: reference
status: active
canonical: mydocs/tech/render_backend.md
last_verified: 2026-08-16
---

# 출력 백엔드 공통 계약 — `RenderBackend`

`src/render_backend/` 가 정의하는 **출력 백엔드 공통 trait** 의 계약 문서다.
기존 SVG·Skia·PDF·Canvas 출력 경로를 고치지 않고 그 위에 얹는 새 추상 계층이며,
이 문서는 (1) 기존 경로 실측, (2) 새 계약과 불변식, (3) 기존 백엔드의 채택
시나리오, (4) 비범위를 담는다.

## 1. 왜 이 계층이 필요한가

[프로젝트 로드맵](../../ROADMAP.md)은 업스트림 책임을 이렇게 적는다.

> | 공통 문서 엔진 | HWP·HWPX·HWP3·HML 파싱, 내부 문서 구조, 조판, 편집, 저장과 **여러 출력 방식** |
> — `ROADMAP.md:187`

"여러 출력 방식"은 이미 여섯 갈래로 자랐지만, 그것들을 **하나로 묶는 계약이 없다**.
그래서 두 가지가 막힌다.

1. **새 백엔드를 붙일 형틀이 없다.** 무엇을 받아 무엇을 내야 하는지, 실패를 어떻게
   말해야 하는지, 페이지 경계를 어떻게 다뤄야 하는지가 문서화된 계약이 아니라
   기존 파일을 읽어 유추할 대상이다.
2. **백엔드 간 정합을 검증할 공통 어휘가 없다.** "SVG 와 PDF 가 같은 페이지를 같게
   그렸는가"를 물으려면 지금은 픽셀을 비교하는 수밖에 없고, 차이가 나도 *조판이 다른
   op 를 준 것*인지 *같은 op 를 다르게 그린 것*인지 갈라낼 수 없다.

`PaintOp` 는 이미 이 계층을 예비해 둔 자리다 — 정의부 주석이 스스로 그렇게 말한다.

> `backend가 재생하는 leaf paint operation.`
> — `src/paint/paint_op.rs:43`

없던 것은 **그 op 를 받는 쪽의 계약**이다. 이 PR 이 그것을 신설한다.

## 2. 1단계 조사 — 기존 출력 경로 실측

### 2.1 진입점·입력·산출·오류

| 백엔드 | 진입점 | 입력 | 산출 | 오류 |
| --- | --- | --- | --- | --- |
| SVG (렌더트리) | `src/renderer/svg.rs:233` `render_tree(&mut self, tree: &PageRenderTree)` | `&PageRenderTree` | 반환 없음 — 내부 `String` 버퍼에 누적, `svg.rs:216` `output() -> &str` 로 꺼냄 | **없음**(무오류 서명) |
| SVG (레이어트리) | `src/renderer/svg_layer.rs:239` `LayerRenderer::render_page` | `&PageLayerTree` | 위와 같은 내부 버퍼 | `HwpError`(실제로는 항상 `Ok`) |
| HTML | `src/renderer/html.rs:47` `render_tree(&mut self, tree: &PageRenderTree)` | `&PageRenderTree` | 내부 `String` 버퍼(`html.rs:42`) | **없음** |
| Canvas(네이티브/테스트) | `src/renderer/canvas.rs:83`·`:88` | 둘 다 | `Vec<CanvasCommand>` | **없음** |
| Web Canvas(wasm) | `src/renderer/web_canvas.rs:471`·`:476` | 둘 다 | **없음 — 부수효과**(`CanvasRenderingContext2d`) | 생성자만 `JsValue`(`web_canvas.rs:388`) |
| Skia(`native-skia`) | `src/renderer/skia/renderer.rs:372` `render_raster_with_options(&self, &PageLayerTree, RasterRenderOptions)` | `&PageLayerTree` + 옵션 | `RasterRenderOutput { bytes, format, width, height, dpi, color_space }` | `HwpError::RenderError` |
| PDF(호환 경로) | `src/renderer/pdf.rs:812` `svgs_to_pdf_with_options(&[String], &PdfExportOptions)` | **SVG 문자열 배열** | `Vec<u8>` | `String` |
| PDF(직접 경로) | `src/renderer/pdf.rs:948` `layer_trees_to_pdf_with_options(&[PageLayerTree], &DirectPdfExportOptions)` | `&[PageLayerTree]` — **문서 단위** | `Vec<u8>` | `String` |

읽어낸 것.

- **산출 방식이 세 종류다.** 내부 버퍼에 쌓고 나중에 꺼내기(SVG·HTML), 소유값
  반환(Skia·PDF), 부수효과만(Web Canvas). `-> Result<Vec<u8>>` 같은 고정 반환형은
  SVG·HTML·WebCanvas 를 담지 못한다 → **연관 타입 `Output` 이 필수다.**
- **오류 타입이 세 종류다.** 없음 / `HwpError` / `String`. → **연관 타입 `Error` 가 필요하다.**
- **PDF 만 문서 단위다.** 나머지는 전부 페이지 한 장 단위다. → 계약은 페이지를 여러 장
  받아 하나의 산출물로 닫을 수 있어야 한다(`(begin_page … end_page)* finish`).
- **`&mut self` 와 `&self` 가 섞인다.** `LayerRenderer::render_page` 는 `&mut self`,
  `LayerRasterRenderer::render_raster` 는 `&self` 다(Skia 는 렌더 중 실제로 불변).
  → 더 넓은 쪽(`&mut self`)으로 잡는다. 불변 백엔드는 손해가 없다.

### 2.2 좌표와 단위가 어디서 바뀌나

| 사실 | 근거 |
| --- | --- |
| 레이어 IR 의 단위는 `px`, 좌표계는 `page-top-left-y-down` | `src/paint/schema.rs:22-23`, `PAGE_LAYER_TREE_UNIT`/`PAGE_LAYER_TREE_COORDINATE_SYSTEM` |
| `BoundingBox` 네 필드 모두 px, 페이지 내 절대 위치 | `src/renderer/render_tree.rs:581-590` |
| HWPUNIT → px 환산은 **렌더러 진입 전**에 끝난다 | `src/renderer/mod.rs:685-686` (`DEFAULT_DPI = 96.0`, `HWPUNIT_PER_INCH = 7200.0`), `mod.rs:1092` `hwpunit_to_px`, `render_tree.rs:619` `from_hwpunit_rect` |
| 백엔드 파일 안에는 HWPUNIT 이 **한 번도 등장하지 않는다** | `svg.rs`·`pdf.rs`·`canvas.rs`·`html.rs`·`skia/renderer.rs` 전수 검색 0건 |
| px → 형식 단위 환산은 백엔드 **안에서** 일어난다 | `src/renderer/pdf.rs:952` `const CSS_PX_TO_PDF_POINT: f64 = 72.0 / 96.0;`, 적용 `pdf.rs:966`·`:984-985` |
| SVG 는 환산 없이 px 를 그대로 쓴다 | `src/renderer/svg.rs:2703` `width="{}" height="{}" viewBox="0 0 {} {}"` — SVG user unit == px |
| 래스터 배율은 옵션으로 따로 받는다 | `RasterRenderOptions.scale`(`layer_renderer.rs:29`), 적용 `skia/renderer.rs:441-443` |

**결론: 이 계층을 통과하는 좌표는 언제나 px 다.** 위로는 HWPUNIT 환산이 끝난 뒤이고,
아래로는 형식 고유 단위 환산이 시작되기 전이다. 이 경계를 trait 문서 주석에 못박았다.

### 2.3 이미 있는 trait 셋과 그 한계

| trait | 위치 | 서명 | 왜 부족한가 |
| --- | --- | --- | --- |
| `Renderer` | `src/renderer/mod.rs:664` | `begin_page(w,h)`, `end_page()`, `draw_text/rect/line/ellipse/image/path(...)` | 원시 도형 단위라 `PaintOp` 어휘를 잃는다. `Result` 가 없어 **실패를 말할 수 없다**. 산출물 타입이 없다. |
| `LayerRenderer` | `src/renderer/layer_renderer.rs:21` | `render_page(&mut self, tree: &PageLayerTree) -> LayerRenderResult<()>` | 페이지 한 장을 통째로 삼킨다. 산출물 타입도 능력 선언도 없고, 여러 페이지를 한 문서로 묶는 개념이 없다. 구현체는 셋뿐이며(`canvas.rs:331`, `svg_layer.rs:239`, `web_canvas.rs:2167`) **SVG 본체·Skia·PDF 는 구현하지 않는다.** |
| `LayerRasterRenderer` | `src/renderer/layer_renderer.rs:73` | `render_raster(&self, tree, options) -> LayerRenderResult<RasterRenderOutput>` | 래스터 전용이라 벡터 백엔드를 담지 못한다. |

즉 **세 trait 중 무엇도 SVG·Skia·PDF 를 함께 담지 못한다.** `RenderBackend` 는 그
사이를 메운다.

### 2.4 재생 순서의 정본

`src/paint/replay_order.rs` 가 z 순서의 단일 출처다.

- `PaintReplayPlane::ORDERED = [Background, BehindText, Flow, InFrontOfText]` — `:19-24`
- `paint_op_replay_plane_with_layer(op, layer)` — `:40`

Skia(`skia/renderer.rs:496`)와 Web Canvas(`web_canvas.rs:492`)는 이 plane 배열을 바깥
루프로 돌린다. 반면 `CanvasRenderer` 는 plane 을 전혀 보지 않고 트리 순서로만 그리며
(`canvas.rs:182`), SVG 는 `PageRenderTree` 쪽의 별도 z 계산(`svg.rs:249`)을 쓴다.
새 구동기 `replay_page` 는 **정본(plane 배열)을 따른다**.

### 2.5 조사에서 드러난 기존 결함 (이 PR 의 범위 밖, 기록만 남긴다)

- `CanvasRenderer` 는 18개 `PaintOp` 중 10개를 조용히 버린다 — `canvas.rs:272-282` 의
  빈 `{}` 팔. 능력 선언이 없으니 소비자는 이 사실을 알 방법이 없다.
- `CanvasRenderer::render_layer_tree`(`canvas.rs:88-91`)와
  `WebCanvasRenderer::render_layer_tree`(`web_canvas.rs:489`)는 `begin_page` 를 부르고
  **`end_page` 를 부르지 않는다.** `Renderer` 의 페이지 짝이 레이어 경로에서 지켜지지
  않는다는 뜻이다. 새 계약은 이 짝을 **강제**한다(§3.1).

## 3. 계약과 불변식

### 3.1 생명주기 (강제되는 불변식)

```text
( begin_page  draw*  end_page )*  finish
```

위반은 전부 오류이며, 조용히 넘어가지 않는다.

| 위반 | 오류 |
| --- | --- |
| `begin_page` 없이 `draw` | `NoOpenPage { call: "draw" }` |
| `begin_page` 없이 `end_page` | `NoOpenPage { call: "end_page" }` |
| 열린 페이지가 있는데 `begin_page` | `PageAlreadyOpen` |
| 열린 페이지를 두고 `finish` | `UnclosedPage { pages_completed }` |
| 유한 양수가 아닌 페이지 치수 | `InvalidPageSize { width, height }` |

**판정은 백엔드마다 재구현하지 않는다.** `render_backend::PageState` 가 상태기를
한 곳에 담고 있고, 이 크레이트의 백엔드 셋은 전부 그것을 품는다. 그래야 "백엔드에
따라 규칙이 다르다"가 생기지 않는다. 실패한 `begin_page` 는 페이지를 열지 않으므로
호출자는 고친 치수로 다시 열 수 있다.

### 3.2 좌표·단위 계약

- 단위는 **px**. HWPUNIT 이 아니다.
- 원점은 **페이지 왼쪽 위**, **y 는 아래로** 증가한다.
- 좌표는 **페이지 절대 좌표**다. `PaintOp` 는 평탄화된 leaf op 이므로 조상 그룹·클립의
  변환이 누적돼 있지 않다.
- px → 형식 고유 단위(pt·device px·mm) 환산은 **백엔드 안에서** 한다. 그 계수는 이
  trait 표면에 드러나지 않는다.

이는 §2.2 에서 실측한 현재 동작과 정확히 같다. 새 규칙을 도입한 것이 아니라
**이미 사실인 것을 계약으로 적은 것**이다.

### 3.3 능력 선언 — 분기 대신 질의

```rust
pub struct BackendCapabilities {
    pub name: &'static str,
    pub vector_text: bool,
    pub embedded_fonts: bool,
    pub gradients: bool,
    pub clipping: bool,
    pub images: bool,
    pub raster_only: bool,
    pub multi_page: bool,
    pub deterministic: bool,
}
```

각 필드는 **최종 산출물이 그 성질을 보존하는가**를 뜻한다(중간 단계가 아니다).
소비자는 `caps.supports(BackendFeature::VectorText)` 또는
`caps.covers(&[..])` 로 **질의**한다. 백엔드 타입으로 `match` 하는 코드는 새 백엔드가
붙을 때마다 전부 고쳐야 하지만, 질의는 고칠 것이 없다.

불변식 하나: `raster_only && vector_text` 는 자기모순이다 —
`BackendCapabilities::is_consistent()` 가 판정한다.

### 3.4 오류

`type Error` 는 백엔드가 고르지만, 생명주기 위반만은 모두 같아야 한다. 그래서
`RenderBackendError` 를 제공하고 양방향 변환을 붙였다.

- `From<HwpError> for RenderBackendError` — 기존 백엔드를 감쌀 때 오류를 잃지 않는다.
- `From<RenderBackendError> for HwpError` — 새 계약을 기존 호출부에 되돌려 넣을 수 있다.

### 3.5 스케치에서 바꾼 것과 그 이유

원안 대비 세 가지를 바꿨다.

1. **`finish_boxed(self: Box<Self>)` 를 추가했다.** `finish(self)` 는 산출물 소유권을
   넘기려면 필요하지만 `Self: Sized` 를 요구해 vtable 에 오르지 못한다. 그러면
   `Box<dyn RenderBackend<..>>` 에서 결과를 꺼낼 수 없다 — 백엔드를 런타임에 고르는
   것이 이 계층의 존재 이유인데 그게 막힌다. `self: Box<Self>` 수신자는 object safe
   이므로 이 한 메서드로 해결된다. 구현은 언제나 `(*self).finish()` 한 줄이다.
2. **`BackendCapabilities` 에 `name`·`clipping`·`images`·`multi_page`·`deterministic`
   를 더했다.** `name` 은 오류 메시지와 회귀 기준선 파일명에 쓸 안정 식별자이고,
   `multi_page` 는 §2.1 에서 드러난 PDF(문서 단위) 대 나머지(페이지 단위)의 차이를
   질의 가능하게 만든다. `deterministic` 은 백엔드 간 정합 시험의 전제 조건이다 —
   기준선을 뜰 수 있는 백엔드를 가려낸다.
3. **`PageSize` 를 `(f64, f64)` 대신 이름 있는 타입으로 두었다.** 기존
   `Renderer::begin_page(width, height)` 는 두 `f64` 를 나란히 받아 뒤바꿔 넣어도
   컴파일된다. 단위 계약을 문서로 못 박을 자리도 필요했다.

## 4. 지금 들어 있는 백엔드

| 백엔드 | `Output` | 무엇에 쓰나 |
| --- | --- | --- |
| `SvgBackend` (`svg_adapter.rs`) | `String` | **레퍼런스 어댑터.** 기존 `SvgLayerRenderer` 를 호출만 해서 진짜 SVG 문서를 낸다. 계약이 실제 백엔드를 감쌀 수 있음을 증명한다. |
| `TraceBackend` (`backends.rs`) | `String` | op 시퀀스를 결정적 문자열로 기록. **백엔드 간 정합 시험의 기준선.** |
| `NullBackend` (`backends.rs`) | `DrawStats` | 그린 op 를 종류별로 세는 계측기. 그리기 비용 없이 조판 산출량을 잰다. |

### 4.1 `TraceBackend` 가 왜 그 자체로 값이 있나

픽셀 비교는 두 백엔드가 다 완성돼야 하고, 달라도 **어디서** 갈렸는지 말해주지 않는다.
`TraceBackend` 는 조판이 내보낸 op 시퀀스 자체를 고정하므로, 두 백엔드의 그림이 다를 때
*같은 입력을 다르게 그린 것*인지 *애초에 다른 입력을 받은 것*인지를 가른다.

출력 형식은 결정적이다.

```text
begin_page 400.00x300.00
  pageBackground bbox=0.00,0.00,400.00,300.00
  rectangle bbox=20.00,20.00,10.00,10.00
end_page ops=2
```

좌표는 항상 `{:.2}` 로 찍어 `f64` 기본 출력의 자릿수 흔들림을 없앤다. op 이름표
(`pageBackground`·`rectangle` …)는 **기존 LayerTree JSON export 의 `"type"` 값과 글자
그대로 같다**(`src/paint/json.rs:507-1036`). 두 어휘가 갈라지지 않게 하려는 것이다.

### 4.2 `SvgBackend` 어댑터의 얇음과 그 대가

어댑터는 `src/renderer/**` 를 한 줄도 고치지 않고 공개 API 만 호출한다
(`SvgLayerRenderer::new` → `LayerRenderer::render_page` → `output()`).
대가는 하나다: 받은 op 들을 `LayerNode::leaf` 하나로 묶어 넘기므로 **원본 트리의
그룹·클립 구조가 이 경로에서 평탄해진다.** 진짜 이관은 §5 다.

### 4.3 구동기 `replay_page`

기존 `PageLayerTree` 한 장을 임의의 백엔드로 재생하는 다리다. 새 백엔드는 트리 순회를
다시 짤 필요 없이 trait 만 구현하면 된다. 순서는 §2.4 의 정본 plane 배열을 따르고,
plane 안에서는 트리 전위 순회를 지키며, 조상의 `LayerNode::layer` 를 자손에게 상속한다.
`B: ?Sized` 라 trait object 에도 그대로 쓰인다.

## 5. 채택 시나리오 — 기존 백엔드가 어떻게 이 뒤로 옮겨가나

이 PR 은 계약만 신설한다. 실제 이관은 아래 순서를 제안한다. 각 단계는 독립 PR 이며,
앞 단계가 뒤 단계의 회귀 그물이 된다.

### 5.1 SVG — `SvgRenderer` 안에서 직접 구현 (가장 먼저)

`SvgRenderer` 는 이미 이 계약과 같은 모양의 내부 생명주기를 갖고 있다.

- `begin_page(width, height)` — `svg.rs:2703` 에서 루트 `<svg>` 를 연다
  (`RenderNodeType::Page` 진입점 `svg.rs:302-304`)
- `end_page()` — `<defs>` 주입과 `</svg>` 닫기 (`svg.rs:851-853`)

즉 SVG 는 `RenderBackend` 를 **직접** 구현할 수 있다. `type Output = String`,
`type Error = RenderBackendError`, `draw` 는 지금의 `paint_op_to_render_node`
(`svg_layer.rs:239` 아래) 대신 `PaintOp` 를 곧바로 SVG 요소로 내보낸다.
그러면 §4.2 의 평탄화 손실이 사라지고, `SvgLayerRenderer` 의
`PageLayerTree` → `PageRenderTree` 역변환(`svg_layer.rs:38-43`, 스스로
"transition renderer" 라 적힌 곳)이 통째로 없어진다.

이관 안전망: 이관 전후로 같은 문서에 `TraceBackend` 를 물려 op 시퀀스가 같은지 먼저
확인하고(입력이 같음을 고정), 그다음 SVG 문자열을 비교한다.

### 5.2 PDF — 문서 단위 산출을 `finish` 로 흡수

직접 PDF 경로는 지금 `&[PageLayerTree]` 를 통째로 받는다(`pdf.rs:948`). 새 계약에서는
페이지마다 `begin_page`/`end_page` 를 받고 `finish` 가 문서 전체를 낸다.

```rust
impl RenderBackend for PdfBackend {
    type Output = Vec<u8>;   // 문서 하나
    type Error = RenderBackendError;
    // begin_page → 새 PDF 페이지 열기 (px → pt: CSS_PX_TO_PDF_POINT, pdf.rs:952)
    // finish     → 문서 직렬화
}
```

`capabilities().multi_page == true` 가 이 성질을 소비자에게 알린다. 호환 경로
(`svgs_to_pdf_with_options`, `pdf.rs:812`)는 그대로 두어도 된다 — SVG 백엔드의
`Output` 이 `String` 이므로 `SvgBackend → svgs_to_pdf` 조합이 계약 안에서 그대로 성립한다.

### 5.3 Skia — `&self` 를 `&mut self` 로 좁히고 옵션을 생성자로

Skia 는 `render_raster_with_options(&self, tree, options)` 로 트리 전체를 받는다
(`skia/renderer.rs:372`). 이관은 옵션을 **생성자**로 옮기는 모양이다.

```rust
let mut backend = SkiaBackend::new(RasterRenderOptions { scale: 2.0, ..Default::default() });
replay_page(&mut backend, &tree)?;
let png: RasterRenderOutput = backend.finish()?;
```

`capabilities()` 는 `BackendCapabilities::raster("skia")` 에서 출발한다
(`raster_only: true`, 따라서 `vector_text: false`). Skia 는 이미 plane 배열을
바깥 루프로 돌리므로(`skia/renderer.rs:496`) `replay_page` 와 재생 순서가 같다 —
이관 시 순서 회귀가 나지 않는다는 뜻이다. `native-skia` 피처 게이트는 그대로 유지하고,
백엔드 구현 전체를 `#[cfg(feature = "native-skia")]` 로 감싼다.

### 5.4 Canvas — 능력 선언으로 결손을 드러내기

`CanvasRenderer` 가 10개 op 를 조용히 버리는 것(§2.5)은 이관 시 두 가지로 바뀐다.
표현 못 하는 op 는 `UnsupportedOp { backend: "canvas", op }` 를 내거나,
`capabilities()` 가 그 결손을 미리 선언한다. 어느 쪽이든 **소비자가 알 수 있게** 된다.
`begin_page`/`end_page` 짝도 계약이 강제하므로 자동으로 고쳐진다.

### 5.5 새 백엔드 (예: 인쇄용 PostScript, 접근성 트리)

기존 코드를 읽을 필요 없이 `RenderBackend` 만 구현하고 `replay_page` 로 구동한다.
`NullBackend` 로 생명주기부터 확인하고, `TraceBackend` 기준선으로 입력이 같음을
고정한 뒤 산출물을 검증하는 것이 권장 순서다.

## 6. 비범위

이 PR 이 **하지 않는** 것.

- **기존 백엔드 수정.** `src/renderer/**` 는 한 줄도 바뀌지 않는다. §5 는 제안이지
  이 PR 의 내용이 아니다.
- **기존 trait 폐기.** `Renderer`·`LayerRenderer`·`LayerRasterRenderer` 는 그대로다.
  경쟁이 아니라 상위 계약이며, 이관이 끝나기 전에는 공존한다.
- **클립·변환·레이어 합성의 계약화.** v1 은 leaf op 재생만 다룬다. 클립 영역은
  `BackendCapabilities::clipping` 으로 **질의만** 가능하고, 클립을 백엔드에 전달하는
  op(`push_clip`/`pop_clip` 같은 것)는 아직 없다. `PaintOp` 자체에 그 개념이 없기
  때문이다.
- **리소스 아레나 전달.** `ResourceArena`(폰트 blob·이미지)는 지금 `PageLayerTree` 에
  달려 있고 trait 표면에 없다. 이미지·내장 폰트를 진짜로 그리는 백엔드는 이 통로가
  필요하다 — 다음 개정에서 `begin_page` 인자나 별도 `set_resources` 로 추가할 자리다.
- **비동기·스트리밍 출력.** 전부 동기다.
- **CLI/MCP 표면 노출.** 이 계층은 아직 라이브러리 내부 계약이다.

## 7. 검증

| 게이트 | 결과 |
| --- | --- |
| `cargo build` | 통과 |
| `cargo test --lib render_backend::` | 14 통과 / 0 실패 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `rustfmt --edition 2021 --check src/render_backend/mod.rs` | 통과(하위 모듈 포함) |

단위 시험이 고정하는 것: trait object 다형 호출(`TraceBackend` 와 `SvgBackend` 를 같은
`Box<dyn ..>` 벡터에 담아 구동), `begin_page` 없는 `draw` 의 오류, 페이지 경계 위반 3종,
치수 유효성, `TraceBackend`/`SvgBackend` 결정성(같은 입력 → 같은 문자열), 능력 질의와
자기모순 판정, `replay_page` 의 plane 재정렬, op 이름표와 LayerTree JSON 어휘의 일치,
`PageState` 계수, `HwpError` 변환.

### 작업 환경 함정

- `cargo fmt --all -- --check` 는 이 환경에서 os error 206 으로 실패한다 →
  `rustfmt --edition 2021 --check <파일>` 을 쓴다.
- `renderer::composer::re_sample_gen::tests::test_gen_re_multisize` 는 이 변경과
  무관하게 원래 실패한다. 회귀로 오인하지 않는다.
- sparse-checkout 에서 `gym/` 을 빼면 `src/mcp_serve.rs:827` 의
  `include_str!("../gym/README.md")` 때문에 **bin 타깃 빌드가 실패한다.**
  `git sparse-checkout add gym` 으로 해소한다(라이브러리 빌드는 영향 없음).
