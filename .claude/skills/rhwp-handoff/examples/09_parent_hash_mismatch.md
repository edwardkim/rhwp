# 09 — 부모 해시가 다르면 --parent 를 더 붙이지 않는다

트리거/갈래: `parent_hash_mismatch`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

레이아웃: `fixtures/layouts/parent-mismatch/`.
자식은 `fixtures/capsules/tamper_parent_sha.capsule.json`.

에디터로 해시를 맞추지 않는다. 새 뿌리를 쓸 수는 있다.
단건 계보는 work-receipt `rhwp lineage` 로 보낸다.
표본 exit 3.
