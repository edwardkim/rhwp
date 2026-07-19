---
kind: canonical
status: active
canonical: mydocs/tech/rendering_engine_design.md
last_verified: 2026-07-16
---

# RHWP 렌더링 엔진 아키텍처 설계서

## 1. 렌더링 백엔드 최종 선정

### 비교 평가

| 평가 항목 | ThorVG (방안 A) | 순수 Rust (방안 B) | Canvas API (방안 C) |
|----------|----------------|-------------------|-------------------|
| 빌드 복잡도 | 높음 (C++ + Rust 혼합) | **낮음** (순수 Cargo) | 낮음 (순수 Cargo) |
| WASM 빌드 | emscripten 필요 | **wasm-pack 직접** | wasm-pack 직접 |
| 벡터 프리미티브 | 풍부 | 충분 | 브라우저 의존 |
| 텍스트 렌더링 | 기본 지원 | 별도 크레이트 | **브라우저 네이티브** |
| 네이티브 빌드 | 가능 | **가능** | 불가 (웹 전용) |
| 유지보수성 | 중 (FFI 관리) | **높음** (순수 Rust) | 중 (web-sys 의존) |
| WASM 크기 | ~150KB + Rust | Rust only | **최소** |

### 최종 선정: 멀티 백엔드 아키텍처

**렌더링 추상화 레이어(Renderer Trait)** 를 도입하여 사용자가 렌더링 백엔드를 옵션으로 선택할 수 있도록 설계한다.

#### 지원 백엔드 (옵션 선택)

| 백엔드 | 출력 형태 | 용도 | 구현 우선순위 |
|--------|----------|------|-------------|
| **Canvas** | Canvas 2D API 직접 그리기 | 실시간 뷰어, 인터랙션 | 1차 |
| **SVG** | SVG 엘리먼트 생성 | 벡터 출력, 확대/축소에 강함, 인쇄 | 2차 |
| **HTML** | DOM 엘리먼트 생성 | 텍스트 선택/복사, 접근성, SEO | 3차 |
| **Vector (tiny-skia)** | 픽셀 버퍼 래스터라이징 | 네이티브/서버사이드 렌더링 | 향후 |
| **ThorVG** | 벡터 엔진 렌더링 | 고급 벡터 기능 필요 시 | 향후 |

#### 설계 원칙

1. **Renderer Trait** 하나로 모든 백엔드를 추상화
2. 파싱 → IR → 레이아웃까지는 백엔드에 무관하게 공통
3. 최종 렌더링 단계에서만 백엔드 분기
4. 사용자가 초기화 시 백엔드 옵션을 선택

## 2. 전체 아키텍처

```
┌─────────────────────────────────────────────────┐
│                    HWP 파일                       │
└────────────────────┬────────────────────────────┘
                     │
            ┌────────▼────────┐
            │   CFB 파서       │  cfb 크레이트
            │   (OLE 컨테이너) │
            └────────┬────────┘
                     │
            ┌────────▼────────┐
            │  레코드 파서      │  src/parser/
            │  (TagID 기반)    │
            └────────┬────────┘
                     │
            ┌────────▼────────┐
            │  중간 표현 (IR)   │  src/model/
            │  Document Model  │
            └────────┬────────┘
                     │
            ┌────────▼────────┐
            │  Paginator       │  src/renderer/pagination.rs
            │  (페이지 분할)    │
            └────────┬────────┘
                     │
            ┌────────▼────────┐
            │  LayoutEngine    │  src/renderer/layout.rs
            │  (렌더 트리 생성) │
            └────────┬────────┘
                     │
            ┌────────▼────────┐
            │ RenderScheduler  │  src/renderer/scheduler.rs
            │ (Observer+Worker)│
            └────────┬────────┘
                     │
            ┌────────▼────────┐
            │  Renderer Trait  │  src/renderer/mod.rs
            │  (추상화 레이어)  │
            └─┬────┬────┬──┬──┘
              │    │    │  │
      ┌───────▼┐ ┌▼───┐│ ┌▼────────┐
      │Canvas  │ │SVG ││ │HTML     │
      │Renderer│ │Rndr││ │Renderer │
      │(1차)   │ │(2차)│ │(3차)    │
      └────────┘ └────┘│ └─────────┘
                  ┌─────▼────┐
                  │ Vector   │  (향후)
                  │ tiny-skia│
                  │ ThorVG   │
                  └──────────┘
```

