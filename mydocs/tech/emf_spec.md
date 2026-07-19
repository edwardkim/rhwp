---
kind: reference
status: active
canonical: mydocs/tech/emf_spec.md
last_verified: 2026-07-17
---

# EMF (Enhanced Metafile) 스펙 정리 — rhwp 1차 구현 범위

> 참조: [MS-EMF] Enhanced Metafile Format (v20180912), Microsoft Open Specifications
> 작업 컨텍스트: Task #195 단계 9 — 스펙 조사
> **본 문서는 rhwp 1차 구현(GDI 기본 레코드)에 필요한 항목만 발췌**한다. 전체 스펙(200+ 레코드)은 다루지 않는다.

## 1. 파일 구조

EMF는 **레코드 시퀀스 기반 메타파일**이다. 구조:

```
[EMR_HEADER (type=1)]                  ← 고정 위치, 최소 88바이트 + 확장
[EMR_* 레코드 ... ]                    ← 드로잉/객체/상태/텍스트/비트맵
[EMR_EOF (type=14)]                    ← 종료
```

- 모든 레코드는 **리틀엔디언**
- 모든 레코드 공통 헤더 8바이트: `Type (u32 LE) + Size (u32 LE, 자신 포함 전체 바이트 수)`
- `Size`는 항상 4의 배수(정렬 요구)

### 레코드 공통 헤더

| 필드 | 타입 | 설명 |
|------|------|------|
| Type | u32 | RecordType enum 값 |
| Size | u32 | 레코드 전체 크기(바이트, 이 헤더 8바이트 포함) |

## 2. EMR_HEADER (RecordType = 1)

파일 선두. 첫 번째 레코드는 반드시 헤더.

### 고정 88바이트 (Header)

| Offset | 필드 | 타입 | 설명 |
|--------|------|------|------|
| 0 | Type | u32 | = 1 |
| 4 | Size | u32 | 이 헤더 전체 크기 (보통 88 또는 108/116 — 확장 포함) |
| 8 | Bounds | RECTL (16B) | 논리 좌표계 bounding box (left, top, right, bottom) |
| 24 | Frame | RECTL (16B) | 물리 단위(0.01mm) bounding box |
| 40 | Signature | u32 | " EMF" = `0x464D4520` |
| 44 | Version | u32 | 보통 `0x00010000` |
| 48 | Bytes | u32 | 파일 전체 바이트 수 |
| 52 | Records | u32 | 전체 레코드 개수 |
| 56 | Handles | u16 | 객체 테이블 최대 핸들 수(+1) |
| 58 | Reserved | u16 | 0 |
| 60 | nDescription | u32 | Description 문자열 문자수(UTF-16) |
| 64 | offDescription | u32 | Description offset (이 헤더 시작부터) |
| 68 | nPalEntries | u32 | 팔레트 엔트리 수 |
| 72 | Device | SIZEL (8B) | 참조 장치 픽셀(예: 1920×1080) |
| 80 | Millimeters | SIZEL (8B) | 참조 장치 mm |

### 확장 Header Extension 1 (선택, Size ≥ 100)

| Offset | 필드 | 타입 | 설명 |
|--------|------|------|------|
| 88 | cbPixelFormat | u32 | PixelFormatDescriptor 크기 |
| 92 | offPixelFormat | u32 | PixelFormatDescriptor offset |
| 96 | bOpenGL | u32 | OpenGL 레코드 포함 여부 |

### 확장 Header Extension 2 (선택, Size ≥ 108)

| Offset | 필드 | 타입 | 설명 |
|--------|------|------|------|
| 100 | MicrometersX | u32 | 참조 장치 X 마이크로미터 |
| 104 | MicrometersY | u32 | 참조 장치 Y 마이크로미터 |

### rhwp 처리 방침

- 최소 필수: Bounds, Frame, Signature 검증, Device/Millimeters
- Description / PixelFormat / OpenGL / Micrometers는 파싱하되 미사용
- Signature 불일치 시 `ParseError::InvalidSignature` 반환

## 3. 주요 구조체

### RECTL (16바이트)

| 필드 | 타입 | 설명 |
|------|------|------|
| left | i32 | 논리 좌표 |
| top | i32 | |
| right | i32 | |
| bottom | i32 | |

### POINTL (8바이트)

| 필드 | 타입 |
|------|------|
| x | i32 |
| y | i32 |

### POINTS (4바이트) — EMR_POLYLINE**16** 등 압축형

| 필드 | 타입 |
|------|------|
| x | i16 |
| y | i16 |

### SIZEL (8바이트)

| 필드 | 타입 |
|------|------|
| cx | i32 |
| cy | i32 |

### XFORM (24바이트) — 2×3 affine 변환

