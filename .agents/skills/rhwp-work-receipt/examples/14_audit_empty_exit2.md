# 14 — 빈 폴더는 사용법

```bash
rhwp audit fixtures/audit-layouts/empty --json
```

exit **2**, stdout 0바이트. 봉투의 `total: 0` 이 아니다 — 봉투 자체가 없다.

대상을 만들고 다시 부른다. 같은 빈 폴더를 루프로 재시도하지 않는다.
