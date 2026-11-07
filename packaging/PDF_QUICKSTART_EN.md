# rhwp 0.9.0 — PDF quick start

This guide ships in the official CLI archives. [한국어](PDF_QUICKSTART.md).
Run from the extracted directory, or add `rhwp` to PATH. On Windows use `.\rhwp.exe` instead of `./rhwp`.

```sh
./rhwp --version
./rhwp capabilities
./rhwp import-pdf input.pdf output.hwpx --json
./rhwp info output.hwpx --json
# Original pages 1, 3 and 4; indexes are zero-based.
./rhwp import-pdf input.pdf selected.hwpx --source-pages 0,2-3 --json
./rhwp import-pdf input.pdf editable.hwpx --layout editable --json
# Linux/macOS: supply the opening password through stdin, never command arguments.
./rhwp import-pdf protected.pdf opened.hwpx --password-stdin < password.txt
./rhwp export-png output.hwpx -p 0 -o preview-png --font-path /path/to/fonts --profile print
./rhwp export-pdf output.hwpx -o preview.pdf --font-path /path/to/fonts
```

PowerShell does not support `< password.txt`. Use Studio's password dialog or host code that safely
supplies standard input. Keep passwords and source documents out of command history, public logs and packages.

PDF import creates an editable document. The default `fidelity` mode prioritizes placement;
`editable` prioritizes flowing paragraphs. Supply actual document fonts when checking the output.

- PDF input, JBIG2, JPEG 2000 and ICC processing are enabled by default. Check the actual build with `capabilities`.
- Standard Security R2–R6 opening passwords are supported. The new HWP/HWPX does not inherit PDF encryption.
- Standard limits include 64 MiB input, 1,024 selected pages, 8,000,000 content operators and 64,000,000 pixels per image. Other cumulative limits also apply.
- For `PDF_RESOURCE_LIMIT`, restore the default `standard` profile and split selected pages if needed. JSON overrides lower limits within the profile ceiling.
- Raising limits does not fix unsupported PDF painting operations. Scanned text is not OCRed; inspect complex masks, rotated text and inferred tables in the destination editor.

See the [PDF developer onboarding guide](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pdf_developer_onboarding_EN.md)
for Rust/npm/WASM, Worker deployment, cancellation, page progress and embedded Studio.
Keep JS, WASM, Worker and Studio on the same release. The repository Worker module is not automatically
included in `@rhwp/core`; `@rhwp/editor` requires a separately hosted Studio.

[CLI reference](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/cli_commands.md) ·
[Support boundaries](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/security_feature_limits.md) ·
[Release and contributors](https://github.com/edwardkim/rhwp/releases/tag/v0.9.0)

Third-party notices: `notices/THIRD_PARTY_LICENSES.md` and the original notice files under `notices/`.
