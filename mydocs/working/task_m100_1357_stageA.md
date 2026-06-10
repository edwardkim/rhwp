# Stage A 완료보고서 — Task #1357 시각 회귀 하니스 + 베이스라인

## 하니스
기존 `scripts/task1274_visual_sweep.py` 재사용 (PDF↔native SVG 페이지 비교: frame/red/
line/column/eq/title/order/tail/question 메트릭). 본 타스크 대상 샘플을 TARGETS 에 추가:
`2024-09-below20above20` (`3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`).

실행: `python3 scripts/task1274_visual_sweep.py --target all --out output/task1357/<tag>`

## 베이스라인 (수정 전, base=stream/devel — #1355 미포함)

| target | flagged | 상세 |
|--------|---------|------|
| 2022-09 | 1/23 | column[10] order[10] |
| 2023-09 | **0/20** | clean |
| 2024-09-below20 | 1/23 | column[10] order[10] |
| 2024-09-between20 | 1/24 | line[11] order[11] |
| 2022-10 | **0/18** | clean |
| 2022-11-practice | 1/21 | line[13] column[13] tail[13] |
| **2024-09-below20above20** | **7/23** | red[9,13] line[9,10,13,17,22] column[9,13,18,19,22] order[10] tail[9,17,18,19,22] question[9,13,22] |

## 관찰
- 회귀 가드 6종은 0~1 flag 로 안정 → 수정이 이들 flag 를 **늘리면 회귀**.
- 대상 샘플은 7 페이지가 PDF 와 divergent. 단 red/question/title flag(9,13,22)는
  **#1355(미주 제목 gap 이중계상)** 영향으로 추정 — task1357 base 에 #1355 미포함.
  → 깨끗한 #1357 베이스라인을 위해 task1357 을 **#1355 포함 base 로 rebase** 후 재측정.
- #1357 의 핵심 대상은 **column/tail overflow flag**(누적 과소충전→본문 하단 초과).

## 다음
Stage B 전, base 정리(#1355 포함) + 재베이스라인 → col0 tail 예측 블록 구현 → Stage C
하니스 재측정으로 무회귀 확인.
