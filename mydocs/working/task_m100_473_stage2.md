# Task #473 Stage 2 완료 보고서

## 단계 목표

`src/renderer/svg.rs:2385-2404` `compute_image_crop_src` scale 계산을 96-DPI 관행 (75 HU/px) 기반으로 변경.

## 변경 내용

```rust
const HWP_CROP_DPI_SCALE: f64 = 75.0; // 7200 HU/inch / 96 px/inch
let scale_from_orig = original_size_hu
    .filter(|(ow, oh)| *ow > 0 && *oh > 0 && img_w_px > 0.0 && img_h_px > 0.0)
    .map(|(ow, oh)| (ow as f64 / img_w_px, oh as f64 / img_h_px))
    .filter(|(sx, sy)|
        (*sx - HWP_CROP_DPI_SCALE).abs() / HWP_CROP_DPI_SCALE < 0.05
            && (*sy - HWP_CROP_DPI_SCALE).abs() / HWP_CROP_DPI_SCALE < 0.05);
let (scale_x, scale_y) = scale_from_orig.unwrap_or((HWP_CROP_DPI_SCALE, HWP_CROP_DPI_SCALE));
```

`original_size_hu / img_px` 가 75 ± 5% 안에 있을 때만 채택 (역호환), 아니면 75 fallback. 이미지 native scale 정확.

## 기존 unit test 갱신

| 테스트 | 입력 (orig/img) | 기존 동작 | 새 동작 | 비고 |
|--------|----|----|----|----|
| `test_compute_image_crop_src_exam_kor_header` | 174000/2320=75 | scale=75 | scale=75 | 영향 없음 ✓ |
| `test_compute_image_crop_src_no_crop_full_image` | 174000/2320=75 | scale=75 | scale=75 | 영향 없음 ✓ |
| `test_compute_image_crop_src_offset_top_left` | 4000/400=10 | scale=10 | scale=75 | **기대값 갱신** (가상 입력) |
| `test_compute_image_crop_src_fallback_when_original_size_missing` | None | scale=cr/img_w | scale=75 | **기대값 갱신** |

가상 입력 두 건은 96-DPI 관행이 아니므로 새 정책이 더 정확.

## 결과

- 신규 통합 테스트 PASS
- 기존 unit test 4건 PASS (2건 갱신)
- 21_언어_기출 12p 그림: nested SVG 미생성 (full crop 으로 src=img 일치) → plain `<image width=355.2 height=264.8>` 으로 정상 렌더

## 다음 단계

Stage 3 — 광범위 샘플 회귀 시각 검증.
