---
kind: decision
status: active
canonical: mydocs/tech/emf_spec.md
last_verified: 2026-07-17
---

# EMF IR (Intermediate Representation) 설계 — rhwp `src/emf/`

> Task #195 단계 9 산출물
> 상위 문서: [emf_spec.md](emf_spec.md)

## 1. 모듈 구조

```
src/emf/
├── mod.rs                         공개 API: parse_emf, convert_to_svg, Error
├── parser/
│   ├── mod.rs                     레코드 디스패처, 스트림 reader
│   ├── constants/
│   │   ├── mod.rs
│   │   ├── record_type.rs         RecordType enum
│   │   ├── pen_style.rs           PenStyle enum
│   │   ├── brush_style.rs         BrushStyle enum
│   │   ├── map_mode.rs            MapMode enum
│   │   └── text_align.rs          TextAlign flags
│   ├── objects/
│   │   ├── mod.rs
│   │   ├── header.rs              Header (EMR_HEADER + ext1/2)
│   │   ├── rectl.rs               RECTL, POINTL, POINTS, SIZEL
│   │   ├── xform.rs               XFORM
│   │   ├── color.rs               ColorRef → RGB
│   │   ├── logpen.rs              LogPen
│   │   ├── logbrush.rs            LogBrush
│   │   └── logfont.rs             LogFontW
│   └── records/
│       ├── mod.rs                 Record enum + dispatch
│       ├── header.rs              단계 10
│       ├── object.rs              단계 11 (CreatePen/Brush/Font, Select, Delete)
│       ├── state.rs               단계 11 (SaveDC/RestoreDC, SetWorldTransform, Window/Viewport)
│       ├── drawing.rs             단계 12 (MoveTo/LineTo, Rect/Ellipse/Arc, Polyline16)
│       ├── path.rs                단계 12 (BeginPath/EndPath/FillPath)
│       ├── text.rs                단계 13 (ExtTextOutW)
│       └── bitmap.rs              단계 13 (StretchDIBits)
└── converter/
    ├── mod.rs                     Player (Record → SVG 노드)
    ├── device_context.rs          DeviceContext, DcStack, ObjectTable
    └── svg/
        ├── mod.rs                 SvgBuilder, 노드 생성
        ├── shape.rs               shape 변환 (rect, ellipse, path, ...)
        ├── text.rs                텍스트 변환
        └── image.rs               DIB → PNG base64
```

**WMF 모듈(`src/wmf/`)과 완전히 독립** — 코드/타입 공유 없음.

## 2. 공개 API

### 최상위

```rust
// src/emf/mod.rs
pub mod converter;
pub mod parser;

pub use parser::Header;
pub use parser::records::Record;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid EMF signature")]
    InvalidSignature,
    #[error("unexpected EOF at offset {0}")]
    UnexpectedEof(usize),
    #[error("unknown record type {0} at offset {1}")]
    UnknownRecord(u32, usize),
    #[error("io error: {0}")]
    Io(String),
}

/// EMF 바이트를 레코드 시퀀스로 파싱.
pub fn parse_emf(bytes: &[u8]) -> Result<Vec<Record>, Error>;

/// EMF 바이트 + 렌더 영역(pt) → SVG fragment 문자열.
/// - `render_rect`: rhwp 렌더 트리에서 전달되는 (x, y, w, h), 단위는 pt.
/// - 반환: `<g ...>...</g>` 형태의 SVG fragment (viewBox 미포함 — 상위에서 배치).
pub fn convert_to_svg(bytes: &[u8], render_rect: (f32, f32, f32, f32)) -> Result<String, Error>;
```

### Record enum

