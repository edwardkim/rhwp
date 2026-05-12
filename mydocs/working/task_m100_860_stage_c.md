# Task #860 Stage C 단계 보고서 (진척)

**선행**: Stage A (BMP URI 미지원 본질), Stage B (정정 후보 1)
**브랜치**: `local/task860`
**작성일**: 2026-05-13

## 작업 요약

후보 1 (BMP → PNG 변환) 적용. 결과: **inner image MIME `bmp` → `png` 정합 확인**. 그러나 **시각 결과는 동일** (박스 외곽만, 콘텐츠 누락). 추가 결함 발견.

## 적용한 fix

### 1. `src/emf/converter/player.rs:368-393` `dib_to_bmp_data_url`

```rust
if let Some(png) = crate::renderer::svg::bmp_bytes_to_png_bytes(&bmp) {
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
    format!("data:image/png;base64,{b64}")
} else {
    // fallback: BMP
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bmp);
    format!("data:image/bmp;base64,{b64}")
}
```

### 2. `src/wmf/converter/svg/util.rs:20-27` `Bitmap::as_data_url`

```rust
if let Some(png) = crate::renderer::svg::bmp_bytes_to_png_bytes(self.as_slice()) {
    format!("data:image/png;base64,{}", STANDARD.encode(&png))
} else {
    format!("data:image/bmp;base64,{}", STANDARD.encode(self.as_slice()))
}
```

### 3. `src/emf/tests.rs:712` test assert update

```rust
assert!(
    svg.contains("href=\"data:image/png;base64,")
        || svg.contains("href=\"data:image/bmp;base64,"),
    "expected PNG (or BMP fallback) data URI"
);
```

## 검증 결과

**cargo test**: 1230 passed, 0 failed, 2 ignored — **회귀 0**.

**inner SVG 의 image MIME 변화**:
- Pre-fix: `data:image/bmp;base64,...`
- **Post-fix: `data:image/png;base64,...`** ✓

**시각 결과**: **여전히 동일** (page 2 PNG: 박스 외곽만, BMP 내부 텍스트 누락).

## 추가 결함 발견

inner SVG 의 image element 의 **width attribute 누락**:

```xml
<image height="768" href="data:image/png;base64,...">
       ^^^^^^^^^^^^
       width 없음
```

rsvg-convert 가 width 없는 image 의 intrinsic size 처리 미흡 → image 렌더링 누락.

### 추정 원인

WMF converter 의 3 image generation path:
1. `ternary_raster_operator.rs:95-101` (SRCCOPY) — `set("x", "y", "width", "height", "href")`
2. `util.rs:100` (Brush::DIBPatternPT) — `set("x", "y", "width", "height", "href")`
3. `util.rs:187` (Brush::Pattern) — `set("x", "y", "width", "height", "href")`

모두 width 명시. 그러나 inner SVG 의 image 에 width 부재.

가능성:
- (a) Node::set 의 BTreeMap iteration 에서 width 누락 (debug 필요)
- (b) WMF record 의 dest_width / width 가 어떤 edge case 에서 attribute 출력 차단
- (c) WMF 의 다른 path (예: META_SETDIBTODEV — 현재 `not implemented` 로그) 사용

### 조사 미완료

WMF 의 record 처리 순서 또는 SVG output 의 actual generation path 추가 trace 필요.

## 다음 단계 옵션

- **A**: Stage C 추가 진단 — WMF→SVG converter 의 image width 누락 본질 추적
- **B**: postprocess workaround — `convert_wmf_to_svg` 결과 SVG 의 image element 에 width 자동 추가 (viewBox width 또는 PNG natural width)
- **C**: 현재 fix (BMP→PNG MIME) 만 commit + width 누락은 별도 task 등록
- **D**: 본 task fix 전체 revert + 별도 task 신설

## 산출 아티팩트

- `src/emf/converter/player.rs` (BMP → PNG 변환)
- `src/wmf/converter/svg/util.rs` (BMP → PNG 변환)
- `src/emf/tests.rs` (test assert update)
- `/tmp/860_fix2/p2.png` (rhwp page 2 PNG, fix 후, 여전히 박스 외곽만)
- `/tmp/860_fix2/inner.svg` (inner SVG, image PNG 임)
- 본 보고서
