# 최종 보고서 — Task #683

**이슈**: [edwardkim/rhwp#683](https://github.com/edwardkim/rhwp/issues/683)
**브랜치**: `task683-clean` (stream/devel 기반)
**완료일**: 2026-05-08

## 결과 요약

`samples/pr-149.hwp` 의 **빈 paragraph + Para-relative TopAndBottom 그림** cluster 간 거리가 한컴 한글 2022 PDF 대비 -1584 HU (1 line) 부족하던 결함 정정. 모든 요소 ±1 px 정합 + 회귀 없음.

## 원인

빈 paragraph (text_len=0) 가 Para-relative TopAndBottom 그림(treat_as_char=false, caption 없음) 만 포함할 때:

- **rhwp pre-fix**: `result_y = pic_y + image_height` — 그림 paragraph 의 line baseline (line_height + line_spacing) 누락
- **한컴 한글 2022 PDF**: `result_y = pic_y + image_height + line(lh+ls)` — 그림 다음에 paragraph baseline 1줄 추가

Cluster 거리: rhwp 17280 HU vs PDF 18864 HU → **1 line (1584 HU) 부족**.

## 수정 내역

### `src/renderer/layout.rs::layout_shape_item` Picture 비-TAC + Para-relative 분기

`result_y = self.layout_body_picture(...)` 직후, 다음 가드 모두 만족 시 `result_y += line_height + line_spacing`:
- `pic.common.treat_as_char == false`
- `pic.common.text_wrap == TextWrap::TopAndBottom`
- `pic.common.vert_rel_to == VertRelTo::Para`
- `pic.caption.is_none()`
- 부모 paragraph 의 visible 텍스트 0

### `src/renderer/layout/integration_tests.rs`

신규 테스트 `test_task683_pr149_image_cluster_spacing` — pr-149.hwp SVG 출력의 그림 cluster 거리 검증 (PDF 18864 HU ±3 px).

## 검증

### 정합 (pr-149.hwp, 150 dpi)

| 요소 | PDF | rhwp 수정 후 | 차이 |
|------|-----|-------------|------|
| 그림1 top | 273 | 273 | 0 px |
| 그림2 top | 666 | 667 | +1 px |
| 그림3 top | 1059 | 1060 | +1 px |
| "회색조:" | 634 | 634 | 0 px |
| "흑백:" | 1028 | 1027 | -1 px |
| "입니다." | 1454 | 1454 | 0 px |
| Cluster 거리 | 18864 HU | 18896 HU | +32 HU (sub-pixel) |

### 회귀

- `cargo test --release` 모든 스위트 0 failures
- 동일 패턴 보유 샘플 회귀 없음 (exam_science, exam_eng, hwp-img-001, k-water-rfp 등)

## 영향 범위

| 항목 | 영향 |
|------|------|
| HWP3 / HWPX | 동일 IR 사용 → 자동 적용 |
| 머리말/꼬리말, 바탕쪽 | 영향 없음 (별도 layout 경로) |
| 표 셀 내부 그림 | 영향 없음 (`cell_ctx.is_some()`) |
| TAC 그림, caption 그림, Square/BehindText/InFrontOfText wrap | 영향 없음 (가드) |
| Skia 네이티브 렌더러 | 페이지네이션/레이아웃 결과 사용 → 자동 적용 |

## 후속 과제 (별개 이슈)

본 task 와 별개이지만 동일 샘플(pr-149.hwp) 에서 발견된 차이:

1. **흑백(BlackWhite) 효과 디더링** — SVG `feComponentTransfer discrete` 하드 임계값 vs 한컴 디더링. 별도 이슈 등록 권장.
2. **회색조(GrayScale) 효과** — 색상 매트릭스 자체는 BT.601 표준 정확. librsvg 렌더링 차이일 가능성 — 브라우저 검증 필요.

## 산출물

- 코드: `src/renderer/layout.rs`, `src/renderer/layout/integration_tests.rs`
- 문서: `mydocs/plans/task_m100_683.md`, `task_m100_683_impl.md`
- 단계별 보고: `mydocs/working/task_m100_683_stage{1,2,3}.md`
- 최종 보고: 본 문서

## 참고

- 한컴 한글 2022 PDF (정답지): `pdf/pr-149-2022.pdf`
- Stage 1 진단 데이터: `mydocs/working/task_m100_683_stage1.md`
