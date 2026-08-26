# 픽스처 인덱스

추출 출처는 모두 `mydocs/manual/agent_codex/` 생성 장이다.

## envelopes

| 파일 | 명령 | 장 | exit | untrustedContent |
|---|---|---|---|---|
| `envelopes/info.json` | `info` | 10_조회.md | 0 | True |
| `envelopes/word_count.json` | `word-count` | 10_조회.md | 0 | False |
| `envelopes/bookmarks.json` | `bookmarks` | 10_조회.md | 0 | True |
| `envelopes/headers_footers.json` | `headers-footers` | 10_조회.md | 0 | False |
| `envelopes/header_footer.json` | `header-footer` | 10_조회.md | 0 | False |
| `envelopes/charts.json` | `charts` | 10_조회.md | 0 | False |
| `envelopes/explain.json` | `explain` | 10_조회.md | 0 | True |
| `envelopes/explore.json` | `explore` | 10_조회.md | 0 | False |
| `envelopes/digest.json` | `digest` | 10_조회.md | 0 | True |
| `envelopes/search.json` | `search` | 10_조회.md | 0 | True |
| `envelopes/export_text.json` | `export-text` | 10_조회.md | 0 | True |
| `envelopes/export_structure.json` | `export-structure` | 10_조회.md | 0 | True |
| `envelopes/fields.json` | `fields` | 10_조회.md | 0 | True |
| `envelopes/export_tables.json` | `export-tables` | 20_표와_데이터.md | 0 | True |
| `envelopes/table_to_csv.json` | `table-to-csv` | 20_표와_데이터.md | 0 | True |
| `envelopes/extract_data.json` | `extract-data` | 20_표와_데이터.md | 0 | True |
| `envelopes/chart_to_csv.json` | `chart-to-csv` | 20_표와_데이터.md | 0 | True |
| `envelopes/edit_replace_text.json` | `edit replace-text` | 30_편집과_계획.md | 0 | False |
| `envelopes/edit_set_cell.json` | `edit set-cell` | 30_편집과_계획.md | 0 | True |
| `envelopes/edit_fill_fields.json` | `edit fill-fields` | 30_편집과_계획.md | 0 | False |
| `envelopes/edit_redact.json` | `edit redact` | 30_편집과_계획.md | 0 | False |
| `envelopes/run.json` | `run` | 30_편집과_계획.md | 0 | False |
| `envelopes/convert.json` | `convert` | 40_변환과_렌더.md | 0 | False |
| `envelopes/ir_diff.json` | `ir-diff` | 50_검증_사다리.md | 0 | False |
| `envelopes/replay.json` | `replay` | 50_검증_사다리.md | 0 | False |
| `envelopes/inspect_injection.json` | `inspect injection` | 60_보안.md | 0 | False |
| `envelopes/inspect_hidden_text.json` | `inspect hidden-text` | 60_보안.md | 0 | False |
| `envelopes/inspect_unicode.json` | `inspect unicode` | 60_보안.md | 0 | False |
| `envelopes/export_provenance_map.json` | `export-provenance-map` | 70_자기서술.md | 0 | False |
| `envelopes/export_plan_schema.json` | `export-plan-schema` | 70_자기서술.md | 0 | False |
| `envelopes/export_agent_manifest.json` | `export-agent-manifest` | 70_자기서술.md | 0 | False |

## traces

