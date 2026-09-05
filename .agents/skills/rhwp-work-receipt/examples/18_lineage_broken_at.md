# 18 — `brokenAt` 명세

부모를 포맷터로 저장한 뒤:

```bash
rhwp lineage b.capsule.json --json
```

- exit 3
- `valid: false`
- `brokenAt` 이 깨진 캡슐 경로
- 해당 링크 `parentOk: false` 또는 `lineageOk: false` 또는 `error`

머리 캡슐이 없으면 exit **1** (IO). 인자가 없으면 exit **2**.
누락 `parent.sha256` 은 생략이 아니라 fail-closed (exit 3).
