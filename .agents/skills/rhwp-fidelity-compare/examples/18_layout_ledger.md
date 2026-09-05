# 예제 — --layout-ledger

이슈 #5329. 실 에이전트 경로. gym 아님.

## 명령

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-plan
ls /tmp/rhwp-fidelity-plan/*candidates.tsv
```

`layout-candidates.tsv`, `table-fragment-candidates.tsv`,
border-clip, cell-overlap, float-owner-shift 가 생긴다.
행은 전부 candidate. PDF 시트 없이 결함 확정 금지.

관련: `references/20_outputs.md`.
정지 F02.
