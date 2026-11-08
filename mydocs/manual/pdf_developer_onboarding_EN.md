---
kind: guide
status: active
canonical: mydocs/manual/pdf_developer_onboarding.md
last_verified: 2026-11-08
---

# PDF import: developer quick start for 0.9.0

[Complete guide and examples (Korean)](pdf_developer_onboarding.md). These instructions describe
the **0.9.0** API and consumption contract. Check each registry or store for availability of that version.
Pin CLI, JS glue, WASM, Worker and Studio to the same release. Initial crates.io publication is included in the 0.9.0 release.

PDF import produces editable Document IR and saves HWP/HWPX. It is not an embedded PDF viewer or OCR.
Scanned text remains an image; reconstructed paragraphs and tables may differ from the original authoring document.

## CLI

Build from the checkout with Rust 1.93.1:

```sh
cargo build --locked --release --bin rhwp --features native-skia
./target/release/rhwp capabilities
./target/release/rhwp import-pdf input.pdf output.hwpx --json
./target/release/rhwp import-pdf input.pdf selected.hwpx --source-pages 0,2-3 --layout editable --json
./target/release/rhwp import-pdf protected.pdf opened.hwpx --password-stdin < password.txt
./target/release/rhwp info output.hwpx --json
```

`export-png -o` takes a directory for per-page PNG files; `export-pdf -o` takes a file.


PDF import, JBIG2, JPEG 2000, ICC conversion and bounded xref recovery are enabled by default.
`native-skia` adds PNG, direct PDF, Sixel and Kitty output. Supply document fonts with `--font-path`
when rendering. A terminal font does not supply fonts to the renderer. Do not redistribute fonts without permission.

Source pages are zero-based, strictly increasing and inclusive at both ends of ranges.
`fidelity` (default) positions recognized body regions in text boxes. `editable` moves recognized body
paragraphs into section flow with columns and page/column breaks; other objects may remain positioned.
Standard Security R2–R6 opening passwords are supported. Empty opening passwords are tried automatically.
Imported output does not inherit PDF encryption. Keep password input files and document diagnostics out of public logs.

## Rust

The 0.9.0 release includes initial crates.io distribution. After publication, use:

```toml
[dependencies]
rhwp = { version = "0.9.0", default-features = false, features = ["pdf-import", "pdf-jbig2", "pdf-jpeg2000", "pdf-icc"] }
```

While publication is pending, replace `version = "0.9.0"` with `path = "/path/to/rhwp"` to consume the matching checkout.

Use `DocumentCore::from_pdf_bytes_with_policy(bytes, &options, policy)` with `PdfImportOptions` and
`PdfImportPolicy::for_profile(PdfImportProfile::Constrained)`. Serialize `core.document()` with
`rhwp::serializer::serialize_hwpx`. See the [complete Rust example](pdf_developer_onboarding.md#2-rust-라이브러리).
The direct API is synchronous and does not automatically provide process isolation. For untrusted uploads,
use a supervised worker; see the bundled [isolation example](../../packaging/crates-io/examples/pdf_isolation_host.rs).

## JavaScript / WASM

After npm publication, install `@rhwp/core@0.9.0` and serve its WASM file alongside matching JS glue:

```js
import init, { HwpDocument } from '@rhwp/core';
await init({ module_or_path: '/rhwp_bg.wasm' });
const pdfBytes = new Uint8Array(await (await fetch('/input.pdf')).arrayBuffer());
const doc = HwpDocument.importPdf(pdfBytes, {
  sourcePages: [0], layout: 'fidelity', resourceProfile: 'constrained',
});
try {
  const hwpx = doc.exportHwpx();
} finally {
  doc.free();
}
```

This API is synchronous: use a module Worker for large files, cancellation and responsive UI.
The repository's `rhwp-shared/pdf/PdfImportSession` is **not an npm export**. Serve `rhwp-shared/pdf/`
and `pkg/` at their original relative paths, on the same origin, with module Workers allowed by CSP.
See the [working Worker example](pdf_developer_onboarding.md#취소와-쪽별-진행률이-필요한-앱).
Progress supplies `sourcePage` (one-based), `sourcePageCount`, `completedPages` and `totalPages`.
The percentage uses selected pages; 100% page processing may precede Core construction and saving.
Cancellation terminates the candidate Worker; a subsequent import creates a new candidate.

For an editor UI, `@rhwp/editor` embeds Studio via `viewerUrl`. Host matching 0.9.0 Studio assets and
use `editor.loadFile(pdfBytes, 'input.pdf', { suppressDialogs: false })`. PDF settings/password dialogs
still require user input. The iframe request timeout and PDF Worker timeout are separate.
VS Code retains `.hwp`/`.hwpx` file associations; open the converted document there.

## Limits and migration

The standard profile allows 64 MiB input, 1,024 selected pages, up to 8 million content operators and
64 million pixels per image. Independent buffer/work/output limits still apply. On resource errors,
restore the standard profile and remove lower overrides, then reduce the selected range if necessary.
Custom JSON only lowers profile limits. CLI accepts `{ "policy": { ... }, "execution": { ... } }`;
WASM `resourceLimits` takes the **inner policy object**. Studio timeout defaults to 30 seconds and
can be set to 1–300 seconds. These are not global memory/RSS guarantees.

`PDF_PASSWORD_REQUIRED`/`PDF_PASSWORD_INVALID` require a password or retry. `PDF_*_UNSUPPORTED`
indicates unsupported content/authentication, not a quota that can be raised. Some clipping/mask,
rotated text, table reconstruction and encryption combinations remain unsupported. Other editors may
not preserve RHWP text-paint extensions. Verify results in the target application.

[Resource policy](pdf_resource_policies.md) · [Codecs and interoperability](pdf_image_codecs.md) ·
[Security boundaries](security_feature_limits.md) · [Rust distribution contract](crates_io_guide.md).
