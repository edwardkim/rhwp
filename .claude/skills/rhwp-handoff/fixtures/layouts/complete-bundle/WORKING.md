---
kind: working
status: active
issue: 5339
handoffTrigger: context_budget
taskId: t-ord
---

# complete bundle

## 인계 머리
- result: fixtures/layouts/complete-bundle/result.json
- capsule: fixtures/layouts/complete-bundle/session.capsule.json
- parent: none
- journal: fixtures/layouts/complete-bundle/handoff.journal.ndjson

## 남은 목표
후임은 세 파일을 읽고 다음 명령 하나만 실행한다.

## 다음 명령
python tools/handoff/orchestrator.py --verify-journal fixtures/layouts/complete-bundle/handoff.journal.ndjson --json

## 하지 말 것
- DocumentCore 편집 로직 발명 금지
- git add -A 금지
- 이름 붙은 워킹트리 checkout 금지
