# Task #477 Stage 2 — 정정 적용 (범위 확장: Task #430 회귀 포함)

## 정정 요약

본 정정 진행 중 작업지시자 시각 검증에서 추가 결함 정황 발견:
- **k-water-rfp 1쪽 pi=31 K-water 로고**가 박스보다 확대되어 표시
- 작업지시자 통찰: *"이미지를 crop 시킨 후 확대하는 잘못된 해석"*

회귀 origin 추적 결과 **PR #434 / Task #430 (commit a5541f9, @planet6897)** 가 회귀 origin 으로 확정. 본 #477 작업 범위에 Task #430 회귀 정정 통합 (작업지시자 옵션 B 선택).

## 회귀 origin

| 항목 | 정합 |
|------|------|
| PR | **#434** (cherry-pick @planet6897 5 commits) |
| Task | **#430** "그림 자동 크롭 미구현 — exam_kor 헤더" |
| 회귀 origin commit | **`a5541f9`** "Task #430 Stage 2: ImageNode.original_size_hu + svg.rs crop 공식 교정" |
| 머지 시점 | 2026-04-29 02:39 |

## 변경 파일 (4개소)

| # | 파일 | 변경 내용 |
|---|------|----------|
| 1 | `src/renderer/svg.rs` | `compute_image_crop_src` 를 HWP 표준 75 HU/px 룰 기반 단일 계산식으로 정합화 |
| 2 | `src/renderer/svg/tests.rs` | 단위 테스트 갱신 (75 HU/px 룰 정합) + pi=31 케이스 추가 |
| 3 | `src/renderer/layout/table_layout.rs` | TAC 표 셀 안 단독 이미지 클램프 (라인 1532-1548) |
| 4 | `src/renderer/layout/table_partial.rs` | TAC 분할 표 셀 안 단독 이미지 클램프 (라인 707-720) |
| 5 | `src/renderer/layout/shape_layout.rs` | TAC 도형 컨테이너 안 인라인 그림 클램프 (라인 1515-1527) |

## 정정 1 — `compute_image_crop_src` 룰 정합화

**HWP 표준 룰**: 1 inch = 7200 HU = 96 px → **75 HU/px** (DPI 96).

```rust
// 정정 후 (75 HU/px 룰)
const HU_PER_PX: f64 = 75.0;
let scale_x = HU_PER_PX;
let scale_y = HU_PER_PX;
let src_x = cl as f64 / scale_x;
let src_y = ct as f64 / scale_y;
let src_w = (cr - cl) as f64 / scale_x;
let src_h = (cb - ct) as f64 / scale_y;
```

`original_size_hu` 인자는 라운드트립 보존 메타로만 유지하며 계산에는 사용하지 않는다.

### 룰 검증 — 두 케이스 모두 정합

| 케이스 | image bin 본질 | crop_w_hu / 75 | image 폭 | 결과 |
|--------|---------------|---------------|---------|------|
| **A** k-water pi=31 | PNG = crop 후 image | 12660 / 75 = **168.8** | 169 | viewBox ≈ image 전체 → 정상 |
| **B** exam_kor 헤더 | PNG = 원본 image | 102366 / 75 = **1364.88** | 2320 | viewBox = image 좌측 58.8% → 정상 (Task #430 의도 보존) |

## 정정 2 — 호출부 셀 폭 클램프 (3개소)

```rust
// [Task #477] 셀/도형 컨테이너 폭 초과 시 비율 유지 클램프
let clamped_w = pic_w.min(inner_area.width);
let clamped_h = if pic_w > 0.0 { pic_h * (clamped_w / pic_w) } else { pic_h };
let pic_area = LayoutRect { x: inline_x, y: tac_img_y, width: clamped_w, height: clamped_h };
self.layout_picture(tree, &mut cell_node, pic, &pic_area, ...);
inline_x += clamped_w;
continue;
```

## 정정 작동 확인

### k-water-rfp 1쪽 pi=31 (Task #430 회귀 정정)

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| nested SVG | `<svg width=162 viewBox="0 0 151.54 83.35">` | (없음 — 단순 `<image width=162>`) |
| image 표시 영역 | image 좌측 89.66% 만 (1.069배 stretch) | image 전체가 외부 박스에 stretch ✅ |
| 작업지시자 시각 판정 | (베이스라인 결함) | 정상 ✅ |

### k-water-rfp 16쪽 pi=186 (본 #477 본질)

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| 외부 SVG 폭 | **623.61 (셀 폭 619.76 초과!)** | **606.16 (inner_area.width)** ✅ |
| 외부 SVG 높이 | 133.93 | 130.18 (비율 유지 축소) |
| 작업지시자 시각 판정 | (베이스라인 결함) | 정상 ✅ |

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1078 passed** (1075 + 단위 테스트 3 추가) ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |
| WASM 빌드 | 4,203,122 bytes ✅ |

## 트러블슈팅 / 위키 작성

- 트러블슈팅: [`mydocs/troubleshootings/image_crop_scale_rule.md`](../troubleshootings/image_crop_scale_rule.md)
- 위키: [HWP 그림 Crop Scale 룰](https://github.com/edwardkim/rhwp/wiki/HWP-%EA%B7%B8%EB%A6%BC-Crop-Scale-Rule)

## 다음 단계

- **Stage 4 (작업지시자 시각 검증)**: 통과 — 1쪽 + 16쪽 표 안 이미지 모두 정상화
- **Stage 5**: 최종 결과보고서 + 오늘할일 갱신 + 머지 + 이슈 close
