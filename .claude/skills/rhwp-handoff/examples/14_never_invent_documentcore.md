# 14 — 세션이 바뀌어도 코어를 고치지 않는다

트리거/갈래: `no-documentcore`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

다음 명령이 `DocumentCore::apply_*` 이거나
`src/document_core/` 패치면 거부.
편집은 `rhwp edit` / `rhwp run` 만.
표본: `fixtures/envelopes/documentcore_invented.json`.
