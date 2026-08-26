# 10 — 실산출은 `run`, 증명은 `replay`

단: 캡슐. `replay` 는 임시 산출만 해시한 뒤 지운다. 다음 계획의 `input` 이
파일이어야 하면 먼저 `run` 한다.

```bash
rhwp run planA.json --json          # O1 생성, outputSha256 저널
rhwp replay --plan-json '<계획A>' --capsule a.capsule.json --json
# 계획B.input = O1
rhwp replay --plan-json '<계획B>' --capsule b.capsule.json --parent a.capsule.json --json
rhwp lineage b.capsule.json --json
```

`lineageOk` 는 **부모 산출 해시 == 자식 입력 해시** 다. run↔replay 교차
결정론이 이 등식을 받친다 (`tests/lineage_contract.rs`).
