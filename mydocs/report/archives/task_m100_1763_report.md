# Task #1763 최종 보고서 — 셀 선언높이 권위: trailing ls 초과 확장 교정

## 요약
#1760 행 분해의 후속 정밀 조사에서 확정한 결함 수정. 다문단 셀의 콘텐츠 높이 측정이
**셀 마지막 줄 trailing line_spacing 을 포함**(#874/#1086 보존 조건)해 선언 셀높이를
초과 확장하던 문제를, **선언높이 권위 가드**(초과분이 전적으로 trailing ls 이면 선언으로
clamp)로 교정. 대표 2501937 row0 +7px 해소(142.2px = 한글 정합), #1759 하니스 재측정에서
대형 표 높이 오차 2건(+7.2/+10.9 → +0.3/+0.9) 소멸.

## 원인
`height_measurer.rs` 측정: `include_trailing_ls = !is_cell_last_line || cell_para_count > 1`
— 다문단 셀은 마지막 줄 trailing ls 포함 → required(콘텐츠+trailing+pad) > 선언 cell.height
→ 행 확장. 줄 간격 자체는 저장 LINE_SEG 와 전부 일치(21.3/56.2/21.4/21.3px) — 저장
cell.height 는 trailing 미포함 원칙(주석 825행)과 측정의 충돌.

## 수정
1. `height_measurer.rs`: `cell_last_trailing_ls` 산출(포함 조건 동일) + 가드 —
   `required > 선언` 이고 `콘텐츠−trailing+pad ≤ 선언` 이면 선언높이로 clamp.
   콘텐츠가 진짜 초과하는 기존 보존 케이스(aift/KTX)는 불변.
2. `typeset.rs` (Stage 2b): 측정 정밀화로 드러난 #1753 prefill 상호작용 회귀 수정 —
   prefill fit 의 여분 4px 마진 제거(후보는 저장 vpos 같은-쪽 보증 존재).

## 검증
| 항목 | 결과 |
|------|------|
| 재현 (render tree) | row0 149.1 → **142.2px** (선언 = 한글 142.1) |
| #1759 하니스 재측정 | 2501937 +7.2→+0.3, 3018147 +10.9→+0.9 |
| cargo test --lib / 통합 15크레이트 | 2051 / 114 passed, 0 failed |
| 페이지 게이트 10종 (국제고속선기준 251 포함) | 무회귀 |
| 코퍼스 MATCH 표본 150건 | 150/150 유지 |
| 코퍼스 mismatch | 총계 동일. 36383764 pi25 +1 (기존 razor 문서의 경계 이동 — stage3 문서화) |
| rustfmt / clippy | 통과 |

## 한계 / 후속
- rowspan 병합 셀 경로(2-b)는 동일 패턴 미적용(재현 사례 rs=1) — 잔여 대형 dh
  (17931383 12×4 +5.2 등)와 함께 #1759 잔여로.
- 36383764 pi25: razor 가족 2차 효과 — #1759 프로그램 소관.

## 산출물
- 소스: `src/renderer/height_measurer.rs`, `src/renderer/typeset.rs`
  (+ `tests/issue_1763_cell_trailing_ls_expand.rs`)
- 재현: `samples/task1763/cell_trailing_ls_expand.hwp` + README
- 데이터: `output/poc/drift_survey2/`, `output/poc/hwpdocs_pipage/*_recheck_1763.tsv`
