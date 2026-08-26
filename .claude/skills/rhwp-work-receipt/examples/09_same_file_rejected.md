# 09 — 부모 덮어쓰기 방지

```bash
rhwp replay --plan-json '<계획>' --capsule a.capsule.json --parent a.capsule.json
```

기대: exit **2**, 부모 파일을 덮어쓰지 않는다.
픽스처: [../fixtures/envelopes/replay_parent_same_file.json](../fixtures/envelopes/replay_parent_same_file.json).

새 작업은 새 파일명이다. `a2.capsule.json --parent a.capsule.json`.
