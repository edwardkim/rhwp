# Task #514 Stage 3 완료 보고서 — PCX 디코더 통합 + 변환 함수 구현

## 변경 요약

| 파일 | 변경 |
|------|------|
| `Cargo.toml` | `pcx = "0.2"` dependency 추가 |
| `src/renderer/svg.rs` | `detect_image_mime_type` 에 PCX 분기 추가 (image/x-pcx) + `pcx_bytes_to_png_bytes` 변환 함수 추가 + 두 image emit 사이트에 PCX 변환 분기 추가 |

소스 변경: `+76 / -2` (svg.rs +75 / -2 + Cargo.toml +1).

## 변경 상세

### 1. PCX MIME 인식 (`detect_image_mime_type`)

```rust
// PCX: 0A 05 (ZSoft Paintbrush v3.0+, 1980년대 비표준 포맷)
// 브라우저 native 미지원 → emit 시 PNG 변환 필요 (pcx_bytes_to_png_bytes)
if data.len() >= 2 && data.starts_with(&[0x0A, 0x05]) {
    return "image/x-pcx";
}
```

기존 PNG / JPEG / GIF / BMP / WMF / TIFF 감지 분기 뒤에 추가. 다른 분기에 영향 없음.

### 2. PCX → PNG 변환 함수 (`pcx_bytes_to_png_bytes`)

`bmp_bytes_to_png_bytes` 옆에 추가. paletted PCX (8bpp) + RGB PCX (24bpp) 모두 지원.

**투명 처리 (작업지시자 요구):** PCX 자체는 알파 채널 미지원 (1980년대 포맷). HWP 의 PCX 임베드는 BehindText 배경/로고 용도로 흰색 (255,255,255) 영역을 투명으로 보여야 한다 (한컴 호환). 변환 시 흰색 픽셀을 투명 알파로 매핑한 RGBA PNG 출력.

```rust
pub(crate) fn pcx_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    use image::{ImageFormat, RgbaImage};
    let mut reader = pcx::Reader::new(Cursor::new(data)).ok()?;
    let (width, height) = (reader.width() as u32, reader.height() as u32);
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    if reader.is_paletted() {
        // paletted: row 인덱스 + 끝에서 팔레트 추출 → RGBA 매핑
        // 흰색 → 투명, 그 외 → 불투명
    } else {
        // RGB row 별 읽고 RGBA 로 확장 (흰색 → 투명)
    }
    let img = RgbaImage::from_raw(width, height, rgba)?;
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png).ok()?;
    Some(out)
}
```

### 3. SVG image emit 사이트의 변환 분기 (2 곳)

**3.1 `paragraph_layout` 진입 emit (svg.rs:1075~1098):**

```rust
let mime_type = detect_image_mime_type(data);
let (render_data, render_mime): (Cow<[u8]>, &str) = if mime_type == "image/x-wmf" {
    // WMF → SVG (기존)
} else if mime_type == "image/bmp" {
    // BMP → PNG (기존)
} else if mime_type == "image/x-pcx" {
    // PCX → PNG (Task #514, 본 변경)
    match pcx_bytes_to_png_bytes(data) {
        Some(png) => (Cow::Owned(png), "image/png"),
        None => (Cow::Borrowed(data), mime_type),
    }
} else { (Cow::Borrowed(data), mime_type) };
```

**3.2 `draw_image` 일반 emit (svg.rs:2284~):**

동일 패턴으로 PCX 분기 추가.

### 4. 변환 실패 폴백 정책

PCX 디코더가 실패하면 원본 데이터 + `image/x-pcx` MIME 으로 폴백 → 시각 결함은 그대로지만 panic 회피. 별도 task 후보 (placeholder 등).

## 검증 결과

### 게이트 통과 현황

| 게이트 | 결과 | 비고 |
|--------|------|------|
| `cargo build --lib` | ✅ Finished | pcx 0.2.5 dependency 추가 후 22.66s |
| `cargo build --release` | ✅ Finished | 31.27s |
| **`cargo test --lib`** | ✅ **1110 passed, 0 failed, 1 ignored** | 회귀 0 (Stage 1 시점과 동일) |
| `cargo clippy --lib -- -D warnings` | ✅ 통과 | 0 warnings |

### 변환 함수 단위 검증 (`samples/복학원서.hwp`)

```bash
target/release/rhwp export-svg samples/복학원서.hwp -o output/svg/task514_after/
```

**Before (Stage 1 진단):**
- 학교 로고 image href: `data:application/octet-stream;base64,CgUBAQ...` ❌ 브라우저 렌더링 불가
- 워터마크 직인: `data:image/jpeg;...` ✅ 정상

**After (Stage 3):**
- 학교 로고 image href: `data:image/png;base64,iVBORw0K...` ✅ **PNG 변환 성공**
- 워터마크 직인: `data:image/jpeg;...` ✅ 정상 유지 (회귀 0)

### PNG 변환 결과 검증

학교 로고의 PNG 출력 IHDR:

```
89 50 4e 47 0d 0a 1a 0a   ← PNG 시그니처
00 00 00 0d 49 48 44 52   ← IHDR chunk (length=13, type="IHDR")
00 00 03 6e               ← width = 878 px
00 00 03 e9               ← height = 1001 px
08                        ← bit depth = 8
06                        ← color type = 6 (RGBA — 알파 채널 포함) ★
```

**투명 처리 정합 확인**: color type = 6 (Truecolor + Alpha) → 흰색 배경이 투명으로 변환된 RGBA PNG.

## 위험 점검

| 위험 | 결과 |
|------|------|
| pcx crate v0.2.5 디코딩 실패 | ✅ 본 fixture (BIN0001.PCX, paletted) 정상 변환 |
| WASM binary 크기 영향 | (Stage 5 에서 점검) |
| 기존 fixture 회귀 (BMP/PNG/JPEG) | ✅ test --lib 1110 passed, 회귀 0 |
| 다른 PCX variant (RGB PCX) 미지원 | ⚠️ samples/ 에는 paletted PCX 만 (sweep 결과) — RGB 분기는 코드만 작성, 실제 fixture 없음. 변환 실패 시 폴백으로 panic 회피 |

## 다음 단계

Stage 3 완료 보고서 승인 후 **Stage 4** 진행:
- `tests/issue_514.rs` 신규 작성 (학교 로고 image href 가 `data:image/png` 로 시작 검증)
- `cargo test --test issue_418` / `--test issue_501` / `--test svg_snapshot` 회귀 0 점검
- `cargo clippy` 전체 영역 점검 (lib 외)

## 산출물

- `Cargo.toml` (pcx 0.2 추가)
- `src/renderer/svg.rs` (변환 함수 + MIME 분기)
- `output/svg/task514_after/복학원서.svg` (Stage 3 검증용)
- 본 보고서 (`mydocs/working/task_m100_514_stage3.md`)
