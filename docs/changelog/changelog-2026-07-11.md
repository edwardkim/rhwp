# 2026-07-11

## HML document open support - Frame

- Decision: stop before product implementation until three redistributable Hancom-authored HML fixtures and provenance-labelled render references are available.
- Why: the request explicitly forbids inferred XML, the repository contains no standalone HML corpus, and upstream issue #1157 records the same maintainer gate.
- Alternatives rejected: synthetic-only parser work, JavaScript HML-to-HWPX conversion, ZIP wrapping, and copying third-party samples without verified licensing/provenance.
- Architecture direction: validate and decode in `src/parser/hml/`, reuse inventory-proven HWPX semantic helpers, carry structured import metadata beside `Document`, and keep renderer/layout format-agnostic.
- Safety direction: bounded XML parsing, DTD/entity denial, deny-by-default external resources, and mandatory HML Save As so the original handle is never overwritten.
- Evidence and frozen implementation plan: `docs/changelog/2026-07/11-hml-document-open/`.

## HML corpus audit

- Accepted one byte-identical public fixture, `samples/hml/aligns.hml`, from
  `ohah/hwpjs@e2beadb2cfbbae6c814c4db6644383f054903c3c`; upstream is MIT and its
  notice is preserved beside the fixture.
- Provenance boundary: the file strongly indicates a Hancom export, but the exact Hancom version,
  OS, and a Hancom PDF/PNG oracle are not documented upstream. It is valid structure evidence, not
  completion evidence for the three-fixture/render-oracle gate.
- Format decision: the accepted HML has no namespace and uses uppercase
  `HWPML/HEAD/BODY/TAIL`, so signature detection cannot require an HWPX namespace and HWPX XML
  cannot be parsed directly without a legacy-HML adapter.
- Coverage boundary: the fixture covers UTF-8 BOM, paragraphs, shape tables, and rectangle text
  boxes. Table, image, equation, binary/external resources, UTF-16, version diversity, and visual
  oracle coverage remain unproven.
- Rejected alternatives: unlicensed `ruby-hwp` HML/PDF pairs, AGPL `hwpkit` corpus without a
  compatibility decision, and generated `node-hwp` HML that is not a Hancom-saved source document.
- Second-pass corpus decision: accepted byte-identical `samples/hml/formatting_table.hml` from
  `osik-kwon/osk_filter@8b483dc73edfe31b34c9c2324e1096be474fa341` under the Unlicense. It adds
  one real 1x1 table and one rectangle textbox, but still has no documented Hancom version/OS or
  Hancom PDF/PNG oracle.
- Second-pass rejection: `md2hml` is unlicensed and generated; the MIT `Markdown2HWP_Program`
  templates may have Hancom ancestry but are directly modified by its HML generator, so their exact
  current bytes are not accepted as Hancom ground truth. Unlicensed picture/binary candidates and
  the `osk_filter` privacy sample were also excluded.
- Remaining corpus gate: two lawful fixtures are now present. Image, equation, embedded binary,
  UTF-16, version diversity, and provenance-labelled visual oracle coverage still require a third
  independently sourced Hancom-saved file; no substitute was fabricated.
- Oracle decision: opened both unchanged HML fixtures in Hancom Office HWP Viewer 12.31.7
  (build 6383) on macOS 26.5.1 and printed sanitized A4 PDF references. `aligns` is 16 pages;
  `formatting_table` is 1 page. Author/title metadata was removed before committing the artifacts.

## HML Studio input and save safety

- Signature decision: binary magic remains first; XML is decoded from a bounded prefix with
  UTF-8/UTF-16 BOM support, then classified by its root before Content-Type. HML requires either a
  Hancom HWPML namespace or the verified legacy `HWPML Version + SubVersion + Style` root contract.
  `HEAD/BODY` are parser validation, not signature requirements, because they may occur after the
  64 KiB detection prefix.
- False-positive boundary: an XML declaration alone, a nested HWPML element, or a namespace-free
  incomplete/empty HWPML root remains ordinary XML; XML-declared HTML remains an HTML error page.
- Open-path decision: `.hml` is exposed consistently in native picker, hidden input, drag/drop, and
  PWA file-handler metadata. Extension only admits the local file into the loader; byte/parser
  validation remains authoritative.
