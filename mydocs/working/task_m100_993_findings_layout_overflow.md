# 원인 분석 — task993 LAYOUT_OVERFLOW 회귀 (page 22 등)

- 타스크: task993 후속 — 사용자가 `rhwp-studio`에서 페이지 22 본문 영역 초과를 발견
- 브랜치: `local/devel` HEAD `45584a25` (task993 머지)
- 작성일: 2026-05-20
- 결론: task993 컷 모델 도입으로 `LAYOUT_OVERFLOW` 50건 회귀. 정합에는 `HeightMeasurer`(MeasuredCell) ↔ `cell_units` 측정 시스템 통일이 필요 — task993 범위 밖, 후속 타스크로 분리 권장.

## 1. 측정 (`samples/2. 인공지능(AI) 기반 재정통합시스템 구축 용역 제안요청서.hwpx`)

`export-svg` 실행 중 렌더러 자체 `LAYOUT_OVERFLOW` 검증기 보고:

| 상태 | LAYOUT_OVERFLOW 건수 |
|------|---------------------|
| task992 종료 (Stage 2 = `8be5e0c2`) | **0** |
| task993 머지 (`45584a25`, 현재 HEAD) | **50** |

회귀 50건 분류:

| 항목 타입 | 건수 |
|----------|-----|
| `PartialTable` | 24 |
| `FullParagraph` | 11 |
| `Table` (비-분할) | 4 |
| `PartialParagraph` | 3 |
| `DRAW`(저수준 중복) | 8 |

크기 분포: `<5px` 12 / `5~25` 22 / `25~100` 12 / `100~500` 1 / **`>500` 3**.

`viewBox` 기준 스캔(구현계획서 §4)으로는 2건 0.7~2.1px만 잡혔으나, 본문 기준 50건이 있다.

## 2. 페이지 22 깊이 조사

페이지 22 항목(베이스라인·현재 동일, 6 항목):

```
PartialTable pi=221 rows=1..6 cont=true 6x3 (header repeat)
FullParagraph pi=222 (빈) h=17.3
Table          pi=223 6x3 632.7×288.1px TAC
FullParagraph pi=224 (빈) h=17.3
FullParagraph pi=225 (빈) h=17.3
PartialTable pi=226 rows=0..1 cont=false 6x3 (TAC, attr=0)
```

`used=902.2px`(베이스라인·현재 동일), 본문 941.1px.

### 렌더러 y_offset 진행

| 항목 | 누적 y | 항목 높이(렌더러) |
|------|--------|-------------------|
| 시작 | 105.8 | — |
| pi=221 | 614.3 | 508.5 |
| pi=222 | 645.5 | 31.2 |
| pi=223 | 965.2 | 319.7 |
| pi=224 | 996.4 | 31.2 |
| pi=225 | 1041.4 | 45.0 |
| pi=226 | 1065.2 | 23.8 |

본문 바닥 1046.9 — pi=226 마치고 **18.3px 초과**.

### 원인 분해

- 페이지네이터(컷 모델): pi=226 row 0 = `cut_row_h[0]` = `advance_row_cut(0,&[],MAX).consumed_height + max_padding = 21.9px` (셀 `h=1643`HU=21.94px).
- 렌더러 비분할 행: `resolve_row_heights` = `MeasuredTable.row_heights[0]` ≈ **40px**.
- 차이 ≈ 18.3px → 페이지 22 초과량과 일치.

pi=222·223·224·225(`FullParagraph`/`Table` 비분할 — task993 직접 변경 없음)도 렌더러 vs paginator 차이가 항목당 14~31px 누적. 베이스라인에서는 paginator가 OLD px 모델(`mt.range_height`)로 누적해 렌더러와 일치했으나, task993 cut walk(`advance_row_cut(MAX).consumed_height + padding`)는 다른 값을 산출 → 누적 drift → 본문 초과.

## 3. 시도한 정합과 트레이드오프

| 시도 | 결과 |
|------|------|
| 2b 오버라이드 전 렌더 행 확장 (페이지 13 회귀 수정 때 도입) | 50건 — 이미 해결책 시도 |
| 2b 오버라이드 → 분할 행만 (현 시도 1) | 51건 (동일 수준) |
| `cut_row_h[r] = mt.row_heights[r]` (현 시도 2) | 50→36건. 페이지 수 184→190 (rows fits 후에도 vpos 리셋 분할). |
| 위 + walk에서 `hit_hard_break` 검사를 fits 분기 후로 (시도 3) | 50→36건. 페이지 수 184→187. `test_page12_enter_table_placement` 회귀(kps-ai.hwp pi=198 배치 시프트). |

어느 부분 정합도 50→0으로 가지 못함. PartialTable·Table·FullParagraph·PartialParagraph 4종 타입 모두에서 `HeightMeasurer` ↔ `cell_units` 측정 미세 차이가 누적되기 때문.

## 4. 진정한 원인

`MeasuredCell.line_heights`/`total_content_height`(HeightMeasurer)와 `cell_units` 줄 높이(`corrected_line_height + line_spacing + cell.height filler`)가 같은 셀 콘텐츠에 대해 다른 px 합을 산출한다. 베이스라인은 이 차이가 양쪽 측정 일관성으로 가려졌으나(둘 다 mt 사용), task993의 cut walk가 노출시켰다.

특히 비분할 행에서 cell.height 필러 + corrected_line_height가 mt 측정과 어긋난다. mt는 베이스라인 px 모델 시절의 가정(LINE_SEG vpos·padding·trailing line_spacing 등)을 내장한다.

## 5. 권장 → 후속 타스크 등록 완료

**M100 #1022** 등록 — "HeightMeasurer ↔ cell_units 측정 정합".
<https://github.com/edwardkim/rhwp/issues/1022>

범위:

- `MeasuredCell` 산출 로직 분석(`HeightMeasurer::measure_section` 등).
- `cell_units` 줄 단위가 `MeasuredCell.line_heights`와 bit 정합하도록 `compose_paragraph` 기반 측정 통일 또는 `cell_units`를 `MeasuredCell` 기반으로 재구성.
- 통일 후 cut 모델 = mt 측정 → paginator·renderer 단일 측정 공간 → LAYOUT_OVERFLOW 0.

현 페이지 22의 18.3px 본문 초과는 viewBox(1122) 안 → 페이지 경계 밖은 아님(실측 max y=1065). 시각상 본문 바닥 ~18px 침범 — 서브픽셀이 아닌 실 회귀이나 다른 ~50건과 동일 부류.

## 6. 단기 옵션

1. **현 상태 유지 + 후속 타스크 등록**(권장). task993은 종결 상태, 회귀 50건은 별도 작업.
2. **task993 일부 되돌리기**: 컷 walk를 베이스라인 px 모델로 복원하면 50건 0으로. 단 task993의 핵심(form-002 vpos-리셋 분할 정합·무한 루프 제거)도 함께 사라진다.
3. **시도 3 부분 정합 적용**: 50→36으로 줄이고 `test_page12` 갱신. 가장 큰 3건(>500px)도 잔존.
