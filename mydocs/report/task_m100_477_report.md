# Task #477 최종 결과보고서 — 표 셀 안 그림 클램프 + 이미지 Crop Scale 룰 정합화

## 결과 요약

본 작업 시작점은 **k-water-rfp.hwp 16쪽 두 번째 표 셀 안 그림이 셀 경계 초과**한 결함 정정. 작업 진행 중 시각 검증에서 **1쪽 K-water 로고 등 다른 케이스 결함**도 발견 → **PR #434 / Task #430 회귀** 가 origin 으로 확정. 본 #477 작업 범위에 Task #430 회귀 정정 통합.

**최종 정정**: HWP 표준 75 HU/px 룰 기반 단일 계산식 + 호출부 셀 폭 클램프.

## 정정 본질

### 1) 회귀 origin — Task #430 (PR #434, commit a5541f9)

**잘못된 가정**: PNG/JPEG 이 **HWP 의 원본 image** 이고 `pic.crop` 이 PNG 안에서 잘라낼 영역.

```rust
// 잘못된 계산식 (Task #430)
scale_x = orig_w_hu / img_w_px   // 14119 / 169 = 83.54 HU/px
src_w = (cr - cl) / scale_x      // 12660 / 83.54 = 151.54 (image 폭의 89.66%)
```

→ viewBox = `0 0 151.54 83.35` → image 의 **좌측 89.66% 만** 외부 박스에 1.069배 stretch 표시 = **확대된 모습**

### 2) HWP 의 두 가지 image bin 케이스

| 케이스 | image bin 본질 | 예 |
|--------|---------------|------|
| **A** | PNG = crop 적용 후 image | k-water-rfp pi=31 (K-water 로고), 표 셀 안 그림 다수 |
| **B** | PNG = 원본 image | exam_kor 헤더 ("국어 영역(A 형)"), Task #430 정정 의도 케이스 |

Task #430 은 케이스 B 만 가정 → 케이스 A 회귀.

### 3) 정합 룰 — HWP 표준 75 HU/px (DPI 96)

**1 inch = 7200 HU = 96 px → 75 HU/px**.

```rust
const HU_PER_PX: f64 = 75.0;
let scale_x = HU_PER_PX;
let scale_y = HU_PER_PX;
let src_x = cl as f64 / scale_x;
let src_y = ct as f64 / scale_y;
let src_w = (cr - cl) as f64 / scale_x;
let src_h = (cb - ct) as f64 / scale_y;
```

두 케이스 모두 자동 정합 처리:
- **A**: `crop_w_hu / 75 ≈ image 폭` → src_w ≈ img_w → image 전체 표시
- **B**: `crop_w_hu / 75 < image 폭` → src_w 가 image 좌측 일부 → 좌측 영역만 표시

## 변경 파일 (5개소)

| 파일 | 변경 |
|------|------|
| `src/renderer/svg.rs` | `compute_image_crop_src` 75 HU/px 룰 정합화 |
| `src/renderer/svg/tests.rs` | 단위 테스트 4 갱신 + pi=31 케이스 1 추가 |
| `src/renderer/layout/table_layout.rs` | TAC 표 셀 안 그림 클램프 |
| `src/renderer/layout/table_partial.rs` | TAC 분할 표 셀 안 그림 클램프 |
| `src/renderer/layout/shape_layout.rs` | TAC 도형 컨테이너 안 그림 클램프 |

## Stage 별 결과

| Stage | 내용 | 결과 |
|-------|------|------|
| 1 | 베이스라인 측정 | k-water-rfp 16쪽 pi=186 외부 SVG 폭 623.61 (셀 폭 619.76 초과) 결함 확정 |
| 2 | 정정 적용 + 범위 확장 | 호출부 3개소 셀 폭 클램프 + Task #430 회귀 정정 (75 HU/px 룰) |
| 3 | 광범위 회귀 검증 | cargo test 1078 passed (1075 + 단위 테스트 3 추가), 광범위 byte 회귀 정합 |
| 4 | 작업지시자 시각 검증 | 1쪽 + 16쪽 모두 정상 ✅ |
| 5 | 최종 결과보고서 + 머지 | 본 문서 |

## 정정 작동 확인

### k-water-rfp 1쪽 pi=31 (Task #430 회귀)

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| nested SVG | `<svg width=162 viewBox="0 0 151.54 83.35">` | (없음 — 단순 `<image width=162>`) |
| image 표시 영역 | image 좌측 89.66% (1.069배 확대) | image 전체가 외부 박스에 stretch |

### k-water-rfp 16쪽 pi=186 (본 #477 본질)

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| 외부 SVG 폭 | 623.61 (셀 폭 619.76 초과!) | 606.16 (inner_area.width) |
| 외부 SVG 높이 | 133.93 | 130.18 (비율 유지 축소) |

### 단위 테스트 — 두 케이스 정합 보존

- `test_compute_image_crop_src_kwater_pi31` (신규): 케이스 A 회귀 정정 검증 (12660/75 = 168.8)
- `test_compute_image_crop_src_exam_kor_header` (갱신): 케이스 B 정정 의도 보존 (102366/75 = 1364.88)

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1078 passed** ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ (Task #418 정정 보존) |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |
| WASM 빌드 | 4,203,122 bytes ✅ |
| 작업지시자 시각 검증 | 통과 ✅ |

## 회귀 origin 보존

- **트러블슈팅**: [`mydocs/troubleshootings/image_crop_scale_rule.md`](../troubleshootings/image_crop_scale_rule.md)
- **위키**: [HWP 그림 Crop Scale 룰](https://github.com/edwardkim/rhwp/wiki/HWP-%EA%B7%B8%EB%A6%BC-Crop-Scale-Rule)

## 메모리 원칙 정합

- **`feedback_hancom_compat_specific_over_general`**: 일반화 알고리즘 (Task #430 의 `orig/img_w` scale) 보다 명시 룰 (75 HU/px) 우선
- **`feedback_v076_regression_origin`**: 외부 컨트리뷰터 PR 가 자기 환경에서 정합인 계산식이 다른 케이스 회귀 origin → 머지 전 작업지시자 시각 판정 게이트
- **`feedback_visual_regression_grows`**: 자동 검증 (cargo test, byte 비교) 만으로는 시각 결함 검출 불가, 작업지시자 시각 판정이 절차의 핵심 게이트
- **`feedback_word_choice_calculation`**: "산수" 대신 "계산" 사용 — 본 작업 중 작업지시자 피드백 보존

## 작업지시자 통찰

본 작업의 본질 진단은 작업지시자의 통찰로 도출:

1. *"컨트리뷰터들 중 이미지 처리쪽 PR 처리 후 발생되는 문제입니다"* — 회귀 origin 가설
2. *"이미지를 crop 시킨 후 확대하는 잘못된 해석"* — 결함 본질
3. *"이건 휴리스틱이 아닙니다. 룰입니다."* — 정정 방향 (HWP 표준 75 HU/px)

## 다음 단계

- 이슈 #477 close
- `local/task477` → `local/devel` 머지
- 오늘할일 갱신
