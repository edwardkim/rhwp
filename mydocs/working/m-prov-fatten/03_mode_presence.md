# 모드 존재표 — 같은 명령도 표지가 갈린다

`untrustedFields` 는 선언 목록을 베끼지 않는다. 그 봉투에 실제로 값이 실린 경로만 남긴다.

| 명령 | 모드 | untrustedContent | 경로 |
| --- | --- | --- | --- |
| `info` | `default` | true | `title`, `fonts[]` |
| `info` | `empty-title` | true | `fonts[]` |
| `word-count` | `default` | false | ∅ |
| `bookmarks` | `named` | true | `bookmarks[].name` |
| `bookmarks` | `empty` | false | ∅ |
| `form-value` | `present` | true | `name`, `value`, `text`, `caption` |
| `form-value` | `missing` | false | ∅ |
| `charts` | `default` | false | ∅ |
| `headers-footers` | `default` | false | ∅ |
| `header-footer` | `with-text` | true | `text` |
| `header-footer` | `absent` | false | ∅ |
| `export-text` | `pages` | true | `pages[].text` |
| `export-text` | `batch-record` | true | `text` |
| `export-structure` | `outline` | true | `structure.preamble[]`, `structure.roots[].heading`, `structure.roots[].marker`, `structure.roots[].body[]`, `structure.roots[].children[]` |
| `export-structure` | `empty-body` | false | ∅ |
| `digest` | `default` | true | `outline[]`, `excerpt` |
| `digest` | `sections` | true | `outline[]`, `excerpt`, `sections[].heading`, `sections[].excerpt` |
| `digest` | `pages` | true | `excerpt` |
| `search` | `hits` | true | `matches[].text`, `matches[].context` |
| `search` | `no-hits` | false | ∅ |
| `extract-data` | `raw-hits` | true | `items[].raw`, `items[].unit` |
| `extract-data` | `none` | false | ∅ |
| `fields` | `with-fields` | true | `fields[].name`, `fields[].guide`, `fields[].memo`, `fields[].command`, `fields[].value` |
| `fields` | `confusable-names` | true | `fields[].name`, `textSecurity.findings[].names[]` |
| `explain` | `with-fields` | true | `fields[]`, `summary` |
| `explain` | `no-fields` | true | `summary` |
| `explore` | `default` | false | ∅ |
| `export-tables` | `cells` | true | `tables[].caption`, `tables[].cells[].text`, `tables[].cells[].nested[]` |
| `export-tables` | `empty` | false | ∅ |
| `table-to-csv` | `csv` | true | `tables[].csv` |
| `table-to-csv` | `file-only` | false | ∅ |
| `csv-to-table` | `changed` | true | `changed[].oldText` |
| `csv-to-table` | `dry-run` | true | `changed[].oldText` |
| `chart-to-csv` | `csv` | true | `charts[].csv` |
| `csv-to-chart` | `changed` | true | `changed[].from` |
| `dump-pages` | `preview` | true | `pages[].columns[].items[].textPreview` |
| `inspect` | `hidden-text` | true | `hiddenText[].excerpt` |
| `inspect` | `injection` | true | `injectionSignals[].excerpt`, `injectionSignals[].matched` |
| `inspect` | `unicode` | true | `findings[].excerpt`, `findings[].rendered`, `findings[].raw`, `findings[].hidden` |
| `inspect` | `clean` | false | ∅ |
| `armor` | `fenced` | true | `armoredText`, `injectionSignals[].excerpt`, `injectionSignals[].matched` |
| `edit` | `set-cell` | true | `oldText` |
| `edit` | `fill-fields` | true | `confusable[].lookalikes` |
| `edit` | `redact` | true | `findings[].raw`, `findings[].masked` |
| `edit` | `sanitize` | true | `removed[].before` |
| `edit` | `replace-text` | false | ∅ |
| `run` | `set-cell` | true | `steps[].oldText` |
| `run` | `fill-fields` | true | `steps[].confusable[].lookalikes` |
| `run` | `dry-run` | false | ∅ |
| `replay` | `receipt` | false | ∅ |
| `audit` | `default` | false | ∅ |
| `lineage` | `default` | false | ∅ |
| `keygen` | `default` | false | ∅ |
| `verify-signature` | `default` | false | ∅ |
| `harness` | `default` | false | ∅ |
| `harness-status` | `default` | false | ∅ |
| `anchor` | `default` | false | ∅ |
| `gate` | `default` | false | ∅ |
| `bundle` | `default` | false | ∅ |
| `disclose` | `default` | false | ∅ |
| `settle` | `default` | false | ∅ |
| `audit-report` | `default` | false | ∅ |
| `recall-scope` | `default` | false | ∅ |
| `conformance` | `default` | false | ∅ |
| `ir-diff` | `with-diff` | true | `categories` |
| `ir-diff` | `identical` | false | ∅ |
| `verify` | `field` | true | `expectations[].actual` |
| `verify` | `contains` | true | `expectations[].actual` |
| `render-diff` | `default` | false | ∅ |
| `layout-anomaly` | `default` | false | ∅ |
| `thumbnail` | `embedded` | true | `base64`, `dataUri` |
| `thumbnail` | `file-only` | false | ∅ |
| `batch` | `export-text` | true | `text` |
| `batch` | `info` | true | `title`, `fonts[]` |
| `batch` | `search` | true | `matches[].text`, `matches[].context` |
| `scan` | `probe-error` | true | `files[].probe.error` |
| `scan` | `list-only` | false | ∅ |
| `threat-scan` | `remote` | true | `findings[].detail` |
| `threat-scan` | `macro-only` | false | ∅ |
| `export-svg` | `manifest` | false | ∅ |
| `export-pdf` | `manifest` | false | ∅ |
| `export-markdown` | `manifest` | false | ∅ |
| `export-hwpx` | `manifest` | false | ∅ |
| `export-hml` | `manifest` | false | ∅ |
| `export-doclang` | `manifest` | false | ∅ |
| `extract-pages` | `manifest` | false | ∅ |
| `convert` | `manifest` | false | ∅ |
| `build-from-ingest` | `manifest` | false | ∅ |
| `scaffold` | `manifest` | false | ∅ |
| `capabilities` | `default` | false | ∅ |
| `export-ir-schema` | `default` | false | ∅ |
| `export-capabilities-schema` | `default` | false | ∅ |
| `export-provenance-map` | `default` | false | ∅ |
| `export-ontology` | `default` | false | ∅ |
| `export-agent-manifest` | `default` | false | ∅ |
| `export-plan-schema` | `default` | false | ∅ |

## 읽을 때

- dry-run / -o / 0건 / exists=false 는 같은 명령의 다른 부분집합이다.
- 선언 목록을 표지에 그대로 복사하면 있지도 않은 필드를 광고하게 된다.
- 키 부재는 이 표의 false 가 아니라 미표기다.
