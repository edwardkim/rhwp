# 빈 번호 머리의 원본 거리 보존

원본은 `tests/fixtures/numbering_levels/levels.hwp`·`levels.hwpx`이고,
독립 Print는 `pdf/semanticist21-20261105/pr8527/levels-hwp-2020.pdf`·`levels-hwpx-2020.pdf`다.
입력·PDF SHA는 [현재 측정 신원](../measurements.md), 전체 TSV는 [tsv](../tsv)에서 두 key를 찾을 수 있다.

R5는 Native 1쪽 95.16596%였으나 빈 9·10수준 본문 간격 약 6.67 CSSpx가 누락됐다.
R5 pinned 라이브러리 SHA `9300b684b8145abf1c2626dd2143a3bd002f0035fbd13b793189f624f7811f2e`의
최소 공개 글꼴 관계 검사는 원본 HWP/HWPX 2건 모두 FAIL했다.
R6 공통 보정 후 두 형식의 Native/fresh WASM 1쪽은 모두 raw 100.00%다.
본문 원점과 기존 가시 번호·내용 순서를 실제 review·overlay로 확대 대조했다.
[같은 페이지 패널](../panels.md)과 [구현 분석](../../empty-numbering-head-analysis.md)을 연결한다.

기존 3건+새 5건, 총 8건이 최소 공개 글꼴 환경에서 PASS·실패 0·exit 0이다.
원본 거리 50%와 거리 0 개입은 공개 set_document로 파생 번호 정보를 재구축해 구분했다.
독립 Print 본문 시작 180.022435pt·189.977040pt를 96/72로 환산한 ±1 CSSpx 검사와
줄·문단 소속·전체 내용 보존 관계를 함께 유지한다. 잘못된 setup·run 분리 가정을 보정했으며
기존 독립 기대값이나 오차를 넓히지 않았다. [완료 페이지 조건](../qualification.md)을 따른다.

보정 전 R5 Native 1쪽: [review](../../validation-r5/png/pr8527/native/8527-levels-hwp-2022/review_001.png) · [overlay](../../validation-r5/png/pr8527/native/8527-levels-hwp-2022/overlay_001.png). 현재 R6 패널과 source·raw를 구분한다.
