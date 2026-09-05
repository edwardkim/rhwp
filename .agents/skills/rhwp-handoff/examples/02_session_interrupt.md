# 02 — 끊긴 세션은 세 파일부터

트리거/갈래: `session_interrupt`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

후임 첫 턴:

1. `output/handoff/t-ord/result.json`
2. `output/handoff/t-ord/session.capsule.json`
3. `mydocs/working/agent_handoff.md`

`outcome == accepted` 이면 수거물 해시 대조 후 working doc 의
다음 명령 하나만. 대화 요약을 스크랩하지 않는다.

세 파일 중 캡슐이 없으면 08. 부모 해시가 다르면 09.
픽스처: `fixtures/triggers/session_interrupt.json`,
`fixtures/incoming/read-order.json`.