```rust
// src/emf/parser/records/mod.rs
#[derive(Debug, Clone)]
pub enum Record {
    // 헤더 / 제어
    Header(Header),
    Eof,

    // 객체 (단계 11)
    CreatePen { handle: u32, pen: LogPen },
    CreateBrushIndirect { handle: u32, brush: LogBrush },
    ExtCreateFontIndirectW { handle: u32, font: LogFontW },
    SelectObject { handle: u32 },
    DeleteObject { handle: u32 },

    // 상태 (단계 11)
    SaveDC,
    RestoreDC { relative: i32 },
    SetWorldTransform(XForm),
    ModifyWorldTransform { xform: XForm, mode: u32 },
    SetMapMode(u32),
    SetWindowExtEx(SizeL),
    SetWindowOrgEx(PointL),
    SetViewportExtEx(SizeL),
    SetViewportOrgEx(PointL),
    SetTextColor(u32),
    SetBkColor(u32),
    SetBkMode(u32),
    SetTextAlign(u32),

    // 드로잉 (단계 12)
    MoveToEx(PointL),
    LineTo(PointL),
    Rectangle(RectL),
    RoundRect { rect: RectL, corner: SizeL },
    Ellipse(RectL),
    Arc { rect: RectL, start: PointL, end: PointL },
    Chord { rect: RectL, start: PointL, end: PointL },
    Pie { rect: RectL, start: PointL, end: PointL },
    Polyline16 { bounds: RectL, points: Vec<(i16, i16)> },
    Polygon16 { bounds: RectL, points: Vec<(i16, i16)> },
    PolyBezier16 { bounds: RectL, points: Vec<(i16, i16)> },

    // 패스 (단계 12)
    BeginPath,
    EndPath,
    CloseFigure,
    FillPath(RectL),
    StrokePath(RectL),
    StrokeAndFillPath(RectL),

    // 텍스트 (단계 13)
    ExtTextOutW(ExtTextOutW),

    // 비트맵 (단계 13)
    StretchDIBits(StretchDIBits),

    // 1차 미구현 — raw 바이트로 보존 (후속 이슈)
    Unknown { record_type: u32, payload: Vec<u8> },
}
```

## 3. DeviceContext 설계

```rust
// src/emf/converter/device_context.rs
#[derive(Clone, Debug)]
pub struct DeviceContext {
    pub pen:          Option<LogPen>,
    pub brush:        Option<LogBrush>,
    pub font:         Option<LogFontW>,
    pub text_color:   u32,
    pub bk_color:     u32,
    pub bk_mode:      BkMode,          // Transparent / Opaque
    pub text_align:   u32,             // bitflags

    // 좌표계
    pub world_xform:  [f32; 6],        // [a, b, c, d, tx, ty]
    pub map_mode:     MapMode,
    pub window_org:   (i32, i32),
    pub window_ext:   (i32, i32),
    pub viewport_org: (i32, i32),
    pub viewport_ext: (i32, i32),

    // 커서
    pub current_pos:  (i32, i32),
}

impl Default for DeviceContext {
    fn default() -> Self {
        Self {
            pen: None, brush: None, font: None,
            text_color: 0x000000, bk_color: 0xFFFFFF,
            bk_mode: BkMode::Opaque, text_align: 0,
            world_xform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],  // identity
            map_mode: MapMode::Text,
            window_org: (0, 0), window_ext: (1, 1),
            viewport_org: (0, 0), viewport_ext: (1, 1),
            current_pos: (0, 0),
        }
    }
}

pub struct DcStack {
    current: DeviceContext,
    stack:   Vec<DeviceContext>,
}

impl DcStack {
    pub fn save(&mut self)   { self.stack.push(self.current.clone()); }
    pub fn restore(&mut self, relative: i32) {
        // relative<0: 상대 깊이, relative>0: 절대 깊이 (MS-EMF 2.3.11)
        // 1차 구현은 relative=-1만 (가장 최근 save) 지원
        if let Some(dc) = self.stack.pop() { self.current = dc; }
    }
    pub fn current(&self)    -> &DeviceContext     { &self.current }
    pub fn current_mut(&mut self) -> &mut DeviceContext { &mut self.current }
}
```

## 4. ObjectTable 설계

```rust
// src/emf/converter/device_context.rs
#[derive(Clone, Debug)]
pub enum GraphicsObject {
    Pen(LogPen),
    Brush(LogBrush),
    Font(LogFontW),
}

pub struct ObjectTable {
    handles: HashMap<u32, GraphicsObject>,
}

impl ObjectTable {
    pub fn insert(&mut self, handle: u32, obj: GraphicsObject);
    pub fn get(&self, handle: u32) -> Option<&GraphicsObject>;
    pub fn remove(&mut self, handle: u32);
}
```

- `handle`은 EMR_CREATEPEN 등의 `ihPen` 필드(u32)
- 스톡 객체(handle & `0x80000000`)는 별도 분기로 미리 정의된 객체 참조

## 5. Player (레코드 → SVG)

