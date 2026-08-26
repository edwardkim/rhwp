# 10 FDE · Chief · Strategist 층 분리

세 capability 가 같은 "고객 문서"를 만지므로 경계가 흐려지기 쉽다.
층을 섞으면 증상 티켓에 전망이 들어가거나, 목표 보고서에 응급처치가
섞인다.

## 한 줄씩

- **FDE (CAP-4893)** — 지금 이 파일에서 보이는 **증상**. 재현, 우회,
  에스컬레이션. 도구: `tools/fde/triage.py`.
- **Chief (CAP-4900)** — **요청 큐**. 트리아지 게이트, goal 라우팅,
  needs-agent. 도구: `tools/chief/service_loop.py`.
- **Strategist (CAP-4903)** — **목표 + 코퍼스**. 전수 지도, 근거 대장,
  §5 게이트. 도구: `tools/strategist/engagement.py`.

## 분류 질문

1. 고객이 지금 깨진 화면/파일을 가리키는가 → FDE
2. 여러 요청이 줄 서 있고 오늘 처리할 것을 고르는가 → Chief
3. "이 더미로 수주/전략 문서를 만들고 출처를 남겨라" → Strategist

애매하면 Chief 가 받아 분류한다. 목표형(`~하고 싶다`)은
`needs-agent` 로 이 스킬/에이전트에 넘긴다.

## 인계 시 가져가는 것 / 두고 가는 것

Strategist 가 FDE 로부터 받을 수 있는 것: 읽을 수 있는 파일 경로,
암호 여부, "이 파일은 info 실패"라는 사실.

받아서는 안 되는 것: FDE 의 응급 패치 추측을 전략 주장으로 승격.

Chief 로부터 받을 것: `objective` 초안, 코퍼스 폴더, 고객이 던진 질문.

받아서는 안 되는 것: 큐 우선순위를 근거 대장에 적는 일.

## 금지 교차

- strategist 스킬 본문에 fde triage 사다리를 복사하지 않는다.
- fde/bug-hunter 스킬을 이 파동에서 수정하지 않는다.
- chief 의 요청 상태 머신을 여기 구현하지 않는다.

예제: [examples/10_fde_symptom_not_strategy.md](../examples/10_fde_symptom_not_strategy.md),
[examples/11_chief_goal_handoff.md](../examples/11_chief_goal_handoff.md).

다음: [11_sws_audit.md](11_sws_audit.md).
