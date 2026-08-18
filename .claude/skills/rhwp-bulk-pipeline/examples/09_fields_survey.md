# 예제 09 — 서식 일괄 조사

```bash
rhwp batch fields --json < examples/lists/fields_pair.txt \
  | jq -c '{source, fieldCount}'
```

field-01 은 11, hwp3-sample 은 0. 0 을 실패로 세지 않는다.
누름틀 있는 파일만 채움 후보로 `rhwp-form-fill` 또는 `batch fill` 에 넘긴다.
전사 `T09.ndjson`.

이슈 #5311. gym 아님. 새 CLI 아님.
