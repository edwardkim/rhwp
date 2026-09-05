# 23 — 축별 봉투

성공 레코드는 단건 `--json` 과 동형. 실패는 공통.

## 실패 (모든 축)

```json
{
  "error": "문서를 열 수 없습니다: 지정된 파일을 찾을 수 없습니다. (os error 2)",
  "exitClass": "runtime",
  "schemaVersion": "1.0",
  "source": "samples/없는파일.hwp",
  "untrustedContent": false,
  "untrustedFields": []
}
```

필수: schemaVersion, source, error, exitClass. `exitClass` = `runtime`.

## 성공 키

| 축 | 키 | 단건 동형 |
| --- | --- | --- |
| `info` | `schemaVersion`, `source`, `format`, `pageCount` | info --json |
| `export-text` | `schemaVersion`, `source`, `pageCount`, `text` | export-text --json 의 문서 단위 축약(pages[] 대신 text) |
| `export-structure` | `schemaVersion`, `source`, `mode` | export-structure --json |
| `export-tables` | `schemaVersion`, `source`, `tableCount`, `tables` | export-tables --json |
| `fields` | `schemaVersion`, `source`, `fieldCount`, `fields` | fields --json |
| `search` | `schemaVersion`, `source`, `query`, `matchCount`, `matches` | search --json |
| `extract-data` | `schemaVersion`, `source`, `kind`, `itemCount`, `totalItemCount`, `truncated`, `counts`, `items` | extract-data --json |
| `convert` | `schemaVersion`, `source`, `format`, `output`, `bytes` | convert --json |
| `fill` | `schemaVersion`, `source`, `row`, `dryRun`, `filledCount`, `filled`, `notFound`, `ambiguous` | edit fill-fields --json + row |

fill 추가 키: `row`.
스키마 필드 추가만 허용. 삭제·변경은 `tests/cli_json_contract.rs`.
## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `23_envelopes.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
