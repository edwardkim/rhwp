# Task #860 Stage A 단계 보고서

**선행**: Task #860 계획서 (수행+구현 승인 완료)
**브랜치**: `local/task860`
**작성일**: 2026-05-13

## 작업 요약

`samples/hwp3-sample14.hwp` 의 그림 (bin_id=2) 내부 콘텐츠 렌더링 누락의 본질 정밀 진단.

## 진단 결과

### A1. 한컴 PDF page 2 의 image stream

```bash
pdfimages -all "pdf/hwp3-sample14-hwp5-2022.pdf" output/860/pdf_img
```

추출 결과: 다수 JPEG + PNG (정상 raster image).

### A2. rhwp SVG 의 image element 구조

rhwp SVG `<image>` href 가 **2단 wrap** 형식:

```
<image href="data:image/svg+xml;base64,..." />
  ↓ (inner SVG decode)
<svg viewBox="0 0 4231 1189">
  <image height="768" href="data:image/bmp;base64,..." />
</svg>
  ↓ (BMP decode)
BMP file (135734 bytes, 353×128, 24bpp, no compression)
```

BMP 콘텐츠 (직접 `sips` 로 PNG 변환 + 시각 확인): **terminal screenshot 정확 표시** — "If you leave me now / You'll take away the biggest part of me / Ooh-ooh, baby ase pl[c]don't go".

→ **BMP image data 자체는 정상**.

### A3. rsvg-convert 의 BMP 렌더링 검증

inner SVG 만 별도 추출 + rsvg-convert 로 PNG 변환:

```bash
rsvg-convert -d 96 /tmp/860/inner.svg -o /tmp/860/inner.png
```

결과: 박스 외곽 (점선) + 화살표 (▲) + 라벨 텍스트 표시. **그러나 BMP image 누락**.

→ **rsvg-convert (SVG renderer) 가 `data:image/bmp` URI 미지원**.

### A4. 가설 결판

| 가설 | 검증 결과 |
|---|---|
| H1 BinData 추출 실패 | ✗ (BMP 데이터 정상 추출) |
| H2 Vector primitives 미지원 | ✗ (이건 raster BMP) |
| H3 Image format 미지원 (decoder 측) | ✗ (BMP 데이터 자체 정상) |
| **H4 SVG embedding 실패** | **△ partially** (embed 됐지만 BMP URI) |
| **H5 SVG renderer 의 BMP URI 미지원** | **✓ 확정** |

**본질 위치**: `src/emf/converter/player.rs:368-384` 의 `dib_to_bmp_data_url` 함수.

EMF (Enhanced Metafile) 의 `StretchDIBits` record 처리 시 DIB (Device Independent Bitmap) 를 BMP 파일 헤더 + bits 형태로 wrap 한 후 `data:image/bmp;base64,...` URI 생성. rsvg-convert (및 다수 SVG renderer) 가 BMP MIME type 미지원 → image 누락.

## 정정 방향

**BMP → PNG 변환 후 data URI embed**:

```rust
// 현재: data:image/bmp;base64,{bmp_b64}
// 변경: data:image/png;base64,{png_b64}
//   - BMP 데이터를 image crate 로 decode
//   - PNG 로 encode
//   - base64 인코딩
```

`image` crate (Cargo.toml 의존성 이미 존재, features = ["bmp", "png"]) 사용으로 변환 가능.

## 영향 범위

- 본 fix 는 `src/emf/converter/player.rs` (EMF 처리 공통 모듈) 위치
- HWP3 / HWPX / HWP5 모두의 EMF 안의 DIB image 렌더링에 영향
- 모든 sample 에서 BMP → PNG 로 embed 형식 변경

### CLAUDE.md 규칙 정합 확인

CLAUDE.md: "HWP3 전용 로직은 `src/parser/hwp3/` 안에서만 구현".

본 fix 는 EMF converter (공통 모듈) 변경 — HWP3 전용 로직 아님. EMF 는 모든 포맷 (HWP3, HWP5, HWPX) 에서 vector graphic 으로 사용. 공통 모듈 fix 가 적절.

## 회귀 위험

- BMP → PNG 변환은 lossless (같은 24bpp RGB) → 시각 동일
- rsvg-convert 의 PNG 지원 명확 → 다른 sample 에 정합 (또는 개선)
- 회귀 위험: **낮음**
- 검증 필수: 기존 EMF image 보유 sample (모든 sample) 회귀 검사

## 산출 아티팩트

- `output/860/pdf_img-*` (PDF page 2 의 image stream 추출)
- `/tmp/860/img0.bmp` (rhwp SVG 의 inner BMP 추출)
- `/tmp/860/img0.png` (BMP → PNG 변환, terminal screenshot 시각)
- `/tmp/860/inner.svg` (rhwp SVG 의 inner SVG)
- `/tmp/860/inner.png` (inner SVG → rsvg-convert, BMP 누락 확인)
- 본 보고서

## 후속 단계

Stage B: 정정 후보 (`dib_to_bmp_data_url` → BMP→PNG 변환) 평가 + 회귀 위험.