- Save decision: Ctrl+S and the generic Save As command on an imported `.hml` first ask whether to
  export HWP or HWPX, then force a new target. The original handle is never written; after explicit
  conversion, later saves keep the converted target and format. Converted targets are tracked by
  the successfully written handle, not inferred from its extension, so misleading `.hwp` input
  names cannot bypass Save As; opening a handle as a new document clears its prior target marker.
- Picker safety: save pickers hide the arbitrary-file option and validate both the returned file
  extension and file-entry identity before opening a writable stream. A browser or user attempt to
  return/reselect the HML source is rejected before any bytes are written.
- Fallback decision: browser-download fallback names are normalized to the chosen `.hwp` or `.hwpx`
  extension, including HWPX output through the legacy filename dialog.
- Alternatives rejected: treating every XML document as HML, requiring a namespace that the real
  legacy fixture lacks, reusing the `.hml` handle for converted bytes, and trusting picker filters
  without validating the returned handle.

## HML Rust parser - authorized first vertical slice

- Scope decision: supersede the earlier implementation stop only for a bounded HWPML 2.9
  text-document tracer. The parser accepts the verified uppercase `HWPML/HEAD/BODY/SECTION/P/TEXT/CHAR`
  structure with default style references and converts it to the existing `Document` IR.
- Safety decision: decode UTF-8 BOM and UTF-16 LE/BE BOM explicitly, cap bytes/depth/attributes/text
  nodes, reject DTDs and non-built-in entities, and return typed errors for malformed XML or missing
  structural regions.
- Loss boundary: non-default resource references and unsupported body elements are errors, not
  silently flattened content. The real `aligns.hml` fixture is detected as HML but reports
  `UnsupportedVersion("2.91")`; its rectangle controls are not claimed as parsed.
- Reuse decision: keep the adapter in `src/parser/hml/` and reuse the common `Document` model.
  HWPX header/section helpers were rejected for this slice because their namespaced lowercase ZIP
  parts are not structurally compatible with legacy uppercase standalone HWPML.
- Alternatives rejected: accepting every HWPML version, copying the HWPX parser, inventing mappings
  for the 2.91 rectangle controls, and returning a partial document after dropping unsupported data.

## HML WASM source-format safety

- Decision: expose `FileFormat::Hml` as `"hml"` from the existing WASM document instance.
- Why: the previous non-HWPX fallback returned `"hwp"`, bypassing Studio's forced Save As policy
  and allowing HWP bytes to be written through the original `.hml` file handle.
- Rejected alternative: infer HML from the filename in Studio. The byte-derived parser format stays
  authoritative and also handles misleading extensions.
- Test: added the failing WASM source-format regression first, then verified an opened HML document
  reports `"hml"`.

## HML shared-core and CLI integration

- Decision: classify HML and HWPX as XML imports only at the `DocumentCore::from_bytes` seam, where
  missing line segments and paragraph bookkeeping are normalized before composition. Renderer and
  layout code remain format-agnostic.
- CLI contract: `info` adds stable `format: HML`, `hwpml_version`, `sections`, and `pages` fields;
  help and command-local usage now advertise `.hml` for `info`, `dump`, `export-svg`, and
  `export-pdf`.
- Resource safety: CLI render paths do not implicitly load sibling files for HML. Such paths are
  untrusted document input and require a future explicit resolver policy.
- Metadata boundary: HWPML version already survives in `Document.doc_info`, but encoding, resource
  count, and structured HML warnings are lost when `parse_document` unwraps `HmlParseResult` to a
  bare `Document`. CLI output deliberately does not claim those fields until an import-metadata
  envelope is propagated through `DocumentCore`.
- Alternative rejected: adding HML conditionals to renderer/layout. The behavior needed here is a
  shared XML-import normalization concern, not a rendering-format concern.

## HML Rust parser - lawful HWPML 2.91 expansion

- Decision: expand the authorized parser slice only against the two redistributable 2.91 fixtures.
  The shared `Document` IR now receives their verified font, character-shape, paragraph-shape,
  style, page, paragraph, rectangle-textbox, and 1x1 table structures.
