# 27 — jq 게이트 레시피

## Q01 — 실패 경로

```bash
jq -r 'select(.error) | .source'
```

## Q02 — 성공 경로+쪽

```bash
jq -r 'select(.error|not) | "\(.source)\t\(.pageCount)쪽"'
```

## Q03 — 성공 수

```bash
jq -s '[.[]|select(.error|not)]|length'
```

## Q04 — 실패 수

```bash
jq -s '[.[]|select(.error)]|length'
```

## Q05 — 검색 히트만

```bash
jq -c 'select(.matchCount > 0) | {source, pages:[.matches[].page]}'
```

## Q06 — 서식만

```bash
jq -c 'select(.fieldCount>0) | {source, fieldCount}'
```

## Q07 — 표만

```bash
jq -c 'select(.tableCount>0) | {source, tableCount}'
```

## Q08 — 10쪽 이상

```bash
jq -r 'select(.pageCount >= 10) | .source'
```

## Q09 — 절단된 extract

```bash
jq -c 'select(.truncated==true) | {source, itemCount, totalItemCount}'
```

## Q10 — fill 실패 행

```bash
jq -c 'select((.notFound|length>0) or (.ambiguous|length>0) or .error)'
```

## Q11 — verify 불일치

```bash
jq -c 'select(.verify != null and .verify.identical==false)'
```

## Q12 — exitClass

```bash
jq -r 'select(.error) | [.source, .exitClass, .error] | @tsv'
```

원본: `fixtures/jq_recipes.json`.
## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `27_gate_recipes.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
