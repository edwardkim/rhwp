# Task #1750 재현 샘플

## split_guard_spacing_before.hwp
- 출처: 국가법령정보센터 `[별표 5] 수소전기자동차의 에너지소비효율 측정방법(제4조제1항제5호 관련)
  (자동차의 에너지소비효율 및 온실가스 배출량 시험방법 등에 관한 고시).hwp` (공개 서식, HWP5/OLE)
- 결함(수정 전): 1쪽 말미 pi22 문단(sb=9.3px, 첫 줄 25.6px)이 분할 진입 가드의
  spacing_before 미반영으로 페이지 넘김을 생략, 분할 루프가 첫 줄을 무조건 배치
  → 1쪽 used 1010.9px > avail 1005.1px (5.8px overfill), PartialParagraph pi=22 lines=0..1.
- 기대(한글 정합): pi22 전체가 2쪽 시작 (한글 OLE 캐럿·저장 LINE_SEG 다음 줄 vpos=700 모두 새 쪽 상단).
- 검증: `rhwp dump-pages samples/task1750/split_guard_spacing_before.hwp` /
  `python tools/verify_pi_page_vs_hangul.py --files samples/task1750/split_guard_spacing_before.hwp -o out.tsv`
