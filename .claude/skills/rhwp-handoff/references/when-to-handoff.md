# 언제 세션을 넘기는가

세션 핸드오프는 습관이 아니라 **트리거**다. 세 가지만 인정한다. 셋 모두
outgoing 이 인계 묶음을 닫은 뒤에만 incoming 이 재개한다.

## 1. 컨텍스트 예산 (context budget)

창(토큰·첨부·툴 출력)이 가득 차기 **전**에 넘긴다. 신호가 오면 새 작업을
시작하지 말고 절차 A 를 닫는다.

인정하는 신호 (하나면 충분):

- 호스트가 compact/summarize 를 예고했다
- 같은 파일을 세 번 이상 다시 읽었다
- `result.json` 과 working doc 이 이미 있고, 다음 할 일이 새 탐색이다
- 오케스트레이터 저널이 한 task 의 수용/`selfExecute` 까지 도달했다

하지 않는 것:

- "조금만 더 보고 넘긴다" — 예산을 넘기면 last result 가 부분 기억이 된다
- 대화 요약을 인계로 대체 — 요약은 파일이 아니다
- 새 CLI 로 예산을 조회 — 그런 명령은 없다

outgoing 체크리스트:

1. 진행 중 위임이 있으면 `python tools/handoff/orchestrator.py --task … --json` 으로
   `result.json` 을 남긴다
2. 이번 세션 실작업을 `rhwp replay --capsule session.capsule.json` 으로 고정한다
3. 이전 세션 캡슐이 있으면 `--parent` 를 붙인다
4. working doc 에 남은 목표와 세 파일 경로를 적는다
5. 그 다음 턴에서 새 탐색을 하지 않는다

픽스처: `fixtures/triggers/context_budget.json`.
워크스루: [`../examples/01_context_budget.md`](../examples/01_context_budget.md).

## 2. 세션 중단 (session interrupt)

호스트 재시작, 연결 끊김, 사용자 중단, 강제 compact, 프로세스 교체.
복구 가능한 마지막 산출을 파일로 고정한다. 기억이 아니라 디스크다.

인정하는 신호:

- 세션이 실제로 끊겼고 후임이 "이어서" 를 요청한다
- outgoing 이 중단 직전 `result.json` 을 쓰지 못했다면 incoming 은
  **추측 재개 금지**. 캡슐·저널·working doc 중 남은 것만 읽는다
- 부분 저널(`handoff.journal.ndjson`)만 있으면 `--verify-journal` 먼저

하지 않는 것:

- 대화 로그를 스크랩해 목표를 재구성
- dirty named worktree 를 reset 해서 "깨끗한 상태로" 위장
- 없는 캡슐을 빈 `workCapsule` 로 날조

픽스처: `fixtures/triggers/session_interrupt.json`.
워크스루: [`../examples/02_session_interrupt.md`](../examples/02_session_interrupt.md).

## 3. 시트 리필 (seat refill)

같은 목표를 **다른 좌석·다른 에이전트 프로세스**에 넘긴다. 권한·비밀·전체
대화·원본 절대경로는 넘기지 않는다. 오케스트레이터의 입력 경계와 같다 —
task `inputs` 에 열거된 파일만 sandbox `inputs/` 사본으로 간다.

인정하는 신호:

- 사용자가 "다른 에이전트에게 이어서" / "시트 리필" 을 말한다
- 같은 isolation 워킹트리에서 후임이 시작된다 (이름 붙은 트리를 넘기지 않는다)
- 후임에게 주는 것은 폴더 경로 하나다: `output/handoff/<taskId>/`

하지 않는 것:

- 원본 작업트리 경로를 후임 `.git` 로 넘긴다
- `C:\Users\swsz9\rhwp` 같은 이름 붙은 트리를 checkout 하게 한다
- 시트만 바꾸고 인계 묶음을 생략한다

픽스처: `fixtures/triggers/seat_refill.json`.
워크스루: [`../examples/03_seat_refill.md`](../examples/03_seat_refill.md).

## 넘기지 않는 때

| 상황 | 보내는 곳 |
|---|---|
| 단건 편집이 끝났고 증명만 필요 | `rhwp-work-receipt` (`rhwp replay` / `audit` / `lineage`) |
| 외부 전문 에이전트에게 task 1건 위임, 같은 세션이 결과를 소비 | `orchestrator.py` 만. 세션 묶음 불필요 |
| 문서 하나를 처음 본다 | `rhwp-doc-triage` / `rhwp-explore` |
| 코어 버그를 고치고 싶다 | 이 스킬 범위 밖. DocumentCore 를 발명하지 말 것 |

## 트리거 필드 (픽스처 계약)

`fixtures/triggers/*.json` 과 `fixtures/catalog.json` 의 `triggers` 는 아래
세 문자열만 쓴다.

```
context_budget
session_interrupt
seat_refill
```

다른 트리거 이름을 발명하지 않는다. 봉투 `_skillMeta.trigger` 도 이 셋이다.
