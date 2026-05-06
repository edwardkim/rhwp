# rhwp export-png Command Manual

## Overview

A native Skia raster backend tool for exporting HWP documents to PNG images. Introduced in PR #599 as a `PageLayerTree`-based raster output. Unlike SVG, it produces deterministic raster output at the pixel level — useful for printing, image verification, and public-asset pipelines, and especially for **AI pipelines + VLM (Vision-Language Model) integration**.

## Prerequisites

- **`native-skia` feature build required** (not included in default builds)
- Build: `cargo build --release --features native-skia`

## Usage

```bash
rhwp export-png <file.hwp> [options]
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--output <dir>` | `-o` | Output directory (default: `output/`) |
| `--page <num>` | `-p` | Export only a specific page (0-based) |
| `--font-path <path>` | | Font file search path (can be specified multiple times) |
| `--scale <factor>` | | Render scale (default: 1.0) |
| `--max-dimension <px>` | | Maximum pixels per side (longest edge). Auto scale calculation |
| `--vlm-target <preset>` | | VLM (Vision-Language Model) input preset |

### VLM Presets

For AI pipelines + VLM integration. `--vlm-target` automatically adjusts input specifications.

| Preset | Max edge | Pixel limit | Notes |
|---|---|---|---|
| `claude` | 1568 px | 1.15 MP | Claude Vision (Anthropic) compliant |

