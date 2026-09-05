# 예제 11 — 암호 문서 분리

```bash
# 금지
rhwp batch info --password secret --json < 목록.txt   # exit 2

# 맞음
rhwp batch info --json < 목록.txt > meta.ndjson
jq -r 'select(.error) | .source' meta.ndjson > maybe-secret.txt
# 암호 신호를 사람이/상류가 확인한 뒤
rhwp info 보호.hwp --password-stdin --json < password.txt
# 평문 나머지만 다시 batch
jq -r 'select(.error|not) | .source' meta.ndjson > plain.txt
rhwp batch export-text --json < plain.txt
```

전사 T07 은 빈 stdout. `fixtures/password_reject.json`.

이슈 #5311. gym 아님. 새 CLI 아님.
