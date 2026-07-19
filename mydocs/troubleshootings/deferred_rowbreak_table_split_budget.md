---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# Task #1860 — 분할 예산 계측·근본원인 (Stage 1)

**대상**: #1860 RowBreak valign=Center 라벨 세로 위치 −23/+42pt
**fixture**: `samples/issue1853_caption_precedes_body_split.hwpx` (PR #1855 merge 772dc2c7)
**권위**: `pdf/issue1853_caption_precedes_body_split-2024.pdf` (한글 2024)

## 계측 도구

- `RHWP_TABLE_DRIFT=1 rhwp export-pdf …` → `TABLE_SPLIT_AVAIL` (typeset 예산 항, pi 포함)
- `RHWP_CUT_DBG=1 …` → `CUT_DBG`(advance_row_cut 유닛/avail — pi 미포함, avail 로 대조)
- `tools/compare_line_baselines.py rhwp.pdf hangul.pdf --page N`
- `pdf_lines.py`(임시): PDF 줄별 (y,x,text)

## 실측 (pi=371 ci=1, 3×2 RowBreak, p44 첫 fragment)

`TABLE_SPLIT_AVAIL: pi=371 cursor_row=0 cont=false cur_h=65.5 table_avail=918.5`
`caption=0.0 host_before=0.0 vert_off=57.7 page_avail=795.2 avail_for_rows=795.2`

`CUT_DBG row=1 cell=1 avail=763.0 units=[42 × h=22.4 … unit37=hb …]`
→ 용량컷 j=34 (34×22.4=761.6 ≈ avail 763) → **end_cut=34**.
소스 hard_break(한컴 저장 분할점) = unit 37 (≈829px).

- rhwp p44 라벨 공공데이터법 y=422.0pt / 한글 445.0pt (Δ −23.0)
- 본문 내용 줄은 rhwp/한글 **동일 y**(예 "7.제17조" 428.0/428.2) → 라벨만 어긋남
- rhwp 마지막 내용줄 y≈697pt, 한글 ≈747pt, body 하단 765.4pt
  → rhwp 는 하단에 ~68pt(≈91px) 여백을 남기고 컷(용량 미달 아님, **예산이 짧음**)

## 근본원인

`page_avail = table_available − current_height − caption_extra − host_before − vert_offset_overhead`
(typeset.rs `typeset_block_table`, ~12334). 빈 host out-of-flow para float 은
`para_start + v_off` 에 앵커되므로(#986/#1088/#157), 분할 예산은 **para_start** 기준이
맞다. current_height 기준이면 같은 문단의 선행 in-flow inline(tac 캡션)이 이미 흐름을
para_start 위로 밀어놓은 만큼 이중차감된다.

- 본 표 pi=371: 같은 문단 ci=0 = inline tac 캡션(참고|유사입법례, 65.5px)이 흐름을
  0→65.5px 전진(current_height=65.5). float ci=1 은 para_start(0)+v_off(57.7)=57.7px
  에 배치(렌더 측정 header 63.7px body-rel = v_off+outer 로 확인).
- 그런데 pi=371 ci=1 은 **#1855 지연 co-anchored 경로**(is_deferred_coanchored_
  rowbreak_table)로 배치되고, 지연 핸들러(typeset.rs ~10637)가
  `para_start_height = st.current_height`(=65.5) 로 **참 para_start(0)를 덮어씀**.
  → page_avail = 918.5 − 65.5 − 57.7 = **795.2**(참값 918.5 − 0 − 57.7 = 860.8).
  ~65px 과소.
- 내용셀 avail 763 → 용량컷이 소스 hard_break(unit37, 한컴 저장 분할점) 3줄 앞
  (unit34)에서 발동 → end_cut=34.

이 단일 예산 과소가 두 증상을 모두 설명:
1. p44 fragment 3줄 짧음 → valign=Center 라벨(정상 로직)이 짧은 fragment 중앙 → −23pt
2. 밀려난 3줄이 p45 최상단 → 이하 균일 +40.8pt

**valign 로직(table_partial.rs)은 정상 — 원인은 지연 경로의 para_start 소실로 인한
분할 예산 과소(current_height 이중차감).**

## 교정 (구현 — 예산 전용 참 para_start)

**중요 함정**: 처음엔 지연 핸들러의 `para_start_height` 자체를 참값으로 바꿨으나,
이 값은 **렌더 위치**(place_table_with_text)에도 쓰여, 지연 배치되는 대형 visible-host
표(admrul_1065 pi=0 giant table)의 배치가 어긋나 **무한 재배치 루프**를 유발했다
(render-diff 20분+ 미완). 따라서 렌더 위치는 불변으로 두고 **예산 계산에만** 참
para_start 를 쓰도록 분리한다.

1. `DeferredTableControl` 에 `para_start_height` 필드 추가 — 원 배치 시점의 참
   para_start 보존(typeset.rs 154, push).
2. `typeset_block_table` 에 **예산 전용** `budget_para_start_height: f64` 파라미터 추가.
   렌더용 `para_start_height` 는 종전대로(지연 경로 = 라이브 current_height) 유지.
3. 두 호출부: 비지연 → `para_start_height`(그 자체가 참 para_start),
   지연 → `deferred.para_start_height`(원 배치 시점 참값).
4. page_avail: 빈 host out-of-flow float(`is_empty_host_column_float`) 만
   `current_height` 대신 `budget_para_start_height` 기준으로 차감. 선행 inline 없으면
   두 값이 같아 종전과 동일(#874 pi=584/242 불변). visible-host 표(admrul_1065/1234
   pi=0/33/35)는 `is_empty_host_column_float=false` → else 분기(불변) → 루프 없음.

## 결과 (계측·권위 대조)

| | 수정 전 | 수정 후 | 한글 |
|---|---|---|---|
| pi=371 page_avail | 795.2 | **860.8** | (참값 860.8) |
| end_cut | 34 | **36** | ~37 |
| p44 라벨 공공데이터법 Δ | −24.6pt | **−7.8pt** | 0 |
| p45 균일 오프셋 | +40.8pt | **+7.2pt** | 0 |
| pi=117 (empty-host, 캡션 別문단) page_avail | 850.1 | 850.1(불변) | — |

- **잔여 −7.8/+7.2pt = 1줄 razor**: end_cut 36 vs 37(소스 hard_break). 내용셀
  avail 828.6 vs 필요 828.8(0.2px). advance_row_cut 이 hard_break 도달 직전 용량컷.
  = 내용셀 per-line 측정 미세 과대 누적(#1759/#1760/#1763 계열, 별개 축). 본 예산
  교정 범위 밖.

## 회귀 관점

- 영향면: 빈 host + out-of-flow para float(TopAndBottom+vert=Para+v_off>0) **且
  같은 문단 선행 in-flow inline 존재** 표만. 그 외(#874 등 선행 inline 없음)는 불변.
  검증: pi=117(캡션이 別문단)=budget 불변, 오버플로 미발생.
- 게이트: cargo test, big_hwpx/big_hwp render-diff, #874/#1022/#1046/#1855.