## 3. 모듈 구조

```
src/
├── lib.rs              # WASM 진입점, 모듈 등록
├── main.rs             # 네이티브 CLI (export-svg, info)
├── wasm_api.rs         # WASM ↔ JavaScript 공개 API
│                         HwpDocument, HwpViewer, HwpError
├── parser/
│   ├── mod.rs          # 파서 모듈
│   └── header.rs       # 파일 헤더 파싱, 시그니처 검증
├── model/              # 중간 표현 (IR) - 12개 파일
│   ├── mod.rs          # HwpUnit, ColorRef, Point, Rect, Padding
│   ├── document.rs     # Document, Section, SectionDef, FileHeader, DocInfo
│   ├── paragraph.rs    # Paragraph, LineSeg, CharShapeRef, RangeTag
│   ├── table.rs        # Table, Cell, TableZone
│   ├── shape.rs        # ShapeObject(7종), CommonObjAttr, TextBox, Caption
│   ├── image.rs        # Picture, CropInfo, ImageAttr, ImageData
│   ├── style.rs        # CharShape, ParaShape, Style, Font, BorderFill, Fill
│   ├── page.rs         # PageDef, PageBorderFill, ColumnDef, PageAreas
│   ├── header_footer.rs # Header, Footer
│   ├── footnote.rs     # Footnote, Endnote, FootnoteShape
│   ├── control.rs      # Control(18종), Field, Bookmark, Hyperlink, Ruby
│   └── bin_data.rs     # BinData, BinDataContent
└── renderer/           # 렌더링 엔진 - 8개 파일
    ├── mod.rs          # Renderer Trait(8 메서드), TextStyle, ShapeStyle,
    │                     LineStyle, PathCommand, RenderBackend, 단위 변환
    ├── render_tree.rs  # RenderNode(dirty flag), RenderNodeType(18종),
    │                     BoundingBox, PageRenderTree
    ├── page_layout.rs  # PageLayoutInfo, LayoutRect, 다단 영역 계산
    ├── pagination.rs   # Paginator, PaginationResult, PageContent, PageItem
    ├── layout.rs       # LayoutEngine (IR → 렌더 트리 변환)
    ├── scheduler.rs    # RenderScheduler, RenderObserver, RenderWorker,
    │                     RenderTask, Viewport, RenderPriority(3단계)
    ├── canvas.rs       # CanvasRenderer (Canvas 2D, 1차)
    ├── svg.rs          # SvgRenderer (SVG 문자열, 2차)
    └── html.rs         # HtmlRenderer (HTML DOM, 3차)
```

## 4. 핵심 인터페이스

### Renderer Trait

```rust
pub trait Renderer {
    fn begin_page(&mut self, width: f64, height: f64);
    fn end_page(&mut self);
    fn draw_text(&mut self, text: &str, x: f64, y: f64, style: &TextStyle);
    fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64, style: &ShapeStyle);
    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: &LineStyle);
    fn draw_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, style: &ShapeStyle);
    fn draw_image(&mut self, data: &[u8], x: f64, y: f64, w: f64, h: f64);
    fn draw_path(&mut self, commands: &[PathCommand], style: &ShapeStyle);
}
```

### WASM 공개 API

