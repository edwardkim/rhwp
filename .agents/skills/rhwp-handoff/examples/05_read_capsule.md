# 05 — 세션 캡슐 읽기

트리거/갈래: `incoming`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

표본: `fixtures/capsules/s03.capsule.json`.

확인:

- `kind` == `workCapsule`
- `receipt.planSha256` == SHA-256(`planText`)
- `plan` == JSON.parse(`planText`)
- `parent` 가 있으면 상대 경로를 **이 파일 기준**으로 푼다

단건 재현이 필요하면 work-receipt 로 보낸다. 여기서
`reproducedRate` 를 계산하지 않는다.
