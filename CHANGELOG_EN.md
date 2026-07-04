# Changelog

This document records the major changes of the rhwp project.

> 한국어 버전은 [CHANGELOG.md](CHANGELOG.md) 를 참조하세요.

## [0.7.17] — 2026-06-23

> Patch following v0.7.16 — first OOXML chart render-fidelity work, legacy-shape shapeComment
> serialization, WASM options-object APIs, many rhwp-studio table/picture/cursor editing fixes,
> render-authority hardening, and a dependency bump batch. Public APIs remain backward-compatible
> (positional APIs kept) — PATCH. Browser extension 0.2.6 included.

### API
- Added options-object variants `*Ex(options_json[, image_data])` for 26 high-arity (7+) WASM
  public APIs (#1413). Existing positional APIs are kept (backward-compatible); `*Ex` performs
  the same operation via JSON options, making downstream less fragile to mid-signature changes.
  Consumer (@rhwp/core) README guidance + consumer edit API manual added (#1445).
  Convention: `mydocs/manual/wasm_api_options_convention.md`.

### Rendering (charts)
- Routed 7 of the 27 OOXML chart types whose data was already extracted (4 3D-bar, 1 3D-pie,
  2 ofPie) to 2D-approximation rendering — removes the "chart (unsupported)" placeholder
  (#1453, C1a / #1431 Track C).
- Bar charts now honor `c:grouping` (stacked/percentStacked) — 6 stacked/percent bar types (#1453).

### HWPX save contract (serializer fidelity)
- Fixed missing `hp:shapeComment` serialization on legacy shapes (ellipse/arc/polygon/curve/
  chart/ole) routed through `render_common_shape_xml` — round-trip preserved (#1451).
- Excluded false `ir-diff` differences on tab_extended reserved fields [3,4,5] (#1473).

### Rendering
- Keep v2 font authority on fallback, expand CanvasKit replay contract guards (#1429/#1447/#1469).
- TAC picture vs text vertical alignment in table cells (#1352). Reflow on un-inline picture (#1459).

### rhwp-studio
- Autosave + recovery UI for unsaved documents (#1448). Local-font detection consent (opt-in) (#1328).
- Page-border preview toggle restore (#1426). Picture insert / inline cursor fidelity (#1452).
- Table row/column insert/delete regression fix (preserve table height when adding rows/cols right
  after creation) + Hancom-style unified dialog/shortcuts (#1481).
- Table-cell drag selection + Hancom-compatible table editing (#1443), cell protection (#493).
- Block manipulation of size-locked objects (#1436), inline-picture wrap + paragraph border (#1440).
- Style apply / table caption / format copy (#1470), platform-specific menu shortcut display (#1476).

### Browser extension (0.2.6)
- Fixed viewer inline-script CSP violation (theme-init.js split) + dark icon asset recovery (#1444).
- Removed the global side effect of the Chrome `onDeterminingFilename` listener — switched to
  `onCreated`/`onChanged` observers so other extensions' `download({filename})` subfolder saves
  are no longer disrupted (#1471).

### Infra
- Track `Cargo.lock` in git — reproducible builds + stable CI cache keys (#1423, macOS FFI excluded).
- Dependency bump batch: zip 8.6.0, serde_json 1.0.150, snafu 0.9.1, subsetter 0.2.6,
  skia-safe 0.99.0, unicode-segmentation 1.13.3, wasm-bindgen-test 0.3.75, @types/chrome 0.2.0
  (#1461–#1468).

### Contributors

Contributor PRs merged in this cycle (since v0.7.16; GitHub handles, alphabetical):

- @jangster77 (Taesup Jang) — table row/column insert-delete regression (#1481), table-cell drag
  selection & editing (#1443) / cell protection (#493), TAC picture alignment / size-lock /
  inline-wrap (#1352/#1436/#1440), autosave recovery (#1448), picture/cursor fidelity (#1452/#1459),
  style/caption/shortcut (#1470/#1476)
- @johndoekim — OOXML chart C1a routing + bar stacking (#1453)
- @oksure (Hyunwoo Park) — exclude false ir-diff on tab_extended reserved fields (#1473)
- @postmelee — local-font consent (#1328), page-border toggle (#1426), Chrome download
  interceptor side-effect fix (#1471), PR review workflow docs (#1425)
- @seo-rii — keep v2 font authority (#1429), expand CanvasKit replay guards (#1447/#1469)

## [0.7.16] — 2026-06-19

> Patch following v0.7.15 — refines the HWPX save contract (serializer fidelity), fixes
> ClickHere guide-text binding in the Hancom editor, adds a drag-and-drop security gate to
> rhwp-studio, and lands many rendering/table/picture fixes plus external contributor PRs.
> Public APIs remain backward-compatible — PATCH.

### HWPX save contract (serializer fidelity)
- Preserve controls inside cell/text-box subLists, original linesegs, and table/picture/group captions (#1379/#1380/#1387/#1403).
- Emit secPr page margins and body column (colPr) definitions from the IR instead of template hardcoding (#1388/#1407).
- Preserve picture size elements (curSz/imgRect/imgDim), MEMO field parameters, shapeComment, borderFill/numbering registration axis, and table pageBreak (#1389/#1391/#1392/#1384/#1409/#1393).
- Make parser autoNum width consistent, fix newNum slot position (#1382/#1407); add an enum-token surface-format check (#1402).
- Lossless roundtrip for DocInfo, numbering paraHead, cellzoneList, useKerning/useFontSpace (#1405/#1350), and hp:tc cell field-name parsing (#1401).

### Hancom compatibility
- Fix the ClickHere guide-text (Direction) command format to match Hancom's reference output — resolves guide text not binding in the Hancom editor (#1434).

### rhwp-studio security & UX
- Exclude drag-and-drop local file loading from default behavior; require explicit opt-in via a modal confirm dialog before loading (#1439). Works in both extension and web modes.
- ClickHere form-mode/boundary editing, edit caret/focus, object aspect-ratio lock / size protection, table-cell TAC picture vertical alignment (#1419/#1428/#1430/#1436/#1352/#258).
- Dark theme support and residual UI contrast cleanup (#1420/#1422). Fix replaceAll save loss, image Shift resize, post table-create F5 handling (#1398/#1400/#1404).

### Rendering
- Native PDF export API (DocumentCore) + report-only PDF visual diff (#1359). Text IR v2 font-proof gates, exact font replay proof, glyph orientation/transform authority (#1421/#1429/#1312).
- Endnote height-model measurement SSOT / gate recalibration, official endnote shape model normalization, integral glyph (#1363/#1370/#1410/#1314/#1377); page-area-limited table-cell / rotated-cell picture placement (#1282).

### Other
- Add a 27-sample chart corpus (OOXML + legacy) verification fixture (#1431, P-1); split endnote dump / sweep verification infra (#1395).
- Preserve mixed page sizes when printing (#1383); Onsamiro picture wrap / paragraph border alignment (#1441).

### Contributors

External contributor PRs merged in this cycle (after v0.7.15; GitHub handles, alphabetical):

- @Martinel2 — useFontSpace IR field + HWP5/HWPX parser & serializer (#1350)
- @Mireutale — HWPX table-cell tab/line-break inline serialization, picture effects/shadow roundtrip (#1360/#1349)
- @jangster77 (Taesup Jang) — endnote shape model normalization, ClickHere form/edit, dark theme, table-cell picture / size protection, Onsamiro alignment (#1410/#1419/#1420/#1427/#1430/#1435/#1437/#1441), verification infra (#1395)
- @johndoekim — 27-sample chart corpus (#1431 / PR #1432)
- @msjang (Minseok Jang) — preserve mixed page sizes when printing (#1383)
- @mrshinds — TAC table host-line spacing (#1376)
- @oksure (Hyunwoo Park) — replaceAll save loss, createEmpty default section, image resize, hp:tc cell name, post-create F5, caption parse/serialize (#1398/#1399/#1400/#1401/#1404/#1406)
- @physwkim (Sang Woo Kim) — HWPX lossless roundtrip (DocInfo/cellzoneList/useKerning, etc.) (#1405)
- @planet6897 (Jaeuk Ryu) — endnote height SSOT/gate, integral glyph, endnote divergence diagnosis/closure (#1314/#1371/#1374/#1377 / PR #1390)
- @postmelee — rhwp-studio dark-mode residual UI contrast (#1422 / PR #1424)
- @seo-rii (Seohyun Lee) — renderer baseline sweep, native PDF export API, Text IR v2 font-proof gates (#1312/#1359/#1421/#1429)

## [0.7.15] — 2026-06-06

> Security patch following v0.7.14 — hardens browser-extension service-worker fetch paths,
> refines equation TAC flow/caret movement, and continues HWPX save-contract stabilization.
> Public APIs remain backward-compatible — PATCH.

### Security
- Hardened Chrome/Firefox extension document-fetch paths in the service worker (#1307).
  - Added message sender validation to separate extension viewer and content-script callers.
  - Blocked localhost, loopback, link-local, private-network, and internal-host URLs.
  - Revalidates the final URL after redirects with the same policy.
  - Uses `credentials: "omit"` for extension-side fetches.
  - Keeps automatically extracted thumbnail data out of the page DOM.
- Prepared Chrome/Edge/Firefox extension `0.2.4`: no new permissions and no new external network endpoints.

### Equation and Endnote Flow
- Improved wrapping and paragraph-indent handling for equation TAC-only lines, and fixed endnote-area caret navigation regressions (#1310).
- Reduced duplicate stops/skips when moving the caret across forced line breaks and paragraph boundaries around TAC equations (#1308/#1310).
- Follow-up fixes for endnote equation scripts, continuation spacing, and superscript alignment (#1301/#1303/#1306).

### HWPX Save Contract
- Fixed hard-coded flip/rotation and missing `isEmbeded` output in HWPX picture serialization (#1309).
- Preserved HWPX diagonal cell-border `hh:slash` / `hh:backSlash` type values (#1311).
- Preserved zero-length HWPX field ordering (#1299).

### rhwp-studio and Docs
- Separated paragraph left-margin and hanging-indent dialog bindings.
- Improved visual-sweep contributor guidance, including rsvg/font setup (#1292).
- Expanded CLI analysis/debugging command guides.

### Contributors
Thanks to Dangel for the security report and validation help, and to all external contributors and
Dependabot for this patch cycle.

## [0.7.14] — 2026-06-05

> Patch cycle following v0.7.13 (May 26–Jun 5) — focused on endnote (explanation) flow/spacing
> alignment, equation rendering/layout refinements, in-cell picture editing (insert/copy/hit-test)
> Hancom parity, HWPX save-contract extensions, and multiple external contributor PRs. Public API
> stays backward-compatible — PATCH.

### Endnote flow & spacing
- Aligned compact-endnote question-title gaps (7mm between-notes), multi-line paragraph leading,
  consecutive inline-equation multi-row merge, and below-separator margins to Hancom
  (#1240, #1241, #1247, #1255, #1259, #1262, tasks #1245/#1248/#1256/#1257/#1258)
- Corrected end-of-column line position and overflow in multi-column (EACH_COLUMN) endnote flow

### Equation rendering
- Script token handling: root/sqrt & relational glued-split, rm+bar(overline) leak, prime/cdots gluing (#1208)
- Trailing scripts bound to LEFT-RIGHT delimiter groups (`|x|^3`) (#1226)
- Big-operator (Σ/∏/∫) operand spacing; resolved Hangul compression/overlap on equation lines (#1235, #1223)
- Restored HWPX endnote/footnote prefixChar marker glyph ('문') (#1202)

### In-cell picture editing — Hancom parity
- Table + picture insert/toggle/visual/click parity (#1177), in-cell shape object-properties dialog (#1150)
- Nested-cell picture copy (Ctrl+C) + floating-object paste cascade (#1228)
- Rectangle-textbox picture click hit-test/properties/insert (#1254), nested-cell paste path preservation (#1207)
- HWP5 wrap=Square host cursor advance (answer↔question overlap), equation-only cell z-table row compression (#1220, #1225)

### Layout & rendering
- HWPX curve `<hp:seg>` outline rendering (#1203), textFlow roundtrip preservation (#1213)
- BehindText/InFrontOfText z-order composition, paper-relative BehindText z-order, master-page textbox numbering (#1163, #1252)
- Fixed 90°/270° rotated image bbox double-rotation (#1102), RawSvg(OLE/chart) first-load blank render (#1182)
- Font fidelity: Hancom Dotum fallback → Noto Sans KR ExtraLight (#1234)
- Hancom-style grid view & page borders, ghost-image fix (#1137, #1164)

### HWPX save contract
- Bookmark/Field dispatcher wiring, OLE chart, rotated picture, facing-page margin alternation, masterpage idRef (#1289, #1242)
- Globally-unique body/table-cell paragraph ids (#1222), external image reference/bytes injection contracts (#1142/#1143)

### rhwp-studio
- Reduced input-edit re-render cost (narrow invalidation) (#1212), shared modal-dialog drag
- mac window resize centering, find/goto dialog Enter handling, hit-test caret snapping (#1193, #1281, #1291)

### Infra & docs
- Dependabot coupled-dependency grouping, vite/puppeteer-core dev-dep bumps (#1214, #1216)
- macOS headless Skia font-lookup hang prevention (task #823), Rust test warning cleanup (#1180)
- Fixed ClickHere field-value file corruption (#1076)

### Contributors
@planet6897, @postmelee, @jangster77, @johndoekim, @Martinel2, @Mireutale, @chkwon, @oksure,
@seo-rii, @xogh3198, @twoLoop-40, @lidge-jun, @humdrum00001010, @HaimLee-4869, @wonbbnote
and Dependabot. Thank you.

## [0.7.13] — 2026-05-26

> Patch cycle following v0.7.12 (May 18–26) — focused on HWPX rendering/save compatibility, exam/public-agency document regressions, and multiple external contributor PR cherry-picks.

### Key Changes

- **HWPX → HWP save compatibility**
  - Added/adjusted table and cell contracts: table-axis behavior, cell LIST_HEADER materialization, gradient `BORDER_FILL`, cell inner margins, and cell background image fill mode.
  - Added memo control serialization, TOC field marker/page text output, page-number hide/restart controls, and related paragraph-control save paths.
  - Resolved multiple Hancom corruption/interrupted-render cases across `hwpx-h-01/02/03`, `mel-001`, `aift`, `exam_kor`, and `exam_social` fixtures.
- **HWPX rendering parity**
  - Implemented/improved master pages (even/odd/last), headers/footers, paragraph numbering, textbox positioning/gradient/corner radius, paragraph borders, and exam passage boxes.
  - Improved SVG and web-canvas visual parity for Hancom-converted fixtures such as `exam_kor.hwpx`, `exam_social.hwpx`, and `hwp3-sample16-hwp5.hwpx`.
- **Pagination and layout fixes**
  - Fixed HWPX `treat_as_char` table LINE_SEG lh over-inflation, nested table page splitting, picture pushdown/vpos double counting, multi-column endnote vpos, and bottom-overflow measurement consistency.
  - Added analysis infrastructure for HWP3/HWP5-converted sample16 page breaks and paragraph spacing.
- **rhwp-studio / extension UX**
  - Improved TAC shape cursor movement and repeated-space navigation behavior.
  - Chrome extension now guides users when local `file://` access is disabled and suppresses duplicate local downloads for HWP/HWPX files (#1131/#1132).
- **Infrastructure and PR intake**
  - Added CI runner disk-space mitigation (#1109).
  - Reviewed/cherry-picked external PRs including #1077/#1078/#1080/#1081/#1117/#1120/#1125/#1132.
  - Advanced CanvasKit glyph payload gating and COLRv1 glyph gradient replay.

### Remaining

- GitHub Actions outage may delay remote CI runs at v0.7.13 preparation time; local build/test/WASM validation is used as a fallback.
- The `exam_social` HWPX → HWP page-3 odd-header textbox height issue remains split into a follow-up issue.

## [0.7.12] — 2026-05-18

> Patch cycle following v0.7.11 (May 12–18) — 19 external contributor PRs merged + @jangster77 PR series of 7 (#956–#968). 416 files / +64383 / -3323.

### Key Changes

- **Original Issue #952 (1 umbrella → 5 separated defects) completed** — @jangster77 diagnostic methodology (partial fix + clear separation):
  - Issue 1 (#956): force paper-based page border outline — fixes `#920` bit-interpretation regression (verified against 5+ samples in Hancom viewer)
  - Issue 2 (#958, #957): sample16 page 18 empty caption phantom advance fix
  - Issue 3 (#961, #959): exam page 1 Q9 — horz_rel_to=Column picture out-of-column emit advance skip
  - Issue 4 (#963, #960): exam page 2 cases formula off-by-one — include end-position TAC in last run of has_line_break line
  - Issue 5 (#964, #962): exam page 2 example textbox inline equation duplicate emit block
- **WMF SetTextAlign vertical bits fix** (#966, #965): `mode & VTA_TOP(=0)` always-true bug → WMF [MS-WMF] 2.1.2.18 spec compliance (ported root cause ~60 lines from large PR #918 Stage 33-A)
- **HWP3 sample18 page count +2 inflate fix** (#968, #967): block standalone page for empty paragraph + [page break] + overflow case (v2 refinement — resolves aift.hwp snapshot regression)
- **Release build LTO + codegen-units=1 + strip** (#818, #790): rhwp CLI -28% (14→10 MB) / WASM -6.5% (4.6→4.3 MB)
- **rhwp-studio new features** (May 12–18): F5 block selection + F3 expand (#811) + menu hotkey infra (#810) + start with new page number (#809) + searchAllText API + rhwpDev.goto (#814) + Task #571 document compare/history split PR 1/3 (#799)

## [0.7.11] — 2026-05-11

> Patch cycle following v0.7.10 (May 10–11) — 30+ external contributor PRs merged. (CHANGELOG_EN.md retroactive supplement — omitted at v0.7.11 release)

- **Skia native raster incremental progress** (Issue #536): P8 (#761) Layer IR contract hardening + P9 (#769) text replay parity + P11 (#797) Text IR v2 compatibility contract
- **HWP3 native rendering** (#753): hwp3-sample10.hwp Oracle 763-page 8-stage fix + Git LFS pdf-large/ isolation
- **rhwp-studio interactions** (#781/#786–#818): scrollbar drag + chord key Ctrl+N→Ctrl+M + Korean IME chord e.code detection + Alt/Option+Arrow word navigation (#794) + table cell drag context (#795)

## [0.7.10] — 2026-05-06

> Post-v0.7.9 patch cycle — Absorbed 7 external contributors (13 PR cherry-picks) + introduced AI pipeline / VLM integration + CLI binary release pipeline (Issues #608/#612).

### New Features

- **CLI binary releases** (Issue #608/#612, by [@almet](https://github.com/almet)'s request)
  - 4-platform GitHub Release assets attached (Linux x86_64 / macOS x86_64+aarch64 / Windows x86_64)
  - SHA-256 checksums included
  - New `.github/workflows/release-binary.yml`
- **PNG raster backend** (PR #599, [@seo-rii](https://github.com/seo-rii)) — render P4 stage
  - Native Skia-based `PageLayerTree` → PNG export
  - `native-skia` feature gate (zero impact on default build, opt-in)
  - `DocumentCore::render_page_png_native(page)` API
  - **AI pipeline + VLM (Vision-Language Model) integration introduced** (maintainer follow-up fixes)
    - `--vlm-target claude` (1568 longest edge / 1.15 MP, Claude Vision compliant)
    - `--scale <factor>` / `--max-dimension <px>` (auto scale calculation)
    - `export-png` CLI command + manual (Korean + English dual)
    - Korean font fallback chain + per-character fallback (whitespace tofu fix) + `--font-path` dynamic font loading

### External PR cherry-picks (13 PRs / 7 contributors)

#### [@planet6897](https://github.com/planet6897) / Jaeook Ryu — collaborative flow

- PR #587 — HWP 5.0 spec 0x18/0x1E swap (hyphen ↔ non-breaking space)
- PR #589 (Task #511 v2 + #554) — HWP3 Square wrap supplementary fix + conversion identification heuristic
- PR #561 (Task #548) — Cell inline TAC Shape margin + indent
- PR #564 (Task #521) — TAC table outer_margin_bottom missing
- PR #570 (Task #568) — Inline table + equation paragraph right-shift
- PR #575 (Task #573) — Choice cell fraction paragraph routing
- PR #580 (Task #577) — Cell-internal standalone TopAndBottom image 1-line offset
- PR #584 (Task #574) — HY견명조 heavy display misclassification (TDD)
- PR #592 (Task #588) — exam_eng.hwp p7 arrow missing (PUA U+F003B mapping)
- PR #593 (Task #590) — Square wrap table horz_rel_to=Column attribute
- PR #567 (Task #565) — Inline equation render-missing

#### [@oksure](https://github.com/oksure) (Hyunwoo Park)

- PR #600 (closes #513) — Supplementary PUA-A SVG output fix

#### [@seo-rii](https://github.com/seo-rii)

- PR #599 (refs #536) — native Skia PNG raster backend

### Maintainer Fixes

- 5 Skia font area fixes (PR #599 follow-up, `876d820`): Korean font fallback chain / `--font-path` dynamic loading / per-character fallback / VLM options / `export-png` CLI

### Infrastructure

- CI build stability (`[[example]] required-features`)
- Wide pagination regression sweep tooling (164 fixtures / 1,614 pages auto-verification)

### Follow-up Issues

- [#613](https://github.com/edwardkim/rhwp/issues/613) — VLM preset expansion
- [#614](https://github.com/edwardkim/rhwp/issues/614) — DPI metadata option
- [#615](https://github.com/edwardkim/rhwp/issues/615) — `pua_oldhangul.rs` Hancom alignment
- [#598](https://github.com/edwardkim/rhwp/issues/598) — rhwp-studio footnote deletion (open)

### Remaining PRs (deferred to v0.7.11)

- PR #601, #602 (@oksure) / PR #607 (@dicebattle) / PR #609 (@jangster77) / PR #611 (@kihyunnn)

---

## [0.7.9] — 2026-05-01

> Post-v0.7.8 cycle — Task #501 (Hancom defensive logic for cell.padding) + cherry-pick of PR #428/#494/#478/#498 + 4 external contributors

### Regression Fixes (Maintainer)

- **Task #501 — mel-001.hwp page 2 table cell height regression** (closes #501)
  - Root cause: HWP cell IR with `cell.padding.top + bottom > cell.height` (mel-001 cell[21] r=2 c=2 "현 원": pad=(141,141,1700,1700), cell.h=1280 HU). HWPX `hasMargin="0"` confirmed.
  - Regression origin: Task #347's `prefer_cell_axis` guard applied cell-priority even for abnormal padding → row_heights inflated → TAC table proportional shrink (scale 0.45) → all rows reduced to 12-20px + cell entry failure.
  - Fix: Added **Hancom-defensive-logic mimic** guard at the end of `resolve_cell_padding` — if pad_top + pad_bottom > cell.height, scale them down proportionally to half of cell.height. Added the same guard in `measure_table_impl` step 1-b as a safety net.
  - Maintainer insight: *"What if Hancom handles this case with its own defensive logic?"* — Preserved Task #347 guard (KTX TOC R=1417 HU compatibility) + added Hancom-behavior-mimic guard
  - Wrote troubleshooting and wiki page ([HWP Cell Padding Defensive Logic](https://github.com/edwardkim/rhwp/wiki/HWP-%EC%85%80-Padding-%EB%B0%A9%EC%96%B4-%EB%A1%9C%EC%A7%81))

### External PR Cherry-picks (3 PRs / 17 commits)

- **PR #428 — Picture serialization within group** (by [@oksure](https://github.com/oksure))
  - Implemented the empty-TODO `ShapeObject::Picture` branch in `serialize_group_child` — fixes data loss for pictures inside groups when saving HWP
  - Added SHAPE_COMPONENT + SHAPE_COMPONENT_PICTURE records (matching Chart/OLE child pattern) + magic constant cleanup (`tags::SHAPE_PICTURE_ID`)

- **PR #494 — Paragraph::utf16_pos_to_char_idx public API** (#484, by [@DanMeon](https://github.com/DanMeon))
  - External binding work in the same vein as PR #405 — encapsulated `helpers::utf16_pos_to_char_idx` (pub(crate)) algorithm as `Paragraph::utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize` (pub)
  - 6 unit tests added, semver MINOR-compatible scope (+1 method, no algorithm change)

- **PR #478 — Layout/equation fixes bundled** (by [@planet6897](https://github.com/planet6897))
  - From the 9-Task / 97-commit accumulated PR, cherry-picked **7 Tasks / 10 commits** that don't directly affect page layout (5-stage merge)
  - **#488** (equation tokenizer font-style keyword prefix split + svg/canvas renderer italic honor) — 14 unit tests added
  - **#490** (alignment for empty-text + TAC equation cells) — exam_science p1 cells 7/11 with 28/36 equation now centered
  - **#483** (footnote multi-paragraph line_spacing + trailing line_spacing follow-up)
  - **#489** (Picture+Square wrap host paragraph text LINE_SEG cs/sw applied)
  - **#495** (cell paragraph inline-Shape branch guard — partial fix; remainder split into [issue #502](https://github.com/edwardkim/rhwp/issues/502))
  - **#480** (wrap=Square table paragraph margin reflected in x coordinate)
  - **#476** (PartialParagraph inline Shape page routing, +881/-4)
  - Not absorbed: #479 (paragraph trailing line_spacing / HWP vpos) — requires Hancom 2020 reference visual verification, split into [issue #503](https://github.com/edwardkim/rhwp/issues/503)

### Regression Verification Infrastructure (External)

- **PR #498 — Canvas visual diff pipeline** (relates #364, by [@seo-rii](https://github.com/seo-rii))
  - Follow-up P3 verification layer for PR #456 (P2 PageLayerTree replay transition) — added rhwp-studio E2E with **automated pixel-diff comparison** of legacy Canvas vs PageLayerTree replay Canvas + GitHub Actions Render Diff workflow
  - 7 commits split (test + diagnostics + docs + CI runner + 3 security hardening)
  - Scope: JS E2E + CI workflow + docs + Vite config (zero Rust changes)
  - Confirmed 0 diff on the 3 default fixtures (KTX / biz_plan / tac-case-001)

### Split Follow-up Issues

- [#502](https://github.com/edwardkim/rhwp/issues/502) — Inline TextBox TextRun handling within a paragraph (Task #495 remainder)
- [#503](https://github.com/edwardkim/rhwp/issues/503) — Task #479 essential fix absorption (requires Hancom 2020 reference visual verification)

### Wiki Updates

- [HWP Cell Padding Defensive Logic](https://github.com/edwardkim/rhwp/wiki/HWP-%EC%85%80-Padding-%EB%B0%A9%EC%96%B4-%EB%A1%9C%EC%A7%81) (new)
- [Hancom PDF Environment Dependency](https://github.com/edwardkim/rhwp/wiki/%ED%95%9C%EC%BB%B4-PDF-%ED%99%98%EA%B2%BD-%EC%9D%98%EC%A1%B4%EC%84%B1) — added Case IV (21_언어_기출: Hancom 2020 = rhwp output)

### Verification

- cargo test --lib 1086 → **1102 passed**
- cargo test --test svg_snapshot 6/6, issue_418 1/1, issue_501 PASS (new integration test)
- cargo clippy --lib -- -D warnings 0 warnings
- WASM 4,206,487 bytes (after Task #501) → 4,202,430 bytes (after PR #478 5th merge) → 4,211,280 bytes (after #480)

## [0.7.8] — 2026-04-29

> Post-v0.7.7 cycle — multiple external contributors, maintainer regression fixes, and wiki/README organization

### External PR cherry-picks (15 items)

Library core fixes (typesetting / pagination / serialization):

- **PR #391 Multi-column section accumulation formula regression fix** (#391, by [@planet6897](https://github.com/planet6897))
  - `src/renderer/typeset.rs` accumulation formula branched by `col_count`: single-column → `total_height`, multi-column → `height_for_fit` (suppresses trailing_ls inflation)
  - exam_eng (2-column): 11 → **8 pages**, single 1-item column issues (p3/p5/p7) all resolved

- **PR #396 Equation rendering improvements** (#174, #175, by [@oksure](https://github.com/oksure))
  - Set inline equation height based on `eq.common.height` (HWP authoritative value) + apply X/Y scaling simultaneously
  - Disable italic styling and width compensation for CJK characters in equations — fixes CASES Korean line overlap
  - Maintainer follow-up fixes (3 items): Canvas fraction line y / Equation scale / Limit fi=fs

- **PR #395 Image brightness/contrast effects in SVG output** (#150, by [@oksure](https://github.com/oksure))

- **PR #397 Equation ATOP parsing and rendering fix** (by [@cskwork](https://github.com/cskwork))
  - **First external contributor to this repository** — `EqNode::Atop` AST parsing + above/below layout without fraction line (separates HWP's ATOP / OVER semantics)

- **PR #400 HWPX equation serialization preservation** (#286, by [@cskwork](https://github.com/cskwork))
  - Fix `render_paragraph_parts` ignoring controls + parser XML entity restoration
  - Verified: Hancom Hangul 2020 normal viewing + PDF match (added Hancom-origin hwp roundtrip regression commit `ecd7d9a`)

- **PR #401 v2 Table page split rowspan>1 cell unit policy** (#398, by [@planet6897](https://github.com/planet6897))
  - `BLOCK_UNIT_MAX_ROWS=3` threshold — protect small blocks (≤3 rows) only, allow row-unit split for large rowspans (≥4 rows) — Hancom-compatible
  - synam-001.hwp page 5 regression fixed (35→37→**35** pages)

- **PR #406 Inline TAC image pagination fix in same paragraph** (#402, by [@planet6897](https://github.com/planet6897))
  - Fixed second inline image in the same paragraph being drawn at the same y-coordinate as the first, causing overlap/overflow
  - 27→30 pages (split normalized)

- **PR #408 heading-orphan vpos-based correction** (#404, by [@planet6897](https://github.com/planet6897))
  - vpos-based 5-condition AND trigger (current fits + vpos overflow + next substantial + next doesn't fit + single column non-wrap) — only 1 of 41 vpos overflow cases is a true orphan
  - Page 9 pi=83 heading → pushed to page 10, placed together with subsequent table

- **PR #410 TopAndBottom Picture vert=Para chart fix + atomic TAC top-fit** (#409, by [@planet6897](https://github.com/planet6897))
  - v1: Extend `prev_has_overlay_shape` guard (Picture + TopAndBottom + vert=Para)
  - v2: `typeset_section` controls loop chart height accumulation
  - v3: `typeset_paragraph` atomic TAC top-fit semantics (60px tolerance)

- **PR #415 Task #352 dash sequence Justify width inflation fix** (#352, by [@planet6897](https://github.com/planet6897))
  - dash leader elastic Justify distribution (PDF-mimicking), exam_eng Q32 dash advance 12.11 → 7.06 px

- **PR #424 Multi-column right column single-line paragraph line spacing fix (vpos correction anchor)** (#412, by [@planet6897](https://github.com/planet6897))
  - layout.rs vpos correction formula fix — introduce `col_anchor_y` (preserves anchor right after body_wide_reserved push), prefer `curr_first_vpos`, separate page_path/lazy_path
  - exam_eng p1 right column item 7 ①~⑤ 15.33→**22.55px uniform**, left column item 1 catch-up 28.56→21.89

- **PR #427 SvgRenderer defs deduplication unified to HashSet** (#423, by [@oksure](https://github.com/oksure))
  - `arrow_marker_ids: HashSet<String>` → unified `defs_ids: HashSet<String>`, O(n)→O(1)

- **PR #434 Image auto-crop (FitToSize+crop) formula correction + paragraph border inner padding** (#430, by [@planet6897](https://github.com/planet6897))
  - svg.rs / web_canvas.rs crop scale formula correction (`cr/img_w` → `original_size_hu/img_size_px`) + helper `compute_image_crop_src` extraction (single source of truth for SVG/Canvas)
  - Separate fix: paragraph border inner padding (text sticking to border)

API additions / tooling:

- **PR #405 `Paragraph::control_text_positions` added** (#390, by [@DanMeon](https://github.com/DanMeon))
  - API refactor for external binding exposure

- **PR #411 `editor.exportHwp()` API added** (by [@ggoban](https://github.com/ggoban))
  - First-time contributor — exposed exportHwp() on iframe wrapper `@rhwp/editor`

- **PR #413 rhwp-studio PWA support** (#383, by [@dyjung150605](https://github.com/dyjung150605))
  - First-time contributor — vite-plugin-pwa, manifest scope `/rhwp/`, icon 192/512/maskable, registerType=autoUpdate, WASM precache

- **PR #419 PageLayerTree generation API introduced** (#364, by [@seo-rii](https://github.com/seo-rii))
  - New `paint` module (2,376 lines, builder/json/layer_tree/paint_op) — PageRenderTree → PageLayerTree conversion
  - opt-in transition adapter (`svg_layer.rs`, `RHWP_RENDER_PATH=layer-svg`)
  - Existing 5 renderer files unchanged (0 lines), 309 pages SVG byte-identical across the board (fidelity analysis report)

### Maintainer work (3 items)

- **Task #394 Disable cell-entry transparent border auto-on logic** (#394)
  - input-handler.ts 5 areas commented out — Hancom output alignment

- **Task #416 `find_bin_data` guard defect fix** (#416)
  - Removed `c.id == bin_data_id` guard — `c.id` is storage_id, bin_data_id is index. sparse id range branching (preserves HWPX chart 60000+N). 7 unit tests added

- **Task #418 `hwpspec.hwp` p20 empty paragraph + TAC Picture double emit fix** (#418)
  - Task #376 fix commit was not merged into devel (closed but only existed on temporary branch) → same defect recurred
  - Added paragraph_layout set_inline_shape_position + already_registered guard in layout.rs::layout_shape_item
  - New memory (verify devel-merged commit on close) + new troubleshooting document

### Maintenance / documentation

- **Wiki page [Hancom PDF Environment Dependency](https://github.com/edwardkim/rhwp/wiki/한컴-PDF-환경-의존성) enhanced**
  - Added "Discovery II (PR #434 / Issue #430)" section — Hancom 2010 ↔ 2020 ↔ Hancom Docs render the same hwp differently. Re-confirms the limit of the single-Hancom-reference assumption.
  - rhwp's current output may better match the test sheet author's intent (preserves original JPEG "(A type)" residue)

- **README.md / README_EN.md enhanced**
  - Added "Hancom PDFs are not authoritative ground truth" item to Contributing section
  - New "Wiki Resources" subsection (9 wiki page links)

- **samples reference materials added** — shared with all contributors and fork users
  - `samples/2010-exam_kor.pdf` (Hancom 2010, 4.57 MB)
  - `samples/2020-exam_kor.pdf` (Hancom 2020, 4.57 MB)
  - `samples/hancomdocs-exam_kor.pdf` (Hancom Docs, 6.05 MB)
  - `samples/복학원서.pdf` (Issue #421 Hancom reference)
  - `samples/synam-001.hwp` (PR #401 regression verification)
  - `samples/atop-equation-01.hwp` (PR #397 visual judgment)

### Verification

- `cargo test --lib`: **1066 passed** (1008 → +58, 0 regressions)
- `cargo test --test svg_snapshot`: 6/6 passed
- `cargo test --test issue_418`: 1/1 passed (Task #418 regression preserved)
- `cargo clippy --lib -- -D warnings`: 0 warnings
- WASM build: 4,182,395 bytes (delta +47 KB)
- Wide byte-level comparison: 10 samples / 309 pages SVG regression verification (per-PR verification gate)
- Maintainer SVG + Canvas dual-path visual judgment (PR #401 v2 / #406 / #408 / #410 / #415 / #424 / #434)

### Acknowledgments to External Contributors

External contributors in this cycle (alphabetical):
[@cskwork](https://github.com/cskwork), [@DanMeon](https://github.com/DanMeon), [@dyjung150605](https://github.com/dyjung150605), [@ggoban](https://github.com/ggoban), [@oksure](https://github.com/oksure), [@planet6897](https://github.com/planet6897), [@seo-rii](https://github.com/seo-rii)

In particular, [@cskwork](https://github.com/cskwork) became the **first external contributor to this repository** with two merged PRs (#397, #400), and [@planet6897](https://github.com/planet6897) diagnosed and fixed the majority (8 PRs) of external PRs in this cycle.

## [0.7.7] — 2026-04-27

> v0.7.6 regression fix cycle (restoring missing semantics after TypesetEngine default switch)

### Fixes — TypesetEngine regression corrections

- **Pagination fit accumulation drift fix** (#359)
  - Separate fit determination from accumulation in typeset: fit uses `height_for_fit` (excluding trailing_ls), accumulation uses `total_height` (full)
  - Added single-item page block guard — skip empty paragraphs / disable safety margin once when next pi's vpos-reset guard is about to trigger
  - **k-water-rfp**: LAYOUT_OVERFLOW 73 → 0 (drift 311px corrected)
  - **kps-ai**: 60 → 4

- **TypesetEngine page_num + PartialTable fit safety margin** (#361)
  - Aligned NewNumber application conditions in `finalize_pages` with Paginator semantics (`prev_page_last_para` tracking, applied once per page)
  - Disabled fit safety margin (10px) right after PartialTable — PartialTable's cur_h is row-accurate
  - **k-water-rfp**: 28 → 27 pages (page_num updated correctly)
  - **kps-ai**: page_num 1, 2, 1, 1, 2~8 normal (NewNumber control handling)

- **kps-ai PartialTable + Square wrap handling** (#362, 8 cumulative items)
  - **wrap-around mechanism (Square wrap) port** ★ — Ported wrap zone matching + activation semantics from Paginator engine.rs:288-372 to TypesetEngine. Paragraphs beside outer table absorbed without consuming height
  - Outer cell vpos guard — exclude LineSeg.vertical_pos in nested table cells (blocks p56 clip)
  - Allow nested PartialTable split — display split instead of atomic deferral for nested tables larger than one page (blocks p67 empty page)
  - Accurate PartialTable remaining height calculation — new `calc_visible_content_height_from_ranges_with_offset`
  - Strengthened nested table cell capping (cap by outer row height)
  - Added hide_empty_line to TypesetEngine (max 2 empty lines at page start with height=0)
  - vpos-reset guard ignored within wrap zone (blocks misfire)
  - Strengthened empty paragraph skip guard — paragraphs with table/shape controls are not skipped (blocks pi=778 table omission)
  - **kps-ai**: 88 → 79 pages (matches Paginator's 78, LAYOUT_OVERFLOW 60→5)

### Security

- **rhwp-firefox/build.mjs CodeQL Alert #17 resolved** (#354)
  - `execSync` shell usage → `execFileSync` (`shell: false`)

### Verification

- `cargo test --lib`: 1008 passed, 0 failed
- `cargo test --test svg_snapshot`: 6/6
- `cargo test --test issue_301`: 1/1
- WASM build passed
- Maintainer visual judgment passed (kps-ai p56, p67-70, p72-73, k-water-rfp full)

## [0.7.6] — 2026-04-26

> Multiple external contributors + typesetting precision cycle

### Added
- **`replaceOne(query, newText, caseSensitive)` WASM API** (#268)
  — Analyzed and implemented by [@oksure](https://github.com/oksure) (new contributor)
  - Resolved crash from position-based vs query-based call mismatch in `replaceText`
  - 100% backward compatibility preserved with new API
  - 5 unit tests (including Korean multi-byte boundaries)

- **SVG/HTML draw_image base64 embedding** (#335)
  — Analyzed and implemented by [@oksure](https://github.com/oksure)
  - Existing placeholders (`<rect>`/`<div>`) → actual image base64 data URI embedding
  - Backend alignment with `render_picture` / `web_canvas`

### Fixed
- **TOC reader dots + page number right-tab alignment** (#279)
  — Analyzed and implemented by [@seanshin](https://github.com/seanshin)
  - Express `fill_type=3` dotted reader as round-cap dots (Hancom-equivalent)
  - Excluded `find_next_tab_stop` RIGHT-tab clamping — corrects page number alignment in indented paragraphs
  - Maintainer enhancements: cell-padding-aware leader semantics, leader length differentiation by page number width, blank-only run carry-over

- **form-002 inner table page split defect** (#324)
  — Analyzed and implemented by [@planet6897](https://github.com/planet6897)
  - `compute_cell_line_ranges` rewritten from residual-tracking to cumulative-position (`cum`)-based
  - `layout_partial_table` `content_y_accum` update + unified split-start row calculation
  - Author self v1 → v2 → v3 enhancements

- **typeset path PageHide / Shape / duplicate emit defects** (#340)
  — Analyzed and implemented by [@planet6897](https://github.com/planet6897)
  - Unified diagnosis of three defects as common cause (typeset.rs omissions)
  - Alignment with `engine.rs` (PageHide collection + `pre_text_exists` guard + Shape inline registration)

- **Firefox AMO warning resolved (rhwp-firefox 0.2.1 → 0.2.2)** (#338)
  — Analyzed and implemented by [@postmelee](https://github.com/postmelee)
  - manifest `strict_min_version` raised to 142 (`data_collection_permissions` compatibility)
  - sanitized unsafe `innerHTML` / `Function` / `document.write` in `viewer-*.js`
  - rhwp-studio 28-file DOM/SVG API replacement + Reviewer Notes (KO/EN)

- **Task #321~#332 cumulative cleanup + vpos/cell padding regression resolution** (#342)
  — Analyzed and implemented by [@planet6897](https://github.com/planet6897)
  - Bidirectional vpos correction guard + cell padding aim explicit-value precedence policy
  - typeset/layout drift alignment + KTX TOC results (#279) restored per maintainer review feedback

### Other
- **New contributor welcome** — README.md / README_EN.md Contributing section explicitly states PR base=devel (follow-up improvement after #330 close)

## [0.6.0] — 2026-04-04

> Typesetting quality improvements + non-functional foundation — "Breaking the egg, into the world"

### Added
- **GitHub Actions CI**: Build + test + Clippy strict mode (#46, #47)
- **GitHub Pages demo**: https://edwardkim.github.io/rhwp/ (#48)
- **GitHub Sponsors**: Sponsor button activated
- **Image cropping**: SVG viewBox / Canvas drawImage image crop rendering (#43)
- **Image border**: Picture border_attr parsing + border rendering (#43)
- **Header/footer Pictures**: non-TAC image absolute positioning, TAC image inline placement (#42)
- **Logo asset management**: assets/logo/ source-managed, favicon generation
- **Non-functional work plan**: 13 items in 6 areas, 3-stage milestones (#45)

### Fixed
- **Same-paragraph TAC + block table**: Prevented intermediate TAC vpos gap negative regression (#41)
- **Split-table cell vertical alignment**: Forced Top in split rows, reflected nested table height (#44)
- **TAC table trailing ls**: Boundary condition cyclic error resolved (#40)
- **Currency symbol rendering**: ₩€£¥ Canvas Malgun Gothic fallback, SVG font chain (#39)
- **Half-/full-width precision**: Removed Bold-fallback compensation, half-width smart quotes / middle dot (#38)
- **Font-name JSON escaping**: Fixed font-name load failure with backslash (#37)
- **Header table cell image**: Fixed bin_data_content propagation path (#36)
- **Clippy warnings removed**: 6 issues including unnecessary_unwrap, identity_op (#47)

## [0.5.0] — 2026-03-29

> Skeleton complete — reverse-engineered HWP parser/renderer

### Core features
- **HWP 5.0 / HWPX parser**: OLE2 binary + Open XML format support
- **Rendering engine**: paragraphs, tables, equations, images, charts, header/footer/master pages/footnotes
- **Pagination**: multi-column split, table row split, shape_reserved handling
- **SVG export**: CLI (`rhwp export-svg`)
- **Canvas rendering**: WASM/Web-based
- **Web editor**: rhwp-studio (text editing, formatting, table creation)
- **hwpctl-compatible API**: 30 Actions, Field API (Hancom Web Hangul-compatible)
- **VS Code extension**: HWP/HWPX viewer (v0.5.0~v0.5.4)
- **755+ tests**

### Typesetting engine
- Line spacing (fixed/percent/by-character), paragraph margins, tab stops
- Table cell merging, border styles, cell formula calculation
- Multi-column layout, paragraph numbering / bullets
- Vertical text, object placement (block/in-line/in-front-of-text/behind-text)
- Inline TAC tables / pictures / equations rendering

### Equation engine
- Fractions (OVER), roots (SQRT/ROOT), subscripts/superscripts
- Matrices: MATRIX, PMATRIX, BMATRIX, DMATRIX
- Cases (CASES), alignment (EQALIGN), integral/sum/product operators
- 15 text decorations, Greek letters, 100+ math symbols
