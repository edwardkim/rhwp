# 구현계획서 — Task #683

**이슈**: [edwardkim/rhwp#683](https://github.com/edwardkim/rhwp/issues/683)
**브랜치**: `task683-clean` (stream/devel 기반)
**수행계획서**: `mydocs/plans/task_m100_683.md`

## 사전 조사

- `src/renderer/layout/picture_footnote.rs::layout_body_picture` 반환값 (L387):
  `(VertRelTo::Para, _) => base_y + total_height` — pic_height + caption stuff 만 포함
- `src/renderer/layout.rs::layout_shape_item` Picture 비-TAC + Para-relative 분기 (L2978~):
  `result_y = self.layout_body_picture(...)` — 즉 image_height 만 진행
- 한글 2022 PDF 는 그림 다음에 paragraph 의 line(lh+ls) 1줄을 추가 진행

## 채택 산식

빈 paragraph (visible 텍스트 0) + Para-relative TopAndBottom 그림 (caption 없음) 의 layout 진행량:

```
result_y = base_y + image_height + caption_overhead   // 기존
         + line_height + line_spacing                 // 신규 (Task #683)
```

가드:
- `pic.common.treat_as_char == false`
- `pic.common.text_wrap == TextWrap::TopAndBottom`
- `pic.common.vert_rel_to == VertRelTo::Para`
- `pic.caption.is_none()`
- 부모 paragraph 의 visible 텍스트 0

## 구현 단계

### Stage 1 — 진단 및 산식 확정

- PDF/SVG 정밀 측정 (150dpi PIL 분석)
- 동일 패턴 다른 샘플 식별
- 산식 (A: measure_paragraph) vs (B: layout_shape_item caller) 비교 → **(B) 채택**

### Stage 2 — 산식 구현

- `layout.rs::layout_shape_item` Picture 분기에 가드 + line_advance 추가
- 단위 테스트 추가 (`test_task683_pr149_image_cluster_spacing`)

### Stage 3 — 시각 검증 및 회귀 테스트

- pr-149.hwp ±10 px 정합 확인
- `cargo test` 전체 통과 검증
- 동일 패턴 보유 다른 샘플 시각 회귀 검증

### Stage 4 — 마무리 및 보고

- 최종 보고서 + orders 갱신
- 커밋 + PR

## 위험 요소

1. 다른 샘플 회귀 — 가드 조건이 좁아 영향 범위 작음, 광범위 회귀 검증으로 대응
2. caption 보유 그림 — 가드로 제외 (별도 케이스)
3. HWPX 동일 패턴 — 같은 IR 사용으로 자동 적용 예상
