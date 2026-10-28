# Task #1759 Stage 1 완료보고서 — 드리프트 서베이 하니스

## 수행 내용
- `tools/metric_drift_survey.py` 신설: 한글 PDF(pyhwpx 배치) + PyMuPDF(baseline·find_tables)
  ↔ rhwp(export-svg baseline · export-render-tree Table bbox) 대조. 산출: pages.tsv
  (페이지별 pitch delta) / tables.tsv (표별 dw/dh/dy) / summary.tsv. PDF 배율 경고·쪽수
  불일치 파일은 SKIP 표기, 표 미검출(무테 결재란 등)은 unmatched 정직 표기.

## 기지값 재현 게이트 (36388181)
- 1쪽 pitch delta **+0.80px/줄** — 사전 측정과 정확 일치.
- 경계 표(pi18, 9×6) 매칭: dh **+1.6px**(rhwp 표가 큼), dy **-4.1px**(rhwp 시작이 높음)
  → bottom 차 -2.5px 재현 = 사전 측정(1017.6 vs 1020.1)과 일치. 계획서의 "표 -2.5px"는
  bottom 기준이었음을 dh/dy 분해로 정밀화 — 하니스가 y 표류(상류 흐름)와 높이 표류
  (표 내부)를 분리 측정.
- py_compile 통과, rust 소스 무변경.

## 상태
완료. Stage 2 (1차 서베이: razor 8건 + MATCH 대조군 30건) 진행.
