# PDFOxide - The Fastest PDF Toolkit for 20 Languages — Python, Rust, Go, JS/TS, C#, Java, Kotlin, Swift, C++ & more, plus CLI & AI

> **New in v0.3.69 — eleven new language bindings.** PDFOxide now ships idiomatic bindings for **C++, Swift, Kotlin, Dart, R, Julia, Zig, Scala, Clojure, Objective-C, and Elixir**, each built over the stable C ABI with its own CI workflow, api-coverage tests, and runnable examples. That brings the toolkit to **20 languages** (Rust core + 19 bindings). Want another language? [Open an issue](https://github.com/yfedoseev/pdf_oxide/issues/new) and tell us.

The fastest PDF library for text extraction, image extraction, and markdown conversion. A Rust core with bindings for **19 languages** — Python, Go, JavaScript / TypeScript, C# / .NET, Java, Kotlin, Scala, Clojure, Ruby, PHP, C++, Objective-C, Swift, Dart, R, Julia, Zig, Elixir, and WASM — plus a CLI tool and MCP server for AI assistants. 0.8ms mean per document, 5× faster than PyMuPDF, 15× faster than pypdf. 100% pass rate on 3,830 real-world PDFs. MIT licensed.

[![Crates.io](https://img.shields.io/crates/v/pdf_oxide.svg)](https://crates.io/crates/pdf_oxide)
[![PyPI](https://img.shields.io/pypi/v/pdf_oxide.svg)](https://pypi.org/project/pdf_oxide/)
[![PyPI Downloads](https://img.shields.io/pypi/dm/pdf-oxide)](https://pypi.org/project/pdf-oxide/)
[![npm](https://img.shields.io/npm/v/pdf-oxide-wasm)](https://www.npmjs.com/package/pdf-oxide-wasm)
[![Documentation](https://docs.rs/pdf_oxide/badge.svg)](https://docs.rs/pdf_oxide)
[![Build Status](https://github.com/yfedoseev/pdf_oxide/workflows/CI/badge.svg)](https://github.com/yfedoseev/pdf_oxide/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses)
<!-- [![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/yfedoseev/pdf_oxide/badge)](https://scorecard.dev/viewer/?uri=github.com/yfedoseev/pdf_oxide) -->
<!-- [![OpenSSF Best Practices](https://www.bestpractices.dev/projects/NNNN/badge)](https://www.bestpractices.dev/projects/NNNN) -->

## Quick Start

### Python
```python
from pdf_oxide import PdfDocument

with PdfDocument("paper.pdf") as doc:
    print(len(doc))                          # number of pages
    for page in doc:
        text = page.text                     # lazy property
        chars = page.chars                   # lazy property
        md = page.markdown(detect_headings=True)

# Direct page access by index
doc = PdfDocument("paper.pdf")
page = doc[0]
text = page.text
```

```bash
pip install pdf_oxide
```

### Rust
```rust
use pdf_oxide::PdfDocument;

let mut doc = PdfDocument::open("paper.pdf")?;
let text = doc.extract_text(0)?;
let images = doc.extract_images(0)?;
let markdown = doc.to_markdown(0, Default::default())?;
```

```toml
[dependencies]
pdf_oxide = "0.3"
```

### CLI
```bash
pdf-oxide text document.pdf
pdf-oxide markdown document.pdf -o output.md
pdf-oxide search document.pdf "pattern"
pdf-oxide merge a.pdf b.pdf -o combined.pdf
```

```bash
brew install yfedoseev/tap/pdf-oxide
```

### MCP Server (for AI assistants)
```bash
# Install
brew install yfedoseev/tap/pdf-oxide   # includes pdf-oxide-mcp

# Configure in Claude Desktop / Claude Code / Cursor
{
  "mcpServers": {
    "pdf-oxide": { "command": "crgx", "args": ["pdf_oxide_mcp@latest"] }
  }
}
```

## Why PDFOxide?

- **Fast** — 0.8ms mean per document, 5× faster than PyMuPDF, 15× faster than pypdf, 29× faster than pdfplumber
- **Reliable** — 100% pass rate on 3,830 test PDFs, zero panics, zero timeouts
- **Complete** — Text extraction, image extraction, PDF creation, and editing in one library
- **Multi-platform** — 20 languages (Rust core + 19 bindings: Python, Go, JS/TS, C#/.NET, Java, Kotlin, Scala, Clojure, Ruby, PHP, C++, Objective-C, Swift, Dart, R, Julia, Zig, Elixir, WASM), plus a CLI and MCP server for AI assistants
- **Permissive license** — MIT / Apache-2.0 — use freely in commercial and open-source projects

## Performance

Benchmarked on 3,830 PDFs from three independent public test suites (veraPDF, Mozilla pdf.js, DARPA SafeDocs). Text extraction libraries only (no OCR). Single-thread, 60s timeout, no warm-up.

### Python Libraries

| Library | Mean | p99 | Pass Rate | License |
|---------|------|-----|-----------|---------|
| **PDFOxide** | **0.8ms** | **9ms** | **100%** | **MIT** |
| PyMuPDF | 4.6ms | 28ms | 99.3% | AGPL-3.0 |
| pypdfium2 | 4.1ms | 42ms | 99.2% | Apache-2.0 |
| pymupdf4llm | 55.5ms | 280ms | 99.1% | AGPL-3.0 |
| pdftext | 7.3ms | 82ms | 99.0% | GPL-3.0 |
| pdfminer | 16.8ms | 124ms | 98.8% | MIT |
| pdfplumber | 23.2ms | 189ms | 98.8% | MIT |
| markitdown | 108.8ms | 378ms | 98.6% | MIT |
| pypdf | 12.1ms | 97ms | 98.4% | BSD-3 |

### Rust Libraries

| Library | Mean | p99 | Pass Rate | Text Extraction |
|---------|------|-----|-----------|-----------------|
| **PDFOxide** | **0.8ms** | **9ms** | **100%** | **Built-in** |
| oxidize_pdf | 13.5ms | 11ms | 99.1% | Basic |
| unpdf | 2.8ms | 10ms | 95.1% | Basic |
| pdf_extract | 4.08ms | 37ms | 91.5% | Basic |
| lopdf | 0.3ms | 2ms | 80.2% | No built-in extraction |

### Text Quality

99.5% text parity vs PyMuPDF and pypdfium2 across the full corpus. PDFOxide extracts text from 7–10× more "hard" files than it misses vs any competitor.

### Corpus

| Suite | PDFs | Pass Rate |
|-------|-----:|----------:|
| [veraPDF](https://github.com/veraPDF/veraPDF-corpus) (PDF/A compliance) | 2,907 | 100% |
| [Mozilla pdf.js](https://github.com/mozilla/pdf.js/tree/master/test/pdfs) | 897 | 99.2% |
| [SafeDocs](https://github.com/pdf-association/safedocs) (targeted edge cases) | 26 | 100% |
| **Total** | **3,830** | **100%** |

100% pass rate on all valid PDFs — the 7 non-passing files across the corpus are intentionally broken test fixtures (missing PDF header, fuzz-corrupted catalogs, invalid xref streams).

## Features

| Extract | Create | Edit |
|---------|--------|------|
| Text & Layout | Documents | Annotations |
| Images | Tables | Form Fields |
| Forms | Graphics | Bookmarks |
| Annotations | Templates | Links |
| Bookmarks | Images | Content |

## Python API

### Page-oriented API

```python
from pdf_oxide import PdfDocument

with PdfDocument("report.pdf") as doc:
    print(len(doc))          # page count
    print(doc.version())

    # Iterate or index pages
    for page in doc:
        text   = page.text                      # str, lazy
        chars  = page.chars                     # list[TextChar], lazy
        words  = page.words                     # list[Word], lazy
        lines  = page.lines                     # list[TextLine], lazy
        tables = page.tables                    # list[Table], lazy
        images = page.images                    # list[Image], lazy
        md     = page.markdown(detect_headings=True)
        html   = page.html()
        print(f"Page {page.index}: {page.width:.0f}×{page.height:.0f} pts")

    # Direct index access (supports negative indices)
    first = doc[0]
    last  = doc[-1]
```

### Scoped extraction

```python
# Extract from a region: (x, y, width, height) in PDF points
header = doc.within(0, (0, 700, 612, 92)).extract_text()
region = doc.within(0, (50, 400, 500, 200))
region_words  = region.extract_words()
region_images = region.extract_images()
```

### Extraction profiles

```python
from pdf_oxide import ExtractionProfile

# Pre-tuned profiles for different document types
words = doc.extract_words(0, profile=ExtractionProfile.form())
lines = doc.extract_text_lines(0, profile=ExtractionProfile.academic())

# Override adaptive thresholds (in PDF points)
words = doc.extract_words(0, word_gap_threshold=2.5)
lines = doc.extract_text_lines(0, word_gap_threshold=2.5, line_gap_threshold=4.0)
params = doc.page_layout_params(0)
print(f"word gap: {params.word_gap_threshold:.1f}")
```

### Form Fields

```python
# Extract form fields
fields = doc.get_form_fields()
for f in fields:
    print(f"{f.name} ({f.field_type}) = {f.value}")

# Fill and save
doc.set_form_field_value("employee_name", "Jane Doe")
doc.set_form_field_value("wages", "85000.00")
doc.save("filled.pdf")
```

## Rust API

```rust
use pdf_oxide::PdfDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("paper.pdf")?;

    // Extract text
    let text = doc.extract_text(0)?;

    // Character-level extraction
    let chars = doc.extract_chars(0)?;

    // Extract images
    let images = doc.extract_images(0)?;

    // Vector graphics
    let paths = doc.extract_paths(0)?;

    Ok(())
}
```

### Form Fields (Rust)

```rust
use pdf_oxide::editor::{DocumentEditor, EditableDocument, SaveOptions};
use pdf_oxide::editor::form_fields::FormFieldValue;

let mut editor = DocumentEditor::open("w2.pdf")?;
editor.set_form_field_value("employee_name", FormFieldValue::Text("Jane Doe".into()))?;
editor.save_with_options("filled.pdf", SaveOptions::incremental())?;
```

## Installation

### Python

```bash
pip install pdf_oxide
```

Wheels available for Linux, macOS, and Windows. Python 3.8–3.14.

### Rust

```toml
[dependencies]
pdf_oxide = "0.3"
```

### JavaScript/WASM

```bash
npm install pdf-oxide-wasm
```

```javascript
const { WasmPdfDocument } = require("pdf-oxide-wasm");
```

### CLI

```bash
brew install yfedoseev/tap/pdf-oxide    # Homebrew (macOS/Linux)
cargo install pdf_oxide_cli             # Cargo
cargo binstall pdf_oxide_cli            # Pre-built binary via cargo-binstall
```

### MCP Server

```bash
brew install yfedoseev/tap/pdf-oxide    # Included with CLI in Homebrew
cargo install pdf_oxide_mcp             # Cargo
```

### Other languages

Established bindings:

- **Go** — `go get github.com/yfedoseev/pdf_oxide/go` — see [go/README.md](go/README.md)
- **JavaScript / TypeScript (Node.js)** — `npm install pdf-oxide` — see [js/README.md](js/README.md)
- **C# / .NET** — `dotnet add package PdfOxide` — see [csharp/README.md](csharp/README.md)
- **Java (JDK 11+)** — Maven coords `fyi.oxide:pdf-oxide:0.3.70` — see [java/README.md](java/README.md)
- **Ruby** — `gem install pdf_oxide` — see [ruby/README.md](ruby/README.md)
- **PHP** — `composer require oxide/pdf-oxide` — see [php/README.md](php/README.md)

New in v0.3.69 (all over the stable C ABI):

- **C++** (header-only, CMake / Conan) — see [cpp/README.md](cpp/README.md)
- **Swift** (SwiftPM) — see [swift/README.md](swift/README.md)
- **Kotlin** (`fyi.oxide:pdf-oxide-kotlin:0.3.70`) — see [kotlin/README.md](kotlin/README.md)
- **Scala** (`fyi.oxide %% pdf-oxide-scala`) — see [scala/README.md](scala/README.md)
- **Clojure** (`fyi.oxide/pdf-oxide-clojure` on Clojars) — see [clojure/README.md](clojure/README.md)
- **Dart / Flutter** (`dart pub add pdf_oxide`) — see [dart/README.md](dart/README.md)
- **R** (`install.packages("pdfoxide")`) — see [r/README.md](r/README.md)
- **Julia** (`Pkg.add("PdfOxide")`) — see [julia/README.md](julia/README.md)
- **Zig** (`build.zig.zon`) — see [zig/README.md](zig/README.md)
- **Objective-C** (CocoaPods) — see [objc/README.md](objc/README.md)
- **Elixir** (`{:pdf_oxide, "~> 0.3.70"}` on Hex) — see [elixir/README.md](elixir/README.md)

```xml
<!-- Java (Maven) -->
<dependency>
  <groupId>fyi.oxide</groupId>
  <artifactId>pdf-oxide</artifactId>
  <version>0.3.70</version>
</dependency>
```

```gradle
// Kotlin (Gradle, Kotlin DSL)
implementation("fyi.oxide:pdf-oxide-kotlin:0.3.70")
```

Every binding shares the same Rust core, so a bug fix in one lands in all of them — everything you read in this README applies, just with each language's native naming conventions. Publishing details for each registry are in [docs/RELEASING-bindings.md](docs/RELEASING-bindings.md).

## CLI

22 commands for PDF processing directly from your terminal:

```bash
pdf-oxide text report.pdf                      # Extract text
pdf-oxide markdown report.pdf -o report.md     # Convert to Markdown
pdf-oxide html report.pdf -o report.html       # Convert to HTML
pdf-oxide info report.pdf                      # Show metadata
pdf-oxide search report.pdf "neural.?network"  # Search (regex)
pdf-oxide images report.pdf -o ./images/       # Extract images
pdf-oxide merge a.pdf b.pdf -o combined.pdf    # Merge PDFs
pdf-oxide split report.pdf -o ./pages/         # Split into pages
pdf-oxide watermark doc.pdf "DRAFT"            # Add watermark
pdf-oxide forms w2.pdf --fill "name=Jane"      # Fill form fields
```

Run `pdf-oxide` with no arguments for interactive REPL mode. Use `--pages 1-5` to process specific pages, `--json` for machine-readable output.

## MCP Server

`pdf-oxide-mcp` lets AI assistants (Claude, Cursor, etc.) extract content from PDFs locally via the [Model Context Protocol](https://modelcontextprotocol.io/).

Add to your MCP client configuration:

```json
{
  "mcpServers": {
    "pdf-oxide": { "command": "crgx", "args": ["pdf_oxide_mcp@latest"] }
  }
}
```

The server exposes an `extract` tool that supports text, markdown, and HTML output formats with optional page ranges and image extraction. All processing runs locally — no files leave your machine.

## Building from Source

```bash
# Clone and build
git clone https://github.com/yfedoseev/pdf_oxide
cd pdf_oxide
cargo build --release

# Run tests
cargo test

# Build Python bindings
maturin develop

# Build the shared library for Go, JS/TS, and C# bindings
cargo build --release --lib
# Output: target/release/libpdf_oxide.{so,dylib} or pdf_oxide.dll
```

## Documentation

- **[Full Documentation](https://pdf.oxide.fyi)** — Complete documentation site
- **[Getting Started (Rust)](docs/getting-started-rust.md)** — Rust guide
- **[Getting Started (Python)](docs/getting-started-python.md)** — Python guide
- **[Getting Started (Go)](go/README.md)** — Go guide
- **[Getting Started (JavaScript / TypeScript)](js/README.md)** — Node.js guide
- **[Getting Started (C# / .NET)](csharp/README.md)** — .NET guide
- **[Getting Started (WASM)](docs/getting-started-wasm.md)** — Browser and Node.js WASM guide
- **[API Docs](https://docs.rs/pdf_oxide)** — Full Rust API reference
- **[Performance Benchmarks](https://pdf.oxide.fyi/docs/performance)** — Full benchmark methodology and results

## Use Cases

- **RAG / LLM pipelines** — Convert PDFs to clean Markdown for retrieval-augmented generation with LangChain, LlamaIndex, or any framework
- **AI assistants** — Give Claude, Cursor, or any MCP-compatible tool direct PDF access via the MCP server
- **Document processing at scale** — Extract text, images, and metadata from thousands of PDFs in seconds
- **Data extraction** — Pull structured data from forms, tables, and layouts
- **Academic research** — Parse papers, extract citations, and process large corpora
- **PDF generation** — Create invoices, reports, certificates, and templated documents programmatically
- **PyMuPDF alternative** — MIT licensed, 5× faster, no AGPL restrictions

## Notable Users

PDFOxide is a production dependency of RAG engines, collaborative editors, AI coding agents, and arXiv.org's own submission pipeline. Every entry below is verified against that project's public dependency manifest — no self-reported logos, no "trusted by" wall.

Star counts checked 2026-08-21:

- **[RAGFlow](https://github.com/infiniflow/ragflow)** (89k★) — one of the most popular open-source RAG engines; PDFOxide is the primary PDF engine in its `deepdoc` parsing pipeline ([go.mod](https://github.com/infiniflow/ragflow/blob/main/go.mod))
- **[AFFiNE](https://github.com/toeverything/AFFiNE)** (71.7k★) — open-source Notion/Miro alternative; powers document extraction via AFFiNE's own [`doc_extractor`](https://crates.io/crates/doc_extractor) crate ([Cargo.toml](https://github.com/toeverything/AFFiNE/blob/canary/Cargo.toml))
- **[grok-build](https://github.com/xai-org/grok-build)** (25.8k★) — xAI's coding-agent TUI; uses PDFOxide's rendering feature to read PDFs in a repo ([Cargo.toml](https://github.com/xai-org/grok-build/blob/main/Cargo.toml))
- **[create-context-graph](https://github.com/neo4j-labs/create-context-graph)** (Neo4j Labs, 712★) — GraphRAG context-builder; PDFOxide is its primary PDF parser ([pyproject.toml](https://github.com/neo4j-labs/create-context-graph/blob/main/pyproject.toml))
- **[arXiv submission-tools](https://github.com/arXiv/submission-tools)** — arXiv.org's own PDF validation and LaTeX-to-PDF submission pipeline

Using PDFOxide in production? [Open an issue](https://github.com/yfedoseev/pdf_oxide/issues/new) to be added here.

## Why I built this

I needed PyMuPDF's speed without its AGPL license, and I needed it in more than one language. Nothing existed that ticked all three boxes — fast, MIT, multi-language — so I wrote it. The Rust core is what does the real work; the bindings for Python, Go, JS/TS, C#, and WASM are thin shells around the same code, so a bug fix in one lands in all of them. It now passes 100% of the veraPDF + Mozilla pdf.js + DARPA SafeDocs test corpora (3,830 PDFs) on every platform I've tested.

If it's useful to you, a star on GitHub genuinely helps. If something's broken or missing, [open an issue](https://github.com/yfedoseev/pdf_oxide/issues) — I read all of them.

— Yury

## License & trademark

The **code** is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option. Unlike AGPL-licensed alternatives, pdf_oxide can be used freely in any project — commercial or open-source — with no copyleft restrictions.

The **name and brand** are not covered by the code license: **"PDFOxide"** (the product name) and **"pdf_oxide"** (the package name), together with any associated logo, are trademarks of Yury Fedoseev. You may say your product "uses PDFOxide"; please don't name a different or modified product "PDFOxide" or "pdf_oxide". See [TRADEMARKS.md](TRADEMARKS.md), and [FORKS.md](FORKS.md) if you are forking.

© 2025–2026 Yury Fedoseev and the PDFOxide contributors. Contributions are accepted under the project's [DCO](CONTRIBUTING.md#commits-dco-and-review) and [CLA](CLA.md); contributors retain ownership of their work.

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
cargo build && cargo test && cargo fmt && cargo clippy -- -D warnings
```

## Citation

```bibtex
@software{pdf_oxide,
  title = {PDFOxide: Fast Multi-Language PDF Toolkit (Rust core, 19 language bindings)},
  author = {Yury Fedoseev},
  year = {2025},
  url = {https://github.com/yfedoseev/pdf_oxide}
}
```

---

**20 languages** (Rust + Python + Go + JS/TS + C# + Java + Kotlin + Scala + Clojure + Ruby + PHP + C++ + Objective-C + Swift + Dart + R + Julia + Zig + Elixir + WASM) + **CLI** + **MCP** | MIT/Apache-2.0 | 100% pass rate on 3,830 PDFs | 0.8ms mean | 5× faster than the industry leaders
