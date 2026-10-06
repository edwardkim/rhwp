# #1937 Stage 1 — 근본 원인 확정 (오라클·실측)

- 브랜치: `local/task1937` / 샘플: `소상공인 중간보고서(2).hwp` (canonical, rhwp 231쪽 vs 한글 50쪽)
- 방법: `dump-pages` 페이지 fill 실측 + `dump -s -p` 표 셀 구조 실측 (코드 무수정)

## 1. 결론 — "행높이 과대"는 맞으나 위치는 **측정 경로(mt.row_heights)**, 원인은 **다문단 셀 콘텐츠 과다측정**

원 이슈의 두 가설(제목=행높이 3배 / 본문=vpos 리셋) 중 **행높이 과다가 맞고, vpos 리셋은 아님**.
단, 최종 렌더 단독 문제가 아니라 **측정 경로가 split·render 를 모두 오도**한다.

## 2. 실측 근거

### 2-1. 페이지 fill — 대부분 페이지가 거의 비어 있음
`dump-pages` 전수 집계 (231쪽):
- **중앙값 fill 0.0%, 평균 9.4%.** under-25% 페이지 203/231(88%).
- p8~p13: 각 `items=1 used=0.0px` 인데 **한 페이지씩 소비** → 전형적 과분할.

### 2-2. 폭주의 정체 — 105행 표 1개가 7쪽에 흩뿌려짐
- 표 pi=99 = **105행×10열**, `쪽나눔=RowBreak`. p7~p13 PartialTable 로 분할:
  p7 rows6..20, p8 19..34, … p13 101..105 → **페이지당 ~13~23행**.
- body 895.8px 에 행 15개면 **행당 ~58px** 소비. 그러나 실제 행 콘텐츠 줄높이는
  `lh=900~1200`(12~16px). → **행당 ~3.8배 과다.**

### 2-3. 저장(한글 권위) 높이와 비교 — rhwp 가 ~3.6배 부풀림
- 표 pi=99 저장 행높이 합 = **119760 HU = 1597px = 1.8쪽분** (한글 50쪽 총량과 정합).
- rhwp 는 같은 표를 **~7쪽(~5800px)** 로 측정 → **저장 대비 ~3.6배.**

### 2-4. 지배 요인 — 다문단 셀
- 행별 셀 최대 문단수: median 2, **max 7**. 예) 셀[30] r4c2 `paras=7`
  (text `제22조(사용료), 제24조…|(분할납부…` — `|`=문단 경계).
- 저장 LINE_SEG 는 `lh=900 ls=540` 로 촘촘. 그러나 rhwp 셀 콘텐츠 높이는
  `calc_cell_paragraphs_content_height` → `recompose_for_cell_width`(table_layout.rs:1702)
  로 **재조판**하여 저장 line_seg 보다 줄/높이를 부풀린다(다문단일수록 누적).

## 3. 코드 경로

- 측정: `table_cell_content.rs:469` → `resolve_row_heights(…, measured_table=None)` →
  step 1-b `calc_cell_paragraphs_content_height`(1546-1568) 가 저장 `cell.height` 를
  **콘텐츠 높이로 override**(1566). 콘텐츠 높이가 과다 → 행 부풀림.
- 이 `mt.row_heights` 가 그대로 ① split(`split_table_rows` engine.rs:2339, `mt.row_heights[cursor_row]`)
  ② 최종 render(`resolve_row_heights` measured_table=Some, 1520 clone) 를 **동시 오도**.

## 4. #1949 와의 관계

- 동일한 `mt.row_heights`/`cut_row_h` 재측정이 #1949(성능 420s)의 원인이기도 하다.
  행높이 산정을 저장값 정합으로 **정확+저렴**하게 만들면 #1937(정확성)·#1949(성능) 동시 개선 가능.

## 5. Stage 2 제안 (승인 요청)

1. **정확 지점 특정**: `recompose_for_cell_width` 재조판이 저장 line_seg 대비 줄수/줄높이를
   부풀리는지, 아니면 문단별 `spacing_before/after`·빈문단 fallback(400HU)이 누적되는지
   `export-render-tree` 셀 y좌표 실측으로 1개 항으로 좁힌다.
2. **최소 수정 설계**: 셀 콘텐츠 높이가 저장 line_seg(한글 권위 줄배치)를 신뢰하도록
   보정 — 다문단 셀에서 저장 높이를 부당하게 초과하지 않게. 정당하게 큰 표(랩 텍스트 실셀)
   회귀 없도록 양방향 게이트.
3. **검증**: canonical(231→~50 목표)·+88/+84 서베이 표본 + #1658 byeolpyo 양 게이트 +
   과소/과대 양방향 회귀.

**리스크**: 콘텐츠 높이 로직은 광역. 정당하게 큰 셀(실제 랩 다행 텍스트) 축소 위험 →
저장 line_seg 존재 시에만 신뢰하고 부재 시 기존 fallback 유지하는 국소 보정으로 한정.

→ Stage 2(정확 지점 특정 + 수정 설계) 진행 승인 요청.
