# Task #525 Stage 1 — 진단·재현·영향 범위

## 요약

**원인 확정.** 비-TAC Picture wrap=Square host paragraph 가 `layout_wrap_around_paras` 를 **두 곳에서 중복 호출** 받는다 — `layout.rs:3106` (`layout_shape_item`) 과 `layout.rs:3534` (`layout_column_shapes_pass`). 결과 동일 paragraph 가 `layout_composed_paragraph` 를 3번 호출 받음 (정상 PageItem::FullParagraph + 중복 wrap-around 2회).

**영향 범위 광범위.** 7 샘플 170 페이지 중 **37 페이지 / 205 dup-instances** 가 동일 결함. 사용자는 시각 가장 두드러진 exam_science p2 8번 문제 (pi=37) 만 보고했으나 실제 광범위 회귀.

## 1. 본질 — 3 호출 재현

`para_index == 37` 에 임시 backtrace 로깅 추가 후 `RHWP_LAYOUT_DEBUG=1 export-svg samples/exam_science.hwp -p 1` 결과:

```
TASK525_LAYOUT_COMPOSED: pi=37 col_x=70.7 col_w=422.6  mc_w_hu=None
   CALL_STACK:
       layout_composed_paragraph
       layout_partial_paragraph
       layout_column_item             ← 정상 PageItem::FullParagraph 경로
       build_single_column

TASK525_LAYOUT_COMPOSED: pi=37 col_x=70.7 col_w=233.9  mc_w_hu=None
   CALL_STACK:
       layout_composed_paragraph
       layout_wrap_around_paras
       layout_column_item             ← layout_shape_item:3106 경로 (Picture Square wrap 그림 처리 시)
       build_single_column

TASK525_LAYOUT_COMPOSED: pi=37 col_x=70.7 col_w=233.9  mc_w_hu=None
   CALL_STACK:
       layout_composed_paragraph
       layout_wrap_around_paras
       build_single_column            ← layout_column_shapes_pass:3534 경로 (별도 컬럼-레벨 패스)
```

호출 출처:

| # | col_w | 출처 | 의도 |
|---|-------|------|------|
| 1 | 422.6 | `layout_column_item` → `layout_partial_paragraph` (PageItem::FullParagraph) | 정상 paragraph 처리 |
| 2 | 233.9 | `layout_column_item` → `layout_shape_item:3106` (Picture Square wrap 그림 PageItem::Shape 처리 시) | wrap-around 텍스트 렌더 |
| 3 | 233.9 | `build_single_column` → `layout_column_shapes_pass:3534` (별도 컬럼-레벨 패스) | wrap-around 텍스트 렌더 (typeset 경로 fallback) |

**호출 2 와 호출 3 가 중복**. 둘 다 같은 paragraph (pi=37) 의 텍스트를 같은 col_x=70.7, col_w=233.9 영역에 렌더링.

## 2. 줄 emit 분포

`layout_composed_paragraph` 안 `has_picture_shape_square_wrap` 분기 (paragraph_layout.rs:822, 973-982):
- `comp_line.segment_width < col_area_w_hu - 200` 이면 effective_col = (col_area.x + cs_px, sw_px) — LINE_SEG.cs/sw 기반 좁은 영역
- 아니면 effective_col = (col_area.x, col_area.width) — 호출자 col_area 그대로

pi=37 ls 분포:
- ls[0..5] sw=17546 (그림 옆 좁은 영역, ~234 px)
- ls[6..7] sw=31692 (그림 아래 전체 column, ~423 px)

3 호출의 ls 별 effective_col_w 와 emit 결과:

| ls | 호출 1 (col_w=422.6) | 호출 2 (col_w=233.9) | 호출 3 (col_w=233.9) | distinct x 위치 |
|----|----------------------|----------------------|----------------------|----------------|
| 0~5 | sw<col_w-200 → eff=233.9 | sw<col_w-200? 17546<17346 false → eff=233.9 | 동일 | 모두 같은 좌표 → 1 위치 (3번 emit, 시각 동일) |
| 6~7 | sw>col_w-200 → eff=422.6 (전체) | sw>col_w-200 → eff=233.9 (좁게) | 동일 | **2 위치 (호출1 + 호출2,3) → 시각 중첩** |

ls[0..5] 는 같은 좌표에 3번 emit 되어도 시각적으로 동일. ls[6..7] 만 두 다른 col_w 에서 정렬 차이로 distinct x 발생 → SVG 데이터에서 본 "옥 x=139.28 + x=143.74 (4.46px 오프셋)" 패턴 정확히 일치.

## 3. SVG 좌표 검증 (pi=37 ls[6], y=775.01)

`/tmp/svg_diff_after/exam_science/exam_science_002.svg` 추출:

| 글자 | 첫 시퀀스 x (호출 2,3 합) | 둘째 시퀀스 x (호출 1) | 차 |
|------|---------------------------|------------------------|-----|
| 옥 | 139.28 (n=2) | 143.74 (n=1) | +4.46 |
| 텟 | 153.08 (n=2) | 157.54 (n=1) | +4.46 |
| 규 | 170.52 (n=2) | 179.45 (n=1) | +8.93 |
| 칙 | 184.32 (n=2) | 193.25 (n=1) | +8.93 |
| 을 | 198.12 (n=2) | 207.05 (n=1) | +8.93 |
| 만~이 | 215.56~438.43 (n=2) | 228.95~480.35 (n=1) | +13~+42 (누적) |

