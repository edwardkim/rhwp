# AMO Reviewer Notes for Task #338

Add-on: rhwp Firefox extension

Issue: https://github.com/edwardkim/rhwp/issues/338

## Summary

This update resolves the Firefox AMO validation and review warnings tracked in issue #338.

The changes address:

- the manifest compatibility warning involving `strict_min_version` and `data_collection_permissions`
- stale build artifacts from previous extension builds
- the previous `document.write` usage in the print feature
- remaining `innerHTML` occurrences in the viewer bundle and content script
- the previously reported dynamic `Function` warning, which is no longer reproduced in the rebuilt package

The extension declares no data collection and processes HWP/HWPX documents locally in the browser.

This note is intended for the AMO "Notes for Reviewers" field and therefore describes the submitted extension package. It intentionally does not include a repository build command.

If a source code package is submitted for the bundled viewer code, reproducible build instructions should be provided in the README included with that source submission.

## Manifest Compatibility

The extension keeps:

```json
"data_collection_permissions": {
  "required": ["none"]
}
```

This field is intentionally retained because AMO requires explicit disclosure of data collection behavior.

To make the manifest version requirements consistent with that field, `browser_specific_settings.gecko.strict_min_version` was raised from `112.0` to `142.0`.

Verified before packaging in the source manifest and after packaging in the submitted package manifest:

- source manifest: `rhwp-firefox/manifest.json`
- submitted package manifest: `manifest.json`

Both now contain:

```json
"strict_min_version": "142.0",
"data_collection_permissions": {
  "required": ["none"]
}
```

## Build Artifact Cleanup

The Firefox build script now removes the previous `dist/` directory before generating a new package.

This prevents old hashed viewer bundles and stale test artifacts from being included in the submitted extension package.

After rebuilding, the generated extension package contains only the current viewer assets:

```text
assets/rhwp_bg-DcCngJ7I.wasm
assets/viewer-Dk56CfXJ.js
assets/viewer-Di8-R0fz.css
```

No `*.map`, `*test*`, or `test/` files remain in the submitted package.

## Removed `document.write`

The previous print feature used `printWin.document.write(...)` in:

```text
rhwp-studio/src/command/commands/file.ts
```

That path has been rewritten.

The print document is now assembled with DOM APIs:

- the print window document is built through `document.createElement`
- file names and labels are inserted with `textContent`
- SVG pages are parsed with `DOMParser` as `image/svg+xml`
- parsed SVG nodes are imported into the print window document
- no inline script is injected

Verification:

```bash
grep -R -l --exclude='*.wasm' 'document\.write' .
```

Result: no matches.

## Dynamic `Function` Warning

The rebuilt package does not reproduce the previously reported dynamic `Function` constructor warning.

Verification:

```bash
grep -R -l -E --exclude='*.wasm' 'new Function|Function\(' .
```

Result: no matches.

## Removed `innerHTML`

The rebuilt package no longer contains `innerHTML` string matches.

Verification:

```bash
grep -R -l --exclude='*.wasm' 'innerHTML' .
```

Result: no matches.

The relevant source paths were refactored as follows:

- external command menu labels and shortcuts now use DOM nodes and `textContent`
- canvas container clearing now uses `replaceChildren()`
- table and object selection overlays now use DOM/SVG APIs
- shape, line, polygon, and arc placement previews now use DOM/SVG APIs
- content script comments no longer contain the static analysis keyword
- editor dialogs, dropdowns, and preview widgets now use DOM APIs, `replaceChildren()`, `textContent`, `option` elements, or SVG XML parsing/import instead of `innerHTML`

## Validation Commands

The submitted extension package was checked with the following commands from the package root:

```bash
grep -R -n -E '"strict_min_version"|"data_collection_permissions"' manifest.json
find . -path '*/test/*' -o -name '*test*' -o -name '*.map'
grep -R -l --exclude='*.wasm' 'document\.write' .
grep -R -l -E --exclude='*.wasm' 'new Function|Function\(' .
grep -R -l --exclude='*.wasm' 'innerHTML' .
```

Expected results:

- the manifest shows `strict_min_version: "142.0"` and `data_collection_permissions`
- the `find` command prints no stale test or source map files
- the three `grep -R -l` commands print no matches

Note: `grep` may exit with status `1` when no matches are found. In this case, no output is the expected result.

## Manual Firefox Validation

The updated Firefox extension was loaded and manually tested in a local Firefox window.

The final build was compared against a pre-change build served separately in the local test environment. The concrete local ports are intentionally omitted because they are not required for AMO review.

Validated behavior:

- content script badge rendering and viewer launch
- HWP document loading with `samples/복학원서.hwp`
- print popup rendering after replacing `document.write`
- previous document pages are cleared when opening a new document or replacing the document
- table object and image selection overlays render correctly
- line, rectangle, ellipse, arc, and polygon overlays render correctly
- shape placement previews and size labels render correctly
- shape picker, symbol dialog, table picker, and paragraph dialog display equivalently before and after the DOM API refactor

Known note:

- Table object handles are rendered and hover cursors change correctly.
- Dragging those handles does not resize the entire table object in current `upstream/devel`.
- Source review shows that table object handle hit testing is currently used for cursor changes only. Table cell, row, and column resizing is implemented separately through the table cell resizing path.
- This is not a regression from the DOM API refactor.

## Privacy Statement

The extension declares:

```json
"data_collection_permissions": {
  "required": ["none"]
}
```

HWP/HWPX documents are processed locally in the browser. The extension does not collect or transmit user document contents to a remote service.
