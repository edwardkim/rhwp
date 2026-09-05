# 11 — 폴더 전수 재현율 1.0

단: 감사.

```bash
rhwp audit fixtures/audit-layouts/all-ok --json
```

기대 키: `total`, `reproduced`, `reproducedRate`, `failed`.
이 레이아웃은 직속 캡슐 3, 재현 3, 비율 1.0, `failed: []`, exit 0.

`notes.txt` 는 세지 않는다.
