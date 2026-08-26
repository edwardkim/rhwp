# 07 — s03 → s04 --parent

트리거/갈래: `--parent`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

`fixtures/capsule_index.json` 의 children 항목은
`parentPathRelativeToCapsuleFile: true` 이고
`inputSha256` 이 부모 `outputSha256` 과 같다 (`lineageOk`).

```bash
rhwp replay --plan-json '<s04 계획, input=s03 산출>' \
  --capsule output/handoff/t-ord/session.capsule.json \
  --parent output/handoff/t-ord/parent.capsule.json --json
```

같은 파일을 `--capsule` 과 `--parent` 로 동시에 가리키면 거절.
