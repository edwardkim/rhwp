# Task #473 구현계획서

## 변경 위치

`src/renderer/svg.rs:2385-2404` `compute_image_crop_src`

## 변경 내용

### 핵심 아이디어

HWP crop 은 96-DPI HU 관행 (75 HU/px) 으로 인코딩. `original_size_hu` 가 그 관행과 일치하지 않으면 (사용자가 표시 크기 변경) 무시하고 75 HU/px 적용.

### Before

```rust
pub(crate) fn compute_image_crop_src(
    crop_hu: (i32, i32, i32, i32),
    original_size_hu: Option<(u32, u32)>,
    img_w_px: f64,
    img_h_px: f64,
) -> (f64, f64, f64, f64) {
    let (cl, ct, cr, cb) = crop_hu;
    let (scale_x, scale_y) = match original_size_hu {
        Some((ow, oh)) if ow > 0 && oh > 0 => (ow as f64 / img_w_px, oh as f64 / img_h_px),
        _ => {
            let s = cr as f64 / img_w_px;
            (s, s)
        }
    };
    let src_x = cl as f64 / scale_x;
    let src_y = ct as f64 / scale_y;
    let src_w = (cr - cl) as f64 / scale_x;
    let src_h = (cb - ct) as f64 / scale_y;
    (src_x, src_y, src_w, src_h)
}
```

### After

```rust
pub(crate) fn compute_image_crop_src(
    crop_hu: (i32, i32, i32, i32),
    original_size_hu: Option<(u32, u32)>,
    img_w_px: f64,
    img_h_px: f64,
) -> (f64, f64, f64, f64) {
    let (cl, ct, cr, cb) = crop_hu;
    // HWP crop 은 이미지 native 픽셀을 96-DPI HU 관행 (75 HU/px) 으로 인코딩.
    // [Task #473] original_size_hu (= ShapeComponentAttr.original_width/height) 는
    // 사용자가 삽입한 표시 HU 로, 이미지 native HU 와 다를 수 있음 (사용자가 그림
    // 크기를 변경한 경우). 표시 HU / img_px 가 75 ± 허용오차 일 때만 사용하고,
    // 아니면 96-DPI 관행 (75 HU/px) fallback.
    const HWP_CROP_DPI_SCALE: f64 = 75.0; // = 7200 HU/inch / 96 px/inch
    let scale_from_orig = original_size_hu
        .filter(|(ow, oh)| *ow > 0 && *oh > 0 && img_w_px > 0.0 && img_h_px > 0.0)
        .map(|(ow, oh)| (ow as f64 / img_w_px, oh as f64 / img_h_px))
        .filter(|(sx, sy)| {
            // 96-DPI 관행과 거의 같으면 채택 (오차 5%)
            (*sx - HWP_CROP_DPI_SCALE).abs() / HWP_CROP_DPI_SCALE < 0.05
                && (*sy - HWP_CROP_DPI_SCALE).abs() / HWP_CROP_DPI_SCALE < 0.05
        });
    let (scale_x, scale_y) = scale_from_orig.unwrap_or((HWP_CROP_DPI_SCALE, HWP_CROP_DPI_SCALE));
    let src_x = cl as f64 / scale_x;
    let src_y = ct as f64 / scale_y;
    let src_w = (cr - cl) as f64 / scale_x;
    let src_h = (cb - ct) as f64 / scale_y;
    (src_x, src_y, src_w, src_h)
}
```

### 효과

| 케이스 | original_size_hu | scale_from_orig | 채택 | scale 결과 |
|--------|---|---|---|---|
| exam_kor 헤더 | 174000 / 2320 = 75.00 | (75, 75) | ✓ | 75 (기존과 동일) |
| 21_언어_기출 그림 | 26640 / 2220 = 12.00 | filter 탈락 | ✗ | 75 (보정) |
| original_size_hu 없음 | None | None | - | 75 (fallback) |

기존 테스트 4건 모두 영향 점검:
- `test_compute_image_crop_src_exam_kor_header` (orig=174000): scale=75 그대로 ✓
- `test_compute_image_crop_src_no_crop_full_image` (orig=174000): scale=75 그대로 ✓
- `test_compute_image_crop_src_offset_top_left` (orig=4000, img=400): orig/img=10. **75 와 5% 안에 들어가지 않음** → 75 fallback. 기존 기대값 변경 필요.
- `test_compute_image_crop_src_fallback_when_original_size_missing` (None): 75 fallback (기존 102366/2320=44.12 와 다름). 기존 기대값 변경 필요.

→ 기존 테스트 두 건은 **현실 HWP 데이터를 반영하지 않는 가상 입력** (10 HU/px 등) 이므로 의도 변경 명시 + 기대값 갱신.

## 단계 구성

### Stage 1: 신규 테스트 + 현행 fail 확인

`src/renderer/layout/integration_tests.rs::test_473_picture_crop_viewbox_matches_image_px`

`samples/21_언어_기출_편집가능본.hwp` 페이지 12 SVG 의 `<svg viewBox="...">` 안의 `<image width="...">` 비교. viewBox width / image width 비율이 1.0 ± 0.1 이어야 함 (현행: ~6.25).

### Stage 2: 코드 수정 + green + 기존 unit test 갱신

- `src/renderer/svg.rs:2385-2404` 변경
- `src/renderer/svg/tests.rs::test_compute_image_crop_src_offset_top_left` 기대값 갱신 (10 HU/px 가상 → 75 fallback)
- `test_compute_image_crop_src_fallback_when_original_size_missing` 기대값 갱신

### Stage 3: 광범위 회귀 시각 검증

- exam_kor (헤더 로고 부분 crop) 영향 없음 확인
- 21_언어_기출 12p 그림 정상 표시 확인
- hwpspec, aift, kps-ai, k-water-rfp 그림 비교

## 검증 명령

```bash
cargo build --release
cargo test --release
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 11 -o /tmp/p12/
./target/release/rhwp export-svg samples/exam_kor.hwp -o /tmp/exam_kor_post/
# 시각 비교 후 정리
```
