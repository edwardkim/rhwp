# 12 — 재현율 회계 (실패 포함)

```bash
rhwp audit fixtures/audit-layouts/mixed --json
```

- `total: 3`
- `reproduced: 2`
- `reproducedRate: 0.666…` (= 2/3)
- `failed[0].capsule == "tampered.capsule.json"`
- exit **3**

회계는 봉투로 읽고, 실패 캡슐만 01·02 편의 verify 로 추적한다.
한 건의 실패가 나머지 성공을 지우지 않는다 — `reproduced` 가 그 숫자다.
