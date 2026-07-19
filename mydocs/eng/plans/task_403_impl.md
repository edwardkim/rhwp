# Task 403 Implementation Plan: Delivering rhwp as a VSCode Extension

## Architecture Overview

```
┌───────────────────────────────���─────────────────────┐
│ VSCode Extension Host                               │
│                                                     │
│  extension.ts                                       │
│    └─ HwpEditorProvider (CustomReadonlyEditorProvider)│
│         ├─ openCustomDocument(): Read file binary    │
│         └─ resolveCustomEditor(): Create Webview     │
│              ├─ Send WASM binary (postMessage)       │
│              └─ Send HWP file data (postMessage)     │
���──────────────────���──────────────────────────────────┤
│ Webview (Sandbox)                                    │
│                                                     │
│  viewer.ts                                          │
│    ├─ WASM initialization (WebAssembly.instantiate)  │
│    ├─ HwpDocument creation (file binary → doc object)│
│    ├─ Page rendering (Canvas 2D)                     │
│    ├─ Virtual scroll (render only visible pages)     │
│    └─ Zoom/Navigation UI                             │
└─────────────────────────────────────────────────────┘
```

### Data Flow

```
Double-click .hwp file
  → Extension Host: workspace.fs.readFile() → Uint8Array
  → postMessage({ type: 'load', wasm: wasmBytes, file: hwpBytes })
  → Webview: WebAssembly.instantiate(wasmBytes)
  → Webview: new HwpDocument(hwpBytes)
  → Webview: renderPageToCanvas(pageNum, canvas, scale)
  → Document displayed on Canvas
```

## Implementation Steps

### Step 1: Project Scaffolding and Extension Manifest

**Goal**: Create `rhwp-vscode/` independent package, build VSCode extension basic structure

**Work**:
- Create `rhwp-vscode/` directory
- `package.json` — Extension manifest
  - `contributes.customEditors`: Register `*.hwp`, `*.hwpx`
  - `viewType`: `rhwp.hwpViewer`
  - `priority`: `default` (acts as default viewer)
- `tsconfig.json` — TypeScript configuration
- `webpack.config.js` ��� Dual bundle for Extension Host + Webview
  - Extension Host: `target: 'node'`, `externals: { vscode }`
  - Webview: `target: 'web'`, WASM as `asset/resource`
- `.vscodeignore` — Exclude unnecessary files from distribution
- `src/extension.ts` — Extension entry point (empty activate/deactivate)

**Deliverable**: `npm install && npm run compile` succeeds, extension loads in VSCode

### Step 2: Custom Editor Provider + File Loading

**Goal**: Build pipeline where opening an HWP file creates a Webview and delivers file data

**Work**:
- `src/hwp-editor-provider.ts` — `CustomReadonlyEditorProvider` implementation
  - `openCustomDocument()`: `workspace.fs.readFile(uri)` → `HwpDocument` (extension-side model)
  - `resolveCustomEditor()`: Generate Webview HTML, configure CSP, send file data
- Webview HTML template
  - CSP: `script-src ${cspSource}; style-src ${cspSource}; wasm-unsafe-eval`
  - Nonce-based script loading
- `src/webview/viewer.ts` — Webview entry point
  - `window.addEventListener('message')` → Receive file data
  - Return confirmation message (display filename, size)
- Extension Host → Webview message protocol definition
  ```typescript
  // Host → Webview
  { type: 'load', fileName: string, fileData: Uint8Array }
  // Webview → Host
  { type: 'ready' }
  { type: 'loaded', pageCount: number }
  ```

**Deliverable**: Open .hwp file → Webview displays filename/size

### Step 3: WASM Integration and Single Page Rendering

**Goal**: Load WASM in Webview and render first page on Canvas

**Work**:
- WASM bundling strategy implementation
  - Copy `pkg/rhwp_bg.wasm` to `dist/media/` during extension build
  - Extension Host reads WASM binary and sends to Webview via postMessage
  - Webview initializes with `WebAssembly.instantiate()`
- `src/webview/wasm-loader.ts` — WASM initialization module
  - Call wasm-bindgen generated JS (`rhwp.js`) init function with WASM byte array
  - Create `HwpDocument` instance
- `src/webview/page-canvas.ts` — Single page Canvas rendering
  - Call `HwpDocument.renderPageToCanvas(pageNum, canvas, scale)`
  - Handle DPI scaling (`window.devicePixelRatio`)
- Display page info (total pages, current page)

**Deliverable**: Open .hwp file → First page rendered on Canvas

### Step 4: Virtual Scroll and Multi-Page View

**Goal**: Complete document viewer that scrolls through all pages

**Work**:
- `src/webview/virtual-scroll.ts` — Virtual scroll implementation
  - Calculate total document height (sum heights from per-page `getPageInfo()`)
  - Create/render Canvas only for pages visible in viewport
  - Release Canvas for pages scrolled out, render newly visible pages
- `src/webview/zoom-control.ts` — Zoom control
  - Zoom in/out buttons (or Ctrl+mouse wheel)
  - Canvas scale adjustment based on zoom level
- Page navigation
  - Show current page / total pages in status bar
  - Jump to page on page number click
- Stylesheet (`src/webview/viewer.css`)
  - Use VSCode theme color variables (`--vscode-editor-background`, etc.)
  - Page shadow, spacing, scroll area layout

**Deliverable**: Full document scroll viewing + zoom + page navigation working

### Step 5: Design Document

**Goal**: Document the architecture based on implementation results

**Work**:
- `mydocs/tech/archive/vscode_extension_design.md` — Technical design document
  - Architecture diagram (Extension Host ↔ Webview ↔ WASM)
  - Message protocol specification
  - WASM loading strategy and CSP configuration
  - Directory structure and build pipeline
  - Differences from rhwp-studio and shared scope
  - Future extension roadmap (editing, search, outline)
- Final report

**Deliverable**: Design document complete, final report

## Technical Decisions

| Item | Decision | Reason |
|------|------|------|
| Editor Provider | `CustomReadonlyEditorProvider` | v1 is read-only. Future transition to `CustomEditorProvider` |
| WASM delivery | postMessage (Uint8Array) | asWebviewUri doesn't support WASM MIME type. Requires VSCode 1.57+ |
| Bundler | webpack | VSCode extension official recommendation. Dual bundle for Extension Host + Webview |
| Rendering | Canvas 2D | Directly reuse rhwp WASM's `renderPageToCanvas()` |
| CSP | `wasm-unsafe-eval` | Required to allow WebAssembly.instantiate() in Webview |

## Approval Request

Please review and approve the above implementation plan (5 steps). Implementation will begin from Step 1 upon approval.
