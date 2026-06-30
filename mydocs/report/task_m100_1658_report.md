# 최종 보고서 — #1658 페이지네이션 엔진 개선 (라운드 1) / PR #1670

- 마일스톤: M100 / 브랜치: local/task1658 (upstream/devel) / PR: edwardkim/rhwp#1670
- 일자: 2026-06-30 / 성격: 다세션 연구·개선 프로젝트 **라운드 1 완결**.
- 선행: #1643(검증 도구), #1648(경계 빈 문단), #1653(RCA·통제 하네스, closed). #1658 **존속**.
- 오라클: OLE 한글(Windows+한컴) — 페이지수·줄 baseline·클리핑의 권위 기준.

## 1. LANDED 수정 (소스: table_layout.rs +11줄)
`advance_row_cut`/`advance_row_block_cut` `tiny_fragment_waste` 가드:
거대 표 셀이 페이지를 가로질러 분할될 때, 셀 내용 vpos reset 이 촘촘하면 잔여공간 충분에도 reset
마다 페이지를 끊어 ≤2줄 낭비 페이지 양산 → over-pagination. 현재 fragment 가 ≤2 유닛이고 잔여공간이
tolerance 초과면 그 reset break 를 흡수. #1488(가시 문단 reset 3유닛 후 보존)은 j>start+2 라 무영향.

### 검증
| 항목 | before | after |
|------|--------|-------|
| 별표1(국토부 주차장법) | 5쪽(Δ+1) | **4쪽 일치** |
| 별표4(산업통상부 LPG) | 33쪽(Δ+8) | **28쪽(Δ+3)** |
| 대형 오라클(랜덤 452) | 97.6% | **97.8%**(신규 −1=0) |
| 소형 controlset(92) | 75 | **75 무회귀** |
| lib / hwpx_roundtrip | — | **1984 / 4 passed**(#1488 보존) |

## 2. 측정·검증 인프라 3종 (재사용 자산)
| 도구 | 역할 | 비고 |
|------|------|------|
| `tools/build_page_oracle.py` + `tests/fixtures/render_page_oracle_1658.tsv` | 한글 PageCount 정답지(랜덤 452) | 92건 과적합 방지 |
| `tools/detect_table_clipping.py` | 본문 클리핑(시각 손실) 검출 (SVG body-clip 초과) | 페이지수 게이트가 못 잡는 시각 회귀 |
| `tools/hangul_pdf_baseline.py` | 한글 줄 baseline/pitch 권위 기준(PDF+fitz) | PageCount 가드 |
가이드: `mydocs/manual/visual_clipping_detector.md`, `hangul_pdf_baseline.md`.

## 3. 규명·정정한 것
- **줄높이 fidelity 정상**: byeolpyo1(권위 PDF 4=4) 한글 줄 pitch **28.80 = rhwp 28.80** → drift 없음.
  → #1658 잔여(별표4 Δ+3, 클리핑)는 **줄높이 아니라 cut 로직 / 한글 PDF 배율 아티팩트** (가설 정정).
- **클리핑 ≡ capacity 동일 뿌리·상충**: 별표4 23.5px 클리핑(render>cut)과 capacity 결손(rhwp<한글)은
  같은 cut↔render↔한글 적재량 차이의 양면. 클리핑 수정↔페이지수 상충 → 단일 조정 불가.
- **별표4 over 성격**: avail=full body(tolerance 0). 잔여는 행 전환 tail fragment(선행 행 누적으로
  tail 부족). reset 분류 판별자 없음(vis_start 혼재).

## 4. 배제·차단 (근거 자료)
- 하단 고정 표 over: Phase 0~2, 8종 시도 전부 회귀(fixpoint·진동). 저ROI.
- vpos-snap 이식 / reset-snap: 회귀(증가전용·32px 게이트 회귀).
- 한글 행높이 레퍼런스: COM(CellShape/TableLowerCell) 차단, PDF 배율 100% 강제도 차단(save_as
  PrintMethod 무시, 별표4 13≠25쪽 압축).

## 5. 잔여 (후속 라운드) — 단일 선결 과제로 수렴
- 별표4 Δ+3·클리핑·하단표 모두 **한글 권위 레이아웃 레퍼런스(배율 100% PDF / COM 표 API)** 가
  선행되어야 진척. → 한글 COM PDF 배율 제어 연구가 다음 라운드의 정공법.
- 별표4 23.5px 클리핑은 capacity 와 상충하므로 3자(cut↔render↔한글) 적재량 정합으로만 동시 해결.

## 6. 결론
- 법령 표 행분할 over 의 1·2유닛 낭비 페이지를 **무회귀로 실제 수정**(별표1 일치, 별표4 대폭 개선),
  대형 오라클 정합 97.6→97.8%.
- 페이지네이션 작업의 **측정·검증 삼각대(페이지수+클리핑+baseline)** 구축.
- 잔여의 성격(줄높이 아님, 한글 레퍼런스 선결)을 데이터로 규명. **#1658 존속**, 본 라운드 landed·무회귀.

## 7. 산출물
- 소스: `src/renderer/layout/table_layout.rs`(+11)
- 도구/오라클: `tools/{build_page_oracle,detect_table_clipping,hangul_pdf_baseline}.py`,
  `tests/fixtures/render_page_oracle_1658.tsv`
- 분석: `mydocs/tech/task_m100_1658_*.md`, `working/task_m100_1658_phase1/2/3.md`,
  `manual/{visual_clipping_detector,hangul_pdf_baseline}.md`
