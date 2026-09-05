# 예제 — plan --text-only 전수

이슈 #5329. 실 에이전트 경로. gym 아님.

## 명령

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-plan
```

## 산출

- `text-report.tsv` 35행
- `report.tsv` 의 diff% 는 `not-run`
- 시트 PNG 없음
- `page-count-ledger.tsv`, layout 원장

확정 문장 금지. 상위 소실 쪽만 나중에 시트.

관련: `references/06_text_report.md`.
전사: `fixtures/transcripts/plan_text_only.txt`.
정지 F02.
