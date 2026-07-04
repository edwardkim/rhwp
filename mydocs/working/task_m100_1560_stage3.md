# Task #1560 Stage 3 완료보고서 — 매뉴얼 + 검증 + 최종 보고

## 변경
- 신규 `mydocs/manual/hangul_page_oracle.md` (사용법·판정·종료코드·한계·워크플로 연동).
- 최종 보고서 `mydocs/report/task_m100_1560_report.md`.
- 임시 스크립트(`output/poc/fidelity*/t3_*.py`)를 본 도구로 대체(매뉴얼에 명시).

## 최종 검증
- 배치/인벤토리 두 모드 동작, `--sample 45 --seed 42` 가 v3 결과(1/45=2%) 재현.
- 알려진 붕괴 케이스(36384160·36387103) 검출, secCnt 수정 케이스(36382669) OK.
- 종료 코드 1(게이트), git HEAD(b086bd5a) 기록.

## 결론
한글 페이지 오라클 정식화 완료. IR-blind 페이지 붕괴를 재현·게이트 가능하게 검출.
