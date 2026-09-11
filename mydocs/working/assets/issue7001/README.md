# #7001 Stage 1 정적 목록

기준: `313bd4273dc0cc36a3f3f9b01425797636601b61`. 해설과 판정은 [Stage 1 보고서](../../task_m100_7001_stage1.md).
[원본 JSON](inventory.json)은 228개 Rust 파일, 모듈 선언, 명령의 파일·행, 보조 타깃별 문자 참조 후보를 보존한다.
역할 분류는 Stage 1 잠정 분류이며 삭제·배포 변경 결정이 아니다.

## 재현

저장소 루트에서 `node mydocs/working/assets/issue7001/inventory.mjs`를 실행한다.
표준 출력으로 JSON을 내보내며 파일·제품 코드를 수정하지 않는다. Cargo metadata는 offline/no-deps이다.
분석 대상이 기준 SHA와 달라졌다면 diff 검사로 중단한다. 문서 커밋은 허용된다.
추적 파일만 읽고 generated/submissions·바이너리·2 MiB 초과 텍스트·mydocs 역사 문서는 자동 검색에서 제외한다.
정규식 목록은 해당 기준의 실제 등록부/dispatch 형태에 맞춘 것이며 미래 Rust 문법 전반을 파싱하지 않는다.

문자 참조에는 주석·테스트·실제 호출이 함께 들어간다. 참조 개수는 사용 횟수나 사용자 수가 아니다.
본 CLI의 참조 0은 **수집 생략** 표시다. 본 CLI는 Gym 명령 집계와 보고서의 호출 근거로 별도 확인했다.
Gym JSON에서 run/cmd 배열의 첫 명령만 집계했다. 인자·답안·과제 해답은 복제하지 않았다.
54종은 이 정적 집계의 관측치이며 Python의 동적 생성·MCP 호출까지 포함한 전체 기능 사용 수가 아니다.

## 실행 타깃 26개

required-features는 26개 모두 빈 목록이다. `src/bin`의 24개는 Cargo 자동 탐색 대상이다.
역할: product-cli=제품 CLI, experimental-cli=운영 실험 CLI, developer-diagnostic=개발·진단 도구.

| 타깃 | 진입점 | 잠정 역할 | 외부 파일 문자 참조 수 |
| --- | --- | --- | ---: |
| `font-metric-gen` | `src/tools/font_metric_gen.rs` | developer-diagnostic | 8 |
| `rhwp` | `src/main.rs` | product-cli | 0 |
| `rhwp-agent` | `src/bin/rhwp-agent/main.rs` | experimental-cli | 17 |
| `rhwp-q-char-shape` | `src/bin/rhwp-q-char-shape.rs` | developer-diagnostic | 1 |
| `rhwp-q-control-layout` | `src/bin/rhwp-q-control-layout.rs` | developer-diagnostic | 1 |
| `rhwp-q-cursor-model` | `src/bin/rhwp-q-cursor-model.rs` | developer-diagnostic | 1 |
| `rhwp-q-cursor-rect` | `src/bin/rhwp-q-cursor-rect.rs` | developer-diagnostic | 1 |
| `rhwp-q-font-layout-evidence` | `src/bin/rhwp-q-font-layout-evidence.rs` | developer-diagnostic | 1 |
| `rhwp-q-font-trace` | `src/bin/rhwp-q-font-trace.rs` | developer-diagnostic | 7 |
| `rhwp-q-form-info` | `src/bin/rhwp-q-form-info.rs` | developer-diagnostic | 1 |
| `rhwp-q-hit-test` | `src/bin/rhwp-q-hit-test.rs` | developer-diagnostic | 1 |
| `rhwp-q-kit` | `src/bin/rhwp-q-kit/main.rs` | developer-diagnostic | 5 |
| `rhwp-q-markdown` | `src/bin/rhwp-q-markdown.rs` | developer-diagnostic | 1 |
| `rhwp-q-more` | `src/bin/rhwp-q-more/main.rs` | developer-diagnostic | 1 |
| `rhwp-q-object-cycle` | `src/bin/rhwp-q-object-cycle.rs` | developer-diagnostic | 1 |
| `rhwp-q-objects` | `src/bin/rhwp-q-objects.rs` | developer-diagnostic | 1 |
| `rhwp-q-pack` | `src/bin/rhwp-q-pack/main.rs` | developer-diagnostic | 1 |
| `rhwp-q-page-caret` | `src/bin/rhwp-q-page-caret.rs` | developer-diagnostic | 1 |
| `rhwp-q-page-def` | `src/bin/rhwp-q-page-def.rs` | developer-diagnostic | 1 |
| `rhwp-q-page-images` | `src/bin/rhwp-q-page-images.rs` | developer-diagnostic | 1 |
| `rhwp-q-page-items` | `src/bin/rhwp-q-page-items.rs` | developer-diagnostic | 1 |
| `rhwp-q-para-shape` | `src/bin/rhwp-q-para-shape.rs` | developer-diagnostic | 1 |
| `rhwp-q-scan-items` | `src/bin/rhwp-q-scan-items.rs` | developer-diagnostic | 1 |
| `rhwp-q-section-starts` | `src/bin/rhwp-q-section-starts.rs` | developer-diagnostic | 1 |
| `rhwp-q-text-file` | `src/bin/rhwp-q-text-file.rs` | developer-diagnostic | 1 |
| `rhwp-q-text-layout` | `src/bin/rhwp-q-text-layout.rs` | developer-diagnostic | 4 |

