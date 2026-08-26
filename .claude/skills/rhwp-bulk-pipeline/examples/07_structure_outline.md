# 예제 07 — 개요/조문 일괄

```bash
rhwp batch export-structure --json --mode outline < examples/lists/ok4.txt
rhwp batch export-structure --json --mode clause  < examples/lists/ok4.txt
```

`--mode chapters` 는 exit 2 (B06). 기본값은 `auto`.
전사 `T11.ndjson`.

이슈 #5311. gym 아님. 새 CLI 아님.