- Order invariant: text before and after an inline object remains one paragraph; the object keeps an
  eight-code-unit control slot in `char_offsets`, while textbox and cell paragraphs stay nested in
  their owning controls.
- Detection boundary: uppercase `HWPML` plus a non-empty `Version` root attribute is required.
  This preserves detection of older versions for a typed `UnsupportedVersion` result while
  rejecting ordinary XML and an unversioned lookalike root.
- Loss policy: `PICTURE`, `EQUATION`, embedded `BINDATA`, script code, and other unsupported inline
  controls are not claimed as imported. Each produces an `UnsupportedElement` warning with its XML
  path; omitted inline controls still reserve their source position. Embedded resource occurrences
  are counted, but decoding remains unsupported until a lawful resource fixture establishes the
  representation.
- Reference policy: resource tables are indexed by their verified HML IDs. Out-of-range paragraph,
  style, and character-shape references produce structured `InvalidReference` warnings rather than
  a panic or silent fallback.
- Alternatives rejected: flattening textbox/cell text into the body, pretending unsupported
  picture/equation bytes were imported, accepting `<HWPML/>` by name alone, and adding HML branches
  to renderer or `DocumentCore`.

## HML rectangle visual semantics from lawful fixtures

- Decision: map only the rectangle `LINESHAPE`, `TEXTMARGIN`, and `WINDOWBRUSH` values observed in
  `aligns.hml` and `formatting_table.hml` into the existing shared shape IR.
- Evidence: red-green tests proved the prior losses as width `0` versus `33`, textbox margin `0`
  versus `283`, and `FillType::None` versus `FillType::Solid`; the complete HML parser target passes
  20/20 afterward.
- Boundary: unobserved line styles/caps/alpha values and hatch styles produce warnings. Gradients,
  image fills, pictures, equations, and resources remain unimplemented without lawful corpus bytes.
- Rejected alternatives: guessing enum mappings from names alone, treating header brushes as shape
  fills, and adding HML-specific renderer or `DocumentCore` behavior.

## HML parser edge ownership and materialization

- Ownership decision: `SIZE` and `POSITION` map to the nearest open `RECTANGLE` or `TABLE` in XML
  ancestry. Global rectangle-first selection was rejected because a table inside a rectangle text box
  overwrote the enclosing rectangle and left its own `Table.common` empty.
- Size decision: when HML `SHAPECOMPONENT` provides original dimensions but omits or zeros current
  dimensions, materialize effective current size from the original dimensions, as the shared HWPX
  parser does for layout safety. If object `SIZE` is absent, original dimensions also supply the
  rectangle's common size.
- Round-trip boundary: do not set HWPX-specific `current_width_was_zero` or
  `current_height_was_zero` flags for HML input. HML serialization is unsupported, and omission in
  HML is not evidence of an explicit HWPX zero sentinel.
- Evidence: focused REDs showed rectangle ownership changed from `1000x500` to nested table
  `300x200`, and missing current size remained `0x0` instead of `600x400`. Both are GREEN; signed
  object and shape-component offsets survive, the full HML parser target is 22/22, and targeted
  clippy is warning-free.
- Inspected without changes: empty/self-closing elements already share the depth gate, and warning
  paths already preserve live XML ancestry for unsupported elements and attributes.

## HML CLI source metadata boundary

- Decision: HML `info` suppresses only the generic `Document.header` version, compression,
  encryption, and distribution lines. For imported HML these values are common IR defaults rather
  than HWPML source facts.
- Preserved output: file size, both section/page summaries, page geometry, fonts, styles, and object
  diagnostics continue through the shared path. HML-specific format, HWPML version, encoding,
  resource count, warnings, and warning paths remain visible.
- Compatibility evidence: direct HWP and HWPX `info` smokes retain the existing generic header
  lines; the condition is limited to `FileFormat::Hml`.
- Rejected alternatives: exposing synthetic HWP flags as HML metadata, hiding all generic document
  diagnostics, or changing common IR header defaults that other formats rely on.

## HML border-fill edge materialization

- Root cause: HML resource parsing allocated each `BorderFill` with its shared-model default but
  ignored the four edge children. Because `BorderLineType::default()` is `Solid`, lawful paragraph
  border ID 2 acquired visible lines despite declaring `Type="None"` on every side.
