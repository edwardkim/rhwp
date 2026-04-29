# Changelog

This document records the major changes of the rhwp project.

> 한국어 버전은 [CHANGELOG.md](CHANGELOG.md) 를 참조하세요.

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