n=2 = 호출 2와 3 의 동일 좌표 emit (동일 col_w=233.9). n=1 = 호출 1 의 다른 col_w=422.6 정렬 결과.

(이슈 본문 보고 "1.3~9px 중첩" 은 본 4.46~13px 오프셋의 누적 결과. 정확하다.)

## 4. 영향 범위 — 광범위 layout 회귀

**검출 방법**: 각 SVG 의 모든 행 (y rounded to 0.01px) 에 대해 동일 글자가 **0.1~15px 오프셋의 다중 x 위치** 에 emit 된 인스턴스 카운트 (#525 본 결함 패턴).

7 샘플 170 페이지 결과:

| 샘플 | dup-instances | 영향 페이지 | 비고 |
|------|---------------|-------------|------|
| exam_kor | **130** | **16** / 20 (80%) | 가장 광범위 |
| exam_eng | 25 | 6 / 8 (75%) |  |
| exam_science | 35 | **4 / 4 (100%)** | 본 task 보고 샘플 |
| exam_math | 6 | 5 / 19 (26%) |  |
| 2010-01-06 | 5 | 4 / 6 (67%) |  |
| synam-001 | 2 | 1 / 35 (3%) |  |
| aift | 2 | 1 / 77 (1%) |  |
| **합계** | **205** | **37 / 170 (22%)** |  |

→ **본 결함은 비-TAC Picture wrap=Square host paragraph 가 있는 모든 페이지에 잠복**. 사용자가 보고한 exam_science pi=37 외에 다른 4개 샘플 35 페이지에 동일 패턴.

(주의: exam_science 의 일부 dup 는 #526 정정 후에도 잔존. 본 task 결함은 #526 와 독립.)

## 5. 정정안 비교

세 호출 중 어떤 것을 제거하느냐가 핵심:

### A안 — `layout_column_shapes_pass:3534` 호출 제거

**근거**: layout.rs:3499-3502 주석 "비-TAC Square wrap 그림/도형: 어울림 문단 렌더링. typeset.rs 경로에서 PaginationResult.wrap_around_paras는 항상 비어있으므로 col_content.wrap_around_paras를 직접 사용해야 함." → typeset 경로 fallback 으로 도입된 것. 현재는 `layout_shape_item:3106` 가 동일 처리를 수행.

**위험**: typeset 경로에서만 활성화되는 케이스가 있다면 회귀.

### B안 — `layout_shape_item:3106` 호출 제거

**근거**: `layout_column_shapes_pass:3534` 가 컬럼-레벨에서 wrap-around 처리를 일관 수행. shape_item 안에서 별도 wrap-around 호출은 redundant.

**위험**: shape_item 경로가 wrap_around_paras 가 비지 않은 경우 (Table Square wrap 경로와 다른 흐름) 만 활성화되면 회귀.

### C안 — 호출 1 (PageItem::FullParagraph) 도 같이 처리

호출 1 자체가 `has_picture_shape_square_wrap=true` 분기로 ls[0..5] (좁은 영역) + ls[6..7] (넓은 영역) 모두 적절한 effective_col 로 렌더. **호출 2, 3 모두 불필요**한 가능성.

**위험**: 호출 1 자체가 wrap-around 의 후속 paragraph (wrap_around_paras 안에 있는) 를 처리하지 않음 — wrap-around 후속 paragraph 가 사라질 수 있음.

### 권고

**A안 채택 시도**. 근거:
- 주석상 fallback 으로 명시 (3499-3502)
- `layout_shape_item:3106` (호출 2) 가 PageItem::Shape 처리 흐름 안에 있어 자연스러운 위치
- 회귀 검증으로 확인 가능 (svg_regression_diff)

A안 실패 (다른 샘플 회귀) 시 → B안 → C안 순.

## 6. 결론 — 완료 기준 충족

수행 계획서 Stage 1 완료 기준: "pi=37 ls[6]~7 중복의 정확한 코드 경로 + 영향 범위 식별" — 충족.

- 코드 경로: `layout.rs:3106 layout_shape_item` + `layout.rs:3534 layout_column_shapes_pass` 가 동일 paragraph 에 `layout_wrap_around_paras` 중복 호출
- 영향 범위: 7 샘플 170 페이지 중 37 페이지 / 205 dup-instances (단순 보고된 pi=37 단독 아님)

## 7. Stage 2 진행 시 주의

- 정정안 A 채택 시 광범위 회귀 위험 (37 페이지 SVG 변경 예상)
- svg_regression_diff 에서 변경 페이지는 **모두 의도된 정정** 으로 분류해야 함 (회귀 0)
- 작업지시자 시각 정합 검증 필수 (특히 exam_kor 16 페이지)
- 본 정정의 본질이 사용자 보고 (#525 단일 케이스) 보다 큰 회귀임을 보고서에 명시

---

승인 요청: 본 진단 결과 (광범위 영향 + A안 채택) + Stage 2 정정 진행 가능 여부.
