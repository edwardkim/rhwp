# 16 — 두 링크 연대기

```bash
rhwp lineage b.capsule.json --json
```

링크 판정 3축:

| 축 | 물음 |
|----|------|
| `parentOk` | 부모 파일이 발급 당시 바이트인가 (`parent.sha256`) |
| `lineageOk` | 부모 `outputSha256` == 자식 `inputSha256` |
| `reproduced` | `--deep` 일 때만. 아니면 `null` |

둘 다 true, `valid: true`, `brokenAt: null`, exit 0.
