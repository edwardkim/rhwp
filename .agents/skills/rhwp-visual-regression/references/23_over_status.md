# 23 — OVER

구조(노드 개수·경로)는 같고, 대응 노드 변위가 `--max-disp` 를 넘었다.

같은 글자 수 치환인데 줄바꿈이 달라져 아래 문단이 밀린 경우가 전형이다.
STRUCT 가 아니므로 경로 개수는 맞다. `worst_page` 와 상위 `topDeltas`
의 disp 를 본다.

임계를 헐겁게 하면 OVER 는 사라질 수 있다. 그게 맞는지(폰트 힌팅
노이즈) 실제 여백 회귀인지는 `export-png` 로 그 쪽을 본다.

채움처럼 구조가 바뀌면 OVER 가 아니라 STRUCT 가 먼저 붙는다
(우선순위).

JSON: `status: OVER`, `regression: true`, `hardStructPages: 0`,
`overPages >= 1`. 텍스트 모드 exit 1, `--json` exit 3.
