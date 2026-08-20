# Evidence index — rhwp-rendered PNGs (task_m100_5706)

Scan: `mydocs/report/task_m100_5706/**/*.png` (no new images).
These are document pages from `target/release/rhwp.exe` v0.8.4
`export-svg -p 0 --profile print` → Edge headless screenshot.
Not terminal captures. Portrait pages: 1000×1400 (shared `_renders` copies
and most `skills/*/page.png`) or 1200×1600 (`rhwp-skill-author` and later
skill renders). Landscape 1200×1600: `rhwp-recipes`, `rhwp-strategist`.

**PNG count: 36** (31 under `skills/`, 5 under `_renders/`).
Catalog skills with at least one PNG: **27 / 27**.
Missing skills (no PNG): **none**.

Byte-identical copies of a shared render are listed separately (same size,
same source page). 10 unique SHA-256 payloads.

## Shared renders (`_renders/`)

| Path | Bytes | WxH | Skill / page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/_renders/english.png](_renders/english.png) | 56,844 | 1000×1400 | onboarding · CLI · triage group — `samples/basic/english.hwp` p0 (영어 번역 학습지) |
| [mydocs/report/task_m100_5706/_renders/form.png](_renders/form.png) | 18,209 | 1000×1400 | form-fill (before) — `samples/basic/BlogForm_BookReview.hwp` p0 (빈 서식) |
| [mydocs/report/task_m100_5706/_renders/form-filled.png](_renders/form-filled.png) | 20,849 | 1000×1400 | form-fill (after) — `output/filled-book.hwp` p0 (4칸 기입) |
| [mydocs/report/task_m100_5706/_renders/request.png](_renders/request.png) | 53,446 | 1000×1400 | security · provenance — `samples/basic/request.hwp` p0 (마일리지 카드 가입) |
| [mydocs/report/task_m100_5706/_renders/table.png](_renders/table.png) | 117,880 | 1000×1400 | table-exchange · visual-regression — `samples/basic/issue1994_behindtext_table_20200830.hwp` p0 (표+악보) |

## Unique payloads (SHA-256 prefix)

| SHA-256 | Bytes | WxH | Source page | Copies |
|---|---:|---|---|---:|
| `d3c91a02fff9` | 56,844 | 1000×1400 | `samples/basic/english.hwp` p0 | 9 |
| `093c0b43fdeb` | 59,145 | 1200×1600 | `samples/basic/english.hwp` p0 | 5 |
| `76918d5b0549` | 53,446 | 1000×1400 | `samples/basic/request.hwp` p0 | 3 |
| `329d79a939e0` | 55,567 | 1200×1600 | `samples/basic/request.hwp` p0 | 5 |
| `cf27ab14cee9` | 18,209 | 1000×1400 | `samples/basic/BlogForm_BookReview.hwp` p0 empty | 2 |
| `44241fbec0a5` | 19,998 | 1200×1600 | `samples/basic/BlogForm_BookReview.hwp` p0 empty | 1 |
| `7defbac1103a` | 20,849 | 1000×1400 | `output/filled-book.hwp` p0 filled | 5 |
| `8a26ba338856` | 117,880 | 1000×1400 | `samples/basic/issue1994_behindtext_table_20200830.hwp` p0 | 4 |
| `ed47aa1a52d2` | 125,706 | 1200×1600 | `samples/basic/issue1994_behindtext_table_20200830.hwp` p0 | 1 |
| `ebb42fa9376d` | 271,303 | 1200×1600 | `samples/basic/KTX.hwp` p0 | 1 |

## Per-skill PNGs

Catalog order. Every catalog skill has `skills/<id>/page.png`.

### rhwp-agent-surface

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-agent-surface/page.png](skills/rhwp-agent-surface/page.png) | 59,145 | 1200×1600 | `samples/basic/english.hwp` p0 print |

### rhwp-bug-hunter

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-bug-hunter/page.png](skills/rhwp-bug-hunter/page.png) | 55,567 | 1200×1600 | `samples/basic/request.hwp` p0 print |

### rhwp-bulk-pipeline

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-bulk-pipeline/page.png](skills/rhwp-bulk-pipeline/page.png) | 56,844 | 1000×1400 | `samples/basic/english.hwp` p0 print |

