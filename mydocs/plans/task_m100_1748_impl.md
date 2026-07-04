# Task #1748 구현계획서 — scattered header RowBreak 표 continuation 하단 over-fill

## 전제 (재현 확정)

통합 베이스(`local/task1748` = upstream/devel + 제출 열린 PR 19건)에서 재현 확인:

- `dump-pages -p 5`: `PartialTable pi=9 ci=0 rows=100..140 end_cut=[1,2]`, p7 `start_cut=[1,2]` — 이슈 관찰과 동일.
- 96 DPI dark-pixel bbox (pymupdf, rhwp `export-pdf` vs `pdf/table_scattered_header_rowbreak-2024.pdf`):
  p5 Δtop −1 / Δbot −6, **p6 Δtop −1 / Δbot +13** — 이슈 관찰(rsvg 기준 −2/−5, −2/+13)과 동등.
- 측정 스크립트는 `tools/compare_page_bbox.py` 로 커밋한다 (pymupdf 96 DPI, thresh<160 — 리뷰어 재현용).

## 구조 이해

p6 은 rows 100..140 의 행 범위 PartialTable 이며, 마지막 행(139)은 셀 유닛 단위로
부분 소비(`end_cut=[1,2]`)된다. 행 컷의 단일 권위는
`src/renderer/layout/table_layout.rs` 의 `advance_row_cut`/`advance_row_block_cut`.
이 표는 RowBreak + `row_count=260 > 5` → `relaxed_hard_break=true` 경로이고,
초과 수용 후보는:

- ① `ROWBREAK_VISIBLE_TAIL_OVERFLOW_TOLERANCE_PX=120` grace — `grace_visible_tail_before_spacer` 가 true 면 avail 초과 유닛도 수용 (`h + u.height <= avail + 120`)
- ② 행 적재 판정(typeset.rs 페이지네이터)의 잔여높이(avail) 산정 오차
- ③ `tiny_fragment_waste`/`HARD_BREAK_REMAINING_TOLERANCE_PX=32` 등 인접 관용 경로

## 단계 구성 (3단계)

### 1단계 — 원인 조사·확정

- p6 행 컷 시점의 avail_height, 마지막 행(139) 셀 유닛 시퀀스(`empty_spacer`/`vis_start..vis_end`/height), grace 발동 여부를 로그/일회성 eprintln 계측으로 추적.
- 한글 PDF 기준 p6 마지막 가시 행과 rhwp 마지막 행을 대조해 "초과 1행"이 어느 판정에서 들어왔는지 확정 (후보 ①~③).
- end_cut=[1,2] 의 의미(셀별 유닛 수) 해석 포함.
- 계측 코드는 커밋하지 않는다 (조사 후 제거). 산출: `mydocs/working/task_m100_1748_stage1.md` (조사 보고).

### 2단계 — 수정 구현 + 단위테스트

> **[1단계 조사 후 갱신]** 가설 ①~③ 기각. 행 컷·페이지네이터 회계는 정확
> (consumed=981.1 ≤ avail=988.3, render tree 표 박스도 일치). 결함은 **컷 행에
> 걸친(straddling) rowspan 셀의 렌더 처리** — 컷 페이지에서 가시범위 제한·클리핑
> 없이 전체 렌더(+13px 잉크), 연속 페이지에서 처음부터 재렌더(중복+아래 행 침범).
> 상세: `mydocs/working/task_m100_1748_stage1.md`.

- `src/renderer/layout/table_partial.rs` 비블록 분할 경로에서 걸친 rowspan 셀
  (`cell_end_row > end_row` / `cell_row < start_row`)에 높이 기반 유닛 컷을 적용:
  `cell_units` 를 프래그먼트-가시 높이 예산으로 잘라 su/eu 산출 →
  `cell_line_ranges_from_cut` 재사용 + `clip:true`. 연속분 su 는 이전 프래그먼트
  소비 높이의 결정적 재계산(`row_cut_content_height`, p6 의 end_cut 계산과 동일 식)으로
  p6.eu == p7.su 를 보장.
- 페이지네이션(typeset.rs, advance_row_cut)은 불변.
- 단위/통합 테스트: 걸친 rowspan 셀의 컷 페이지 줄 제한 + 연속 페이지 이어그리기
  (중복 없음) 검증.
- 산출: 소스 커밋 + `mydocs/working/task_m100_1748_stage2.md`.

### 3단계 — 회귀 게이트 검증 + 최종 보고

| 게이트 | 기준 |
|--------|------|
| 본 건 | p6 Δbot +13 → **≤ ~5px**, p5 Δbot −6/Δtop −1 무회귀, 전체 53쪽 유지 여부 확인(행 밀림 cascade 점검) |
| giant (`samples/task1718/table_giant_cell_overfill.hwp`) | 42쪽 불변, p5/p6 Δtop −4 불변 (#1728 v2) |
| 별표 (`samples/byeolpyo1.hwp`, `samples/byeolpyo4.hwp`) | 쪽수 4/27 무회귀 (#1658 게이트) |
| 테스트 | `cargo test` 전체 무회귀, `cargo fmt --check` |

- cascade 주의: p6 에서 1행이 빠지면 p7 이후 행 배분이 전부 이동한다. 한글 PDF 와 총
  쪽수(53)·후반 페이지 bbox 를 함께 대조해 개선이 국소가 아닌 전역 정합인지 확인.
- 산출: `mydocs/report/task_m100_1748_report.md`, 필요 시 재현 샘플은 이미
  `samples/table_scattered_header_rowbreak.hwp` 존재 (PR 포함 규칙 충족).

## 리스크

- grace 축소는 #1718 이 겪은 대로 별표류(over-pagination 방지) 회귀와 상충 위험 —
  게이트 3종(giant/별표1/별표4)으로 방어.
- 120px tolerance 자체를 줄이는 방식은 광범위 회귀 우려 → 판정 조건(구조 판별)을
  좁히는 방식을 우선한다.
