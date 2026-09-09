# Task #1765 구현계획서 — 병합 셀 경로 가설 검증

수행계획서: `mydocs/plans/task_m100_1765.md`

## Stage 1 — 재현 동결 + 실패 확인
- `samples/task1765/merged_cell_trailing_ls.hwp` (17931383) + README.
- render-tree/하니스로 dh +5.2 확인.
- 이 시점의 병합 셀 trailing ls 설명은 초기 가설로 기록한다.

## Stage 2 — 가설 검증 + 기각
- 행높이 대조로 17931383 후보 표의 셀이 전부 `rs=1` 임을 확인했다.
- +5.2px 는 병합 셀 경로가 아니라 row9 단일 행의 per-line 콘텐츠 측정 누적으로 재분류했다.
- 2-b 경로 가드는 실증 재현 부재로 제외하고, 시도한 소스 변경은 되돌렸다.

## Stage 3 — 샘플 보강 + 최종보고
- HWP/HWPX 샘플과 한컴 2024 기준 PDF를 남긴다.
- visual sweep 으로 문제 페이지 구조를 대조한다.
- 최종 보고서는 소스 무변경, #1759 per-line 누적 대표 사례로 정리한다.
