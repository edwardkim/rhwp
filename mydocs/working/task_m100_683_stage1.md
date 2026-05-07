# Stage 1 — 진단 및 산식 확정 (Task #683)

**브랜치**: `local/task683`
**관련**: `mydocs/plans/task_m100_683.md`, `mydocs/plans/task_m100_683_impl.md`

## 요약

`samples/pr-149.hwp` 의 그림 cluster 간격 차이는 **빈 paragraph + `wrap=TopAndBottom` 그림** 의 layout 시 그림 다음에 빈 문단의 line baseline 이 추가되지 않아서 발생.

수정 위치는 **`src/renderer/layout/picture_footnote.rs::layout_body_picture` 의 반환값** 또는 **`src/renderer/layout.rs::layout_shape_item` 의 result_y 처리부** 둘 중 하나로 좁혀짐. Stage 2 에서 후자(caller 수정)를 채택 — 빈 paragraph 판정 정보가 caller 에 있고 영향 범위가 명확.

## 진단 데이터

### 1. PDF (한글 2022) 정밀 측정 — 150dpi

| 요소 | y 범위 | 높이 |
|------|--------|------|
| Title | 207..226 | 20 px (≈ 960 HU) |
| "원본:" | 241..259 | 19 px |
| **그림1** | 273..600 | 327 px (= 15696 HU) |
| "회색조:" | **634..649** | 16 px |
| **그림2** | 666..993 | 327 px |
| "흑백:" | 1028..1042 | 15 px |
| **그림3** | 1059..1387 | 328 px |
| "입니다." | 1454..1472 | 19 px |

그림 간 거리: 393 px = **18864 HU** (그림1→2, 그림2→3 동일).

### 2. rhwp SVG 측정 — 150dpi

| 요소 | y 범위 |
|------|--------|
| Title | 206..226 |
| "원본:" | 240..259 |
| 그림1 | 273..600 (동일) |
| "회색조:" | **600..617** ← **PDF보다 34px 위** |
| 그림2 | 633..961 ← PDF보다 33px 위 |
| "흑백:" | 961..978 |
| 그림3 | 994..1321 |
| "입니다." | 1354..1373 |

그림 간 거리: 360 px = **17280 HU**.

### 3. 차이 분석

- 그림1 위치: PDF=SVG (273 px). **첫 그림은 정합.**
- 그림1 bottom (600) → "회색조:" top:
  - PDF: 34 px (= 1632 HU ≈ **1 line**)
  - SVG: 0 px
- 누적 차이: cluster 당 **+1632 HU ≈ 1600 HU (1 line)** 가 PDF 에 추가됨

### 4. file 의 LINE_SEG vpos (참조)

```
p2 (그림1 paragraph)  vpos=3200
p3 ("회색조:")         vpos=18896  (= 3200 + 15696, 그림1 height)
p4 (그림2 paragraph)  vpos=20496  (= 18896 + 1600, "회색조:" line)
p5 ("흑백:")           vpos=36192  (= 20496 + 15696)
...
```

**file 자체는 그림 paragraph 가 image_height 만 차지** 하도록 vpos 인코딩되어 있으나, 한글 2022 layout 은 이를 **무시하고 +1 line 을 그림 다음에 추가** 한다.

## 코드 추적

### 현재 rhwp 동작 (layout)

1. `pagination/engine.rs::process_controls::Picture` (L1069~)
   - 빈 paragraph p2 의 `para_height` (line_height + line_spacing = 1600 HU) 를 `paginate_text_lines` 에서 가산 (L671 `st.current_height += para_height`)
   - Picture branch 에서 `pic_h + margin` 추가 (L1083 `st.current_height += pic_h + margin_top + margin_bottom`)
   - **pagination 누적**: cluster 당 1600 + 15696 = 17296 HU
2. `layout.rs::layout_shape_item` (L3037~) — Picture (TAC=false) 분기
   - `pic_y = para_start_y[para_index]` = paragraph 시작 y
   - `result_y = self.layout_body_picture(...)`
3. `layout/picture_footnote.rs::layout_body_picture` 반환값 (L387)
   - `(VertRelTo::Para, _) => base_y + total_height` (= pic_height + caption stuff)
   - **layout 실 진행**: 그림 paragraph 당 image_height 만 (line_height 가산 안 됨)