- Decision: track the currently open border-fill ID and map only direct `LEFTBORDER`, `RIGHTBORDER`,
  `TOPBORDER`, and `BOTTOMBORDER` children. This binds edges by XML ancestry and leaves rectangle
  `WINDOWBRUSH` ownership unchanged.
- Reuse decision: normalize HML `Width="...mm"` values with the existing `border_width_index`
  function rather than maintaining a second width table.
- Boundary: materialize only the lawful `None` and `Solid` types; warn and preserve the existing
  solid fallback for unproved types. Border colors, diagonal/center lines, shadows, and fill-brush
  variants remain unimplemented, and no renderer branch was added.
- Evidence: the focused test failed on ID 2's default-solid edges, then passed with ID 2 as four
  `None`/`0.1mm` edges and table/cell ID 3 as four `Solid`/`0.12mm` edges. The HML parser target is
  23/23 green; targeted clippy and rustfmt are clean.

## HML public support boundary

- Decision: document HML as a limited HWPML 2.9/2.91 import path in both public READMEs, including
  the proven text, formatting, table, and rectangle-textbox coverage.
- Safety boundary: state next to the feature that pictures, equations, and embedded/external resources
  are skipped with warnings and that the original HML is never overwritten.
- Rejected alternative: adding HML to the headline feature list without its version, loss, and Save As
  limits; that would overstate compatibility beyond the lawful two-file corpus.

## HML semantic serializer - S2

- Decision: serialize only the fields consumed by `src/parser/hml/reader.rs`: root metadata,
  mapping-table fonts/border edges/character shapes/paragraph shapes/tab IDs/styles, page geometry,
  paragraph text and character-shape runs, rectangles/textboxes, and tables/cells. The output is
  canonical HWPML 2.91 UTF-8, while lawful `SubVersion` and `Style` root attributes are retained in
  HML import metadata and re-emitted.
- Loss decision: `DocumentCore::export_hml_native` accepts only HML-origin cores. Any non-preserved
  import warning blocks export with a structured `code`, `xml_path`, and `message`; unsupported IR
  controls also fail explicitly instead of disappearing.
- Preservation boundary: captured HEAD/TAIL fragments are emitted in preserved-fragment order.
  The current lawful proof covers verbatim `TAIL/SCRIPTCODE`; arbitrary placement among modeled
  siblings is not claimed because the S1 envelope stores preserved-fragment order, not every child
  index.
- Alternatives rejected: inventing `DOCSETTING` values the reader never maps, translating through
  OWPML/HWPX tags, flattening unsupported controls, returning a generic render-error string, and
  claiming byte-for-byte document identity.
- TDD evidence: the first public round-trip test failed because `export_hml_native` did not exist;
  the loss-gate test then failed because warning kinds were collapsed to a generic blocker code.
  The minimal fixes made both lawful fixtures, an applied edit, the path-bearing loss gate, and the
  non-HML origin refusal pass through public `DocumentCore` behavior.

## HML serializer reader-domain loss gate - S2 full-spec pass

- Root cause: `write_border_line` mapped every public `BorderLineType` except `None` to HML
  `Type="Solid"`. A caller could edit an imported HML document to `Dash`, export successfully, and
  reparse a different value, violating the loss gate.
- Decision: allow only the `None` and `Solid` values the current HML reader reconstructs. Refuse
  every other border type with a structured `HML_UNSUPPORTED_IR` blocker at the exact edge path.
- Proof decision: compare all reader-mapped table, cell, common-object, rectangle, shape-component,
  line, and fill values on lawful round-trip; require preserved fragments byte-exactly once and
  blocker messages for every refusal case.
- Alternatives rejected: silently canonicalizing unsupported border styles, guessing legacy HML
  spellings the reader does not accept, or widening the reader without lawful corpus evidence.

## HML CLI/WASM export edge safety - S3

- File-identity decision: output protection compares Unix device/inode identity as well as lexical
  and canonical paths. A hard link is the same underlying file even when both path strings differ,
  so allowing it would violate the original-preservation contract.
