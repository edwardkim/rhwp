# 최종 결과 보고서 — Task #1860

**이슈**: #1860 분할(RowBreak) 행 valign=Center 라벨 세로 위치 −23~+42pt 어긋남
**브랜치**: `local/task1860` (통합 베이스 `local/prstack-0703` + #1841 워킹트리)
**마일스톤**: M100
**작성일**: 2026-07-03

---

## 1. 요약

78842 유사입법례 표(3×2 RowBreak, PR #1855 fixture)에서 `valign=Center` 라벨
공공데이터법(−23pt)·전자정부법(+42pt)이 한글 2024 대비 반대방향으로 어긋나던 문제를,
**분할 예산의 para_start 소실(current_height 이중차감)** 로 규명하고 교정했다.

**valign 로직은 결함이 아니었다** — 라벨은 각 fragment 중앙에 정상 배치되며, fragment
높이가 잘못(짧게) 계산된 것이 근본. 원인은 #1855 지연 co-anchored 경로가 out-of-flow
float 의 참 para_start 를 current_height(선행 tac 캡션 포함)로 덮어써, 분할 예산이
캡션 높이만큼 이중차감되어 RowBreak 컷이 소스 hard_break 보다 3줄 조기 발동한 것.

| 지표 | 수정 전 | 수정 후 | 한글 |
|------|--------|--------|------|
| pi=371 page_avail | 795.2px | **860.8px** | (참값 860.8) |
| RowBreak end_cut | 34 | **36** | ~37 |
| p44 라벨 공공데이터법 Δ | −24.6pt | **−7.8pt** | 0 |
| p45 균일 오프셋 | +40.8pt | **+7.2pt** | 0 |

잔여 −7.8/+7.2pt = **1줄 razor**(end_cut 36 vs 소스 hard_break 37, 내용셀 avail
828.6 vs 필요 828.8, 0.2px). = 내용셀 per-line 미세 과대 누적(#1759/#1760/#1763 계열,
별개 축) — 본 예산 교정 범위 밖으로 잔존.

## 2. 근본원인

`typeset_block_table`(typeset.rs)의 첫 fragment 분할 예산:
`page_avail = table_available − current_height − caption − host − vert_offset_overhead`.

빈 host out-of-flow para float(TopAndBottom+vert=Para+v_off>0)은 `para_start + v_off`
에 앵커되므로(#986/#1088/#157) 예산 기준은 **para_start** 여야 한다. current_height
기준이면 같은 문단 선행 in-flow inline(tac 캡션)이 이미 흐름을 para_start 위로 밀어놓은
만큼 이중차감된다.

pi=371 ci=1 은 **#1855 지연 co-anchored 경로**(is_deferred_coanchored_rowbreak_table)
로 배치되는데, 지연 핸들러(typeset.rs ~10637)가 `para_start_height = st.current_height`
(=65.5, 선행 캡션 참고|유사입법례 포함)로 **참 para_start(0)를 소실**. → page_avail
795.2(참값 860.8). 내용셀 avail 763 → 소스 hard_break(unit37, 한컴 저장 분할점) 3줄
앞(unit34)에서 용량컷 → fragment 3줄 짧음:
- p44: 짧은 fragment 중앙 → 라벨 −23pt
- p45: 밀려난 3줄 → 이하 균일 +40.8pt

상세 계측: `mydocs/tech/task_m100_1860_split_budget.md`.

## 3. 수정 내용

`src/renderer/typeset.rs` — **예산 전용 참 para_start** 분리:
1. `DeferredTableControl` 에 `para_start_height: f64` 필드 추가(원 배치 시점 참값).
2. `typeset_block_table` 에 예산 전용 `budget_para_start_height` 파라미터 추가.
   **렌더 위치용 `para_start_height` 는 종전대로 유지**(지연 경로 = 라이브
   current_height).
3. 두 호출부: 비지연 → para_start_height, 지연 → deferred.para_start_height.
4. page_avail: 빈 host out-of-flow float(`is_empty_host_column_float`)만
   `current_height` 대신 `budget_para_start_height.min(current_height)` 기준으로
   차감. 선행 inline 없으면 두 값 동일 → 종전 불변(#874 pi=584/242).

**함정 노트(중요) ①**: 지연 핸들러의 `para_start_height` 자체를 참값으로 바꾸면 그 값이
**렌더 위치**에도 쓰여, 지연 배치 대형 visible-host 표(admrul_1065 pi=0)가 무한
재배치 루프(render-diff 20분+ 미완)에 빠진다. 반드시 **예산 전용**으로만 분리해야 함.

**함정 노트(중요) ② — current_height 클램프 필수**: 지연 배치가 **페이지/컬럼 경계를
넘은 뒤** 실행되면 저장된 para_start(원 페이지 흐름 좌표)가 새 페이지에서 무효가 된다.
CI 회귀(issue_1686, samples/pr-1674 35→36쪽)로 발견: pi=27 빈 host 에 형제 float 표
2개(ci=0 v_off=0, ci=1 v_off>0) — ci=1 지연 배치가 새 페이지(cur_h=0)에서 stale
para_start 583.8 을 차감해 예산 −584px → 조기 분할 → +1쪽. 참 para_start 는 현재
흐름 높이를 초과할 수 없으므로 `min(current_height)` 상한. 클램프 후 pr-1674 hwp/hwpx
35쪽(오라클 정합) 복원 + fixture −7.8/+7.2pt 유지 + issue_1686 4/4.

`src/renderer/layout/table_partial.rs` — **노드-자식 포섭 불변** (pi=28 회귀 해소):
5. 예산 교정이 co-anchored 표(pi=28, rowbreak-problem-pages)의 분할점을 이동시켜,
   분할 조각의 셀 내 as-char 텍스트박스 rect 가 유닛 기반 셀 높이보다 ~15px 아래로
   그려져 표 노드 bbox 가 자식을 clip(page17). 렌더 완료 후 표 노드 bbox 높이를 모든
   자손의 최하단까지 확장(확장만, 축소 없음)해 자식 포섭 불변을 보장.
   - 시도했다 폐기: split-높이 계산(row_cut_content_height) 확장 — 표 노드 높이는
     유닛 합을 이미 포함(clip 원인은 shape **렌더 배치**), 높이 계산 수정은 무효.

**table_partial.rs valign 로직은 불변** (증상 아님).

## 4. 검증

### 단위/통합 테스트
- 예산 교정판 `cargo test --release` 전 195 바이너리 **4878 통과 / 0 실패**.
- 예산+포섭 최종판 `cargo test`(debug) 전 스위트 **통과**(doc-test 까지 exit 0),
  `issue_rowbreak_chart_overlap` **20/20**(pi=28 회귀 해소), visual_roundtrip_baseline
  포함 모두 ok.
- 핵심: svg_snapshot(골든 8), issue_1753_deferred_table_fill_ahead(지연 경로),
  issue_874(예산 불변 케이스), issue_1748/1156/1488(RowBreak 분할),
  issue_1073/1133(nested split/valign), issue_1772(om sync) 모두 통과.

### fixture (권위: 한글 2024 PDF)
- p44 라벨 공공데이터법 −24.6→−7.8pt, p45 +40.8→+7.2pt.
- pi=117(캡션이 別문단, para_start==current_height) budget 850.1 불변,
  이전 broad 시도의 51.2px 오버플로 없음(회귀 방지 확인).

### 코퍼스 render-diff (roundtrip 안정성, baseline A/B 클린 비교)
- **big_hwpx** (2500 doc): 수정 전/후 summary 동일(PASS 2480 / OVER 9 /
  STRUCT 11 / PAGE_MISMATCH 0 / max_disp 690.33). timing 제외 per-doc diff =
  **3 doc만 page 수 ±1 변동**(admrul_0326 7→6, seoul_0022 6→5, admrul_0650 134→135)
  — 전부 PASS·max_disp 0(roundtrip 안정). STRUCT/OVER/PAGE_MISMATCH 회귀 0.
- **big_hwp** (native, 2500 doc): 최종판 A/B — summary baseline 과 **동일**
  (PASS 2494 / OVER 4 / STRUCT 2 / PAGE_MISMATCH 0 / max_disp 465.00). timing 제외
  per-doc diff = **3 doc page ±1**(admrul_0360 9→10, admrul_0527 2→3,
  admrul_0549 13→14) 전부 PASS·max_disp 0. STRUCT/OVER/PAGE_MISMATCH 회귀 0.
  admrul_1065 무한루프 해소 확인. 대형표(admrul_1234, baseline ~19분) 저속은
  선존 성능 이슈로 무관.

> 주: render-diff 는 원본 IR render vs roundtrip render 의 내부 정합(회귀)만 본다.
> 본 수정은 두 render 를 대칭 변경하므로 roundtrip 지표는 불변이며(=회귀 없음),
> 3 doc page 수 변동은 예산 교정으로 분할점이 이동한 정상 효과다.

### 성능 함정(무한루프 회피 검증)
- native 코퍼스 스캔 중 admrul_1065(baseline 12쪽/32s)·admrul_1234(baseline
  77쪽/**1132s≈19분**)가 초기 fix(지연 렌더 para_start 변경본)에서 무한 재배치
  루프 발생. **예산 전용 분리 후 admrul_1065 정상**(12쪽 PASS, baseline 동일),
  admrul_1234 는 else 분기라 불변(baseline 동일 저속). 초대형 표 저속은 선존 성능
  이슈로 본 타스크 무관.

## 4-b. 회귀 발견 및 해소 — pi=28 텍스트박스 클리핑

`cargo test`(debug) 에서 `rowbreak_page17_split_table_covers_visible_textbox_shape`
(samples/rowbreak-problem-pages.hwpx, pi=28 ci=0) **1건 실패**.

- pi=28 = pi=371 과 **동일 구조 클래스**(빈 host, 지연 co-anchored, tac ci=1 +
  float ci=0). 내 예산 교정이 pi=28 예산도 165.2→295.1(+130px)로 키움.
- 결과: page17 float 조각이 짧아져 셀 내 고정높이 텍스트박스(하단 760.8pt)를
  조각 하단(749.4pt)이 ~11px 클리핑. baseline(작은 예산)은 우연히 클리핑 회피 →
  테스트 통과했었음.
- **권위 대조(pdf/rowbreak-problem-pages-2024.pdf) page17**: 내 fix 본문 텍스트는
  한글과 정합(max bottom 796.2 vs 799.0, median −3.24pt) — **분할점 자체는 한글
  충실**. 클리핑은 분할 셀이 내부 고정 shape 를 못 감싸는 별개 엣지.

**해소**: 예산 교정은 pi=28 에도 원칙상 옳다(float 은 para_start+v_off 앵커). 클리핑은
분할 조각의 셀 노드 bbox 가 자식 shape rect 를 못 감싸던 **선존 렌더 불변 결함**이
예산 교정으로 표면화된 것 → **노드-자식 포섭 불변**(위 §3.5)으로 해소. 재빌드 후
`issue_rowbreak_chart_overlap` **20/20 통과**(형제 회귀 0).

## 5. 잔여/후속

- **잔여 1줄 razor(−7.8/+7.2pt)**: 내용셀 per-line 측정 미세 과대 누적으로 end_cut 이
  소스 hard_break(37) 직전(36)에서 용량컷. #1759/#1760/#1763 계열 별개 축.
  후속: advance_row_cut 이 임박 hard_break(한컴 저장 분할점)을 소량 tolerance 내에서
  스냅하도록 하는 게이트 검토(공유 함수라 별도 신중 검증 필요) — 본 타스크 범위 외.

## 6. 파일

- `src/renderer/typeset.rs` (예산 교정)
- `src/renderer/layout/table_partial.rs` (노드-자식 포섭 불변)
- `mydocs/plans/task_m100_1860.md`, `_impl.md`
- `mydocs/tech/task_m100_1860_split_budget.md`
- `mydocs/report/task_m100_1860_report.md`
