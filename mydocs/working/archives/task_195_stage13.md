# Task #195 단계 13 완료보고서 — 텍스트·비트맵 레코드

## 수행 내용

`EMR_EXTTEXTOUTW` (UTF-16 텍스트) + `EMR_STRETCHDIBITS` (비트맵) 파서와 Player 출력 로직을 추가했다. 텍스트는 DC의 LOGFONTW 속성을 SVG `<text>`의 font-family/size/weight/style에 매핑하고, 비트맵은 DIB(BITMAPINFO + 비트)를 BMP 파일 포맷으로 래핑하여 `data:image/bmp;base64,...` URL로 임베딩한다.

## 추가 파일

| 파일 | 역할 |
|------|------|
| `src/emf/parser/records/text.rs` | EMR_EXTTEXTOUTW 파서 (EmrText 고정부 40B + OutputString UTF-16 추출) |
| `src/emf/parser/records/bitmap.rs` | EMR_STRETCHDIBITS 파서 (72B 고정부 + BMI/bits 슬라이싱) |

## 수정 파일

| 파일 | 변경 |
|------|------|
| `src/emf/parser/mod.rs` | `Cursor::full_buf`, RT_EXT_TEXT_OUT_W(0x54) / RT_STRETCH_DI_BITS(0x51) 분기 |
| `src/emf/parser/records/mod.rs` | `Record::ExtTextOutW(ExtTextOut)`, `Record::StretchDIBits(StretchDIBits)` variant + re-export |
| `src/emf/converter/player.rs` | `emit_text` (SVG `<text>` + 폰트 매핑), `emit_bitmap` (DIB→BMP data URL), `dib_to_bmp_data_url` 헬퍼 |
| `src/emf/tests.rs` | 단계 13 테스트 5건 추가 (총 25건), 기존 Unknown 테스트 레코드 타입 0x54 → 0x70으로 교체 |

## 지원 레코드 (단계 13)

| RecordType | 이름 | 필드 |
|-----------|------|------|
| 0x54 | EMR_EXTTEXTOUTW | Bounds, GraphicsMode, ScaleX/Y, Reference, Options, Rectangle, Text (UTF-16→UTF-8) |
| 0x51 | EMR_STRETCHDIBITS | Bounds, xDest/yDest/cxDest/cyDest, BMI(Vec), Bits(Vec) |

## SVG 출력 매핑

### 텍스트
- `<text x=refX y=refY font-family="..." font-size="..." font-weight="..." font-style="..." fill="rgb(...)">내용</text>`
- `font-family`: DC의 LOGFONTW.face_name (비어있으면 `sans-serif`)
- `font-size`: `|LOGFONTW.height|` (기본 12)
- `font-weight`: `weight ≥ 700` ? `bold` : `normal`
- `font-style`: `italic != 0` ? `italic` : `normal`
- `fill`: DC의 `text_color`를 COLORREF → `rgb(R,G,B)` 변환
- XML 특수문자(`<`, `>`, `&`, `"`, `'`)는 `escape_xml`로 이스케이프

### 비트맵
- `<image x=xDest y=yDest width=cxDest height=cyDest preserveAspectRatio="none" href="data:image/bmp;base64,..."/>`
- DIB → BMP 변환: 14B `BITMAPFILEHEADER` (`"BM"` + fileSize + reserved=0 + dataOffset) + BMI + bits
- base64 인코딩: `base64` crate v0.22 (기존 의존성)
- 브라우저/SVG viewer는 data:image/bmp를 지원

## 검증 결과

```
$ cargo build --release       → OK
$ cargo test --release emf::  → 25 passed; 0 failed (기존 20 + 신규 5)
$ cargo test --release --lib  → 915 passed; 0 failed; 1 ignored
```

WMF 회귀 없음.

## 신규 단위 테스트

| 테스트 | 검증 |
|--------|------|
| parses_ext_text_out_w_ascii | "Hello" UTF-16 → UTF-8 추출, Reference 좌표 |
| parses_ext_text_out_w_korean | "가나다" UTF-16 서러게이트 없이 한글 3자 디코딩 |
| convert_to_svg_text_uses_font_face_name | CreateFont(Arial, bold 700) + SetTextColor(red) + ExtTextOutW → `font-family="Arial" font-weight="bold" fill="rgb(255,0,0)"` |
| parses_stretch_di_bits_and_emits_image | 2×2 BGRA 비트맵 → `<image>` 출력 + data URL 헤더 검증 |
| text_xml_special_chars_are_escaped | `a<b&c` → `a&lt;b&amp;c` 이스케이프 확인 |

## 설계 결정 사항

- **offset 해석**: EMR_EXTTEXTOUTW의 `offString`, EMR_STRETCHDIBITS의 `offBmiSrc/offBitsSrc`는 **레코드 시작(type+size 포함) 기준**. 디스패처는 payload(=record+8)를 전달하므로 `off - 8`로 환산한 인덱스를 사용.
- **Cursor API 확장**: 파서가 payload 전체 슬라이스를 인덱싱할 수 있도록 `Cursor::full_buf()` 추가. 기존 sub-cursor 격리는 유지.
- **비트맵 포맷**: DIB를 PNG로 재인코딩하는 대신 **BMP로 래핑**. 이유: 추가 crate 불필요(기존 `base64`만 사용), 속도 빠름, 정보 손실 없음, 모든 브라우저/SVG 뷰어가 `image/bmp` 지원.
- **폰트 크기**: LOGFONTW.height는 음수(cell height, device units 기준)가 일반적. `unsigned_abs()`로 양수화하여 SVG `font-size`(px)로 사용. 향후 XForm 적용 시 정확한 px 환산 필요.
- **offDx 무시**: 문자별 advance width 배열 미사용. SVG 텍스트는 브라우저 폰트 메트릭에 위임.

## 미해결 이슈

- 한국어 폰트 폴백 — rhwp의 `font_fallback_strategy.md` 체인과 연결되지 않음. `font-family` 값이 CSS 체인으로 감싸지지 않아 Arial 같은 영문 폰트에서 한글 글자 누락 가능. 단계 14에서 shape_layout 통합 시 재검토.
- DIB 상하반전 — 비트맵은 bottom-up 레이아웃이 일반적이나, 현재는 BMP로 그대로 래핑(BMP도 bottom-up)하므로 문제 없음. 만약 PNG로 전환 시 세로 뒤집기 필요.
- 비트맵 RasterOperation 무시 — SRCCOPY 외 래스터 연산자(SRCAND, BLACKNESS 등)는 처리하지 않음. 실 EMF에서는 대부분 SRCCOPY.
- 텍스트 회전 — LOGFONTW.escapement 미적용. 회전된 텍스트는 정위치로 출력됨. 후속 이슈.

## 다음 단계

**단계 14**: `shape_layout.rs` Ole arm에 EMF 폴백 경로 연결 (`emf::convert_to_svg` 호출) + 전체 회귀 + `samples/` export-svg 확인 + 최종 보고서.
