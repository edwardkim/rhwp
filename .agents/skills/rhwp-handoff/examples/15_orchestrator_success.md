# 15 — 수용 봉투를  sav고 소비한다

트리거/갈래: `accepted`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

표본: `fixtures/envelopes/orch_accepted.json`.
`status=ok`, `outcome=accepted`, `code=0`,
`nextAction.action=consume`, `untrustedContent=true`.

outgoing 은 stdout 을 `result.json` 으로 저장한다.
도구가 그 파일을 자동 생성하지는 않는다.