| ID | 명령 | 장 | keys |
|---|---|---|---|
| T001 | `info` | 10_조회.md | fonts, format, pageCount, paraCount, schemaVersion, sections, sizeBytes, source |
| T002 | `word-count` | 10_조회.md | charCount, pageCount, paragraphCount, schemaVersion, sectionCount, source, untrustedContent, untrustedFields |
| T003 | `bookmarks` | 10_조회.md | bookmarks, count, schemaVersion, source, untrustedContent, untrustedFields |
| T004 | `headers-footers` | 10_조회.md | count, headersFooters, schemaVersion, source, untrustedContent, untrustedFields |
| T005 | `header-footer` | 10_조회.md | applyTo, exists, isHeader, schemaVersion, section, source, untrustedContent, untrustedFields |
| T006 | `charts` | 10_조회.md | charts, count, schemaVersion, source, untrustedContent, untrustedFields |
| T007 | `explain` | 10_조회.md | encrypted, endnoteCount, fields, footnoteCount, format, pageCount, paragraphCount, schemaVersion |
| T008 | `explore` | 10_조회.md | affordanceCount, encrypted, format, menu, note, pageCount, schemaVersion, source |
| T009 | `digest` | 10_조회.md | excerpt, format, nextStep, outline, pageCount, paraCount, schemaVersion, source |
| T010 | `search` | 10_조회.md | caseSensitive, matchCount, matches, omittedCount, query, schemaVersion, source, totalMatchCount |
| T011 | `export-text` | 10_조회.md | omittedCount, pageCount, pages, schemaVersion, source, truncated, untrustedContent, untrustedFields |
| T012 | `export-structure` | 10_조회.md | mode, nodeCount, schemaVersion, source, structure, untrustedContent, untrustedFields |
| T013 | `fields` | 10_조회.md | fieldCount, fields, schemaVersion, source, textSecurity, untrustedContent, untrustedFields |
| T014 | `export-tables` | 20_표와_데이터.md | schemaVersion, source, tableCount, tables, untrustedContent, untrustedFields |
| T015 | `table-to-csv` | 20_표와_데이터.md | bom, output, outputFormat, schemaVersion, source, tableCount, tables, untrustedContent |
| T016 | `extract-data` | 20_표와_데이터.md | counts, itemCount, items, kind, schemaVersion, source, totalItemCount, truncated |
| T017 | `chart-to-csv` | 20_표와_데이터.md | bom, chartCount, charts, schemaVersion, source, untrustedContent, untrustedFields |
| T018 | `edit replace-text` | 30_편집과_계획.md | caseSensitive, changedPages, dryRun, find, occurrence, replace, replacedCount, schemaVersion |
| T019 | `edit set-cell` | 30_편집과_계획.md | changedPages, col, dryRun, keepStyle, newText, oldText, overflow, row |
| T020 | `edit fill-fields` | 30_편집과_계획.md | ambiguous, changedPages, confusable, dryRun, filled, filledCount, notFound, schemaVersion |
| T021 | `edit redact` | 30_편집과_계획.md | changedPages, dryRun, findingCount, findings, inPlace, kinds, mask, noRaw |
| T022 | `run` | 30_편집과_계획.md | assertions, changedPages, input, inputSha256, output, outputFormat, outputSha256, planVersion |
| T023 | `convert` | 40_변환과_렌더.md | bytes, format, output, passwordProtected, schemaVersion, source, untrustedContent, untrustedFields |
| T024 | `ir-diff` | 50_검증_사다리.md | a, b, categories, diffCount, identical, schemaVersion, untrustedContent, untrustedFields |
| T025 | `replay` | 50_검증_사다리.md | expectedOutputSha256, input, inputSha256, mode, outputSha256, planSha256, reproduced, schemaVersion |
| T026 | `inspect injection` | 60_보안.md | clean, highestConfidence, includeFields, injectionSignals, minConfidence, scanScopes, schemaVersion, signalCount |
| T027 | `inspect hidden-text` | 60_보안.md | clean, hiddenCharCount, hiddenText, includeOffPage, schemaVersion, source, thresholdPt, untrustedContent |
| T028 | `inspect unicode` | 60_보안.md | clean, findingCount, findings, kindCounts, kindFilter, scannedChars, schemaVersion, severityCounts |
| T029 | `export-provenance-map` | 70_자기서술.md | commands, envelopeFlags, pathSyntax, policy, schemaVersion, tool, untrustedContent, untrustedFields |
| T030 | `export-plan-schema` | 70_자기서술.md | definitionCount, dialect, planSchemaVersion, schema, schemaVersion, untrustedContent, untrustedFields |
| T031 | `export-agent-manifest` | 70_자기서술.md | capabilities, irSchema, missingAxes, planSchema, provenanceMap, schemaVersion, untrustedContent, untrustedFields |
