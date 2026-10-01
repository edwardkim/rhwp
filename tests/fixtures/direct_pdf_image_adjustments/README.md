# Direct PDF image adjustment evidence

These representative images use the existing public fixture
`samples/2025 행정업무운영 편람(최종).hwp`, physical page 3 (`-p 2`).
The source HWP is unchanged; SHA-256:
`40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`.

The existing reference is
`pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`, SHA-256
`1ff2f5b3158c902700c40dde4afa258bc3fef3df61b7081e0d50027c94007cec`.
Its Creator identifies Hwp 2022 and its Producer identifies Hancom PDF
1.3.0.550 despite the filename suffix. Both the reference and parsed source
have 384 pages. The comparison uses matching page content, not a renamed or
regenerated reference PDF.

`native_review_003.png` and `native_overlay_003.png` compare CLI SVG rendered
by Chrome with the Hancom reference. `wasm_review_003.png` and
`wasm_overlay_003.png` use SVG and render-tree data from the freshly built
WASM document. These are compatibility evidence; they do not exercise direct
PDF replay. `direct_pdf_review_003.png` and `direct_pdf_overlay_003.png`
separately show the actual direct PDF replay against the existing SVG image
semantics and the Hancom reference.

The adjustment regression expectations follow normalized Skia matrix offsets
and SVG's default linearRGB filter space. They use the independent sRGB
transfer equations in CSS Color 4, corroborated with Chrome SVG samples.
Zero-adjustment and already-baked image payloads retain their existing path.

Validation commands, with a prepared review worktree and native-skia CLI:

```sh
node scripts/run-rust-test.mjs issue_4764_pdf_raster_fidelity -- \
  --cargo-profile release-test --features native-skia --target-dir <shared-target>
rhwp export-pdf 'samples/2025 행정업무운영 편람(최종).hwp' \
  --backend direct --profile print --font-path ttfs/opensource -p 2 -o page3.pdf
python3 scripts/visual_sweep.py \
  --file-target handbook-page3 'samples/2025 행정업무운영 편람(최종).hwp' \
  'pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf' \
  --rhwp-bin <native-cli> --pages 3 --dpi 96 --out <output>
# Repeat the sweep with --wasm-pkg <fresh-pkg> for WASM evidence.
```

The Hancom page uses Haansoft Batang. The native environment substitutes
Noto Sans KR/Noto Sans CJK KR; the SVG/WASM environment uses the repository's
webfont rules. Font appearance and inherited title clipping differ: the HWP
and existing SVG show `행정업무운영 편람`, whereas the reference visibly clips
the final word. The reference's gray panel also differs from the existing SVG
baseline. This change preserves the SVG image-filter semantics and does not
rewrite document text, font rules, or brightness/contrast metadata to fit the
reference.

This is a focused brightness/contrast improvement, not complete direct-PDF
support for the handbook. Full-document replay still rejects the connector
line style on page 63; selecting `-p 62` reproduces that independent guard.
Unbaked image watermark tone/opacity, the special RealPic page-background
preset, and other unsupported effects remain fail-closed.