## 최상위 명령 등록부와 dispatch

하위 edit/MCP 도구 목록을 최상위 명령 수에 더하지 않는다. 단일 q 실행 파일 20개의 JSON command 값은 원본 JSON의 singleCommand에 있다.

### mainDispatch — 102

`export-svg`, `export-render-tree`, `export-structure`, `export-png`, `export-png-gpu`, `gpu-info`, `export-pdf`, `export-text`, `export-markdown`, `export-tables`, `export-llm`, `table-to-csv`, `csv-to-table`, `chart-to-csv`, `csv-to-chart`, `export-hwpx`, `export-hml`, `export-doclang`, `export-ir-schema`, `export-capabilities-schema`, `export-ontology`, `capabilities`, `export-provenance-map`, `export-agent-manifest`, `mcp-serve`, `batch`, `scan`, `threat-scan`, `info`, `word-count`, `bookmarks`, `charts`, `form-value`, `header-footer`, `headers-footers`, `digest`, `dump`, `dump-note-shape`, `dump-endnote-lines`, `dump-pages`, `dump-extents`, `diag`, `search`, `inspect`, `armor`, `extract-data`, `convert`, `extract-pages`, `build-from-ingest`, `scaffold`, `hwp5-inventory`, `hwp5-inventory-diff`, `hwp5-contract-analyze`, `hwp5-ctrl-data-trace`, `hwp5-contract-probe`, `hwp5-table-probe`, `hwp5-mel-personnel-probe`, `hwp5-borderfill-diagonal-probe`, `hwp5-first-para-control-probe`, `hwp5-anchor-trace`, `hwp5-char-shape-audit`, `hwp5-cell-header-probe`, `dump-records`, `test-shape`, `test-caption`, `gen-table`, `gen-pua`, `test-field`, `ir-diff`, `ir-sweep`, `dump-anchors`, `dump-carets`, `verify`, `hwpx-roundtrip`, `hwp5-roundtrip`, `render-diff`, `layout-anomaly`, `measure-width`, `core-pages`, `bench`, `thumbnail`, `fields`, `explain`, `explore`, `edit`, `run`, `replay`, `audit`, `lineage`, `keygen`, `verify-signature`, `harness`, `harness-status`, `anchor`, `gate`, `bundle`, `disclose`, `settle`, `audit-report`, `recall-scope`, `conformance`, `export-plan-schema`.

### mainCatalog — 102

`info`, `word-count`, `bookmarks`, `header-footer`, `headers-footers`, `charts`, `form-value`, `export-text`, `export-structure`, `digest`, `export-ir-schema`, `run`, `replay`, `lineage`, `keygen`, `verify-signature`, `harness`, `harness-status`, `anchor`, `gate`, `bundle`, `disclose`, `settle`, `audit-report`, `recall-scope`, `conformance`, `audit`, `export-plan-schema`, `capabilities`, `export-provenance-map`, `export-agent-manifest`, `mcp-serve`, `export-svg`, `export-png`, `export-png-gpu`, `gpu-info`, `export-pdf`, `export-markdown`, `export-hwpx`, `export-hml`, `export-doclang`, `export-capabilities-schema`, `export-ontology`, `export-tables`, `table-to-csv`, `csv-to-table`, `chart-to-csv`, `csv-to-chart`, `extract-pages`, `search`, `extract-data`, `fields`, `explain`, `explore`, `inspect`, `armor`, `export-render-tree`, `convert`, `build-from-ingest`, `scaffold`, `thumbnail`, `edit`, `batch`, `scan`, `threat-scan`, `dump`, `dump-pages`, `dump-extents`, `dump-note-shape`, `dump-endnote-lines`, `dump-records`, `diag`, `ir-diff`, `verify`, `render-diff`, `layout-anomaly`, `hwpx-roundtrip`, `hwp5-roundtrip`, `measure-width`, `core-pages`, `bench`, `hwp5-inventory`, `hwp5-inventory-diff`, `hwp5-contract-analyze`, `hwp5-contract-probe`, `hwp5-ctrl-data-trace`, `hwp5-table-probe`, `hwp5-mel-personnel-probe`, `hwp5-borderfill-diagonal-probe`, `hwp5-first-para-control-probe`, `hwp5-anchor-trace`, `hwp5-char-shape-audit`, `hwp5-cell-header-probe`, `test-shape`, `test-caption`, `test-field`, `gen-table`, `gen-pua`, `export-llm`, `ir-sweep`, `dump-anchors`, `dump-carets`.

### agentRegistry — 80

