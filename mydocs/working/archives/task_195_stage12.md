# Task #195 단계 12 완료보고서 — 드로잉 레코드 + SVG 컨버터 1차

## 수행 내용

드로잉 레코드(선/사각형/타원/호/폴리라인16/폴리곤16/폴리베지어16) + 패스 레코드(BeginPath/EndPath/CloseFigure/FillPath/StrokePath/StrokeAndFillPath) 파서를 추가하고, `Player` + `SvgBuilder` 기반 EMF → SVG 변환기를 구현했다. `convert_to_svg` 공개 API를 통해 EMF 바이트를 SVG fragment 문자열로 변환할 수 있다.

## 추가 파일

| 파일 | 역할 |
|------|------|
| `src/emf/parser/records/drawing.rs` | MoveToEx / LineTo / Rectangle / RoundRect / Ellipse / Arc·Chord·Pie / Polyline16·Polygon16·PolyBezier16 파서 |
| `src/emf/parser/records/path.rs` | 패스 bounds RECTL 파서 |
| `src/emf/converter/svg.rs` | `SvgBuilder` + `colorref_to_rgb` + `escape_xml` |
| `src/emf/converter/player.rs` | `Player` — DC/ObjectTable 적용 후 SVG 노드 발행 |

## 수정 파일

| 파일 | 변경 |
|------|------|
| `src/emf/mod.rs` | `convert_to_svg` 공개 API 추가 |
| `src/emf/converter/mod.rs` | Player/SvgBuilder re-export |
| `src/emf/parser/records/mod.rs` | Record enum에 14개 드로잉/패스 variant 추가 |
| `src/emf/parser/mod.rs` | RT_* 상수 및 dispatcher 분기 17종 추가 |
| `src/emf/tests.rs` | 단계 12 테스트 7건 추가 (총 20건) |

## 지원 레코드 (단계 12)

### 드로잉
| RecordType | 이름 | 페이로드 |
|-----------|------|---------|
| 0x1B | EMR_MOVETOEX | PointL |
| 0x2A | EMR_ELLIPSE | RectL |
| 0x2B | EMR_RECTANGLE | RectL |
| 0x2C | EMR_ROUNDRECT | RectL + SizeL(corner) |
| 0x2D | EMR_ARC | RectL + PointL(start) + PointL(end) |
| 0x2E | EMR_CHORD | 동일 |
| 0x2F | EMR_PIE | 동일 |
| 0x36 | EMR_LINETO | PointL |
| 0x55 | EMR_POLYBEZIER16 | RectL + count + POINTS[count] |
| 0x56 | EMR_POLYLINE16 | 동일 |
| 0x57 | EMR_POLYGON16 | 동일 |

### 패스
| RecordType | 이름 |
|-----------|------|
| 0x3B | EMR_BEGINPATH |
| 0x3C | EMR_ENDPATH |
| 0x3D | EMR_CLOSEFIGURE |
| 0x3E | EMR_FILLPATH |
| 0x3F | EMR_STROKEANDFILLPATH |
| 0x40 | EMR_STROKEPATH |

## 공개 API

```rust
pub fn convert_to_svg(
    bytes: &[u8],
    render_rect: (f32, f32, f32, f32),  // (x, y, w, h) in pt
) -> Result<String, Error>;
```

EMF Bounds → render_rect 매핑 행렬을 `<g transform="matrix(sx 0 0 sy tx ty)">`로 감싸 출력한다.

## SVG 출력 매핑

| EMF 레코드 | SVG 출력 |
|-----------|---------|
| Rectangle | `<rect x y width height fill stroke stroke-width>` |
| RoundRect | `<rect ... rx ry>` |
| Ellipse | `<ellipse cx cy rx ry>` |
| Arc | `<path d="M ... A ... "/>` (fill=none) |
| Chord | `<path d="M ... A ... Z"/>` (fill 적용) |
| Pie | `<path d="M cx cy L ... A ... Z"/>` (중심 포함) |
| Polyline16 | `<polyline points>` (fill=none) |
| Polygon16 | `<polygon points>` (fill 적용) |
| PolyBezier16 | `<path d="M ... C ... C ..."/>` |
| LineTo | `<line x1 y1 x2 y2>` (currentPos 업데이트) |
| BeginPath..EndPath..FillPath | 누적된 `<path d>` 한 번에 출력 |

