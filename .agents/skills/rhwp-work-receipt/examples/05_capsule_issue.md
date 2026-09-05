# 05 — 작업 캡슐 발급

단: 캡슐. 목표: 계획+영수증을 자기완결 파일로 남긴다.

권위: [capsule-chain.md](../references/capsule-chain.md).

```bash
rhwp replay --plan-json '<계획A>' --capsule a.capsule.json --json
```

파일 골격:

- `kind: "workCapsule"`
- `parent: null` (뿌리)
- `plan` / `planText` — 객체와 원문이 같아야 한다
- `receipt` — 01 편의 봉투

제3자는 이 파일만 받으면 `audit` 한 폴더 또는 `lineage` 한 머리로 재현할 수 있다.
`plan.output` 은 발급 당시 사용자 경로를 보존한다. 재실행은 임시 경로로 덮어쓴다.

발급 후 파일을 열어서 저장하지 마라. 08 편.
