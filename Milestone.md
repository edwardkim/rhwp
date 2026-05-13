# HWP Writer Milestones

Target branch: `ios/devel`

This file tracks the Swift/iOS writer lane. Check an item only when the named
test, script, or manual gate exists and the result is recorded here or in the
PR.

## Direction

- Primary generation target is `.hwp`, not `.hwpx`.
- Keep HWPX as an input/conversion reference lane, not the first writer output.
- Follow TDD: write the failing test or smoke first, make the narrowest change,
  then refactor only after the proof is green.
- Treat rhwp self-roundtrip as necessary but insufficient. HWP writer completion
  needs Hancom 2020 or Hancom 2022 manual-open evidence before claiming
  compatibility.
- Keep Swift/iOS work based on `ios/devel`.

## Current Baseline

- [x] Swift native read bindings landed in upstream `devel` and were synced to
  `ios/devel`.
- [x] `bindings/swift` exposes `Rhwp.readText(inputFile:page:)` and
  `RhwpDocumentTextView`.
- [x] `bindings/swift/Examples/read_text_ffi.swift` documents direct
  `rhwp_read_text` usage.
- [x] Rust already has the HWP writer entrypoint:
  `src/serializer/cfb_writer.rs::serialize_hwp`.
- [x] WASM already exposes the writer as `HwpDocument.exportHwp()` and
  verification metadata as `HwpDocument.exportHwpVerify()`.
- [x] iOS simulator runtime smoke for the AlHangeul app is reproducible through
  `scripts/ios-smoke-alhangeul.sh`.
- [ ] HWP writer API for Swift/iOS has a byte-returning native ABI.

## Milestone -1: Baseline Inventory

Goal: keep future work grounded in the writer that already exists.

- [x] Record HWP writer entrypoint: `serialize_hwp(doc)`.
- [x] Record public serializer API: `DocumentSerializer`, `HwpSerializer`,
  `HwpxSerializer`, `serialize_document(doc)`.
- [x] Record WASM export API: `exportHwp`, `exportHwpx`, `exportHwpVerify`.
- [x] Record current iOS gap: Swift bindings read/export text but do not return
  generated HWP bytes.
- [ ] Add the first new failing test in the existing writer regression surface,
  preferably `src/serializer/cfb_writer/tests.rs` or a narrowly scoped
  integration test under `tests/`.

## Milestone 0: iOS Test App Harness

Goal: prove the Apple lane can run real native rhwp calls on an iOS simulator
without depending on the production app flow.

- [ ] Add a small iOS smoke target or clearly separated smoke screen under
  `rhwp-ios`.
- [x] Build the native library for Apple Silicon simulator in a script.
- [x] Launch the smoke app on an iOS simulator and render a bundled HWPX sample
  through the native iOS FFI path.
- [x] Save the reproduction command in `scripts/`.
- [ ] Record the simulator, command, and result in the PR body.

Suggested first proof:

```sh
scripts/ios-smoke-alhangeul.sh
```

Latest local proof:

- Date: 2026-05-13
- Simulator: iPhone 17, iOS 26.5 (`566832F4-2C84-4145-8D73-1CEEDA1B9B4E`)
- Result: `sample.hwpx` bundled, app launched, screenshot showed page `1 / 70`
  rendered.

## Milestone 1: HWP Writer Red Tests

Goal: define the writer contract before adding Swift surface area.

- [ ] Add a Rust test that creates an empty document, exports `.hwp`, reloads it
  with `HwpDocument::from_bytes`, and asserts page count and stream presence.
- [ ] Add a Rust test that inserts Korean/English mixed text, exports `.hwp`,
  reloads it, and asserts extracted text contains the inserted text.
- [ ] Add stream-level invariants for FileHeader, DocInfo, BodyText/Section0,
  BinData, and compression behavior where relevant.
- [ ] Add source-format boundary coverage: HWP source is allowed; HWPX source is
  blocked or routed only through a full converter milestone.
- [ ] Add a Native binding test for the missing writer ABI. The first version
  should fail because the symbol does not exist yet.
- [ ] Keep the tests focused on `.hwp` bytes and CFB structure, not HWPX ZIP
  output.

Proposed ABI shape to test first:

```c
unsigned char *rhwp_write_text_hwp(const char *text, size_t *out_len);
void rhwp_bytes_free(unsigned char *ptr, size_t len);
```

## Milestone 2: Minimal HWP Generation

Goal: make the smallest `.hwp` that rhwp can reload and Hancom can open.

- [ ] Implement the native byte-returning writer ABI.
- [ ] Generate a one-page `.hwp` from plain UTF-8 text.
- [ ] Return structured errors for null text, invalid UTF-8, and serialization
  failure.
- [ ] Verify CFB signature, required streams, and rhwp reload.
- [ ] Run Hancom manual-open verification before marking compatibility.
- [ ] Store generated smoke artifacts under a deterministic output folder and
  reference that path in the PR.

## Milestone 3: Swift Writer API

Goal: make HWP generation natural for Swift callers.

- [ ] Add `Rhwp.writeTextHwp(text:) throws -> Data`.
- [ ] Add Swift tests that call the native writer and verify the returned data
  starts with the HWP CFB signature.
- [ ] Add a Swift test that writes the data to a temporary `.hwp`, then reads it
  back through `Rhwp.readText`.
- [ ] Document the API in `bindings/swift/README.md`.

## Milestone 4: Test App Create Flow

Goal: prove the writer is usable from an iOS user flow.

- [ ] Add a smoke UI path that enters text and generates `.hwp` data.
- [ ] Save or share the generated `.hwp` through the iOS document flow.
- [ ] Re-open the generated file in the smoke app and display extracted text.
- [ ] Keep the UX minimal; this is a verification app, not the full editor.

## Milestone 5: Writer Expansion

Goal: expand only after the plain-text path is proven.

- [ ] Paragraph breaks.
- [ ] Basic page settings.
- [ ] Character style defaults.
- [ ] Tables.
- [ ] Images.
- [ ] Picture in table.
- [ ] HWPX-to-HWP conversion, only after the HWP-origin writer path has
  compatibility evidence.

## Do Not Reopen

- Do not start with HWPX generation for the Swift/iOS writer lane.
- Do not treat `from_bytes(export_hwp(doc))` as Hancom compatibility proof.
- Do not use a simple HWPX-to-HWP adapter as the writer foundation.
- Do not mix production iOS app redesign with writer smoke verification.