```rust
#[wasm_bindgen]
pub struct HwpDocument { ... }

#[wasm_bindgen]
impl HwpDocument {
    pub fn new(data: &[u8]) -> Result<HwpDocument, JsValue>;
    pub fn create_empty() -> HwpDocument;
    pub fn page_count(&self) -> u32;
    pub fn render_page_svg(&self, page_num: u32) -> Result<String, JsValue>;
    pub fn render_page_html(&self, page_num: u32) -> Result<String, JsValue>;
    pub fn render_page_canvas(&self, page_num: u32) -> Result<u32, JsValue>;
    pub fn get_page_info(&self, page_num: u32) -> Result<String, JsValue>;
    pub fn get_document_info(&self) -> String;
    pub fn set_dpi(&mut self, dpi: f64);
    pub fn set_fallback_font(&mut self, path: &str);
}

#[wasm_bindgen]
pub struct HwpViewer { ... }

#[wasm_bindgen]
impl HwpViewer {
    pub fn new(document: HwpDocument) -> Self;
    pub fn update_viewport(&mut self, scroll_x: f64, scroll_y: f64, width: f64, height: f64);
    pub fn set_zoom(&mut self, zoom: f64);
    pub fn visible_pages(&self) -> Vec<u32>;
    pub fn pending_task_count(&self) -> u32;
    pub fn render_page_svg(&self, page_num: u32) -> Result<String, JsValue>;
    pub fn render_page_html(&self, page_num: u32) -> Result<String, JsValue>;
}
```

### 에러 처리 구조

```rust
pub enum HwpError {          // 네이티브 (non-WASM 안전)
    InvalidFile(String),
    PageOutOfRange(u32),
    RenderError(String),
}
impl From<HwpError> for JsValue { ... }  // WASM 경계에서만 변환
```

## 5. Observer + Worker 패턴

### Observer 패턴 (변경 감지)

렌더 트리 노드에 `dirty` 플래그를 내장하여 변경된 노드만 선택적으로 재렌더링한다.

```rust
pub struct RenderNode {
    pub dirty: bool,        // 변경 여부
    pub visible: bool,      // 가시성
    // ...
}

pub trait RenderObserver {
    fn on_event(&mut self, event: &RenderEvent);
    fn visible_pages(&self) -> Vec<u32>;
    fn prefetch_pages(&self) -> Vec<u32>;
}

pub enum RenderEvent {
    ViewportChanged(Viewport),
    ZoomChanged(f64),
    ContentChanged(u32),
    InvalidateAll,
}
```

### Worker 패턴 (우선순위 렌더링)

```rust
pub enum RenderPriority {
    Immediate = 0,   // 현재 뷰포트 내 페이지
    Prefetch = 1,    // 뷰포트 인접 ±2 페이지
    Background = 2,  // 나머지 페이지
}

pub trait RenderWorker {
    fn render_page(&mut self, tree: &PageRenderTree) -> Result<(), RenderError>;
    fn get_cached(&self, page_index: u32) -> Option<&PageRenderTree>;
    fn invalidate_cache(&mut self, page_index: u32);
}
```

### RenderScheduler (Observer + Worker 통합)

```
ViewportChanged → RenderScheduler → Task Queue (우선순위 정렬)
                       ↓                    ↓
               visible_pages()        Immediate: 즉시 렌더
               prefetch_pages()       Prefetch: 인접 페이지 사전 렌더
                                      Background: 나머지
```

## 6. 페이지 렌더링 모델

### 페이지 물리 구조

```
┌──────────────────────────────────┐
│           용지 (Paper)            │
│  ┌──────────────────────────┐    │
│  │      위 여백 (Top)        │    │
│  │  ┌──────────────────┐    │    │
│  │  │  머리말 (Header)   │    │    │
│  │  ├──────────────────┤    │    │
│  │  │   본문 영역       │    │    │
│  │  │  (Body Area)     │    │    │
│  │  │  ┌─단1─┐ ┌─단2─┐ │    │    │
│  │  │  │     │ │     │ │    │    │
│  │  │  └─────┘ └─────┘ │    │    │
│  │  ├──────────────────┤    │    │
│  │  │ 각주 구분선/영역   │    │    │
│  │  ├──────────────────┤    │    │
│  │  │  꼬리말 (Footer)   │    │    │
│  │  └──────────────────┘    │    │
│  │      아래 여백 (Bottom)   │    │
│  └──────────────────────────┘    │
└──────────────────────────────────┘
```

### 렌더링 파이프라인