- Argument decision: an option-looking token cannot satisfy `-o`/`--output`. Treating `--bogus` as
  a filename hid user mistakes and performed an unintended write; it now returns usage exit 2.
- Commit decision: HML output is prepared in a create-new sibling temporary file, fully written,
  synced, closed, and only then renamed. Failed preparation or replacement cleans the temporary
  file and leaves the prior destination intact. Rejected: direct `fs::write`, because opening an
  existing file truncates it before the complete replacement is known to be valid.
- WASM diagnostic decision: `exportHml` error text includes the same stable blocker code, XML path,
  and message exposed by metadata and CLI. Rejected: a message-only `JsValue`, because Studio could
  not correlate a trial-export failure with the preflight blocker contract.
- Platform boundary: Unix has standard device/inode and atomic replacement semantics. Non-Unix
  remains conservative through lexical/canonical checks and refusal where the standard rename
  cannot replace an existing target; cross-platform hard-link identity is not claimed by this pass.

## HML serializer edge loss gate and HEAD anchors - S2

- Root cause: the serializer wrote only the reader's narrow HML subset but validated unsupported
  values piecemeal during emission. Section/multi-column breaks and many mutable resource, shape,
  table, and cell fields therefore serialized successfully and reparsed as different public IR;
  emission also stopped at the first detected blocker.
- Decision: run a cohesive reader-domain preflight before creating XML. Collect every unsupported
  value with a typed path, permit only exact reader-reconstructable defaults and enum values, and
  retain the writer's local checks as defensive backstops.
- Placement decision: preserved HEAD metadata records the count of modeled siblings already seen,
  not merely preserved-fragment order or an unqualified raw child index. The writer weaves fragments
  around MAPPINGTABLE from that evidence, preserving before/after placement without guessing the
  positions of ignored schema children.
- Alternatives rejected: silently normalizing unsupported public values, returning only the first
  blocker, treating preserved-fragment order as modeled sibling order, or hard-coding raw child
  indexes against a canonical HEAD that intentionally omits unmodeled children.
- Evidence: independent RED/GREEN slices cover Section, MultiColumn, four HEAD resource domains,
  rectangle semantics, table-plus-cell aggregation, and synthetic HEAD before/after placement. The
  final serializer/parser/CLI targets pass 12/25/5 tests; targeted clippy, rustfmt, and whitespace
  checks are clean.

## HML serializer adversarial hardening - S2

- Root cause: the preflight matrix omitted font/style payloads, section/page flags, and BinData;
  unequal direct text mutation silently regenerated control offsets; generic document children were
  not captured across all three document containers; and the public raw-fragment envelope bypassed
  XML structure, resource-limit, and XML 1.0 character validation.
- Reader-boundary decision: gate every value the current reader cannot reconstruct. This blocks
  unsupported semantics rather than inventing mappings from synthetic XML or canonicalizing public
  IR into a different value.
- Offset decision: control positions are valid only when text UTF-16 starts, eight-unit control
  gaps, and `char_count` agree. Never infer replacement offsets from stale public fields; supported
  edits must use the bookkeeping-preserving `DocumentCore` API.
- Preservation decision: capture unknown direct HEAD/BODY/TAIL subtrees opaquely, anchored by the
  number of modeled siblings already seen. Known wrappers and modeled children are excluded, and
  captured descendants do not enter IR.
- Raw-boundary decision: validate each public `PreservedFragment` once before emission for direct
  parent/path/root evidence, anchor bounds, a single well-formed subtree, default HML limits, no
  declaration/DTD/custom entity, and XML 1.0-valid decoded characters. Structured writer values are
  checked by the same XML character predicate.
- Error decision: preserve warning-only `LossyImport` and IR-only `UnsupportedIr`; introduce the
  explicit `LossyImportAndUnsupportedIr` variant only when both blocker sources are present, so the
  caller receives one complete repair list without losing origin information.
- Alternatives rejected: fallback offset normalization, generic raw passthrough, schema-order
  guesses, capturing known modeled wrappers, first-error returns, and collapsing all refusal kinds
  into one generic error.
- Evidence: public RED/GREEN cases cover seven omitted semantic paths, stale offsets in three owners,
  six anchored opaque fragments, XML characters, ten raw-fragment attacks, and mixed blocker
  aggregation. Final targeted results are serializer/parser/CLI 20/26/5, clippy clean, rustfmt clean,
  and whitespace diff clean.

