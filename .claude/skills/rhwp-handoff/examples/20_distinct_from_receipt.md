# 20 — 단건 증명과 세션 인계를 섞지 않는다

트리거/갈래: `경계`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

`fixtures/catalog.json`:
`receiptSkill: rhwp-work-receipt`,
`receiptIsSingleJobProof: true`,
`sessionHandoffIsNotReceipt: true`.

사용자가 '이 편집 증명해' 라고 하면 이 스킬을 열지 않는다.
이 PR 은 `rhwp-work-receipt/SKILL.md` 를 고치지 않는다.
