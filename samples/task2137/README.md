# Task #2137 재현 샘플

## 156618554_petfood_press.hwp (실문서, 공개 보도자료)
- 출처: korea.kr 공개 동정자료 — 농식품부 '27년 펫푸드 수출액 5억불 지원(2024-03-05).
  hwpdocs `korea_downloads/농림축산식품부/156618554_….hwp` (120KB, 공개 가능 문서).
- 형상: 문서 마지막 pi13 = 빈 앵커 + **소형 부동 글상자**(사각형, 자리차지
  TopAndBottom, vert=문단, 49.8px高). 앵커 저장 vpos=68600 → 줄 경계 70000 ≤ 본문
  70018HU (page-last 증거).
- 결함(수정 전): #2093 신뢰 경로의 controls.is_empty() 배제 + 개체 하단 넘침으로
  앵커+개체 2쪽 단독 (한글 1쪽 — 개체 하단 여백 스필).
- 기대: 1쪽, 개체 여백 스필. visual sweep OK 1=1쪽(88.3%), 오라클 MATCH.
- 검증: `rhwp dump-pages samples/task2137/156618554_petfood_press.hwp` /
  `cargo test --test issue_2137_topbottom_float_anchor_saved_fit`

## 156637323_unification_lecture.hwpx (실문서, 공개 보도자료)
- 출처: korea.kr 통일부 — 국립통일교육원장 한미연합사 특강(2024-06-24).
- 형상: 문서 마지막 pi19 = 빈 문단 + **소형 tac(글자처럼) TopAndBottom 도형**
  (정책브리핑 배너, 61.4px). 저장 lineseg vpos 895.2 + lh 61.4 = 956.6 > 본문
  933.6 — 한글은 하단 여백 스필로 1쪽 유지.
- 결함(수정 전): tac 개체가 #2093 신뢰 경로(controls 게이트)와 #409 atomic
  top-fit(#1027-E2 TopAndBottom Shape 제외) 양쪽에서 배제 → 2쪽 (10k r12
  OVER+SHAPE 잔존 대표).
- 기대: 1쪽, 배너 하단 여백 스필 렌더.