### rhwp-chief

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-chief/page.png](skills/rhwp-chief/page.png) | 59,145 | 1200×1600 | `samples/basic/english.hwp` p0 print |

### rhwp-cli

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-cli/page.png](skills/rhwp-cli/page.png) | 56,844 | 1000×1400 | `samples/basic/english.hwp` p0 print |

### rhwp-codex

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-codex/page.png](skills/rhwp-codex/page.png) | 117,880 | 1000×1400 | `samples/basic/issue1994_behindtext_table_20200830.hwp` p0 print |

### rhwp-contributor

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-contributor/page.png](skills/rhwp-contributor/page.png) | 56,844 | 1000×1400 | `samples/basic/english.hwp` p0 print |

### rhwp-doc-triage

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-doc-triage/page.png](skills/rhwp-doc-triage/page.png) | 56,844 | 1000×1400 | `samples/basic/english.hwp` p0 print |

### rhwp-exam-ingest

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-exam-ingest/page.png](skills/rhwp-exam-ingest/page.png) | 56,844 | 1000×1400 | `samples/basic/english.hwp` p0 print |

### rhwp-explore

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-explore/page.png](skills/rhwp-explore/page.png) | 55,567 | 1200×1600 | `samples/basic/request.hwp` p0 print |

### rhwp-fde

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-fde/page.png](skills/rhwp-fde/page.png) | 59,145 | 1200×1600 | `samples/basic/english.hwp` p0 print |

### rhwp-fidelity-compare

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-fidelity-compare/page.png](skills/rhwp-fidelity-compare/page.png) | 55,567 | 1200×1600 | `samples/basic/request.hwp` p0 print |

### rhwp-form-fill

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-form-fill/before.png](skills/rhwp-form-fill/before.png) | 18,209 | 1000×1400 | `samples/basic/BlogForm_BookReview.hwp` p0 print (empty) |
| [mydocs/report/task_m100_5706/skills/rhwp-form-fill/after.png](skills/rhwp-form-fill/after.png) | 20,849 | 1000×1400 | `output/filled-book.hwp` p0 print (제목/지은이/국적/리뷰) |
| [mydocs/report/task_m100_5706/skills/rhwp-form-fill/filled.png](skills/rhwp-form-fill/filled.png) | 20,849 | 1000×1400 | same filled page as `after.png` |
| [mydocs/report/task_m100_5706/skills/rhwp-form-fill/page.png](skills/rhwp-form-fill/page.png) | 20,849 | 1000×1400 | same filled page as `after.png` |

### rhwp-handoff

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-handoff/page.png](skills/rhwp-handoff/page.png) | 59,145 | 1200×1600 | `samples/basic/english.hwp` p0 print |

### rhwp-knowledge-map

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-knowledge-map/page.png](skills/rhwp-knowledge-map/page.png) | 55,567 | 1200×1600 | `samples/basic/request.hwp` p0 print |

### rhwp-mcp-session

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-mcp-session/page.png](skills/rhwp-mcp-session/page.png) | 56,844 | 1000×1400 | `samples/basic/english.hwp` p0 print |

### rhwp-onboarding

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-onboarding/page.png](skills/rhwp-onboarding/page.png) | 56,844 | 1000×1400 | `samples/basic/english.hwp` p0 print |

### rhwp-provenance

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-provenance/page.png](skills/rhwp-provenance/page.png) | 53,446 | 1000×1400 | `samples/basic/request.hwp` p0 print |

### rhwp-recipes

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-recipes/page.png](skills/rhwp-recipes/page.png) | 125,706 | 1200×1600 | `samples/basic/issue1994_behindtext_table_20200830.hwp` p0 print |

### rhwp-safe-edit

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-safe-edit/page.png](skills/rhwp-safe-edit/page.png) | 20,849 | 1000×1400 | `output/filled-book.hwp` p0 print (`edit fill-fields` `-o` 산출) |

### rhwp-security-sweep

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-security-sweep/page.png](skills/rhwp-security-sweep/page.png) | 53,446 | 1000×1400 | `samples/basic/request.hwp` p0 print |

### rhwp-skill-author

