# Task 392: GitHub Upstream Public Release Preparation Plan

## Goal

Release the rhwp project as MIT-licensed open source on GitHub.
**Without modifying the existing GitLab repo**, construct a **new repo** at `/home/edward/mygithub/rhwp/`.

## Core Principles

```
Existing Repo (GitLab)                  New Repo (GitHub)
/home/edward/vsworks/rhwp/            /home/edward/mygithub/rhwp/
─────────────────────────             ──────────────────────────
CLAUDE.md (contains sensitive info)   → CLAUDE.md (public version, sensitive info removed)
.env.docker (local only)              → .env.docker.example (template)
samples/ (contains sensitive HWP)     → samples/ (self-generated public samples only)
hwp_webctl/ (government code)         → Not copied
mydocs/manual/hwp/Help/ (Hancom CHM)  → Not copied
mydocs/manual/hwpctl/*.hwp (Hancom docs) → Not copied
mydocs/convers/ (conversation logs)   → Not copied
rhwp-studio/dist/fonts/ (commercial)  → Open source fonts only
mydocs/tech/hwp_spec_5.0.md (Hancom spec) → Not copied (redistribution terms unclear)
────────────────────────────────────────────────────────────
src/ (all)                            → Full copy
mydocs/orders/ (daily records)        → Copy (educational material)
mydocs/plans/ (plans)                 → Copy (educational material)
mydocs/feedback/ (feedback)           → Copy (educational, excluding sensitive)
mydocs/report/ (reports)              → Copy (educational material)
mydocs/tech/ (technical docs)         → Selective copy (self-authored only)
mydocs/manual/onboarding_guide.md     → Copy
mydocs/troubleshootings/              → Copy
rhwp-studio/src/ (all)                → Full copy
rhwp-studio/e2e/ (E2E tests)         → Copy
scripts/ (dashboard)                  → Copy
```

## Implementation Method: Copy Script

Write `scripts/prepare-github.sh` to automate selective copying.

```bash
#!/bin/bash
# Usage: ./scripts/prepare-github.sh
# Result: GitHub-ready repo created at /home/edward/mygithub/rhwp/

SRC=/home/edward/vsworks/rhwp
DST=/home/edward/mygithub/rhwp

# 1. Copy source code
rsync -av --exclude-from=github-exclude.txt $SRC/ $DST/

# 2. Replace CLAUDE.md with public version
cp $DST/CLAUDE.public.md $DST/CLAUDE.md

# 3. Copy .env.docker.example
cp $SRC/.env.docker.example $DST/.env.docker.example

# 4. Copy only public samples
cp $SRC/samples/public/* $DST/samples/

# 5. Generate LICENSE
# 6. git init + commit
```

## Build Impact Verification

| Command | Runnable in GitHub Repo | Notes |
|------|------------------------|------|
| `cargo build` | Yes | Source code complete |
| `cargo test` | Yes | Tests referencing sensitive samples use skip |
| WASM build | Yes | Copy `.env.docker.example` → `.env.docker` |
| `rhwp export-svg` | Yes | Use public samples |
| E2E tests | Yes | Use public samples |

## Exclusion Details

### Sensitive Information
- CLAUDE.md GitLab IP/account/password/SSH key paths
- .env.docker environment variable values
- hwp_webctl/ (actual government budget request code+data)
- mydocs/convers/ (work conversation logs)

### Hancom/Commercial Licenses
- `mydocs/manual/hwp/Help/` — Hancom hwpkor.chm extracts (953 HTML files)
- `mydocs/manual/hwpctl/*.hwp` — Hancom hwpctl official document originals
- `mydocs/manual/hwpctl/*.md` — Hancom document conversions
- `mydocs/tech/hwp_spec_5.0.md` — Hancom public spec (redistribution terms unclear)
- `mydocs/tech/hwp_spec_chart.md` — Hancom chart spec
- `mydocs/tech/hwp_spec_equation.md` — Hancom equation spec
- `mydocs/tech/hwp_spec_3.0_hwpml.md` — Hancom HWPML spec
- `rhwp-studio/dist/fonts/h2hdrm*` — Hamchorom Dotum (Hancom font)
- `rhwp-studio/dist/fonts/Times*` — MS commercial font
- `rhwp-studio/dist/fonts/Tahoma*` — MS commercial font
- `rhwp-studio/dist/fonts/Verdana*` — MS commercial font
- `rhwp-studio/dist/fonts/Malgun*` — MS commercial font

### Sensitive Sample Files
- `samples/bodo-01.hwp, bodo-02.hwp` — Actual press releases
- `samples/kps-ai.hwp` — Bid proposal
- `samples/gonggo-01.hwp` — Public notice
- `samples/synam-001.hwp` — Actual document
- `samples/exam_*.hwp` — College entrance exam papers (copyrighted)
- `samples/bsbc01_10_000.hwp, data.json` — Budget forms
- `rhwp-studio/public/samples/kps-ai.hwp` etc. same

### Self-Authored (Can Include)
- `mydocs/tech/hwp_spec_errata.md` — Spec errata summary (self-authored)
- `mydocs/tech/hwp_table_rendering.md` — Table rendering guide (self-authored)
- `mydocs/tech/investigations/issue-139/equation_support_status.md` — Equation support status (self-authored)
- `mydocs/tech/archive/dev_roadmap_20260324.md` ��� Development roadmap (self-authored)
- `mydocs/tech/rendering_engine_design.md` ��� Rendering engine design (self-authored)

## Sub-Task Breakdown

| Task | Content | Priority | Notes |
|--------|------|---------|------|
| 393 | prepare-github.sh script + exclusion list | P0 | Selective copy automation |
| 394 | CLAUDE.public.md + .env.docker.example | P0 | Public version with sensitive info removed |
| 395 | samples/ public sample self-generation | P0 | Generate test HWP files with gen-table, etc. |
| 396 | LICENSE (MIT) + Cargo.toml/package.json update | P0 | |
| 397 | README.md public version | P0 | English or bilingual |
| 398 | CONTRIBUTING.md | P1 | |
| 399 | GitHub Actions CI setup | P1 | |
| 400 | GitHub repo creation + initial push | P0 | /home/edward/mygithub/rhwp/ |
