# tools/work_receipt

MEGA QUEUE M-rcpt (#5478). 기존 `replay` / `audit` / `lineage` CLI 의
픽스처·예외 봉투·작업 문서다. 새 명령은 없다.

```
python tools/work_receipt/fatten_work_receipt.py
python tools/work_receipt/test_fatten_work_receipt.py
```

- `contracts.py` — exit·바늘·검증 함수 (main.rs 대응)
- `catalog.py` — 한국 공공문서 시나리오·예외·레이아웃·토폴로지
- `fixtures/` — 생성된 정본
- `docs/` · `WORKING.md` — 사람용
- `schema/` — JSON 스키마