```
IR(Document Model)
    → Paginator       (페이지 분할: 문단 높이 누적 → 페이지 경계 결정)
    → LayoutEngine    (렌더 트리 생성: 각 요소의 정확한 px 위치/크기 계산)
    → RenderScheduler (우선순위 스케줄링: Immediate → Prefetch → Background)
    → Renderer 백엔드  (Canvas/SVG/HTML 출력)
```

### 렌더 노드 종류 (18종)

| 노드 타입 | 설명 |
|-----------|------|
| Page | 페이지 루트 |
| PageBackground | 배경/테두리 |
| Header / Footer | 머리말/꼬리말 |
| Body | 본문 영역 |
| Column | 단 영역 |
| FootnoteArea | 각주 영역 |
| TextLine | 텍스트 줄 |
| TextRun | 텍스트 런 (동일 글자 모양) |
| Table / TableCell | 표/셀 |
| Line / Rectangle / Ellipse / Path | 그리기 개체 |
| Image | 이미지 |
| Group | 묶음 개체 |

## 7. 폰트 Fallback 전략

### Fallback 체인

```
1. HWP 문서 내 지정 폰트 (CharShape.font_ids)
   ↓ (없으면)
2. 시스템 폰트 매핑 (fontconfig 등)
   ↓ (없으면)
3. 기본 대체 폰트: /usr/share/fonts/truetype/nanum/NanumGothic.ttf
```

### API

```rust
pub const DEFAULT_FALLBACK_FONT: &str = "/usr/share/fonts/truetype/nanum/NanumGothic.ttf";

// 런타임 변경 가능
doc.set_fallback_font("/custom/path/font.ttf");
doc.get_fallback_font();  // 현재 설정 조회
```

## 8. 단위 환산

- HWP 내부 단위: HWPUNIT = 1/7200 인치
- Canvas 렌더링: 픽셀 (DPI 기반 변환)
- 변환 공식: `pixel = hwpunit * dpi / 7200`
- 역변환: `hwpunit = pixel * 7200 / dpi`
- 기본 DPI: 96 (웹 표준)
- A4 용지: 59528 × 84188 HWPUNIT = 793.7 × 1122.5 px (@ 96 DPI)

## 9. CLI 명령어

```bash
rhwp export-svg <파일.hwp> [--output <폴더>] [--page <번호>]
rhwp info <파일.hwp>
rhwp --version
rhwp --help
```

- SVG 내보내기 기본 출력 폴더: `output/`

## 10. 1차 지원 범위

| 요소 | 지원 수준 |
|------|----------|
| 페이지 | 용지 크기, 방향, 여백, 쪽 배경/테두리 |
| 구역 | 구역별 페이지 설정, 단일/다중 구역 |
| 텍스트 | 기본 텍스트, 글꼴, 크기, 색상, 굵게/기울임 |
| 문단 | 정렬, 들여쓰기, 줄간격, 문단간격 |
| 표 | 기본 표 구조, 셀 병합, 테두리 |
| 이미지 | 인라인 이미지 (PNG, JPG) |
| 도형 | 직선, 사각형, 타원, 다각형, 곡선, 호 |
| 머리말/꼬리말 | 기본 머리말/꼬리말 렌더링 |
| 폰트 | NanumGothic fallback, 런타임 변경 가능 |

### 1차 미지원 (향후 확장)
- 수식, 차트, OLE 개체
- 각주/미주
- 표 페이지 분할
- 글자 효과 (그림자, 외곽선, 양각/음각)
- 세로쓰기
- 텍스트 래핑 (개체 주변 텍스트 흐름)

## 11. 빌드 검증 현황

| 대상 | 결과 | 테스트 수 |
|------|------|---------|
| 네이티브 (cargo build) | 성공 | - |
| 테스트 (cargo test) | **88개 통과** | 88 |
| WASM (wasm-pack build) | 성공 | - |

### 테스트 분포

| 모듈 | 테스트 수 |
|------|---------|
| model (12개 파일) | 31 |
| parser | 1 |
| renderer (8개 파일) | 44 |
| wasm_api | 12 |
| 합계 | **88** |
