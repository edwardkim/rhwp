# Task 215 Execution Plan: Single-Pass Layout Engine Phase 2 — Table Typesetting

## Goal

Implement **Break Token based table typesetting** in TypesetEngine to structurally resolve measurement-placement mismatch bugs in the existing 3-stage pipeline.

### Target Bugs

| Document | Symptom | Root Cause |
|----------|---------|------------|
| k-water-rfp.hwp | 14px overflow on pages 14~15, paragraph cropping | Mismatch between pagination's host_spacing calculation and layout's actual spacing |
| k-water-rfp.hwp | TYPESET_VERIFY sec1: 25→27 page difference | TypesetEngine table split logic incomplete |
| kps-ai.hwp | TYPESET_VERIFY sec0: 79→75 difference | Same |
| hwpp-001.hwp | TYPESET_VERIFY sec3: 57→55 difference | Same |

### Design Principles (Learned from Predecessor Engines)

1. **Chromium LayoutNG**: Break Token pattern — `layout(node, constraint) → (fragment, Option<break_token>)`
2. **LibreOffice Writer**: Master/Follow Chain — preserve resumption information during splits
3. **MS Word/OOXML**: cantSplit, tblHeader — row split rules and header row repetition
4. **Common**: Do not separate measurement and placement — **measure while placing**

Details: `mydocs/tech/investigations/issue-397/layout_engine_research.md`

---

## Implementation Plan (4 Steps)

### Step 1: BreakToken Data Structures and format_table() Implementation

**Goal**: Implement format_table() that calculates table height in a single pass

**Work items**:

1. Define `TypesetBreakToken` enum
2. `FormattedTable` struct — format() result for tables
3. `format_table()` implementation
   - Utilize existing height_measurer's MeasuredTable data
   - Calculate host_spacing using **same rules as layout**
   - Integrate spacing_before/after, outer_margin, host_line_spacing

**Verification**: Compare total_height between existing MeasuredTable and FormattedTable (difference report)

### Step 2: typeset_table() — fits → place / split Implementation

**Goal**: Implement table page splitting using Break Token pattern

### Step 3: Header Row Repetition, Footnotes, Multiple TAC Table Handling

**Goal**: Transfer all table split features from existing Paginator to TypesetEngine

### Step 4: Parallel Verification Enhancement and Cleanup

**Goal**: Achieve complete parity between TypesetEngine and Paginator, prepare for transition

## Key Design Points

### A. host_spacing Unification — Most Important Fix

The direct cause of current bugs is that **pagination and layout calculate host_spacing differently**.

In TypesetEngine, **calculate once using identical rules as layout in format_table()**, and use this value in all stages: fits/place/split.

### B. State Transfer via Break Token

Old: Directly calculate start_row/end_row and insert into PartialTable
New: Break Token explicitly conveys "where to resume"

### C. Measurement-Placement Unification

The height value returned by format_table() is **exactly identical** to the height value used by typeset_table() for placement. Since no separate height_measurer is involved, mismatch is structurally impossible.

## Expected Results

| Item | Phase 1 (Current) | Phase 2 (On Completion) |
|------|-------------------|------------------------|
| k-water-rfp OVERFLOW | 6 cases (including 14px) | 0 cases |
| TYPESET_VERIFY table section differences | 3 documents with differences | 0 cases |
| Measurement-placement mismatch possibility | Exists (3-stage pipeline) | None (single pass) |
| Table split accuracy | MeasuredTable dependent | Break Token based |
