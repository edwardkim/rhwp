# 06 — 같은 폴더 해시 체인

단: 캡슐. 목표: 다음 작업의 입력이 이전 실산출일 때 `--parent` 로 잇는다.

```bash
rhwp run planA.json --json
rhwp replay --plan-json '<계획A>' --capsule a.capsule.json --json
rhwp replay --plan-json '<계획B: input=O1>' --capsule b.capsule.json --parent a.capsule.json --json
```

같은 폴더면 `parent.capsule` 은 `a.capsule.json` 이다. 저장·해석은 **캡슐 파일
기준**이지 호출 cwd 가 아니다.

16 편 `lineage` 로 `parentOk` 와 `lineageOk` 를 읽는다.
