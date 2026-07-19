# rhwp Onboarding Guide

This project is developed through a **collaboration between Claude Code (AI agent) and a task director (Human)**.
It differs significantly from traditional development workflows, so please read this document carefully.

## 1. Project Overview

A project that parses and renders HWP (Hangul Word Processor) files using Rust.

```
HWP/HWPX file → Parser → IR (Model) → Paginator → Layout → SVG/Canvas
```

- **Native**: SVG export and IR dump via CLI
- **Web**: WASM build → View/edit documents in rhwp-studio (web editor)

## 2. Setting Up the Development Environment

### 2.1 Required Tools

```bash
# Rust (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Docker (for WASM builds)
# → Install Docker Desktop or Docker Engine

# Node.js (rhwp-studio web editor)
# → Install Node.js 18+
```

### 2.2 Build Verification

```bash
cargo build              # Native build
cargo test               # Run tests
```

### 2.3 WASM Build

```bash
docker compose --env-file .env.docker run --rm wasm
```

### 2.4 Running the Web Editor

```bash
cd rhwp-studio
npm install
npx vite --host 0.0.0.0 --port 7700
# Open http://localhost:7700 in your browser
```

## 3. Task Director–Agent Collaboration Model

### 3.1 Roles

| Role | Who | Responsibilities |
|------|-----|-----------------|
| **Task Director** (Human) | Project owner | Assigns tasks, validates against Hancom, approves/rejects, adjusts direction |
| **Agent** (Claude Code) | AI developer | Code analysis, implementation, testing, documentation, debugging |

### 3.2 Task Workflow

```
1. Task director: Assigns a task (registered in mydocs/orders/)
2. Agent:         Writes an execution plan → Requests approval
3. Task director: Approves or requests changes
4. Agent:         Writes an implementation plan (3–6 stages) → Requests approval
5. Task director: Approves
6. Agent:         Implements stage by stage → Stage completion report → Requests approval
7. Task director: Verifies, then approves or gives feedback
8. Repeat...
9. Agent:         Final report → Commit → Merge to devel
```

### 3.3 Core Principles

- **Follow the task director's instructions**: The agent may suggest "a better approach," but the task director has the final say
- **Immediate course correction**: "Revert that", "You fixed the wrong thing" → Revert immediately and re-analyze
- **Approval-gated progression**: Do not advance to the next stage without stage-level approval
- **Verification by the task director**: Rendering accuracy is validated by comparing against Hancom Word Processor

## 4. Debugging Protocol

This is the most distinctive aspect of the project. The task director and agent share a **common vocabulary for pinpointing problems**, backed by a unified tool chain.

### 4.1 Three-Stage Debugging Workflow

```
Step 1: export-svg --debug-overlay  → Identify paragraphs/tables in the SVG
Step 2: dump-pages -p N             → Inspect the page's layout list and heights
Step 3: dump -s N -p M              → Detailed IR investigation of a specific paragraph
```

The entire process can be performed **without modifying any code**.

### 4.2 Debug Overlay (`--debug-overlay`)

```bash
rhwp export-svg sample.hwp --debug-overlay --output output/
```

Visually marks paragraph/table boundaries and indices on the SVG:

- **Paragraphs**: Alternating-color dashed borders + `s{section}:pi={index} y={coordinate}` label (top-left)
- **Tables**: Red dashed borders + `s{section}:pi={index} ci={control} {rows}x{cols} y={coordinate}` label (top-right)

#### Communication Example

```
Task director: "The gap between paragraph s2:pi=44 and table s2:pi=45 is too large"
Agent:         "dump confirms pi=44 vpos_end=14860, pi=45 vpos=15360,
                gap=500 HU (6.7px) is correct, but layout outputs 172.3px.
                Root cause: shape_reserved was applied twice."
```

Both sides communicate using **quantitative identifiers** like `s2:pi=44` and `y=243.0`, eliminating ambiguity.

### 4.3 Pagination Result Dump (`dump-pages`)

```bash
rhwp dump-pages sample.hwp -p 15
```

```
=== Page 16 (global_idx=15, section=2, page_num=6) ===
  body_area: x=96.0 y=103.6 w=601.7 h=930.5
  Column 0 (items=7)
    FullParagraph  pi=41  h=37.3 (sb=16.0 lines=21.3 sa=0.0)  "Data type description"
    Table          pi=45 ci=0  16x4  492.2x278.7px  wrap=TopAndBottom tac=false
```

