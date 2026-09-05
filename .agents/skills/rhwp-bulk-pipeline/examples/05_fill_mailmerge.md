# 예제 05 — 메일머지 (form+data)

```bash
rhwp batch fill --form samples/form-01.hwp \
  --data examples/data/mailmerge_3.jsonl \
  --out-dir output/filled --name-field 성명 --json
```

stdin 에 `lists/recipe9.txt` 를 넣지 않는다. 데이터는 `data/mailmerge_3.jsonl`
또는 `data/mailmerge_3.csv`. 빈 헤더는 `data/empty_header_only.csv` → exit 2.

dry-run:

```bash
rhwp batch fill --form samples/form-01.hwp \
  --data examples/data/mailmerge_3.jsonl \
  --out-dir output/filled --dry-run --json
```

`--out-dir` 를 빼지 않는다. 전사 `T12.ndjson`(dry-run), `T13.ndjson`(실행).

이슈 #5311. gym 아님. 새 CLI 아님.
