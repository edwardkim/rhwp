<p align="center">
  <img src="assets/logo/logo-256.png" alt="rhwp logo" width="128" />
</p>

<h1 align="center">rhwp</h1>

<p align="center">
  <strong>All HWP, Open for Everyone</strong><br/>
  <em>Open-source HWP document viewer & editor — Rust + WebAssembly</em>
</p>

<p align="center">
  <a href="https://github.com/edwardkim/rhwp/actions/workflows/ci.yml"><img src="https://github.com/edwardkim/rhwp/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://edwardkim.github.io/rhwp/"><img src="https://img.shields.io/badge/Demo-GitHub%20Pages-blue" alt="Demo" /></a>
  <a href="https://www.npmjs.com/package/@rhwp/core"><img src="https://img.shields.io/npm/v/@rhwp/core?label=npm" alt="npm" /></a>
  <a href="https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode"><img src="https://img.shields.io/badge/VS%20Code-Marketplace-007ACC" alt="VS Code" /></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.75%2B-orange.svg" alt="Rust" /></a>
  <a href="https://webassembly.org/"><img src="https://img.shields.io/badge/WebAssembly-Ready-blue.svg" alt="WASM" /></a>
</p>

<p align="center">
  <a href="README.md">한국어</a> | <strong>English</strong>
</p>

---

Open **HWP files anywhere**. Free, no installation required.

**HWP** is the dominant document format in South Korea — used by government agencies, schools, courts, and most organizations. Until now, there has been no viable open-source solution to read or edit these files.

rhwp changes that. Built with Rust and compiled to WebAssembly, it renders HWP documents directly in the browser with accuracy that matches (and sometimes exceeds) the proprietary viewer.

