# 봉투 필드 카탈로그

권위 스키마는 cli_commands.md 와 tests/cli_json_contract.rs 다.
필드를 지어내지 않는다.

## export-svg --json

`schemaVersion, source, format=svg, outputDir, pageCount, renderedCount, overflowCellLines, pages[{page,path,bytes,overflowCellLines}]`

## export-pdf --json

`schemaVersion, source, format=pdf, backend, output, bytes, pageCount, renderedCount`

## export-text --json

`schemaVersion, source, pageCount, truncated, omittedCount, pages[{page,text,truncated?,omittedCount?}]`

## export-markdown --json

`schemaVersion, source, format=markdown, outputDir, pageCount, renderedCount, imageCount, pages[{page,path,bytes}]`

## info --json

`schemaVersion, source, format, sizeBytes, version, sections, pageCount, paraCount, fonts`

## ir-diff --json

`schemaVersion, a, b, identical, diffCount, categories`

## 실패

stdout 없음. 필드 카탈로그를 적용하지 않는다.
