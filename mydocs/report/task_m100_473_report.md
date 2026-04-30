# Task #473 최종 결과 보고서

## 이슈

- 번호: #473
- 제목: 그림 crop 변환의 scale 기준 오류 — original_size_hu(표시 HU) 가 96-DPI native HU 와 일치하지 않을 때 viewBox 과대
- 마일스톤: M100
- 브랜치: `local/task473`

## 증상

`samples/21_언어_기출_편집가능본.hwp` 페이지 12 우측 단 `<보기>` 표(pi=258) 내부 그림이 의도된 크기의 약 16% 비율로 축소 표시.

```xml
<svg viewBox="0 0 13875 10333.75"><image width="2220" height="1654" .../></svg>
```

viewBox(13875) 안에 image(2220) → 그림이 좌측 상단 1/6.25 영역에만 표시.

## 근본 원인

`src/renderer/svg.rs:2385-2404` `compute_image_crop_src` 가 `original_size_hu` (표시 HU, ShapeComponentAttr.original_width/height) 를 crop 좌표 변환 scale 기준으로 사용.

HWP `crop` 은 **이미지 native 픽셀을 96-DPI HU 관행 (75 HU/px) 으로 인코딩**. `original_size_hu` 는 사용자가 지정한 표시 크기로, 이미지 native HU 와 다를 수 있음.

| 케이스 | original_size_hu/img_px | 의도 scale (75) | 기존 scale | 결과 |
|--------|---|---|---|---|
| exam_kor 헤더 (native 삽입) | 174000/2320 = 75 | 75 | 75 (orig=image native HU) | ✓ |
| 21_언어_기출 그림 (축소 삽입) | 26640/2220 = 12 | 75 | 12 (orig=display HU) | ✗ 6.25× viewBox 과대 |

원인 확인:
- 이미지 binary 2220×1654 px × 75 HU/px = 166500×124080 HU (= crop 값과 일치)
- 표시 26640×19860 HU (= 94×70mm) ≠ 이미지 native HU

## 수정

`compute_image_crop_src` scale 산출 로직을 96-DPI 관행 (75 HU/px) 기반으로 변경:

```rust
const HWP_CROP_DPI_SCALE: f64 = 75.0;
let scale_from_orig = original_size_hu
    .filter(|(ow, oh)| *ow > 0 && *oh > 0 && img_w_px > 0.0 && img_h_px > 0.0)
    .map(|(ow, oh)| (ow as f64 / img_w_px, oh as f64 / img_h_px))
    .filter(|(sx, sy)|
        (*sx - HWP_CROP_DPI_SCALE).abs() / HWP_CROP_DPI_SCALE < 0.05
            && (*sy - HWP_CROP_DPI_SCALE).abs() / HWP_CROP_DPI_SCALE < 0.05);
let (scale_x, scale_y) = scale_from_orig.unwrap_or((HWP_CROP_DPI_SCALE, HWP_CROP_DPI_SCALE));
```

`original_size_hu / img_px` 가 75 ± 5% 안일 때만 채택 (역호환), 아니면 75 fallback.

## 추가/갱신 테스트

### 신규
- `src/renderer/layout/integration_tests.rs::test_473_picture_crop_viewbox_matches_image_px`

### 기존 갱신 (가상 입력 → 96-DPI 관행)
- `test_compute_image_crop_src_offset_top_left` — orig=10 HU/px, 75 fallback 적용
- `test_compute_image_crop_src_fallback_when_original_size_missing` — None, 75 fallback 적용

### 영향 없음 (orig=75 HU/px)
- `test_compute_image_crop_src_exam_kor_header`
- `test_compute_image_crop_src_no_crop_full_image`

## 검증 결과

| 항목 | 결과 |
|------|------|
| 신규 통합 테스트 | PASS |
| 기존 unit test 4건 (2건 갱신) | PASS |
| 전체 cargo test (1078건) | 1078 / 1078 PASS |
| 21_언어_기출 12p 그림 시각 확인 | nested SVG 제거, plain `<image 355×265>` 정상 렌더 ✓ |
| 광범위 샘플 nested-svg-image 회귀 | 415 페이지 / 32 nested SVG / **0 viewBox 과대** ✓ |

샘플 7종 검증 (21_언어_기출 / exam_kor / exam_eng / hwpspec / aift / k-water-rfp / kps-ai). exam_kor partial crop 20건 (헤더 로고) 정상 보존.

## 영향 범위

- 모든 `Picture` 컨트롤의 crop 좌표 변환.
- 주로 사용자가 그림 크기를 변경한 케이스에서 visual 정정.
- 표시 HU = 이미지 native HU (75 HU/px) 인 케이스는 영향 없음.

## 학습 (메모리 적용)

`feedback_essential_fix_regression_risk` 메모리 적용:
- 1차 회귀 검사 시 `abs(ratio - 1.0) > 0.1` 로 양방향 검출 → exam_kor 20/20 false alarm.
- partial crop (viewBox < image) 은 정상 동작이므로 검사 기준을 `ratio > 1.1` (그림 축소만) 으로 좁힘.
- **거짓 양성 재해석** 으로 본질 보존 확인.

## 산출물

- `src/renderer/svg.rs` (`compute_image_crop_src` 변경)
- `src/renderer/svg/tests.rs` (unit test 2건 갱신)
- `src/renderer/layout/integration_tests.rs` (테스트 1건 추가)
- `mydocs/plans/task_m100_473{,_impl}.md`
- `mydocs/working/task_m100_473_stage{1,2,3}.md`
- `mydocs/report/task_m100_473_report.md`
