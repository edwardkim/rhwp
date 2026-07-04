# Task #1748 최종 보고서 — scattered header RowBreak 표 continuation 하단 over-fill 수정

## 요약

#1728 잔여 갈래였던 scattered header RowBreak 표(260×9)의 p6 하단 +13px(1행 과다
노출)를 수정. 원인은 이슈의 추정(#1718 grace 계열 행 컷 판정)이 아니라 **컷 행에
걸친(straddling) rowspan 셀의 렌더 처리 결함**이었다. 행 컷·페이지네이터 회계는
정확했고, 걸친 rowspan 셀만 가시범위 제한 없이 전체 렌더(컷 페이지)·전체
재렌더(연속 페이지, 중복)되고 있었다. 높이 기반 유닛 컷 도입으로
**p6 Δbot +13 → −4px** (게이트 ≤~5px 달성), p7 중복 재렌더·아래 행 침범도 해소.

- 이슈: #1748 / 브랜치: `local/task1748` (upstream/devel + 제출 열린 PR 19건 선적용)
- 수정: `src/renderer/layout/table_partial.rs`, `table_layout.rs` (헬퍼 1개) —
  **페이지네이션(typeset.rs) 불변**
- 테스트: `tests/issue_1748_rowbreak_straddle_rowspan.rs` 3건 신규

## 원인 (1단계 조사, `mydocs/working/task_m100_1748_stage1.md`)

- p6 = `PartialTable pi=9 rows=100..140 end_cut=[1,2]` — `RHWP_TABLE_DRIFT` 회계는
  `consumed=981.1 ≤ avail=988.3` 정상, render tree 표 박스도 회계와 일치(1002.0px).
  그러나 래스터 잉크는 표 하단(1077.6px)을 넘어 1094px — **셀 박스 밖 텍스트**.
- 페이지 경계가 rowspan 블록 내부를 per-row 분할할 때(#1022 경로) 컷
  부기(start_cut/end_cut)는 컷 행의 `row_span==1` 셀만 담는다. 경계에 걸친
  rowspan 셀(row 138~, span이 컷 행 139 포함)은
  `is_split_end_row(cell_row == end_row-1)` 판별 밖 → `clip:false`,
  `cut_units:None` → 전체 줄 무제한 렌더(3줄 초과, +44.9px). 연속 페이지 p7은
  같은 이유로 **처음부터 재렌더** → 중복 + 아래 행 침범(+61.9px).
- 한글 2024 PDF는 컷 페이지에 들어가는 줄까지만 렌더하고 나머지를 연속 페이지에
  이어그린다.

## 수정 (2단계, `mydocs/working/task_m100_1748_stage2.md`)

- `table_layout.rs`: `cell_units_fitting_height` — 유닛 누적높이가 예산에 들어가는
  선두 유닛 수 (EPS 0.1px).
- `table_partial.rs`: RowBreak 표 비블록 분할에서 걸친 rowspan 셀 판별
  (`is_rowbreak_straddle`: 시작 걸침 `cell_row < start_row < cell_end_row`,
  끝 걸침 span이 프래그먼트 끝 초과 또는 마지막 span 행이 분할 행) → 높이 기반
  유닛 컷 `su = fit(prior_h − pad_top)`, `eu = fit(prior_h + cell_h − pad_top)`.
  prior_h는 2b 행높이 오버라이드와 동일 식으로 결정적 재계산되어 컷 페이지 eu ==
  연속 페이지 su 가 산술적으로 일치(상태 전달 불필요). 줄 범위 변환은 기존
  `cell_line_ranges_from_cut` 재사용, `clip:true` + Top 정렬 강제.
- 한계(문서화): 한 셀 span이 3개 이상 프래그먼트에 걸치며 중간 span 행 자체가
  또 분할되는 극단 케이스는 prior_h의 pad 중복으로 경계 줄이 ±1줄 어긋날 수 있다
  (이번 샘플 포함 일반 케이스는 정확).

## 검증 (3단계)

### 본 건 (96 DPI dark-pixel bbox vs `pdf/table_scattered_header_rowbreak-2024.pdf`)

| 항목 | 수정 전 | 수정 후 | 게이트 |
|------|--------:|--------:|--------|
| p6 Δbot | **+13** | **−4** | ≤ ~5px ✅ |
| p5 Δtop/Δbot | −1/−6 | −1/−6 | 무회귀 ✅ |
| 전체 쪽수 | 53 | 53 | 불변 ✅ |
| p7 걸친 셀 | 전체 재렌더(중복) | 잔여 줄 이어그리기 | 한글 정합 ✅ |

- 문서 후반부(p8~)는 한글 52쪽 vs rhwp 53쪽의 기존 전역 표류(#1772/#1774 계열)가
  있으며 본 타스크 범위 밖 (수정 전후 페이지네이션 동일하므로 악화 없음).

### 회귀 게이트

| 게이트 | 결과 |
|--------|------|
| 신규 테스트 3건 (`issue_1748_rowbreak_straddle_rowspan`) | 통과 (수정 전 실측 +44.9/+61.9px 초과였던 단언) ✅ |
| `cargo test --release` 전체 | **2748 passed / 실질 실패 0** ✅ |
| svg_snapshot 골든 | 내용 기준 8/8 불변 — 7건 "실패"는 Windows autocrlf CRLF 노이즈, **이슈 #1786 등록** ✅ |
| giant (`task1718/table_giant_cell_overfill.hwp`) | 42쪽 불변, p5/p6 Δtop −3/Δbot −2 (#1728 v2 수준) ✅ |
| byeolpyo1 / byeolpyo4 (#1658 게이트) | 4쪽 / 26쪽 무회귀 ✅ |
| `tools/clipping_gate.py` (controlset 92) | 검사 92 / 회귀 0 / baseline 이탈 0 ✅ |
| `tools/render_page_gate.py` (대형 오라클 452) | **443 일치(98.0%)** — #1658 기록 442 대비 +1 ✅ |
| `tools/render_page_gate.py` (소형 92) | 73 일치 — 통합 베이스(선적용 PR 19건) 상태이며, 본 diff는 렌더 트리 전용(`layout_partial_table`)이라 쪽수 산정 표면 없음 |
| `rustfmt --check` (변경 파일) | 통과 |

## 산출물

- 소스: `src/renderer/layout/table_partial.rs`, `src/renderer/layout/table_layout.rs`
- 테스트: `tests/issue_1748_rowbreak_straddle_rowspan.rs`
- 검증 도구: `tools/compare_page_bbox.py` — 페이지 dark-pixel bbox 비교
  (rhwp export-pdf vs 한글 PDF). 본 건 게이트 재현:
  `python tools/compare_page_bbox.py <out.pdf> pdf/table_scattered_header_rowbreak-2024.pdf --pages 6 --max-dbot 5`
- 문서: 수행·구현 계획서, stage1·2 보고, 본 보고서
- 재현 샘플: `samples/table_scattered_header_rowbreak.hwp` (기존 커밋,
  PR 재현 자료 규칙 충족) / 비교: `pdf/table_scattered_header_rowbreak-2024.pdf`
- 부수 산출: 이슈 #1786 (svg_snapshot 골든 CRLF Windows 환경 실패)

## 후속 제안

- p5 시점부터 한글 대비 1행 선행하는 행높이 미세 표류(밀집 행 누적)는
  #1759/#1760/#1772 추적 프로그램 범위.
- 3+ 프래그먼트 걸침 극단 케이스의 pad 중복 보정은 실사례 확보 시 후속.
