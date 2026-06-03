# Stage 1 — 진단 및 산식 확정 (Task #683)

## 요약

`samples/pr-149.hwp` 의 그림 cluster 간격 차이는 **빈 paragraph + `wrap=TopAndBottom` 그림** 의 layout 시 그림 다음에 paragraph 의 line baseline 이 추가되지 않아 발생.

## 진단 데이터 (150dpi)

### PDF (한글 2022) 정밀 측정

| 요소 | y 범위 |
|------|--------|
| 그림1 | 273..600 |
| "회색조:" | **634..649** |
| 그림2 | 666..993 |
| "흑백:" | 1028..1042 |
| 그림3 | 1059..1387 |
| "입니다." | 1454..1472 |

그림 간 거리: 393 px = **18864 HU**.

### rhwp SVG 측정 (수정 전)

| 요소 | y 범위 |
|------|--------|
| 그림1 | 273..600 (동일) |
| "회색조:" | **600..617** ← PDF보다 34px 위 |
| 그림2 | 633..961 ← PDF보다 33px 위 |
| ... | (누적 차이) |

그림 간 거리: 360 px = **17280 HU** (PDF 대비 -1584 HU).

### file 의 LINE_SEG vpos (참조)

```
p2 (그림1 paragraph)  vpos=3200
p3 ("회색조:")         vpos=18896  (= 3200 + 15696)
p4 (그림2 paragraph)  vpos=20496  (= 18896 + 1600)
```

→ file 자체는 그림 paragraph 가 image_height 만 차지하도록 인코딩되어 있으나, 한글 2022 layout 은 이를 무시하고 +1 line 을 그림 다음에 추가.

## 코드 추적

### 현재 rhwp 동작 (layout)

- `layout.rs::layout_shape_item` Picture 비-TAC + Para-relative 분기:
  - `pic_y = para_start_y[para_index]` = paragraph 시작 y
  - `result_y = self.layout_body_picture(...)`
- `picture_footnote.rs::layout_body_picture` 반환값:
  - `(VertRelTo::Para, _) => base_y + total_height` (= pic_height + caption stuff)
  - **layout 실 진행**: 그림 paragraph 당 image_height 만

### Hancom 한글 2022 추정 동작

빈 paragraph + `wrap=TopAndBottom` 그림 시:
- 그림은 paragraph 시작 y 에 배치 (image1 top 위치 일치)
- 그림 다음에 paragraph 의 line baseline 1줄(line_height + line_spacing) 만큼 추가 진행
- 즉 layout 진행량 = `image_height + line_height + line_spacing`

## 채택 산식

**빈 paragraph (text_len = 0) + TopAndBottom 그림 (treat_as_char = false, caption 없음) 의 layout 진행량:**

```
result_y = base_y + image_height + caption_overhead
         + line_height + line_spacing   ← 신규 추가 (빈 paragraph + caption 없음 한정)
```

## 수정 위치 후보 비교

### 후보 A — `picture_footnote.rs::layout_body_picture` 반환값

- 장점: 모든 caller 에 자동 반영
- 단점: 머리말/꼬리말, 바탕쪽, 표 셀 내부 caller 들이 paragraph 컨텍스트 없이 호출 → 빈 paragraph 판정 불가

### 후보 B — `layout.rs::layout_shape_item` Picture 분기

- 장점: paragraph 객체 접근 가능 → 빈 paragraph 정확히 판정 가능
- 장점: 영향 범위가 본문 + 다단 본문에 한정
- **채택**

## 영향 범위

| 항목 | 영향 |
|------|------|
| HWP3 / HWPX | 동일 IR 사용 → 자동 적용 |
| 머리말/꼬리말, 바탕쪽 | 별도 layout 경로 → 영향 없음 |
| 표 셀 내부 그림 | `cell_ctx.is_some()` 분기 → 영향 없음 |
| TAC, Square/BehindText/InFrontOfText wrap | 가드로 제외 |
| caption 보유 그림 | 가드로 제외 |

## 예상 결과

- 그림1 위치: 273 px (변화 없음)
- 그림2 위치: 633 → 666 (PDF 매칭, +33 px)
- 그림3 위치: 994 → 1059 (PDF 매칭, +65 px)
- 그림 cluster 거리: 17280 → 18896 HU (PDF 18864 매칭, +32 HU sub-pixel)

## 다음 단계

Stage 2 — `layout_shape_item` Picture 분기에 가드 + line_advance 추가.