> **[Live Demo](https://edwardkim.github.io/rhwp/)** | **[VS Code Extension](https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode)** | **[Open VSX](https://open-vsx.org/extension/edwardkim/rhwp-vscode)**

<p align="center">
  <img src="assets/screenshots/render-example-1.png" alt="rhwp rendering example — KTX route map" width="700" />
</p>

## npm Packages — Use in Your Web Project

### Embed a Full Editor (3 lines)

Embed the complete HWP editor in your web page — menus, toolbars, formatting, table editing, everything included.

```bash
npm install @rhwp/editor
```

```html
<div id="editor" style="width:100%; height:100vh;"></div>
<script type="module">
  import { createEditor } from '@rhwp/editor';
  const editor = await createEditor('#editor');
</script>
```

### HWP Viewer/Parser (Direct API)

Use the WASM-based parser/renderer directly to render HWP files as SVG.

```bash
npm install @rhwp/core
```

```javascript
import init, { HwpDocument } from '@rhwp/core';

globalThis.measureTextWidth = (font, text) => {
  const ctx = document.createElement('canvas').getContext('2d');
  ctx.font = font;
  return ctx.measureText(text).width;
};

await init({ module_or_path: '/rhwp_bg.wasm' });

const resp = await fetch('document.hwp');
const doc = new HwpDocument(new Uint8Array(await resp.arrayBuffer()));
document.getElementById('viewer').innerHTML = doc.renderPageSvg(0);
```

| Package | Purpose | Install |
|---------|---------|---------|
| [@rhwp/editor](https://www.npmjs.com/package/@rhwp/editor) | Full editor UI (iframe embed) | `npm i @rhwp/editor` |
| [@rhwp/core](https://www.npmjs.com/package/@rhwp/core) | WASM parser/renderer (API) | `npm i @rhwp/core` |

## Features

### Parsing
- HWP 5.0 binary format (OLE2 Compound File)
- HWPX (Open XML-based format)
- Sections, paragraphs, tables, textboxes, images, equations, charts
- Header/footer, master pages, footnotes/endnotes

### Rendering
- **Paragraph layout**: line spacing, indentation, alignment, tab stops
- **Tables**: cell merging, border styles (solid/double/triple/dotted), cell formula calculation
- **Multi-column layout** (2-column, 3-column, etc.)
- **Paragraph numbering/bullets**
- **Vertical text**
- **Header/footer** (odd/even page separation)
- **Master pages** (Both/Odd/Even, is_extension/overlap)
- **Object placement**: TopAndBottom, treat-as-char (TAC), in-front-of/behind text
- **Image crop & border rendering**

### Equation
- Fractions (OVER), square roots (SQRT/ROOT), subscript/superscript
- Matrices: MATRIX, PMATRIX, BMATRIX, DMATRIX
- Cases, alignment (EQALIGN), stacking (PILE/LPILE/RPILE)
- Large operators: INT, DINT, TINT, OINT, SUM, PROD
- 15 text decorations, full Greek alphabet, 100+ math symbols

### Pagination
- Multi-column document column/page splitting
- Table row-level page splitting (PartialTable)
- shape_reserved handling for TopAndBottom objects
- vpos-based paragraph position correction

### Output
- SVG export (CLI)
- Canvas rendering (WASM/Web)
- Debug overlay (paragraph/table boundaries + indices + y-coordinates)

### Web Editor
- Text editing (insert, delete, undo/redo)
- Character/paragraph formatting dialogs
- Table creation, row/column insert/delete, cell formula
- hwpctl-compatible API layer (Hancom WebGian compatible)

### hwpctl Compatibility
- 30 Actions: TableCreate, InsertText, CharShape, ParagraphShape, etc.
- ParameterSet/ParameterArray API
- Field API: GetFieldList, PutFieldText, GetFieldText
- Template data binding support

## Quick Start (Build from Source)

### Requirements
- Rust 1.75+
- Docker (for WASM build)
- Node.js 18+ (for web editor)

### Native Build

```bash
cargo build                    # Development build
cargo build --release          # Release build
cargo test                     # Run tests (783+ tests)
```

### WASM Build

```bash
cp .env.docker.example .env.docker   # First time: copy env template
docker compose --env-file .env.docker run --rm wasm
```

Build output goes to `pkg/`.

### Web Editor

```bash
cd rhwp-studio
npm install
npx vite --port 7700
```

Open `http://localhost:7700` in your browser.

## CLI Usage

### SVG Export

```bash
rhwp export-svg sample.hwp                         # Export to output/
rhwp export-svg sample.hwp -o my_dir/              # Custom output directory
rhwp export-svg sample.hwp -p 0                    # Specific page (0-indexed)
rhwp export-svg sample.hwp --debug-overlay         # Debug overlay
```

### Document Inspection

```bash
rhwp dump sample.hwp                  # Full IR dump
rhwp dump sample.hwp -s 2 -p 45      # Section 2, paragraph 45 only
rhwp dump-pages sample.hwp -p 15     # Page 16 layout items
rhwp info sample.hwp                  # File info
```

### Debugging Workflow

1. `export-svg --debug-overlay` → Identify paragraphs/tables by `s{section}:pi={index} y={coord}`
2. `dump-pages -p N` → Check paragraph layout list and heights
3. `dump -s N -p M` → Inspect ParaShape, LINE_SEG, table properties

No code modification needed for the entire debugging process.

## Roadmap

```
0.5 ──── 1.0 ──── 2.0 ──── 3.0
Foundation  Typeset   Collab    Complete
```

| Phase | Direction | Strategy |
|-------|-----------|----------|
| **0.5 → 1.0** | Systematize typesetting engine on read/write foundation | Build core architecture solo, make it solid |
| **1.0 → 2.0** | Open community participation on AI typesetting pipeline | Lower the barrier to contribution |
| **2.0 → 3.0** | Community-driven features toward public asset | Reach parity with Hancom |

## Recent Changes (v0.7.3 / extension v0.2.1, 2026-04-19)

**rhwp-studio (library 0.7.3)**
- HWPX-source documents: save disabled with user notification ([#196](https://github.com/edwardkim/rhwp/issues/196)) — prevents data loss until HWPX→HWP full converter ([#197](https://github.com/edwardkim/rhwp/issues/197)) lands
- HWPX→HWP IR mapping adapter assets preserved ([#178](https://github.com/edwardkim/rhwp/issues/178)) — rhwp self-roundtrip 100% recovered, Hancom compatibility deferred to #197
- Rotated shape resize cursor improvement + Flip handling (external contribution by [@bapdodi](https://github.com/bapdodi) — PR [#192](https://github.com/edwardkim/rhwp/pull/192))
- HWP image effects (grayscale/black-and-white) reflected in SVG (external contribution by [@marsimon](https://github.com/marsimon) — PR [#149](https://github.com/edwardkim/rhwp/pull/149))
- Windows CFB path separator fix (external contribution by [@dreamworker0](https://github.com/dreamworker0) — PR [#152](https://github.com/edwardkim/rhwp/pull/152))
- HWPX Serializer — Document IR → HWPX save (external contribution by [@seunghan91](https://github.com/seunghan91) — PR [#170](https://github.com/edwardkim/rhwp/pull/170))
- HWPX ZIP entry decompression cap + strikeout shape whitelist (external contribution by [@seunghan91](https://github.com/seunghan91) — PR [#153](https://github.com/edwardkim/rhwp/pull/153), PR [#154](https://github.com/edwardkim/rhwp/pull/154))
- Shape resize width/height clamp (external contribution by [@seunghan91](https://github.com/seunghan91) — PR [#163](https://github.com/edwardkim/rhwp/pull/163))
- Mobile dropdown menu icon/label overlap fix (external contribution by [@seunghan91](https://github.com/seunghan91) — PR [#161](https://github.com/edwardkim/rhwp/pull/161))

**rhwp-chrome / Edge extension (v0.2.1)**
- Restored "remember last save location" for general file downloads when extension is active ([#198](https://github.com/edwardkim/rhwp/issues/198))
- Options page CSP fix ([#166](https://github.com/edwardkim/rhwp/issues/166))
- HWP files: `Ctrl+S` overwrites the same file directly (external contribution by [@ahnbu](https://github.com/ahnbu) — PR [#189](https://github.com/edwardkim/rhwp/pull/189))
- Thumbnail loading spinner cleanup + options CSP compatibility (external contribution by [@postmelee](https://github.com/postmelee) — PR [#168](https://github.com/edwardkim/rhwp/pull/168))
- Block empty viewer tab on DEXT5-style download handlers

**Thanks to contributors**
This release cycle: [@ahnbu](https://github.com/ahnbu), [@bapdodi](https://github.com/bapdodi), [@dreamworker0](https://github.com/dreamworker0), [@marsimon](https://github.com/marsimon), [@postmelee](https://github.com/postmelee), [@seunghan91](https://github.com/seunghan91)

### Coming Soon

- HWPX direct save support ([#197](https://github.com/edwardkim/rhwp/issues/197)) — full HWPX→HWP converter
- "What's New" notification on extension update
- Additional download handler patterns (user-reported)

## Project Structure

```
src/
├── parser/                    # HWP/HWPX file parser
├── model/                     # HWP document model
├── document_core/             # Document core (CQRS: commands + queries)
├── renderer/                  # Rendering engine (layout, pagination, SVG/Canvas)
├── serializer/                # HWP file serializer (save)
└── wasm_api.rs                # WASM bindings

rhwp-studio/                   # Web editor (TypeScript + Vite)
mydocs/                        # Development documentation (Korean + English)
mydocs/eng/                    # English translations (724 files)
```

## Built with AI Pair Programming

> **This is not vibe coding.** There is no "just accept what AI gives you." Every plan is reviewed. Every output is verified. Every decision has a human behind it.

Vibe coding — hitting accept without reading, letting AI make architectural decisions, shipping code you don't understand — is a trap. It produces code that *looks* right but breaks in ways you can't diagnose, because you never understood it in the first place.

This project takes the opposite approach. A human **task director** maintains full ownership of direction, quality, and architectural decisions, while AI handles implementation at a speed and scale that would be impossible alone. The key difference: **the human never stops thinking.**

This project is developed using **[Claude Code](https://claude.ai/code)** (Anthropic's AI coding CLI) as a pair programming partner. The entire development process is transparently documented — not just the code, but the *thinking process* behind it.

### How It Works

A human **task director** and an AI **pair programmer** collaborate through a structured workflow:

```
Task Director (Human)              AI Pair Programmer (Claude Code)
─────────────────────              ────────────────────────────────
Sets direction & priorities   →    Analyzes, plans, implements
Reviews & approves plans      ←    Writes implementation plans
Provides domain feedback      →    Debugs, tests, iterates
Makes architectural decisions →    Executes with precision
Judges quality & correctness  ←    Generates code, docs, tests
```

### The Process

Every task follows a disciplined cycle:

1. **Task registration** — GitHub Issue with clear scope
2. **Implementation plan** — AI writes, human reviews and approves
3. **Step-by-step execution** — Build → Test → Report at each stage
4. **Code review feedback** — Human provides corrections, AI adapts
5. **Completion report** — Results documented for future reference

### What's in `mydocs/`

The `mydocs/` directory (724 files, English translations in `mydocs/eng/`) contains the complete development record:

| Directory | Contents | What You'll Learn |
|-----------|----------|-------------------|
| `orders/` | Daily task logs | How to structure daily AI collaboration |
| `plans/` | Task plans & implementation specs | How to break complex problems for AI |
| `working/` | Step-by-step completion reports | How AI executes and reports progress |
| `feedback/` | Code review feedback | How to guide AI with corrections |
| `tech/` | Technical research docs | How AI analyzes and reverse-engineers |
| `manual/` | Guides (incl. [AI Pair Programming Guide](mydocs/eng/manual/ai_pair_programming_guide.md)) | Best practices distilled |
| `troubleshootings/` | Debugging records | How AI diagnoses and fixes bugs |

### Why This Matters

Most AI coding demos show simple tasks. This project demonstrates AI pair programming **at production scale**:

- **100K+ lines of Rust** — parser, renderer, pagination, editor
- **783+ tests** with zero clippy warnings
- **Reverse engineering** a proprietary binary format
- **Sub-pixel layout accuracy** matching commercial software
- **Full CI/CD pipeline** — from commit to npm publish to GitHub Pages

The `mydocs/` directory is not documentation about the code — it's documentation about **how to build software with AI**. It's an open-source methodology.

**[Hyper-Waterfall Methodology](mydocs/eng/manual/hyper_waterfall.md)** — Waterfall discipline at the macro level + Agile speed at the micro level, made possible by AI.

### Vibe Coding vs. Directed AI Development

| | Vibe Coding | This Project |
|--|-------------|-------------|
| **Human role** | Accept AI output | Direct, review, decide |
| **Planning** | None — "just build it" | Written plan → approval → execution |
| **Quality gate** | Hope it works | 783 tests + Clippy + CI + code review |
| **Debugging** | Ask AI to fix AI's bugs | Human diagnoses, AI implements fix |
| **Architecture** | Emergent (accidental) | Deliberate (CQRS, dependency direction) |
| **Documentation** | None | 724 files of process records |
| **Outcome** | Fragile, hard to maintain | Production-grade, 100K+ lines |

The difference is not the AI — it's the discipline. AI is a force multiplier, but it multiplies whatever process you have. No process × AI = fast chaos. Good process × AI = extraordinary output.

## Architecture

```mermaid
graph TB
    HWP[HWP/HWPX File] --> Parser
    Parser --> Model[Document Model]
    Model --> DocumentCore
    DocumentCore --> |Commands| Edit[Edit Operations]
    DocumentCore --> |Queries| Render[Rendering Pipeline]
    Render --> Pagination
    Pagination --> Layout
    Layout --> SVG[SVG Output]
    Layout --> Canvas[Canvas Output]
    DocumentCore --> WASM[WASM API]
    WASM --> Studio[rhwp-studio Web Editor]
    Studio --> hwpctl[hwpctl Compatibility Layer]
```

## HWPUNIT

- 1 inch = 7,200 HWPUNIT
- 1 inch = 25.4 mm

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Questions and ideas are welcome on [Discussions](https://github.com/edwardkim/rhwp/discussions).

## Notice

This product was developed with reference to the HWP (.hwp) file format specification published by Hancom (한글과컴퓨터).

## License

[MIT License](LICENSE) — Copyright (c) 2025-2026 Edward Kim