→ pagination 은 1600 추가 누적하지만 layout 은 image_height 만 진행. **layout 이 한 줄 누락**.

### Hancom 한글 2022 추정 동작

빈 paragraph + `wrap=TopAndBottom` 그림 시:
- 그림은 paragraph 시작 y 에 배치 (image1 top 위치는 동일)
- 그림 다음에 paragraph 의 line baseline 1줄(line_height + line_spacing) 만큼 추가 진행
- 즉 layout 진행량 = `image_height + line_height + line_spacing`

## 채택 산식

**빈 paragraph (text_len = 0) + TopAndBottom 그림 (treat_as_char = false) 의 layout 진행량:**

```
result_y = base_y + image_height + caption_overhead
         + line_height + line_spacing   (← 신규 추가, 빈 paragraph 한정)
```

**조건 가드:**
- `pic.common.treat_as_char == false`
- `pic.common.text_wrap == TextWrap::TopAndBottom`
- `pic.common.vert_rel_to == VertRelTo::Para`
- 부모 paragraph 의 `text` 가 visible 글자 0 (`para.text.chars().filter(|c| *c > '\u{001F}' && *c != '\u{FFFC}').count() == 0`)
- (선택) caption 없음 — caption 있는 경우는 별도 케이스, 안전 위해 1차 fix 에서는 caption 있으면 변경 없음

## 수정 위치 후보 비교

### 후보 A — `picture_footnote.rs::layout_body_picture` 반환값

- 장점: 모든 caller 에 자동 반영
- 단점: 머리말/꼬리말, 바탕쪽, 표 셀 내부 caller 들이 paragraph 컨텍스트 없이 호출 → 빈 paragraph 판정 불가

### 후보 B — `layout.rs::layout_shape_item` Picture 분기

- 장점: paragraph 객체 접근 가능 → 빈 paragraph 정확히 판정 가능
- 장점: 영향 범위가 본문 + 다단 본문에 한정 (머리말/꼬리말은 별도 경로)
- 단점: `layout_body_picture` 반환 후 result_y 보정 추가 필요 (1 곳)

→ **후보 B 채택**.

## 영향 범위 평가

### HWP3 / HWPX

같은 IR 로 변환되어 layout 코드 공유. pr-149.hwp 는 HWP5 포맷이지만 HWPX 동일 패턴(빈 문단 + TopAndBottom 그림)이 있으면 동일하게 fix 적용됨. Stage 3 회귀 검증에서 `samples/hwpspec.hwp` 등 동일 패턴 보유 샘플로 시각 확인 예정.

### 다른 wrap 모드

- `Square` wrap: 그림 옆에 텍스트 흐름 → fix 무관
- `BehindText` / `InFrontOfText`: floating, layout 진행 미발생 → fix 무관
- `TopAndBottom`: 본 fix 대상

### 텍스트가 있는 paragraph + TopAndBottom 그림

가드 (`text_len == 0`) 로 인해 영향 없음. 텍스트가 있는 paragraph 의 line baseline 은 paragraph_layout 이 따로 처리.

## 예상 결과 (Stage 2 적용 후)

- 그림1 위치: 동일 (273 px) — paragraph 시작 y 자체는 변경 없음
- 그림2 위치: 633 → 666 (PDF 매칭, +33 px)
- 그림3 위치: 994 → 1059 (PDF 매칭, +65 px)
- 그림 간 거리: 17280 → 18864 (PDF 매칭)
- "회색조:" / "흑백:" 라벨도 1줄 아래로 이동 (PDF 매칭)

## 잔여 리스크

1. 다른 샘플 회귀 — `samples/` 내 빈 문단 + TopAndBottom 그림 보유 파일 (`hwpspec.hwp` 외) 에 시각 변화 가능성
   - 대응: Stage 3 광범위 회귀 검증
2. caption 보유 그림 — 1차 fix 에서는 caption 없는 경우만 처리, caption 있으면 기존 동작 유지
   - Stage 2 가드에서 caption 유무 확인
3. pagination over-estimation 미정합 — pagination 은 17296+1600 가산해 왔는데 fix 후 layout 도 동일하게 진행. **이번 fix 로 정합 일치** ✓

## 다음 단계

Stage 2 — 산식 구현 (`layout.rs::layout_shape_item` Picture 분기에 가드 추가, line_height 가산)

**작업지시자 승인 대기**.
