# Task 181: Hancom Equation Rendering Implementation — Execution Plan

## Background

HWP file equation (Equation) controls are currently not parsed or rendered.

### Current Status

| Item | Binary HWP | HWPX |
|------|-----------|------|
| Parser | `CTRL_EQUATION` not handled → `Control::Unknown` | `parse_equation()` → `Control::Shape(Rectangle)` (layout only) |
| Model | No dedicated model | Same |
| Rendering | Empty rectangle or ignored | Empty rectangle |

### HWP Equation System Overview

Hancom equations are expressed in a **script language** similar to LaTeX:
- `1 over 2` → fraction 1/2
- `sqrt x` → square root
- `E=mc^2` → superscript
- `sum_{i=0}^n` → sigma sum
- `matrix{a & b # c & d}` → matrix

Scripts are stored as UTF-16 strings in `HWPTAG_EQEDIT` records (binary) or `<hp:script>` elements (HWPX).

## Goal

1. Extract equation scripts from both binary HWP and HWPX
2. Parse equation scripts into AST (Abstract Syntax Tree)
3. Render AST as SVG elements (fraction lines, root symbols, subscript/superscript placement, etc.)
4. Integrate equation controls into existing rendering pipeline

## Rendering Strategy

Adopt **HWP equation script → AST → direct SVG rendering** instead of LaTeX conversion.

Reasons:
- Embedding a LaTeX rendering engine (MathJax, etc.) adds excessive dependencies
- In SVG environments, directly placing text, lines, and paths is natural
- Equation layout rules are relatively simple (fraction/subscript/root placement formulas)

## Implementation Steps

### Step 1: Model + Parser (Equation Data Extraction)
Extract equation script strings from binary HWP / HWPX and store in model.

### Step 2: Equation Tokenizer + Symbol Mapping
Convert equation script strings to token streams (porting from Python reference).

### Step 3: Equation Parser (Token → AST)
Convert token streams to AST (EqNode tree) using recursive descent parser.

### Step 4: Equation Layout Engine (AST → Layout Boxes)
Convert AST to sized/positioned LayoutBox tree using font metrics.

### Step 5: SVG Rendering + Pipeline Integration
Convert LayoutBox tree to SVG elements, integrate into existing rendering pipeline.

### Step 6: Testing + Verification
Unit tests for each module + visual verification with equation-containing HWP files.

## Scope

### Included
- Basic commands: OVER, SQRT, ^, _, INT/SUM/PROD, MATRIX, CASES, LEFT/RIGHT
- Decorations: hat, bar, vec, dot, tilde, etc.
- Greek letters + math symbols (Unicode mapping)
- Font styles: rm, bold, it
- Spacing/newlines: ~, `, #, &
- Auto-sized brackets: LEFT/RIGHT

### Excluded (Subsequent tasks)
- LADDER, SLADDER, LONGDIV (special layouts)
- SCALE (size ratio adjustment, HWP3 legacy)
- REL, BUILDREL (text above/below arrows)
- COLOR (color specification)
- Equation editing UI
