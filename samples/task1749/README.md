# Task #1749 재현 샘플

## saved_bounds_cumulative_vpos.hwpx
- 출처: 서울 정보소통광장 정보공개 결재문서(공개) 36371084 — opengov 결재문서 계열,
  PII 방침 A(그대로 동결, `samples/hwpx/opengov/README.md` 선례).
- 특성: **누적좌표 문서** — 저장 LINE_SEG vpos 가 페이지 경계에서 리셋되지 않고 계속 증가
  (pi18 vpos=72626, pi19 vpos=74902 > 본문높이 74265HU). 저장 vpos 가 페이지 배정을
  인코딩하지 않음.
- 결함(수정 전): 1쪽 말미 꼬리 공백 문단 pi18(" ")이 누적높이 검사 탈락(998.7 > 986.2px)
  인데도 `saved_single_line_bottom_fits`(저장 bounds 985.7px ≤ avail) 로 1쪽 배치
  → used 1011.8px > 본문 990.2px overfill.
- 기대(한글 정합): pi18 은 2쪽 시작 (한글 OLE 캐럿 p2).
- 검증: `rhwp dump-pages samples/task1749/saved_bounds_cumulative_vpos.hwpx` /
  `python tools/verify_pi_page_vs_hangul.py --files samples/task1749/saved_bounds_cumulative_vpos.hwpx -o out.tsv`

## saved_bounds_cumulative_page_break.hwpx
- 출처: 서울 정보소통광장 정보공개 결재문서(공개) 36375752 — opengov 결재문서 계열,
  PII 방침 A(그대로 동결).
- 특성: **누적좌표 + 명시적 쪽나누기** — vpos 가 쪽 경계에서도 리셋 없이 증가하고,
  2쪽 마지막 문단 pi=26(vpos=137484) 다음의 pi=27 이 [쪽나누기](column_type=Page).
  저장 lineseg 상 pi=25(134764)와 pi=26 은 한 줄(2720HU) 간격 연속 = 2쪽 배치가 정답.
- 결함(#1749 1차 게이트): `saved_flow_marks_page_last` 가 "vpos 리셋"만 페이지-마지막
  증거로 인정 → 쪽나누기 증거 누락 → pi=26 신뢰 거부 → 누적높이 판정(919.2+36.3 >
  930.5px)으로 3쪽 단독 문단으로 밀림 (5쪽 → 6쪽 회귀).
- 기대(한글 정합): pi=26 은 2쪽 마지막, 전체 5쪽.
- 검증: `rhwp dump-pages samples/task1749/saved_bounds_cumulative_page_break.hwpx` /
  `cargo test --test issue_1749_saved_bounds_page_break`
