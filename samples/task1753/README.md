# Task #1753 재현 샘플

## deferred_takeplace_fill_ahead.hwpx
- 출처: 국가법령정보센터 `[별표 2] 건설사업관리기술인 배치기준 및 건설사업관리 직접경비
  (건설엔지니어링 대가 등에 관한 기준).hwpx` (공개 서식, 21쪽)
- 구조: pi51 = 제목 텍스트("1) 투입인원수 산정기준") + 57×9 자리차지(TakePlace) RowBreak 표
  (vert=문단 +6.1mm) 동일 문단. 표 몸체는 9쪽 잔여 공간에 안 들어가 10쪽으로 이월.
- 기대(한글 정합, PDF 시각 확인): 후속 pi52("※ 시공단계 안전관리 보정계수 및 난이도는 1.0
  이상만 적용")·pi53("2) 보정계수")이 **9쪽 하단**에 렌더 (저장 LINE_SEG vpos=72581/74121).
- 결함(수정 전): rhwp 는 표 fragment(10~11쪽) 뒤 **11쪽**에 배치 — 지연 float 선행 채움 미지원.
- 잔여(범위 밖): pi51 host 제목 줄은 rhwp 가 10쪽 fragment 상단에 렌더(한글 9쪽) — 후속 이슈.
- 검증: `rhwp dump-pages samples/task1753/deferred_takeplace_fill_ahead.hwpx` /
  `python tools/verify_pi_page_vs_hangul.py --files samples/task1753/deferred_takeplace_fill_ahead.hwpx -o out.tsv`
