# Task #195 단계 11 완료보고서 — 객체/상태 레코드 파서 + DC 스택

## 수행 내용

EMF 파서에 객체(펜/브러시/폰트 생성·선택·삭제) + 상태(SaveDC/RestoreDC, WorldTransform, Window/Viewport, Color/Mode) 레코드 분기를 추가했다. 컨버터 기반 구조로 `DeviceContext` / `DcStack` / `ObjectTable`을 신설했다.

## 추가 파일

| 파일 | 역할 |
|------|------|
| `src/emf/parser/objects/xform.rs` | `XForm` (2×3 affine, 24B) + `identity` |
| `src/emf/parser/objects/logpen.rs` | `LogPen` (16B) |
| `src/emf/parser/objects/logbrush.rs` | `LogBrush` (12B) |
| `src/emf/parser/objects/logfont.rs` | `LogFontW` (92B, FaceName UTF-16 → UTF-8) |
| `src/emf/parser/records/object.rs` | CreatePen / CreateBrushIndirect / ExtCreateFontIndirectW / SelectObject / DeleteObject 파서 |
| `src/emf/parser/records/state.rs` | SaveDC / RestoreDC / WorldTransform / Window/Viewport / Color/Mode 파서 |
| `src/emf/converter/mod.rs` | 컨버터 엔트리 |
| `src/emf/converter/device_context.rs` | `DeviceContext`, `DcStack`, `ObjectTable`, `GraphicsObject` |

## 수정 파일

| 파일 | 변경 |
|------|------|
| `src/emf/mod.rs` | `pub mod converter;` 추가 |
| `src/emf/parser/objects/mod.rs` | logpen/logbrush/logfont/xform re-export |
| `src/emf/parser/records/mod.rs` | `Record` enum 14개 variant 추가 |
| `src/emf/parser/mod.rs` | `dispatch` 함수 — RecordType별 분기, sub-cursor로 페이로드 경계 격리 |
| `src/emf/tests.rs` | 단위 테스트 7건 추가 (총 13건) |

## 지원 레코드 (단계 11)

### 객체
| RecordType | 이름 | 페이로드 |
|-----------|------|---------|
| 0x25 | EMR_SELECTOBJECT | handle(u32) |
| 0x26 | EMR_CREATEPEN | handle + LogPen(16B) |
| 0x27 | EMR_CREATEBRUSHINDIRECT | handle + LogBrush(12B) |
| 0x28 | EMR_DELETEOBJECT | handle(u32) |
| 0x52 | EMR_EXTCREATEFONTINDIRECTW | handle + LogFontW(92B) + 선택 확장 스킵 |

### 상태
| RecordType | 이름 | 페이로드 |
|-----------|------|---------|
| 0x09 | EMR_SETWINDOWEXTEX | SizeL(8B) |
| 0x0A | EMR_SETWINDOWORGEX | PointL(8B) |
| 0x0B | EMR_SETVIEWPORTEXTEX | SizeL(8B) |
| 0x0C | EMR_SETVIEWPORTORGEX | PointL(8B) |
| 0x11 | EMR_SETMAPMODE | u32 |
| 0x12 | EMR_SETBKMODE | u32 |
| 0x16 | EMR_SETTEXTALIGN | u32 |
| 0x18 | EMR_SETTEXTCOLOR | u32(COLORREF) |
| 0x19 | EMR_SETBKCOLOR | u32 |
| 0x21 | EMR_SAVEDC | (빈) |
| 0x22 | EMR_RESTOREDC | i32(iRelative) |
| 0x23 | EMR_SETWORLDTRANSFORM | XForm(24B) |
| 0x24 | EMR_MODIFYWORLDTRANSFORM | XForm + u32(mode) |

## 검증 결과

```
$ cargo build --release
    Finished `release` profile

$ cargo test --release emf::
13 passed; 0 failed

$ cargo test --release --lib
903 passed; 0 failed; 1 ignored
```

- 신규 EMF 테스트 7건 추가, 기존 EMF 테스트 6건 유지 → EMF 전체 13건
- 전체 라이브러리 회귀: 903 (기존 896 + EMF 신규 7) 모두 통과
- WMF 테스트 영향 없음(독립 모듈)

## 신규 단위 테스트

| 테스트 | 검증 |
|--------|------|
| parses_create_pen_and_select_and_delete | EMR_CREATEPEN → SELECT → DELETE 시퀀스, LogPen 필드 추출 |
| parses_create_brush_indirect | EMR_CREATEBRUSHINDIRECT LogBrush 추출 |
| parses_ext_create_font_indirect_w | LogFontW 92B + FaceName UTF-16 "Arial" 디코딩 |
| parses_dc_stack_and_world_transform | SaveDC / SetWorldTransform(XForm) / RestoreDC(-1) 시퀀스 |
| parses_window_viewport_and_colors | SetWindowExtEx / SetViewportOrgEx / SetTextColor / SetBkMode |
| dc_stack_save_restore_round_trip | DcStack LIFO 동작 — 중첩 save 후 상대 restore |
| object_table_insert_get_remove | ObjectTable insert/get/remove/len/is_empty |

## 설계 결정 사항

- **sub-cursor 격리**: `dispatch` 함수가 각 레코드의 페이로드를 정확한 길이로 잘라 별도 `Cursor`로 넘긴다. 레코드 파서가 실수로 다음 레코드 영역을 침범하는 것을 구조적으로 방지.
- **RestoreDC iRelative**: MS-EMF 2.3.11 규약에 따라 음수는 상대(pop 개수), 양수는 절대 깊이. 단계 11은 **음수(상대)만 구현**, 양수는 false 반환. 실 EMF에서 대부분 `-1` 사용.
- **LogFontExDv 확장 스킵**: `EMR_EXTCREATEFONTINDIRECTW`는 LogFontW 외에 EliteSpace DV 확장을 가질 수 있으나, 1차 렌더에 불필요하므로 확장부는 `payload_len - 96`만큼 커서만 전진.
- **COLORREF 포맷**: `0x00BBGGRR` 그대로 저장. SVG 변환 시(단계 12) R/G/B 분리.
- **의존성 제약**: `HashMap`만 std 사용. 외부 crate 추가 없음.

## 미해결 이슈

- RestoreDC 양수(절대 깊이) 미지원 — 실 파일 사용 시 추가 고려
- 스톡 객체 핸들(`0x80000000 | STOCK_xxx`) 구분 — 단계 12 드로잉 레코드에서 SelectObject가 스톡 객체를 선택할 때 ObjectTable 조회 실패를 감안해야 함

## 다음 단계

**단계 12**: 드로잉 레코드(선/사각형/타원/패스/폴리라인16) 파서 + SVG 컨버터 `Player` + `SvgBuilder`로 기본 도형 EMF → SVG 출력.
