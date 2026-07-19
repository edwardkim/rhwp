---
kind: snapshot
status: historical
canonical: mydocs/manual/branding_strategy.md
last_verified: 2026-07-17
---

# rhwp Logo Generation — AI Prompts

## Brand Core

- **rhwp** = "R" (sounds like Korean "알" = "All") + HWP
- "All HWP" — open document format for everyone
- Egg (알) — birth, origin, potential, breaking free
- "ㅎ" — first consonant of "한글" (Hangul, Korean writing system)
- Precision — pixel-perfect document typesetting
- Rust + WebAssembly — modern, reliable technology

## Prompts

### Prompt 1: The Hatching ㅎ

```
A minimal, modern logo design for an open-source document viewer called "rhwp".

The logo depicts a smooth egg shape with a subtle crack line running diagonally across it. Through the crack, the Korean character "ㅎ" (a single Hangul consonant, looks like a small circle above a horizontal line with two vertical strokes below) is visible, glowing softly from inside. The egg shell is pearl white with a slight golden shimmer. The "ㅎ" inside is rendered in deep navy blue (#1B3A6B).

Clean flat design, no gradients except a subtle inner glow. Suitable for both light and dark backgrounds. Square format, works at 16x16 favicon size and 512x512. No text, just the symbol.

Style: geometric, minimal, professional. Similar quality to Mozilla, Rust Foundation, or JetBrains logos.
```

### Prompt 2: R + Egg Monogram

```
A sophisticated monogram logo combining the letter "R" and an egg silhouette for an open-source project called "rhwp" (meaning "All HWP" — a Korean document format viewer).

The letter "R" is stylized so its curved stroke forms the outline of an egg. The counter space (hole) of the R contains a subtle Korean character "ㅎ" (a small circle above a horizontal line). Color palette: navy blue (#1B3A6B) for the main form, warm orange (#E8731A) accent on the egg crack line, white negative space.

Flat vector design, no shadows, no 3D effects. Geometric and balanced. Professional tech company quality. Works as app icon, favicon, and print logo.
```

### Prompt 3: Open Egg — The Document Emerges

```
A logo for "rhwp" — an open-source Korean document (HWP) viewer and editor.

Concept: An egg seen from above, cracked open in half. Inside the egg, instead of yolk, there is a small document page with faint horizontal text lines visible. The two shell halves frame the document. The crack line forms a subtle "R" shape.

Colors: Shell in warm ivory/cream, document in clean white with light blue text lines, background transparent. Accent: thin rust-orange (#E8731A) line along the crack edge.

Ultra-minimal, flat, vector style. App icon friendly — must be recognizable at 32x32 pixels. No literal text in the logo, only implied text lines on the tiny document.
```

### Prompt 4: Pure Typography

```
A typographic logo for "rhwp" — an open-source document platform.

The four letters "rhwp" in a custom geometric sans-serif typeface. The "r" is slightly larger and styled as a standalone mark — its stem curves to suggest an egg shape. A hairline crack runs through the "r" from top-right to bottom-left.

Color: Single color, deep navy (#1B3A6B). The crack line in rust orange (#E8731A). Kerning is tight but not touching. Weight is medium — confident but not heavy.

Vector, flat, scalable. Corporate identity quality — suitable for a GitHub README header, app store banner, and business cards.
```

### Prompt 5: Abstract — The Moment of Opening

```
Abstract logo mark for "rhwp", an open-source project that opens closed document formats for everyone.

Two curved shapes facing each other, like an egg split in two, with a narrow gap of light between them. The gap forms the negative space of the Korean character "ㅎ" (a circle above a horizontal line). The overall silhouette is circular/oval.

Left half: navy blue (#1B3A6B). Right half: rust orange (#E8731A). Gap: white/transparent. The two halves don't touch — the opening between them is the core message.

Geometric precision, perfect symmetry, flat vector. No texture, no gradient. Think Paul Rand, Saul Bass — timeless simplicity. Must work as monochrome (single color) version too.
```

## Usage Guide

| Use Case | Recommended Prompt | Size |
|----------|-------------------|------|
| VS Code extension icon | 1 or 2 | 128x128 |
| GitHub repository | 4 (typography) | banner 1280x640 |
| Favicon | 1 or 5 | 32x32, 16x16 |
| Open VSX / Marketplace | 2 or 3 | 256x256 |
| Social media | 5 (abstract) | 1:1 square |
| Document header | 4 with tagline | landscape |

## Tips by AI Tool

| Tool | Tips |
|------|------|
| Midjourney | Append `--style raw --ar 1:1 --v 6`. Use `--style raw` for logos. |
| DALL-E 3 | Use prompts as-is. Add "I NEED this exact design" for better adherence. |
| Stable Diffusion | Negative prompt: "realistic, photo, 3d, gradient, shadow, text" |
| Ideogram | Strong at text-in-logo. Best for Prompt 4. |
| Adobe Firefly | Supports vector style. Add "vector flat logo" keyword. |

## Post-Generation

1. Use AI output as **reference only** — redraw final logo in vector (Figma, Illustrator)
2. Export as SVG for crisp rendering at all sizes
3. Create single-color version (black/white) for favicon and print
4. Create inverted version for dark mode backgrounds
5. Test readability at 16x16, 32x32, 128x128, and 512x512
