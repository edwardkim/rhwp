# 04 — STRUCT_MISMATCH 는 데이터다

`status: STRUCT_MISMATCH` 는 종료 코드 1(텍스트) 또는 3(`--json`)을
낸다. 자동화는 그걸 **실패 신호**로 받을 수 있다. 에이전트는 받아서
**경로를 읽는다.** 반사적으로 롤백하지 않는다.

## 읽는 순서

1. `Δ TextRun: 15→13 (-2)` 같은 타입 증감. 음수=손실, 양수=추가.
2. 변위 큰 노드 경로 상위 몇 개 (`495.93px  Page/Body2/...`).
3. 그 경로가 **방금 편집한 위치**와 같은가.
4. 상위 틀(`Page`, `Page/PageBg0`)이 0.00px 인가.

편집 위치와 같으면 F03 — 정상. 값이 바뀌면 그 자리 구조도 바뀐다.
무관한 머리말·다른 단·로고면 F04 — 진짜 회귀.

## 임계와 무관

`--max-disp 100` 을 줘도 하드 구조 불일치는 STRUCT 로 남는다.
임계는 변위(OVER)만 가른다.

## TextRun ±1

한 페이지의 구조 차이가 TextRun 삽입·삭제 각 최대 1개뿐이면
`WARN_TEXTRUN` (#1773). 하드 실패가 아니다. 다른 페이지에 일반
STRUCT 가 있으면 문서는 여전히 STRUCT_MISMATCH.

## JSON 에서

`pages[].topDeltas[].path`, `pages[].typeDeltas`, `hardStructPages`.
`regression: true` 여도 경로부터 대조한다.