Present (required `english.png` and `page.png`).

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-skill-author/english.png](skills/rhwp-skill-author/english.png) | 59,145 | 1200×1600 | `samples/basic/english.hwp` p0 print |
| [mydocs/report/task_m100_5706/skills/rhwp-skill-author/page.png](skills/rhwp-skill-author/page.png) | 55,567 | 1200×1600 | `samples/basic/request.hwp` p0 print |

### rhwp-skill-router

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-skill-router/page.png](skills/rhwp-skill-router/page.png) | 19,998 | 1200×1600 | `samples/basic/BlogForm_BookReview.hwp` p0 print (empty) |

### rhwp-strategist

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-strategist/page.png](skills/rhwp-strategist/page.png) | 271,303 | 1200×1600 | `samples/basic/KTX.hwp` p0 print |

### rhwp-table-exchange

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-table-exchange/page.png](skills/rhwp-table-exchange/page.png) | 117,880 | 1000×1400 | `samples/basic/issue1994_behindtext_table_20200830.hwp` p0 print |

### rhwp-visual-regression

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-visual-regression/page.png](skills/rhwp-visual-regression/page.png) | 117,880 | 1000×1400 | `samples/basic/issue1994_behindtext_table_20200830.hwp` p0 print |

### rhwp-work-receipt

| Path | Bytes | WxH | Page |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/skills/rhwp-work-receipt/page.png](skills/rhwp-work-receipt/page.png) | 56,844 | 1000×1400 | `samples/basic/english.hwp` p0 print |

## Full PNG inventory (36)

