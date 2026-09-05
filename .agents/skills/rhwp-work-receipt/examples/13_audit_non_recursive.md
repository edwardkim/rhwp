# 13 — 비재귀 `*.capsule.json`

```bash
rhwp audit fixtures/audit-layouts/nested-ignored --json
```

직속 `top.capsule.json` 만 센다. `nested/hidden.capsule.json` 은 없다.
`total: 1`.

하위 폴더를 감사하려면 그 경로로 다시 `audit` 한다. 재귀 플래그는 없다.
이 스킬이 `--recursive` 를 발명하지 않는다.