## HML serializer reconstructability and depth parity - S2 exact fixer

- Root causes: the writer accepted redundant public character-shape entries that the reader
  canonicalizes away, and raw-fragment validation counted depth outside the two container levels
  already open in the generated document while failing to apply the same boundary to empty
  elements.
- Decision: derive the exact reader-visible character-shape sequence from emitted text/control run
  boundaries and reject any different public sequence recursively. Reserve the enclosing document
  depth before validating public raw fragments and enforce one shared start/empty boundary.
- Compatibility: lawful fixtures and edits performed through `DocumentCore` remain savable. The
  serializer refuses only states that would reparse to different public IR or exceed the default
  secure reader limit.
- Structural decision: extract warning-to-blocker conversion from `export_hml_native` and split the
  large comparison/mutation test helpers. Changed production functions are at most 50 lines and
  changed nesting remains within four levels.
- Alternatives rejected: silently canonicalizing public `char_shapes`, preserving redundant runs
  through invented zero-width XML, increasing secure-reader limits, or special-casing empty tags at
  serialization time. Each would hide the contract mismatch instead of enforcing it at preflight.
- Evidence: focused tests failed before the fix and pass afterward; final serializer/parser/CLI
  targets are 22/26/5, scoped clippy and formatting checks are clean, and the independent evaluator
  proves blocker, rejection-boundary, and accepted-boundary secure-reparse behavior.

## HML save surfaces - S3 CLI and WASM

- Shared-gate decision: expose `DocumentCore::hml_export_preflight` and make native export, WASM
  metadata, and callers use the same source/import/IR blocker classification. This prevents dialog
  capability metadata from drifting from the export that ultimately runs.
- CLI decision: use a command-specific `-o`/`--output` parser. The conversion verifier parser does
  not understand valued output options and would incorrectly mix `--verify` semantics into HML
  save. Require an explicit distinct output path and write only after parsing and preflight pass.
- Diagnostic decision: refusal is nonzero and prints each stable blocker code, XML path, and
  message. Do not collapse the typed HML error into a generic serialization message.
- WASM decision: add `exportHml` beside `exportHwp`/`exportHwpx`, and extend the existing metadata
  JSON without removing or renaming fields. `hmlSavable` and `saveBlockers` are exact camelCase
  fields computed from the shared preflight for lawful, import-loss, and edited-IR documents.
- Alternatives rejected: trial export to determine dialog state, duplicating warning-only logic in
  WASM, allowing implicit/same-path CLI output, or reusing `parse_conversion_verify_args`.
- Evidence: initial tests failed on the absent preflight, CLI command, and WASM binding. Focused
  results are CLI/serializer/parser 8/23/26 plus three WASM tests, with clippy and repository fmt
  checks clean. WASM package regeneration remains intentionally deferred to S4/S6.

## HML atomic output basename boundary - S3 adversarial fixer

- Root cause: the same-directory temporary filename embedded the complete destination basename.
  A filesystem-valid destination near `NAME_MAX` therefore became invalid only after the helper
  added its process ID, nonce, attempt number, and suffix.
- Decision: make the temporary basename independent of the destination name, using a fixed prefix
  plus process ID, timestamp nonce, and bounded attempt number. This retains same-directory atomic
  rename and `create_new` collision resistance without consuming the destination's name budget.
- Alternatives rejected: truncating the destination basename, which adds encoding and boundary
  complexity without improving uniqueness, and moving the temp file elsewhere, which can break
  same-filesystem atomic rename.
- Evidence: the public near-`NAME_MAX` CLI test changed from `ENAMETOOLONG` to GREEN; binary unit
  tests pass 4/4 and the full HML CLI target passes 12/12. Fresh S3 exact verification remains the
  next gate.

## Studio three-format semantic save policy - S4 builder

- Root cause: `save-target.ts` had introduced `hml | hwp | hwpx`, but `file.ts` still chose
  exporters, MIME types, names, and picker options through the old `isHwpx` boolean. This left the
  TypeScript build with eight errors and caused a marked `.hml` save handle to resolve as HWP on
  the next Ctrl+S.