| Path | Bytes | WxH | SHA-256 |
|---|---:|---|---|
| [mydocs/report/task_m100_5706/_renders/english.png](_renders/english.png) | 56,844 | 1000×1400 | `d3c91a02fff9` |
| [mydocs/report/task_m100_5706/_renders/form-filled.png](_renders/form-filled.png) | 20,849 | 1000×1400 | `7defbac1103a` |
| [mydocs/report/task_m100_5706/_renders/form.png](_renders/form.png) | 18,209 | 1000×1400 | `cf27ab14cee9` |
| [mydocs/report/task_m100_5706/_renders/request.png](_renders/request.png) | 53,446 | 1000×1400 | `76918d5b0549` |
| [mydocs/report/task_m100_5706/_renders/table.png](_renders/table.png) | 117,880 | 1000×1400 | `8a26ba338856` |
| [mydocs/report/task_m100_5706/skills/rhwp-agent-surface/page.png](skills/rhwp-agent-surface/page.png) | 59,145 | 1200×1600 | `093c0b43fdeb` |
| [mydocs/report/task_m100_5706/skills/rhwp-bug-hunter/page.png](skills/rhwp-bug-hunter/page.png) | 55,567 | 1200×1600 | `329d79a939e0` |
| [mydocs/report/task_m100_5706/skills/rhwp-bulk-pipeline/page.png](skills/rhwp-bulk-pipeline/page.png) | 56,844 | 1000×1400 | `d3c91a02fff9` |
| [mydocs/report/task_m100_5706/skills/rhwp-chief/page.png](skills/rhwp-chief/page.png) | 59,145 | 1200×1600 | `093c0b43fdeb` |
| [mydocs/report/task_m100_5706/skills/rhwp-cli/page.png](skills/rhwp-cli/page.png) | 56,844 | 1000×1400 | `d3c91a02fff9` |
| [mydocs/report/task_m100_5706/skills/rhwp-codex/page.png](skills/rhwp-codex/page.png) | 117,880 | 1000×1400 | `8a26ba338856` |
| [mydocs/report/task_m100_5706/skills/rhwp-contributor/page.png](skills/rhwp-contributor/page.png) | 56,844 | 1000×1400 | `d3c91a02fff9` |
| [mydocs/report/task_m100_5706/skills/rhwp-doc-triage/page.png](skills/rhwp-doc-triage/page.png) | 56,844 | 1000×1400 | `d3c91a02fff9` |
| [mydocs/report/task_m100_5706/skills/rhwp-exam-ingest/page.png](skills/rhwp-exam-ingest/page.png) | 56,844 | 1000×1400 | `d3c91a02fff9` |
| [mydocs/report/task_m100_5706/skills/rhwp-explore/page.png](skills/rhwp-explore/page.png) | 55,567 | 1200×1600 | `329d79a939e0` |
| [mydocs/report/task_m100_5706/skills/rhwp-fde/page.png](skills/rhwp-fde/page.png) | 59,145 | 1200×1600 | `093c0b43fdeb` |
| [mydocs/report/task_m100_5706/skills/rhwp-fidelity-compare/page.png](skills/rhwp-fidelity-compare/page.png) | 55,567 | 1200×1600 | `329d79a939e0` |
| [mydocs/report/task_m100_5706/skills/rhwp-form-fill/after.png](skills/rhwp-form-fill/after.png) | 20,849 | 1000×1400 | `7defbac1103a` |
| [mydocs/report/task_m100_5706/skills/rhwp-form-fill/before.png](skills/rhwp-form-fill/before.png) | 18,209 | 1000×1400 | `cf27ab14cee9` |
| [mydocs/report/task_m100_5706/skills/rhwp-form-fill/filled.png](skills/rhwp-form-fill/filled.png) | 20,849 | 1000×1400 | `7defbac1103a` |
| [mydocs/report/task_m100_5706/skills/rhwp-form-fill/page.png](skills/rhwp-form-fill/page.png) | 20,849 | 1000×1400 | `7defbac1103a` |
| [mydocs/report/task_m100_5706/skills/rhwp-handoff/page.png](skills/rhwp-handoff/page.png) | 59,145 | 1200×1600 | `093c0b43fdeb` |
| [mydocs/report/task_m100_5706/skills/rhwp-knowledge-map/page.png](skills/rhwp-knowledge-map/page.png) | 55,567 | 1200×1600 | `329d79a939e0` |
| [mydocs/report/task_m100_5706/skills/rhwp-mcp-session/page.png](skills/rhwp-mcp-session/page.png) | 56,844 | 1000×1400 | `d3c91a02fff9` |
| [mydocs/report/task_m100_5706/skills/rhwp-onboarding/page.png](skills/rhwp-onboarding/page.png) | 56,844 | 1000×1400 | `d3c91a02fff9` |
| [mydocs/report/task_m100_5706/skills/rhwp-provenance/page.png](skills/rhwp-provenance/page.png) | 53,446 | 1000×1400 | `76918d5b0549` |
| [mydocs/report/task_m100_5706/skills/rhwp-recipes/page.png](skills/rhwp-recipes/page.png) | 125,706 | 1200×1600 | `ed47aa1a52d2` |
| [mydocs/report/task_m100_5706/skills/rhwp-safe-edit/page.png](skills/rhwp-safe-edit/page.png) | 20,849 | 1000×1400 | `7defbac1103a` |
| [mydocs/report/task_m100_5706/skills/rhwp-security-sweep/page.png](skills/rhwp-security-sweep/page.png) | 53,446 | 1000×1400 | `76918d5b0549` |
| [mydocs/report/task_m100_5706/skills/rhwp-skill-author/english.png](skills/rhwp-skill-author/english.png) | 59,145 | 1200×1600 | `093c0b43fdeb` |
| [mydocs/report/task_m100_5706/skills/rhwp-skill-author/page.png](skills/rhwp-skill-author/page.png) | 55,567 | 1200×1600 | `329d79a939e0` |
| [mydocs/report/task_m100_5706/skills/rhwp-skill-router/page.png](skills/rhwp-skill-router/page.png) | 19,998 | 1200×1600 | `44241fbec0a5` |
| [mydocs/report/task_m100_5706/skills/rhwp-strategist/page.png](skills/rhwp-strategist/page.png) | 271,303 | 1200×1600 | `ebb42fa9376d` |
| [mydocs/report/task_m100_5706/skills/rhwp-table-exchange/page.png](skills/rhwp-table-exchange/page.png) | 117,880 | 1000×1400 | `8a26ba338856` |
| [mydocs/report/task_m100_5706/skills/rhwp-visual-regression/page.png](skills/rhwp-visual-regression/page.png) | 117,880 | 1000×1400 | `8a26ba338856` |
| [mydocs/report/task_m100_5706/skills/rhwp-work-receipt/page.png](skills/rhwp-work-receipt/page.png) | 56,844 | 1000×1400 | `d3c91a02fff9` |

## Missing skills (no PNG)

Catalog has 27 skills. All 27 have `skills/<id>/*.png`. Missing: **none**.
