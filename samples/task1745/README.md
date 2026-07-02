# Task #1745 재현 샘플

## table_text_anchor_wrap.hwp
- 출처: 국가법령정보센터 `[별표 2] 업무정지 처분을 갈음하여 부과하는 과징금 산정기준(제33조 관련)(약사법 시행령).hwp` (공개 서식, HWP5/OLE)
- 구조: pi0 = 제목 텍스트 + 48×6 표(wrap=어울림, RowBreak, 1~3쪽 분할)가 **같은 문단**에 anchor,
  pi1 = 빈 문단 (저장 LINE_SEG `vpos=2072, cs=45568, sw=2620` — 1쪽 표 오른쪽 잔여 띠).
- 기대(한글 정합): pi1 줄이 **1쪽** 표 오른쪽 잔여 폭(9.2mm)에 배치.
- 결함(수정 전): Task #362 wrap 흡수의 기준 cs/sw 를 표 문단 첫 LINE_SEG(전폭 제목 줄)에서
  취해 매칭 실패 → pi1 이 표 아래 **3쪽** FullParagraph 로 배치 (PI_MISMATCH).
- 검증: `rhwp dump-pages samples/task1745/table_text_anchor_wrap.hwp` /
  `python tools/verify_pi_page_vs_hangul.py --files samples/task1745/table_text_anchor_wrap.hwp -o out.tsv`
