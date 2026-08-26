# 18 — 저널 체인을 다시 계산한다

트리거/갈래: `journal`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

```bash
python tools/handoff/orchestrator.py \
  --verify-journal output/handoff/t-ord/handoff.journal.ndjson --json
```

ok 표본: `fixtures/journals/ok.ndjson` (entries 2, chainValid).
위조 표본: `fixtures/journals/tampered.ndjson` (brokenAt 2, exit 3).
이 봉투는 last result 가 아니다 (25).