```rust
// src/emf/converter/mod.rs
pub struct Player {
    pub dc_stack: DcStack,
    pub objects:  ObjectTable,
    pub path_active: Option<PathBuilder>,   // BeginPath ~ EndPath 사이
    pub svg:      SvgBuilder,
}

impl Player {
    pub fn new(header: &Header, render_rect: (f32, f32, f32, f32)) -> Self;
    pub fn play(&mut self, records: &[Record]) -> Result<(), Error>;
    pub fn into_svg_fragment(self) -> String;

    fn exec(&mut self, rec: &Record) -> Result<(), Error>;
    // rec 종류별 dispatch
}
```

## 6. 좌표 변환

### 논리 → 페이지 (MM_ANISOTROPIC)

```
scale_x = viewport_ext.x / window_ext.x
scale_y = viewport_ext.y / window_ext.y
page_x  = (logical_x - window_org.x) * scale_x + viewport_org.x
page_y  = (logical_y - window_org.y) * scale_y + viewport_org.y
```

### 페이지 → EMF 프레임 → SVG

1. Player는 **EMF 내부 논리 좌표를 그대로 SVG 좌표로 출력**하고, 최종 `<g transform="matrix(...)">` 로 래핑하여 `render_rect`에 매핑.
2. 매핑 행렬:
   ```
   sx = render_rect.w / (header.bounds.right - header.bounds.left)
   sy = render_rect.h / (header.bounds.bottom - header.bounds.top)
   tx = render_rect.x - header.bounds.left * sx
   ty = render_rect.y - header.bounds.top * sy
   → <g transform="matrix(sx 0 0 sy tx ty)">
   ```
3. WorldTransform은 각 드로잉 레코드 출력 시 별도 `<g transform="matrix(...)">` 적용.

## 7. SVG 출력 규약

- 루트: `<g class="emf-root" transform="matrix(sx 0 0 sy tx ty)">...</g>`
- 좌표 소수점: 최대 2자리 (`{:.2}`)
- 색상: `rgb(R,G,B)` 또는 `#RRGGBB`
- 텍스트: `<text x="..." y="..." font-family="..." font-size="..." fill="rgb(...)">내용</text>`
  - 폰트명은 LOGFONTW.FaceName → rhwp `font_fallback_strategy.md`의 폴백 체인 재사용
- 비트맵: `<image x="..." y="..." width="..." height="..." href="data:image/png;base64,..."/>`

## 8. 에러 처리 방침

- 파싱 실패 시 `Error` 반환 → `convert_to_svg`는 상위에서 **실패 시 기존 placeholder 유지** (shape_layout에서 처리)
- 미지 RecordType은 `Record::Unknown`으로 저장, Player는 경고 로그 후 스킵
- Signature 불일치는 즉시 실패

## 9. 테스트 전략

### 단위 테스트

| 단계 | 테스트 대상 |
|------|------------|
| 10 | EMR_HEADER 파싱 — 고정 88B 픽스처 + 확장 |
| 11 | CreatePen/Brush/Font → ObjectTable 등록, SelectObject → DC 갱신, SaveDC/RestoreDC 스택 |
| 12 | Rectangle/Ellipse/Polyline16 → SVG `<rect>/<ellipse>/<polyline>` 출력 |
| 12 | BeginPath~EndPath~FillPath → `<path d="...">` 출력 |
| 13 | ExtTextOutW UTF-16 디코딩 → `<text>` 출력 |
| 13 | StretchDIBits BI_RGB → base64 PNG |

### 통합 테스트

- `samples/emf/simple-line.emf` — 선 1개 (자체 제작)
- `samples/emf/simple-rect-text.emf` — 사각형 + 텍스트 (자체 제작)
- 1.hwp OlePres000 EMF (로컬, 저작권 이슈로 samples/ 미포함)

### 회귀

- 기존 WMF 테스트 878+15 모두 통과 유지
- 전체 `cargo test --release` green

## 10. 빌드/의존성

기존 프로젝트에 이미 있음:
- `flate2` (stage 6)
- `quick-xml` (stage 8)
- `cfb` (stage 7)

신규 필요 여부:
- **PNG 인코딩**: 이미 사용 중인지 확인 필요. 없으면 `image` crate 추가 (BMP/DIB → PNG 변환용). 단계 13 착수 시 재검토.
- **폰트 처리**: rhwp 기존 폰트 폴백 재사용, 신규 의존성 불필요.