Checking the **measured heights (sb/lines/sa)** of each paragraph/table enables immediate diagnosis of page-break errors.

### 4.4 IR Dump (`dump`)

```bash
rhwp dump sample.hwp -s 2 -p 45
```

```
--- Paragraph 2.45 --- cc=9, text_len=0, controls=1
  [PS] ps_id=32 align=Justify spacing: before=1000 after=0 line=160/Percent
  [0] Table: 16 rows x 4 cols
  [0]   [common] treat_as_char=false, wrap=TopAndBottom, vert=Para(0=0.0mm)
  [0]   [outer_margin] left=1.0mm top=2.0mm right=1.0mm bottom=7.0mm
```

Inspect ParaShape, LINE_SEG, and table properties without any code changes.

### 4.5 Visual Verification (Task Director)

The task director verifies using Hancom Word Processor with methods such as:

1. **Changing text colors**: Applying different colors to header/body/background text to identify rendering sources
2. **Showing control codes**: Checking the positions of tabs, spaces, and line breaks in Hancom
3. **Screenshot comparison**: Side-by-side comparison of Hancom rendering and rhwp rendering
4. **Property inspection**: Hancom object properties → Accurately relay position/size/margin values

#### Verification Communication Example

```
Task director: [screenshot attached]
               "I've colored the odd-page header too. Please re-export page 1 SVG"

Agent:         [after SVG export]
               "The math region at y=149.3 has changed to #2d629c (blue).
                This comes from the odd-page header. The header is still
                being rendered on page 1."

Task director: "The header is defined at pi=41 and should apply from page 2 onward."

Agent:         [after root cause analysis]
               "The pagination engine pre-registered headers via a full scan,
                causing them to appear on pages before the definition. Fixing now."
```

## 5. Feedback System

### 5.1 Feedback Types

| Type | Example | Agent Response |
|------|---------|---------------|
| **Immediate fix** | "Revert that" | `git checkout` the changes immediately |
| **Direction change** | "This approach won't solve it" | Discard the current approach, start fresh analysis |
| **Information** | "Table height in Hancom: 39.01mm" | Verify using the exact value |
| **Bug report** | "Spacing error at s2:pi=365" | Diagnose with debug-overlay + dump |
| **Backlog entry** | "A root fix belongs in the backlog" | Register as B-XXX, scope the current task |

### 5.2 Feedback Documents

- `mydocs/feedback/` — Feedback records
- `mydocs/feedback/r-code-review-*.md` — Code review reports (rounds 1–4)
- `mydocs/orders/` backlog section — Unresolved issue tracking (B-001–B-008)

### 5.3 Verification Tools by Purpose

| Tool | Used by | Purpose |
|------|---------|---------|
| `export-svg --debug-overlay` | Both | Identify paragraph/table positions |
| `dump-pages -p N` | Agent | Inspect pagination results |
| `dump -s N -p M` | Agent | Detailed IR investigation |
| Hancom rendering comparison | Task director | Establish the ground truth |
| WASM build + web check | Task director | Final rendering verification |

## 6. Understanding the Code Structure

### 6.1 Core Modules

```
src/
├── model/           # Pure data structures (no dependencies on other modules)
├── parser/          # HWP/HWPX → model conversion
├── document_core/   # Domain core (CQRS pattern)
│   ├── commands/    # State mutation (editing)
│   └── queries/     # State queries (rendering, cursor)
├── renderer/        # Rendering engine
│   ├── pagination/  # Pagination (9.5/10 — exemplary code)
│   ├── layout/      # Layout (paragraphs, tables, shapes)
│   ├── equation/    # Equation parser/renderer
│   └── svg.rs       # SVG output
└── wasm_api.rs      # WASM thin wrapper
```

### 6.2 Dependency Direction (Must Be Followed)

```
parser → model ← document_core ← renderer
                ↑                ↑
              wasm_api          main.rs
```

- **model imports nothing** (pure data)
- **parser only knows model**
- **renderer knows model and document_core**
- **wasm_api/main.rs knows everything** (adapter layer)

### 6.3 Rendering Pipeline

```
Model → StyleResolver → Composer → HeightMeasurer → Paginator → LayoutEngine → SVG/Canvas
```