| 필드 | 타입 | 의미 |
|------|------|------|
| eM11, eM12 | f32 | [a b] |
| eM21, eM22 | f32 | [c d] |
| eDx, eDy | f32 | [tx ty] |

변환: `x' = x·eM11 + y·eM21 + eDx`, `y' = x·eM12 + y·eM22 + eDy`

### COLORREF (4바이트)

`0x00BBGGRR` (alpha 항상 0, rhwp는 `rgb(R,G,B)`로 변환)

### LOGPEN (16바이트) — `EMR_CREATEPEN` 용

| 필드 | 타입 |
|------|------|
| lopnStyle | u32 (PenStyle) |
| lopnWidth.x | i32 (픽셀 너비) |
| lopnWidth.y | i32 (사용 안 함) |
| lopnColor | COLORREF |

PenStyle 주요 값: `PS_SOLID=0`, `PS_DASH=1`, `PS_DOT=2`, `PS_DASHDOT=3`, `PS_NULL=5`

### LOGBRUSH (12바이트)

| 필드 | 타입 |
|------|------|
| lbStyle | u32 (BrushStyle) |
| lbColor | COLORREF |
| lbHatch | u32 (HatchStyle, Style=Hatched일 때만) |

BrushStyle 주요: `BS_SOLID=0`, `BS_NULL=1`, `BS_HATCHED=2`

### LOGFONTW (92바이트)

| Offset | 필드 | 타입 | 설명 |
|--------|------|------|------|
| 0 | Height | i32 | 폰트 높이 (논리 단위, 음수면 cell height) |
| 4 | Width | i32 | 평균 문자 폭 (0=자동) |
| 8 | Escapement | i32 | 각도(0.1도 단위, 회전) |
| 12 | Orientation | i32 | |
| 16 | Weight | i32 | 굵기 (400=normal, 700=bold) |
| 20 | Italic | u8 | |
| 21 | Underline | u8 | |
| 22 | StrikeOut | u8 | |
| 23 | CharSet | u8 | |
| 24 | OutPrecision | u8 | |
| 25 | ClipPrecision | u8 | |
| 26 | Quality | u8 | |
| 27 | PitchAndFamily | u8 | |
| 28 | FaceName | [u16;32] | UTF-16 폰트명(null-terminated, 최대 32자) |

## 4. RecordType 1차 카탈로그 (rhwp 범위)

| Type | 이름 | 카테고리 | 단계 |
|------|------|---------|------|
| 1 | EMR_HEADER | 헤더 | 10 |
| 14 | EMR_EOF | 제어 | 10 |
| 2 | EMR_POLYBEZIER | 드로잉 | 12 |
| 3 | EMR_POLYGON | 드로잉 | 12 |
| 4 | EMR_POLYLINE | 드로잉 | 12 |
| 21 | EMR_SETWINDOWEXTEX | 상태 | 11 |
| 22 | EMR_SETWINDOWORGEX | 상태 | 11 |
| 23 | EMR_SETVIEWPORTEXTEX | 상태 | 11 |
| 24 | EMR_SETVIEWPORTORGEX | 상태 | 11 |
| 25 | EMR_SETBRUSHORGEX | 상태 | 11 |
| 27 | EMR_SETMAPMODE | 상태 | 11 |
| 17 | EMR_SETBKMODE | 상태 | 11 |
| 22 | (SetTextAlign=22는 중복 — 실제 값은) EMR_SETTEXTALIGN=22 | — | (스펙 원문 참조, 파서에서 정확 값 사용) |
| 33 | EMR_SCALEVIEWPORTEXTEX | 상태 | 11 |
| 34 | EMR_SCALEWINDOWEXTEX | 상태 | 11 |
| 33 | EMR_SETTEXTCOLOR | 상태 | 11 |
| 34 | EMR_SETBKCOLOR | 상태 | 11 |
| 35 | EMR_SAVEDC | 상태 | 11 |
| 36 | EMR_RESTOREDC | 상태 | 11 |
| 37 | EMR_SETWORLDTRANSFORM | 상태 | 11 |
| 38 | EMR_MODIFYWORLDTRANSFORM | 상태 | 11 |
| 39 | EMR_SELECTOBJECT | 객체 | 11 |
| 38 | EMR_CREATEPEN | 객체 | 11 |
| 39 | EMR_CREATEBRUSHINDIRECT | 객체 | 11 |
| 40 | EMR_DELETEOBJECT | 객체 | 11 |
| 82 | EMR_EXTCREATEFONTINDIRECTW | 객체 | 11 |
| 42 | EMR_ELLIPSE | 드로잉 | 12 |
| 43 | EMR_RECTANGLE | 드로잉 | 12 |
| 44 | EMR_ROUNDRECT | 드로잉 | 12 |
| 45 | EMR_ARC | 드로잉 | 12 |
| 46 | EMR_CHORD | 드로잉 | 12 |
| 47 | EMR_PIE | 드로잉 | 12 |
| 54 | EMR_LINETO | 드로잉 | 12 |
| 27 | EMR_MOVETOEX | 드로잉 | 12 |
| 59 | EMR_BEGINPATH | 패스 | 12 |
| 60 | EMR_ENDPATH | 패스 | 12 |
| 61 | EMR_CLOSEFIGURE | 패스 | 12 |
| 62 | EMR_FILLPATH | 패스 | 12 |
| 63 | EMR_STROKEANDFILLPATH | 패스 | 12 |
| 64 | EMR_STROKEPATH | 패스 | 12 |
| 84 | EMR_EXTTEXTOUTW | 텍스트 | 13 |
| 81 | EMR_STRETCHDIBITS | 비트맵 | 13 |
| 86 | EMR_POLYLINE16 | 드로잉 | 12 |
| 85 | EMR_POLYGON16 | 드로잉 | 12 |
| 87 | EMR_POLYBEZIER16 | 드로잉 | 12 |

