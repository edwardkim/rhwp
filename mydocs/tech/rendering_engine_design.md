---
kind: canonical
status: active
canonical: mydocs/tech/rendering_engine_design.md
last_verified: 2026-09-03
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

## 12. 렌더 정규화 derived state 계약

`DocumentCore.document`는 저장·편집 가능한 유일한 권위 IR이다. 렌더 전용 보정은
`RenderNormalizationState`에서 source IR과 revision으로부터 재생성하며 derived state를 다시
source에 쓰거나 편집 때 clone 경로를 직접 mirror하지 않는다.

### 상태와 revision

| 상태 | 의미 | 무효화 |
| --- | --- | --- |
| `document_epoch` | 문서 교체·전체 재초기화 세대 | 모든 section revision 증가, path ledger 초기화 |
| `section_revisions` | section 구조·페이지 기하 세대 | 해당 section derived projection 재생성 |
| `path_revisions` | 구조가 안정적인 셀/caption/textbox 편집 세대 | 해당 logical path 변경 기록 |
| `sections` | #2004 이미지 스택 immutable compatibility projection | `source_revision` 일치 시 같은 `Arc` 재사용 |
| `overlay` | #2195 중첩 표 effective width-scale 희소 projection | 현재 source path에서 재구축, stable entry `Arc` 재사용 |

일반 `mark_section_dirty()`는 pagination dirty와 section normalization revision 증가를 함께 수행한다.
문단/control/cell 수가 바뀌지 않는 deferred cell text edit은 명시적 path revision을 먼저 올리고
pagination만 dirty로 표시한다. #2004 compatibility projection이 실제 존재하는 section은 그
projection만 section revision으로 무효화한 뒤 transient render 전에 source IR에서 다시 만든다.

### kill 범위 규칙 (#4335)

측정 무효화의 owner는 `DocumentCore.dirty_paragraphs`다. source `Table`에는 renderer cache 상태를
두지 않는다. cell control을 바꾸는 명령은 외곽 문단을 dirty로 표시하고, 측정기는 그 문단의 중첩 표를
모두 다시 측정한 뒤 해당 문단 key만 소비한다. 따라서 아직 측정하지 않은 다른 구역의 invalidation을
전역 clear로 잃지 않는다(#4325).

셀 단일줄 overflow memo는 source `Paragraph`가 아니라 layout/measurement session cache가 소유한다.
key는 현재 source snapshot의 paragraph identity와 cell inner width의 `f32` bit다. source/style 변경은
다른 pointer-key layout cache와 같은 session 경계에서 전체 clear되므로 text/char-shape mutation site가
cache 필드를 알거나 개별 invalidation을 수동 호출하지 않는다. fresh composition끼리는 판정을 재사용하되
overflow인 경우의 derived rewrap은 매 composition에 다시 적용한다.

셀 text reflow 뒤 저장 frame을 더는 신뢰하지 않는 provenance도 source `Table`에 두지 않는다.
`RenderNormalizationState.text_reflowed_tables`가 live `Box<Table>` identity를 소유한다. 이 pointee는
문단·control vector 이동에도 안정적이고, 같은 format instance ID를 복제한 서로 다른 표를 섞지 않는다.
overlay 재구축은 이 identity를 현재 source와 immutable normalized snapshot pointer로 투영한다.
Layout과 Typeset은 같은 overlay를 읽는다. snapshot undo/redo는 identity를 당시 logical path로 투영해
보관한 뒤 복원 문서의 새 identity로 다시 해소한다. 표 나누기와 control clipboard paste는 clone에
provenance를 명시 상속하고, 표 붙이기는 어느 쪽이든 가진 provenance를 남은 front table에 합친다.

### persistence lowering 경계

HWP lowering은 `prepare_hwp_export_snapshot`이 만든 `HwpExportSnapshot`에서만 실행한다. 호환을 위해
`&mut self` signature를 유지하는 legacy API도 live `Document`를 adapter 작업 버퍼로 쓰지 않는다.
일반·report·password 저장과 CLI verify는 같은 snapshot type을 소비하며, CLI가 converter를 직접 불러
expected IR을 따로 만드는 경로는 두지 않는다.

### 논리 경로와 lookup

cache identity는 section/상위 문단과 다음 `RenderPathEntry`의 조합이다.

- `TableCell { control_index, cell_index, paragraph_index }`
- `TableCaption { control_index, paragraph_index }`
- `ShapeTextBox { control_index, paragraph_index }`
- `PictureCaption { control_index, paragraph_index }`

표 캡션 API의 legacy cell sentinel은 mutation 경계에서만 해석하며 cache key에는 들어가지 않는다.
각 단계의 control variant와 index를 source IR에서 검증하고, 불일치는 이전 entry를 반환하지 않고
`RenderError`로 표면화한다.

#2195 hot path는 source `Table` 주소로 width-scale을 O(1) 조회하지만 주소는 권위 key가 아니다.
overlay를 만들 때 logical path map과 pointer index를 현재 source IR에서 함께 재구축한다. 이전
projection은 path, source/effective width, 현재 pointer가 모두 같을 때만 `Arc`를 재사용한다.
source path가 사라지면 이전 entry도 함께 사라진다.

### #2004와 #2195의 표현 차이

#2195는 중첩 표의 저장 폭과 부모 셀 실효 폭 사이의 배율만 달라지므로
`NestedTableWidthProjection` 하나로 표현한다. `HeightMeasurer`와 `LayoutEngine`의 열 폭, 셀 내용 폭,
border geometry가 같은 overlay scale을 소비하며 source `Table`/`Cell` 폭은 바뀌지 않는다.

#2004 셀 이미지 스택은 TAC 재분류뿐 아니라 셀 문단 cardinality, 합성 `LINE_SEG`,
`ComposedParagraph`를 함께 바꾸는 구조 projection이다. 현재 renderer/typeset의 slice 계약을
보존하기 위해 영향 section에만 immutable `Arc<Vec<Paragraph>>`와
`Arc<Vec<ComposedParagraph>>`를 둔다. 같은 section revision에서는 재복제하지 않으며 편집 경로를
직접 수정하지 않는다. 따라서 mutable clone coherence 문제는 제거하면서 기존 #2004 조판을
보존한다.

관련 source guard는 `tests/issue_2308_render_normalized_guard.rs`, 기능·재사용·stale fallback 검증은
`renderer::render_normalization::tests`와
`document_core::queries::rendering::tests::issue_2308_*`가 담당한다.