`capabilities`, `doctor`, `scan`, `fingerprint`, `diff-text`, `verify`, `pii-scan`, `chunk-plan`, `context-cost`, `evidence`, `info`, `format`, `pages`, `page-window`, `empty-pages`, `char-count`, `para-count`, `sample-text`, `outline`, `search`, `search-count`, `contains`, `grep-pages`, `grep`, `compare-pages`, `compare-text`, `field-diff`, `fields`, `field-count`, `tables`, `table-count`, `table-inspect`, `hangul-ratio`, `ascii-ratio`, `line-count`, `unique-chars`, `section-count`, `longest-page`, `shortest-page`, `text-hash`, `hash`, `size`, `magic`, `plan-lint`, `envelope-lint`, `nextcall`, `extract-data`, `field-values`, `table-csv`, `form-ready`, `threat-scan`, `injection-scan`, `hidden-text`, `unicode-scan`, `structure`, `explore`, `explain`, `notes`, `bookmarks`, `charts`, `digest`, `page-hashes`, `empty-fields`, `merged-tables`, `encrypted`, `armor`, `stego-scan`, `sweep`, `outline-nav`, `field-locate`, `captions`, `headers-footers`, `batch-info`, `doc-info`, `page-info`, `section-def`, `field-get`, `page-pos`, `para-page`, `chart-data`.

### kitRegistry — 50

`empty-doc`, `cell-shape`, `overlay-images`, `flow-images`, `footnote-info`, `footnote-hit`, `footnote-footholds`, `note-edit-info`, `cursor-in-cell`, `cursor-in-hf`, `cursor-in-footnote`, `cursor-in-note`, `hit-header`, `hit-in-hf`, `hit-in-footnote`, `hit-fn-marker`, `hf-edit-target`, `fn-selection-rects`, `field-info-at`, `field-by-id`, `field-list-json`, `cur-field-state`, `form-at`, `form-value`, `para-bounds`, `line-starts`, `word-starts`, `word-end`, `caret-stops`, `char-index`, `page-text`, `bin-data`, `source-image-bytes`, `overflow-cells`, `table-overlaps`, `page-border-fill`, `covering-pages`, `chart-csv`, `chart-list`, `search-all`, `canvaskit-preflight`, `layer-tree`, `hyperlinks`, `equations`, `pictures`, `hidden-comments`, `index-marks`, `auto-numbers`, `ruby`, `page-hide`.

### moreDispatch — 51

`volume-probe`, `para-empty`, `para-has-ctrl`, `section-para-lens`, `body-text-len`, `ctrl-per-para`, `table-border-fill`, `table-spacing`, `table-attr`, `picture-border-width`, `picture-opacity`, `picture-href-set`, `equation-baseline`, `equation-color`, `equation-attr`, `form-height`, `form-caption`, `form-fore-color`, `field-properties`, `field-ctrl-id`, `ruby-align`, `ruby-pos`, `pagehide-border`, `pagehide-fill`, `pagehide-master`, `autonumber-super`, `char-overlap-border`, `hyperlink-text-len`, `bookmark-empty-name`, `header-nonempty`, `footer-nonempty`, `footnote-nonempty`, `endnote-nonempty`, `hidden-nonempty`, `shape-height`, `table-zones`, `table-grid-len`, `field-extra-props`, `form-back-color`, `equation-version`, `index-both-keys`, `para-char-count`, `section-ctrl-total`, `caption-para-count`, `enabled-forms-only`, `nonempty-urls`, `nonempty-scripts`, `lock-pictures-only`, `picture-crop-left`, `form-name-len`, `field-command-len`.

### packDispatch — 51

`volume-probe`, `forms-all`, `shapes-all`, `char-overlaps`, `headers-list`, `footers-list`, `footnotes-list`, `endnotes-list`, `new-numbers`, `page-num-ctrls`, `page-number-pos`, `column-defs`, `unknown-ctrls`, `tables-model`, `field-ctrls`, `bookmark-names`, `treat-as-char`, `logical-inline`, `picture-crops`, `equation-scripts`, `form-types`, `hyperlink-hosts`, `ruby-mains`, `pagehide-headers`, `autonumber-nums`, `index-second-keys`, `hidden-comment-len`, `table-rows`, `table-cells`, `shape-sizes`, `header-paras`, `footer-paras`, `footnote-paras`, `endnote-paras`, `picture-locks`, `picture-reverse`, `equation-fonts`, `form-enabled`, `field-commands`, `field-ids`, `form-sizes`, `section-defs`, `caption-tables`, `ctrl-kinds`, `page-starts-on`, `hidden-comment-count`, `ruby-ratio`, `char-overlap-len`, `table-cols`, `picture-instance`, `index-first-keys`.


## Gym 관계

추적된 pack JSON 2,092개에서 명령 54종이 관측되었고 전부 본 CLI dispatch에 있다.
명령별 출현 수·파일 수는 원본 JSON의 gym.commandHeads를 본다. 이는 실행 성공 증거가 아니다.
보조 실행 파일 및 DSEL에 대한 Gym 직접 문자 참조는 해당 검색 범위에서 0이다.
