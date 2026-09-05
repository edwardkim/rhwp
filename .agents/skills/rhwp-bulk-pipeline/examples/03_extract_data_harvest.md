# 예제 03 — 날짜·금액 수확

```bash
rhwp batch extract-data --json --limit 3 < examples/lists/recipe9.txt > /tmp/data.ndjson
jq -c 'select(.error|not) | {source, itemCount, totalItemCount, truncated, counts}' /tmp/data.ndjson
```

국립국어원 업무계획은 `totalItemCount: 297`, `itemCount: 3`, `truncated: true`.
`counts` 는 297을 가리킨다. limit 을 배치 전체 상한으로 읽지 말 것.

같은 파일을 stdin 에 두 번 넣으면 두 레코드가 각각 3건이다
(`batch_extract_data_contract`). 전사 `T03.ndjson`.

이슈 #5311. gym 아님. 새 CLI 아님.