- Decision: use one `SaveFormat` descriptor for extension, MIME, and picker type, and one exporter
  dispatcher for the corresponding WASM method. Keep first HML save as forced Save As while passing
  the original handle solely for identity and `isSameEntry` rejection; after a successful save,
  derive repeat-save format from the marked handle's actual `.hml`, `.hwp`, or `.hwpx` extension.
- Capability decision: enable HML only when metadata explicitly says `hmlSavable === true`, the
  blocker array is structurally present, and the loaded WASM document exposes `exportHml`. Missing
  or malformed metadata and stale WASM packages fail closed to HWP, with a visible diagnostic and
  blocker paths. Trial export was rejected because metadata and capability are the intended
  preflight surfaces.
- Compatibility decision: explicit HWP/HWPX Save As keeps its previous behavior; only HML-origin
  forced saves protect the source handle. Non-HML Save As therefore does not acquire the HML source
  rejection rule.
- Copy decision: describe HML as a semantic save that preserves meaning but not original bytes;
  when blocked, keep HWP and HWPX visibly available. Existing dialog styling and motion remain
  unchanged.
- Evidence: the converted `.hml` regression failed as HWP before the fix and passed afterward.
  Focused format, picker, capability, dialog-message, and warning suites pass 30 tests; the full
  Studio suite passes 217/217 and `npm run build` completes. The updated browser scenario passes
  `node --check`; its real execution remains deferred until Docker regenerates the WASM package.

## Studio save policy fail-closed review - S4 full-spec improvement

- Root causes: the filesystem save boundary still accepted omitted `saveFormat` and silently
  selected HWP, while a marked `.hml` handle bypassed capability metadata on later Ctrl+S actions.
  The metadata normalizer also accepted any array contents, so a malformed blocker code could
  incorrectly coexist with `hmlSavable: true`.
- Decision: make `SaveFormat` and the force-save decision required inputs at the write boundary;
  do not retain a compatibility default that can disagree with the selected exporter or MIME.
  Re-evaluate HML capability before every HML write, including repeat saves to a marked handle.
- Loss-routing decision: when repeat HML save becomes unavailable, reopen the same format dialog.
  HML stays disabled with diagnostics, HWP is primary, HWPX remains available, and choosing either
  conversion format forces a new picker so binary output cannot reach the `.hml` handle.
- Metadata decision: accept blocker entries only when `code`, `xmlPath`, and `message` are strings;
  Rust owns the code vocabulary, so Studio intentionally accepts any string code instead of a
  duplicated TypeScript enum. A nonempty valid blocker list always disables HML save.
- Alternatives rejected: relying on `exportHml` to throw after the user chooses save, preserving
  `undefined -> hwp` for older callers, or filtering malformed blockers while leaving HML enabled.
  Each would make the UI's preflight claim differ from the actual save boundary.
- Evidence: the focused RED exposed malformed-blocker enablement and the absent repeat-save policy;
  the corrected focused suites pass 28/28, the full Studio suite passes 218/218, production build
  succeeds, E2E syntax and whitespace checks are clean, and graph backward trace is clean.

## Studio save boundary edge hardening - S4 edge improvement

- Root causes: the current-handle branch wrote bytes before checking that the handle extension
  matched the selected `SaveFormat`; metadata/exporter capability probes could throw and skip the
  fail-closed HML format dialog entirely.
- Decision: run the same extension assertion before both current-handle and picker writes. Read
  metadata and exporter capability independently through a fail-closed context so one stale WASM
  probe cannot hide the other probe's useful blocker diagnostics.
- Alternatives rejected: trusting prior handle bookkeeping, because mutable UI state can become
  inconsistent, and catching all errors around the whole save action, because that converts a
  recoverable HML capability failure into a generic failed save instead of offering HWP/HWPX.
- Dialog boundary: the public dialog helper creates a fresh dialog per call; each instance adds its
  format buttons once and guards Promise settlement. No listener-accumulation change was made
  without a reproducible public-path defect.
- Evidence: both focused cases were RED before their fixes; focused save suites pass 31/31, the
  full Studio suite passes 220/220, and the TypeScript/Vite production build succeeds.

