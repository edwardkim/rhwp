# 예제 06 — 아카이브 전역 검색

```bash
rhwp batch search --query "의" --json < examples/lists/search_pair.txt \
  | jq -c '{source, matchCount}'
```

`--query` 없이 치면 exit 2, 전사 `T06.ndjson` 빈 파일.
히트 문서만 본문:

```bash
jq -r 'select(.matchCount>0) | .source' hits.ndjson | sort -u > hits.txt
rhwp batch export-text --json < hits.txt > hits-text.ndjson
```

이슈 #5311. gym 아님. 새 CLI 아님.
