# 01 — 컨텍스트 예산이 보이면 인계를 닫는다

트리거/갈래: `context_budget`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

호스트가 compact 를 예고했거나 같은 조례 파일을 세 번 다시 읽었다.
새 탐색을 시작하지 않는다.

```bash
python tools/handoff/orchestrator.py \
  --task output/handoff/t-ord/task.json \
  --agent "python worker.py" \
  --work-dir output/handoff/t-ord --json \
  > output/handoff/t-ord/result.json
python tools/handoff/orchestrator.py \
  --verify-journal output/handoff/t-ord/handoff.journal.ndjson --json
rhwp replay --plan-json '<이번 세션 계획>' \
  --capsule output/handoff/t-ord/session.capsule.json \
  --parent output/handoff/t-ord/parent.capsule.json --json
```

working doc 의 `handoffTrigger` 는 `context_budget`.
픽스처: `fixtures/triggers/context_budget.json`,
`fixtures/transcripts/T01_context_budget.json`.

실패: 예산을 넘긴 채 별표까지 '조금만 더' 보다가 세션이 죽으면
`result.json` 이 없다. 그때는 02 가 아니라 08 에 가깝다.