Each stage uses only the output of the previous stage. No reverse dependencies.

## 7. Essential Domain Knowledge

### 7.1 HWPUNIT

```
1 inch = 7200 HWPUNIT
1 inch = 25.4 mm
1 HWPUNIT = 25.4 / 7200 mm ≈ 0.00353 mm
```

Converted in code using the `hwpunit_to_px(hu, dpi)` function.

### 7.2 Key Properties

| Property | Description | Usage |
|----------|-------------|-------|
| `treat_as_char` (TAC) | Treat as character — inline placement | Positioned within text flow |
| `text_wrap` | TopAndBottom, Square, InFrontOfText, BehindText | Placement relative to body text |
| `vert_rel_to` | Paper, Page, Para — vertical reference | Absolute/relative position |
| `horz_rel_to` | Paper, Page, Column, Para — horizontal reference | Absolute/relative position |

### 7.3 HWP Spec Documents

- `mydocs/tech/hwp_spec_5.0.md` — HWP 5.0 file format (note: contains spec errors)
- `mydocs/tech/hwp_spec_equation.md` — Equation spec
- `mydocs/tech/table_layout_rules.md` — Table layout rules
- `mydocs/tech/investigations/issue-139/equation_support_status.md` — Equation support status

> **Important**: The HWP spec documents contain errors. Always verify against actual binary data.
> Known errors are listed in the "Known Spec Errors" section of CLAUDE.md.

## 8. E2E Testing

### 8.1 Running Tests

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700 &   # Vite dev server
node e2e/text-flow.test.mjs              # Run individual test
```

### 8.2 Modes

| Mode | Purpose | Command |
|------|---------|---------|
| `--mode=headless` | CI automation | Chrome inside WSL2 |
| `--mode=host` | Visual inspection | Host Windows Chrome via CDP |

### 8.3 Writing a New E2E Test

```javascript
import { launchBrowser, closeBrowser, createPage, loadApp, screenshot, assert } from './helpers.mjs';

const browser = await launchBrowser();
const page = await createPage(browser);
await loadApp(page, 'sample.hwp');

// Verification
const pageCount = await page.evaluate(() => window.hwpDoc?.pageCount());
assert(pageCount >= 1, `Page count >= 1 (${pageCount})`);

await screenshot(page, 'my-test');
await closeBrowser(browser);
```

## 9. Git Workflow

### Branch Structure

```
main              ← Releases (tags: v0.5.0, etc.)
devel             ← Development integration
local/task{N}     ← Task branches based on GitHub Issue numbers
```

### Task Number Management

- **GitHub Issues** for automatic task numbering (manual number assignment is prohibited)
- **GitHub Milestones** for grouping tasks
- Milestone notation: `M{version}` (e.g., M100 = v1.0.0, M05x = v0.5.x)

### Workflow Steps

```bash
# 1. Register a GitHub Issue
gh issue create --repo edwardkim/rhwp --title "Title" --body "Description" --milestone "v1.0.0"

# 2. Create a task branch
git checkout -b local/task1 devel

# 3. Implement + commit
git commit -m "Task #1: Description"

# 4. Merge to devel + close Issue
git checkout devel && git merge local/task1
git push origin devel    # Auto-closes if commit includes "closes #1"
# Or manually: gh issue close 1

# 5. Release (only when requested by the task director)
git checkout main && git merge devel && git push origin main
```

### Daily Task Notation

In `mydocs/orders/yyyymmdd.md`, reference tasks using the milestone+issue format:

```markdown
## M100 — Typesetting Engine Systematization

| Issue | Task | Status |
|-------|------|--------|
| [#1](https://github.com/edwardkim/rhwp/issues/1) | Baseline reverse engineering | Done |
| [#2](https://github.com/edwardkim/rhwp/issues/2) | Edit gap fix | Pending |
```

## 10. Contribution Guidelines

1. **Maintain model purity**: Do not import other modules from `src/model/`
2. **No enum→bool simplification**: Do not convert types like CharShape's UnderlineType to bool
3. **Do not trust the HWP spec blindly**: Always verify against actual binary data
4. **Avoid unwrap()**: Use Option/Result, especially in parser/renderer code
5. **Clean up debug output**: Remove `eprintln!` calls before committing
6. **Korean documentation**: All project documents are written in Korean
