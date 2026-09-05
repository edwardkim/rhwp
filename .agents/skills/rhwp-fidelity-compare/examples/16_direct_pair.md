# 예제 — REG 없는 쌍

이슈 #5329. 실 에이전트 경로. gym 아님.

## 명령

```bash
RHWP_BIN=target/release-test/rhwp \
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 214 \
  --source samples/input.hwp \
  --reference-pdf pdf/oracle-2020.pdf \
  --label issue-3738-hwp \
  --reference-grade '한컴 2020 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-issue-3738
```

세 플래그 중 하나라도 빠지면 사용법 오류. positional 은 두 개.
`--source` 만 주고 키를 `0` 으로 오인하지 말 것.

관련: `references/19_direct_pair.md`.
전사: `fixtures/transcripts/direct_pair_text.txt`.
정지 F02.