Other VLM presets (GPT-4V / Gemini / Qwen-VL / LLaVA) are tracked as a follow-up task in [issue #613](https://github.com/edwardkim/rhwp/issues/613).

### Option Priority

1. **`--vlm-target <preset>`** — Highest priority. Auto-applies max-dimension + max-pixels
2. **`--max-dimension <px>`** — Explicit limit. Auto-calculates scale (page → fits within limit)
3. **`--scale <factor>`** — Direct scale factor. Overrides max-dimension when specified
4. (no option) — Default (scale 1.0, page native size)

When multiple options are specified, the smallest result is chosen (safety-first).

### Examples

```bash
# Export all pages to PNG (default)
rhwp export-png samples/exam_kor.hwp

# Export only a specific page (page 17 = index 16)
rhwp export-png samples/exam_kor.hwp -p 16

# Specify output directory
rhwp export-png samples/exam_kor.hwp -o my_output/

# When Hancom-specific fonts (HY견명조 etc.) are not in the system, specify ttfs directory
rhwp export-png samples/exam_kor.hwp --font-path /home/edward/mygithub/ttfs

# Specify multiple font directories
rhwp export-png samples/exam_kor.hwp \
  --font-path /home/edward/mygithub/ttfs \
  --font-path /usr/share/fonts/truetype/nanum

# High resolution (2x scale, for printing)
rhwp export-png samples/exam_kor.hwp --scale 2.0

# Limit to 1024 pixels per side (LLaVA-style)
rhwp export-png samples/exam_kor.hwp --max-dimension 1024

# Claude Vision input (auto-adjusted to 1568 px / 1.15 MP)
rhwp export-png samples/exam_kor.hwp --vlm-target claude

# AI pipeline integration (Claude + Hancom fonts)
rhwp export-png samples/exam_kor.hwp \
  --vlm-target claude \
  --font-path /home/edward/mygithub/ttfs \
  -o output/claude_input/
```

### Output Dimension Examples (exam_kor page 17, native 1123 × 1588)

| Options | Output dimension | pixel count |
|---|---|---|
| (default) | 1123 × 1588 | 1.78 MP |
| `--scale 2.0` | 2246 × 3175 | 7.13 MP |
| `--scale 0.5` | 562 × 794 | 0.45 MP |
| `--max-dimension 1024` | 725 × 1024 | 0.74 MP |
| `--vlm-target claude` | 898 × 1269 | 1.14 MP (≤1.15 MP) |

## Output Filename Rules

- Single page (with `-p`): `{filename}.png`
- All pages: `{filename}_001.png`, `{filename}_002.png`, ...

Page numbers start at 1 (user-friendly), internal indices start at 0 (`-p` option).

## Font Fallback Behavior

This tool searches for fonts in the following order:

### 1. User-specified (`--font-path`) — Highest priority

All TTF/OTF/TTC files in directories specified via `--font-path` are loaded into memory. If a typeface matches CharShape.font_family (e.g., "HY견명조"), it is used first.

Recommended path for this environment: `/home/edward/mygithub/ttfs` (contains many Hancom-specific fonts)

### 2. System FontMgr — Korean fallback chain

If CharShape.font_family is not found in the system, the following fallback order is used:

```
[CharShape.font_family,]
Noto Sans KR,
Noto Serif KR,
Noto Sans CJK KR,
Noto Serif CJK KR,
Nanum Gothic,
Nanum Myeongjo,
Malgun Gothic,
맑은 고딕,
Batang,
바탕,
Apple SD Gothic Neo,
AppleMyungjo,
DejaVu Sans,
Arial,
sans-serif
```

### 3. Skia legacy typeface — Final fallback

If all of the above fail, Skia's `legacy_make_typeface` is called. If the system has no Korean-glyph-bearing fonts at all, characters are displayed as squares (tofu).

### Per-character Fallback (Whitespace Tofu Prevention)

Text is rendered character by character. If the primary typeface lacks a glyph for a codepoint (`unichar_to_glyph == 0`), other typefaces in the chain are tried. If no typeface has the glyph, no visible glyph is drawn (preventing tofu for whitespace like NBSP / U+2007 / U+200B).

## Hancom-specific Font Support

Hancom-specific fonts used by Hancom Office (HY견명조, HY헤드라인M, HY견고딕, etc.) may not be auto-registered with the system fontconfig. To accurately reproduce Hancom visuals in PNG:

```bash
rhwp export-png input.hwp --font-path /path/to/ttfs
```

If the ttfs directory contains Hancom-specific fonts, they are matched automatically.

## Output Format

Each PNG:
- **Format**: PNG (RGBA, 8-bit)
- **DPI**: 96 (matches default SVG output)
- **Renderer**: native Skia (skia-safe 0.x)
- **Size**: page size × scale (default 1.0)

## Build Guide

```bash
# Debug build
cargo build --features native-skia

# Release build (recommended)
cargo build --release --features native-skia

# native-skia tests
cargo test --features native-skia skia --lib
```

## Non-Goals (Current PR #599 Stage)

The following are unsupported at the current stage — candidate follow-up tasks:

- Complex text shaping (kerning / GSUB / GPOS)
- Full equation native replay (currently placeholder/fallback)
- raw-svg / form object native replay (currently placeholder)
- CanvasKit (browser/WASM) PNG export
- Skia visual regression fixture pipeline
- Additional VLM presets ([issue #613](https://github.com/edwardkim/rhwp/issues/613))
- DPI metadata option (`--dpi`, [issue #614](https://github.com/edwardkim/rhwp/issues/614))

## Unit Conversion Reference

| Conversion | Formula |
|------|------|
| HWPUNIT → mm | `hu × 25.4 / 7200` |
| HWPUNIT → px (96DPI) | `hu × 96 / 7200` |

## Troubleshooting

### Korean characters appear as squares (tofu)

The system has no Korean fonts, or CharShape.font_family is not registered with system fontconfig.

**Solution:**
1. Specify a Korean font directory with `--font-path` (recommended)
2. Install Korean fonts on your system: Noto Sans KR / Nanum / Apple SD Gothic Neo, etc.
3. Verify system Korean fonts: `fc-list :lang=ko`

### export-png command not recognized

```
Error: export-png command requires the native-skia feature.
```

**Solution:** Rebuild with `cargo build --release --features native-skia`.

### LAYOUT_OVERFLOW warnings

```
LAYOUT_OVERFLOW: page=N, col=M, para=K, ...
```

A warning that page body area was exceeded. Does not affect PNG output, but is shown for layout integrity verification. If it's a known baseline area for the current environment, it can be ignored.

## Related Commands

- `rhwp export-svg` — SVG output (CSS font chain, system font fallback)
- `rhwp dump` — Dump typesetting control structure
- `rhwp dump-pages` — Dump pagination results

## References

- This tool's primary domain: PR #599 (refs #536 — Multi-renderer support tracking issue)
- DTP engine identity (`project_dtp_identity`) — Foundation for multi-layer / WebGPU / master pages
- `feedback_image_renderer_paths_separate` — SVG (`svg.rs`) / Canvas (`web_canvas.rs`) / Skia (`skia/renderer.rs`) have separate image functions; check all paths when fixing visual defects