## Studio semantic save policy - S4 exact verification

- Decision: accept S4 only after the fresh no-edit adversarial pass and an independent exact pass
  both traced the same required `SaveFormat` from command entry through exporter, MIME, extension,
  picker validation, original-handle identity checks, and state mutation. Unit tests alone were not
  treated as type or production-bundle proof, so the TypeScript/Vite build remained mandatory.
- Fail-closed boundary: malformed or throwing metadata and stale exporter bindings disable HML but
  preserve the HWP/HWPX conversion choices. Repeat `.hml` saves recheck this capability; conversion
  after capability loss forces a picker instead of writing binary bytes into the HML handle.
- Alternatives rejected: accepting prior builder evidence as final verification, treating E2E
  script syntax as browser proof, or claiming generated WASM support from the stale checked-in
  package. Each would overstate a boundary that has not yet run.
- Evidence: focused S4 suites pass 33/33, the complete Studio suite passes 220/220, production build
  succeeds, E2E script syntax, whitespace, and run-state JSON are clean, and graph backward trace is
  clean. S4 passes; Docker WASM regeneration and real S6 browser/release verification remain next.

## HML document open/save - S6 Rust release verification

- Decision: accept the Rust release gate only from a complete latest-source run; the prior partial
  execution was not reused as evidence.
- Environment decision: the all-features Clippy command initially failed before compilation because
  the sandbox could not unpack a missing Cargo registry crate. Rerun the same command with approved
  registry access instead of treating the filesystem denial as a source regression.
- Evidence: rustfmt passed; the full release-test suite completed with 3,110 passed, 22 ignored, and
  0 failed in 375.32 seconds; all-targets/all-features Clippy passed without warnings in 65.20
  seconds; `git diff --check` passed. Browser/WASM and non-Unix platform proof remain separate.

## HML document open/save - S6 WASM and browser verification

- Decision: regenerate the ignored WASM package from the latest Rust source before treating Studio
  tests or script syntax as browser evidence. Verify both the success path and a wasm32 error path:
  semantic HML save/reopen proves real output bytes, while an HWP `exportHml` attempt proves the
  structured refusal crosses the `JsValue` boundary.
- Browser decision: keep the deterministic save E2E on a standards-shaped picker handle so written
  bytes, MIME, extension, edit survival, preserved envelope, and source SHA can be asserted. Use a
  separate `playwright-cli` session for visible-state, network, console, and fixed-frame QA evidence.
- Accessibility root cause: the inherited dialog primary color `#6182d6` gave white 12px text only
  3.70:1 contrast. Scope the correction to the HML save primary action instead of changing the
  repository-wide accent token: normal and hover states now use the existing accessible blues
  `#2563eb` and `#1d4ed8` (5.17:1 and 6.70:1).
- Alternatives rejected: lowering the contrast threshold, calling the button large text, changing
  every dialog accent, or treating the Puppeteer functional run as the required Playwright QA.
- Evidence: Docker wasm-pack completed in 5m04s; generated bindings contain `exportHml`; the real
  save/reopen/SCRIPTCODE/source-SHA/HWP E2E passed twice; Playwright observed HTTP 200, no console
  errors, structured metadata, and `[HML_SOURCE_REQUIRED]`. A routed remote `?url=` load with an
  intentionally misleading `Content-Type: text/html` still detected lawful HML, rendered one page
  and its canvas, retained the warning/final filename state, and kept both WASM and document
  requests at HTTP 200. Contrast passed 9/9; Studio passed 220/220 and production build after the
  UI fix; the browser QA gate passed.

## HML fixture byte-preservation rule

- Root cause: the lawful `aligns.hml` fixture contains source-authored BOM, CRLF, and trailing
  whitespace. Normalizing it to satisfy textual diff checks would invalidate the provenance bytes
  and the source-SHA safety oracle used by the browser test.
- Decision: classify `samples/hml/*.hml` as byte-preserved, non-textual diff fixtures in
  `.gitattributes`. Human-readable provenance and behavior remain documented in the adjacent
  README and inventory; tests continue to parse the exact bytes.
- Alternative rejected: rewriting line endings or trimming spaces, because that changes the test
  subject instead of fixing Git's treatment of the corpus.
