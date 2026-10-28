# 세션 핸드오프용 working doc

정본 위치는 `mydocs/working/` 이다. 세션 스킬이 새 문서 종류를 만들지 않는다.
이 파일이 요구하는 것은 **칸**이다. 예시는
[`mydocs/working/agent_handoff.md`](../../../../mydocs/working/archives/agent_handoff.md).

## 최소 칸

```markdown
---
kind: working
status: active
issue: 5339
handoffTrigger: context_budget
taskId: t-session-03
---

# <제목> (#이슈)

## 인계 머리
- result: output/handoff/<taskId>/result.json
- capsule: output/handoff/<taskId>/session.capsule.json
- parent: output/handoff/<taskId>/parent.capsule.json  (없으면 none)
- journal: output/handoff/<taskId>/handoff.journal.ndjson

## 남은 목표
한 줄.

## 다음 명령
기존 CLI 또는 `python tools/handoff/orchestrator.py …` 한 줄.

## 하지 말 것
- DocumentCore 편집 로직 발명 금지
- git add -A 금지
- 이름 붙은 워킹트리 checkout 금지
```

`handoffTrigger` 는 `context_budget` / `session_interrupt` / `seat_refill` 만.

## incoming 이 칸을 읽을 때

- 경로가 상대면 저장소 루트 기준
- 파일이 없으면 그 칸은 예외 갈래로 간다 (추측 경로를 만들지 않는다)
- "다음 명령"이 `rhwp handoff` 이면 발명이다. 거부
- "다음 명령"이 `git checkout C:\Users\swsz9\rhwp` 이면 거부

## 갱신

outgoing 이 인계를 닫을 때마다 이 칸을 **덮어쓴다**. 히스토리는 git 이 가진다.
세션마다 새 파일을 만들 수도 있다 (`agent_handoff_s03.md`). 후임에게는
머리 파일 하나만 가리킨다.
