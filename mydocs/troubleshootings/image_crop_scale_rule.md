# 그림 crop scale 룰 — HWP 표준 75 HU/px

## 결함 정황

`samples/k-water-rfp.hwp` 1쪽 K-water 로고 (pi=31), 16쪽 두 번째 표 셀 안 그림 등이 **셀 / 박스 폭을 초과해 확대된 모습**으로 표시. 한컴 에디터 정답지와 시각상 다름.

## 회귀 origin

| 항목 | 정합 |
|------|------|
| PR | **#434** (cherry-pick @planet6897 5 commits) |
| Task | **#430** "그림 자동 크롭 미구현 — exam_kor 헤더 '국어 영역(A 형)'" |
| 회귀 origin commit | **`a5541f9`** "Task #430 Stage 2: ImageNode.original_size_hu + svg.rs crop 공식 교정" |
| 머지 시점 | 2026-04-29 02:39 |
| 컨트리뷰터 | Jaeook Ryu (planet6897) |

## 잘못된 가정

Task #430 의 `compute_image_crop_src` 계산:

```rust
// 잘못된 계산 (Task #430 도입)
scale_x = orig_w_hu / img_w_px   // 14119 / 169 = 83.54 HU/px
src_w = (cr - cl) / scale_x      // 12660 / 83.54 = 151.54 (image 폭 169 의 89.66%)
```

**가정**: PNG/JPEG 이 **HWP 의 원본 image** 이고 `pic.crop` 이 PNG 안에서 잘라낼 영역.

→ 결과: viewBox = `0 0 151.54 83.35` → **image 의 좌측 89.66% 만** 외부 박스에 1.069배 stretch 표시 = **확대된 모습**

## 본질 — HWP 의 두 가지 image 케이스

| 케이스 | image bin 본질 | 예 |
|--------|---------------|------|
| 케이스 A | **PNG = crop 적용 후 image** (한컴이 BinData 에 저장 시 이미 crop) | k-water-rfp pi=31 (K-water 로고), 16쪽 표 셀 안 그림 |
| 케이스 B | **PNG = 원본 image** (crop 메타가 별도 영역 정의) | exam_kor 헤더 ("국어 영역(A 형)"), Task #430 정정 의도 케이스 |

Task #430 은 케이스 B 만 가정 → 케이스 A 회귀.

## 정합 룰 (Task #477)

**HWP 표준 룰**: 1 inch = 7200 HU = 96 px → **75 HU/px** (DPI 96).

```rust
// 정합 계산 (Task #477)
const HU_PER_PX: f64 = 75.0;
let scale_x = HU_PER_PX;
let scale_y = HU_PER_PX;
let src_x = cl as f64 / scale_x;
let src_y = ct as f64 / scale_y;
let src_w = (cr - cl) as f64 / scale_x;
let src_h = (cb - ct) as f64 / scale_y;
```

`original_size_hu` 인자는 라운드트립 보존 메타로만 유지하며 계산에는 사용하지 않는다.

## 룰 검증 — 두 케이스 모두 정합

### 케이스 A (k-water pi=31)
- crop_w = 12660 HU → src_w = 12660 / 75 = **168.8 ≈ image 폭 169**
- src_w 가 image 폭과 거의 같음 → `is_cropped = false` → 단순 `<image>` 출력
- **결과**: image 전체가 외부 박스 (162 × 89.11) 에 stretch — 정상

### 케이스 B (exam_kor 헤더)
- crop_w = 102366 HU → src_w = 102366 / 75 = **1364.88** (image 폭 2320 의 58.8%)
- src_w 가 image 보다 충분히 작음 → `is_cropped = true` → nested SVG + viewBox
- **결과**: image 의 좌측 58.8% 만 외부 박스에 stretch — Task #430 정정 의도 보존

## 재발 방지

이미지 처리 PR 머지 시 점검 항목:

1. **HWP 표준 DPI 96 (75 HU/px) 룰 정합 확인** — `compute_image_crop_src` 의 scale 계산식이 표준 룰 외 다른 식이면 재검토
2. **두 케이스 모두 시각 검증**:
   - PNG = crop 후 image (k-water-rfp pi=31, 16쪽 셀 안 그림)
   - PNG = 원본 image (exam_kor 헤더)
3. **단위 테스트** — `compute_image_crop_src_kwater_pi31` (회귀 정정 검증) + `compute_image_crop_src_exam_kor_header` (Task #430 의도 보존)

## 일반화된 원칙

**휴리스틱이 아닌 룰** — HWP 표준 (1 inch = 7200 HU = 96 px) 은 명세이며, image 처리에서 일관 적용해야. PR 의 정정이 일부 케이스를 정정해도 **명세 룰을 벗어난 계산**라면 다른 케이스에서 회귀 위험.

작업지시자 통찰: *"이미지 컨트롤의 크기는 같지만, 한컴은 전체 이미지가 커진 것이고, rhwp 는 이미지를 crop 시킨 후 확대하는 잘못된 해석"*.

## 관련 메모리

- `feedback_hancom_compat_specific_over_general` — 일반화 알고리즘 (Task #430 의 `orig/img_w` scale) 보다 명시 룰 (75 HU/px) 우선
- `feedback_v076_regression_origin` — 외부 컨트리뷰터 PR 가 자기 환경에서 정합인 계산식이 다른 케이스 회귀 origin
- `feedback_visual_regression_grows` — 자동 검증 (cargo test, byte 비교) 만으로는 시각 결함 검출 불가, 작업지시자 시각 판정 게이트 필수