> **주의**: 위 표의 일부 Type 값은 빠른 참조용이다. 구현 시 MS-EMF 공식 카탈로그에서 최종 확정한다(값 충돌 불가). 현재 WMF RecordType은 포맷이 달라 재사용 불가.

## 5. EMF vs WMF 차이 요약

| 항목 | WMF | EMF |
|------|-----|-----|
| 좌표 크기 | 16bit (i16) | 32bit (i32) |
| 레코드 크기 필드 | u32 (word 단위) | u32 (byte 단위) |
| 헤더 | 18바이트 META_PLACEABLE(옵션) + 18바이트 META_HEADER | 88+ 바이트 EMR_HEADER |
| 좌표계 단위 | device/logical, 애플리케이션 의존 | 0.01mm 물리 단위까지 명시 (Frame) |
| 파일 매직 | `0x9AC6CDD7` (placeable) 또는 없음 | " EMF" (offset 40) |
| RecordType enum | WMF-specific | EMF-specific, 완전히 별개 |
| 객체 핸들 | 로컬 테이블 | 동일 개념, 크기 명시(nHandles) |
| 패스 레코드 | 없음 (PolyPath 제한적) | BeginPath/EndPath/FillPath 풍부 |
| 텍스트 | ETO_OPAQUE 등 플래그 제한 | EXTTEXTOUTW + 옵션 풍부 |
| 변환 행렬 | 없음 (MapMode로만) | XFORM 레코드 직접 |

결론: **파서·컨버터 코드 공유 금지**, 독립 `src/emf/` 모듈로 구축.

## 6. 좌표계 변환 파이프라인

EMF의 좌표는 다음 단계로 SVG viewport에 매핑:

```
논리 좌표 (Bounds 기반)
  ↓ World Transform (SetWorldTransform / ModifyWorldTransform)
월드 좌표
  ↓ Window/Viewport (SetWindowExt/Org, SetViewportExt/Org, MapMode)
페이지 좌표
  ↓ Frame (EMR_HEADER.Frame, 0.01mm)
물리 좌표
  ↓ rhwp render_rect (외부에서 전달)
SVG viewport
```

rhwp 1차 구현 단순화:
- MapMode는 `MM_ANISOTROPIC`만 지원 (Window/Viewport 스케일 계산)
- World Transform은 2×3 affine 행렬로 SVG `transform="matrix(...)"` 직접 출력
- Frame은 bounding box 확정용으로만 사용

## 7. 1차 구현 제외 목록

| 제외 항목 | 사유 | 후속 처리 |
|----------|------|-----------|
| EMF+ (EMR_COMMENT_EMFPLUSRECORD) | GDI+ 확장, 별도 렌더 엔진 필요 | 후속 이슈 |
| EMR_ALPHABLEND | 고급 합성 | 후속 이슈 |
| EMR_GRADIENTFILL | 그라데이션 | 후속 이슈 |
| EMR_MASKBLT, EMR_PLGBLT | 복잡한 비트맵 합성 | 후속 이슈 |
| ICM 색상 관리 | 정밀 색 재현 | 범위 외 |
| 클리핑 리전 (EMR_SELECTCLIPRGN 등) | 복잡한 클립 합성 | 후속 이슈 |
| OpenGL 레코드 | 특수 | 범위 외 |
| PixelFormat | OpenGL 종속 | 범위 외 |

## 8. 참조

- [MS-EMF]: <https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-emf/>
- [MS-WMF]: <https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/>
- rhwp 기존 WMF 파서: `src/wmf/` (레이아웃 참조, 코드는 공유하지 않음)
- 1.hwp OlePres000 실측 (Task #195 단계 7 보고서)