### 색상 매핑
- COLORREF `0x00BBGGRR` → `rgb(R, G, B)` (lowest byte = Red)
- Pen style PS_NULL(5) → stroke=none
- Brush style BS_NULL(1) → fill=none

## 검증 결과

```
$ cargo build --release       → OK
$ cargo test --release emf::  → 20 passed; 0 failed (기존 13 + 신규 7)
$ cargo test --release --lib  → 910 passed; 0 failed; 1 ignored
```

WMF 회귀 없음(독립 모듈 유지).

## 신규 단위 테스트

| 테스트 | 검증 |
|--------|------|
| parses_rectangle_and_ellipse | EMR_RECTANGLE/ELLIPSE RectL 필드 추출 |
| parses_polyline16_with_points | EMR_POLYLINE16 bounds + 점 3개 |
| parses_path_begin_end_fill | BeginPath/EndPath/FillPath 시퀀스 |
| convert_to_svg_emits_rect_with_stroke_and_fill | Pen+Brush+Rectangle → `<rect>` 속성 검증 (fill=rgb(255,0,0), stroke, width) |
| convert_to_svg_polyline_and_ellipse | Polyline16+Ellipse → `<polyline>/<ellipse>` 좌표 검증 |
| convert_to_svg_polygon_closes_shape | Polygon16 → `<polygon>` (폐곡선) |
| colorref_conversion_low_byte_is_red | COLORREF → rgb() 변환 정확성 |

## 설계 결정 사항

- **좌표 매핑 전략**: Player는 EMF 논리 좌표를 그대로 SVG 출력에 사용하고, `<g transform="matrix(...)">` 1회 래핑으로 render_rect에 배치. World/Window/Viewport 변환은 DC에 저장만 하고 단계 13~14에서 개별 요소에 적용 예정.
- **패스 상태 머신**: `Player.path_active + path_d` 멤버로 BeginPath~EndPath 구간 수집. FillPath 등에서 한 번에 `<path d>` 노드 출력.
- **스톡 객체 미구현**: handle의 상위 비트(`0x80000000`) 체크 없음. 실 EMF에서 SelectObject가 스톡 객체를 참조하면 ObjectTable miss로 DC의 pen/brush가 갱신되지 않아 기본값(검정 stroke) 사용됨.
- **PolyBezier 제어점 해석**: EMF 규약대로 첫 점이 시작점, 이후 3점씩(제어1, 제어2, 끝점) C 커맨드로 변환.
- **Arc/Chord/Pie**: SVG `<path>` elliptical arc 명령 `A rx ry 0 0 1 x y` 사용. 시작/끝 각도 재계산 없이 EMF의 start/end 포인트를 그대로 경로 끝점으로 사용.

## 미해결 이슈

- Arc 각도 계산 정밀도 — 현재는 EMF 시작/끝 포인트를 그대로 SVG arc 끝점으로 사용하지만, 엄격한 정확도는 각도 → 반타원 경계점 계산 필요. 실문서 검증 시 재평가.
- WorldTransform 적용 보류 — 개별 도형에 `<g transform>`을 추가로 감싸야 함. 단계 13에서 텍스트 레코드와 함께 구현.
- 스톡 객체(`0x80000000 | STOCK_*`) 구현 — 단계 13에서 DC 기본값 채우기 필요.

## 다음 단계

**단계 13**: 텍스트 레코드(`EMR_EXTTEXTOUTW`) + 비트맵 레코드(`EMR_STRETCHDIBITS`) 파서 + DIB → PNG base64 → `<image>` 임베딩 + 폰트 폴백 체인 연결.
