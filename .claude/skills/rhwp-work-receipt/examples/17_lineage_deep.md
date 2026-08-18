# 17 — `--deep` 재실행

```bash
rhwp lineage b.capsule.json --deep --json
```

각 링크의 `reproduced` 가 true/false 로 채워진다. 비용은 링크 수다.
한 링크가 false 면 `valid: false`, `brokenAt` 이 그 캡슐, exit 3.

얕은 lineage 가 이미 깨졌으면 deep 을 돌릴 필요가 없다.
