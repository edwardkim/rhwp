# Hancom WebGian Replacement Software Development Strategy

## 1. Background

### 1.1 Market Status
- Hancom WebGian is deployed and in operation at most government agencies
- Near-monopoly position resulting in virtually no customer feedback adoption
- Price increase notification for H1 2026 (annual subscription model, over $28,000/year)

### 1.2 Customer Pain Points
| Item | Details |
|------|---------|
| Performance | Structural limitations of JS-based architecture cause frequent complaints about slow speed |
| Feedback | Customer requests and improvement feedback are almost entirely ignored |
| Cost | Expensive subscription fees + continuous increases, vendor lock-in due to lack of alternatives |

### 1.3 Market Competition Analysis (HWP 5.0 Binary Format)

**Note**: While HWPX (XML-based) has multiple implementations, the situation for HWP 5.0 binary (.hwp) is quite different.

**Commercial Products -- Full "Read+Edit+Save+Rendering" Implementation**:
| Product | Developer | Notes |
|---------|----------|-------|
| Hangul/WebGian | Hancom | Original developer. Server-dependent OT architecture |
| A commercial web office | — | Independent engine (no Hancom SDK). Serverless WASM browser editing. Mobile/web/desktop support. Expanding B2B web office in the commercial space |

**Commercial Products -- Partial Implementation (Viewer/Conversion, Limited Editing)**:
| Product | Developer | Notes |
|---------|----------|-------|
| Document Viewer/Editor | S Co. | HWP 96-2024 viewer, document conversion server, Document AI. Government viewer market presence. Programmatic HWP binary generation API not confirmed. Server-based SaaS architecture |

**Open Source -- Only Partial Implementations Exist (No Rendering/Typesetting)**:
| Project | Language | Read | Write | Rendering | License |
|---------|----------|------|-------|-----------|---------|
| hwplib (neolord0) | Java | O | O | X | Apache 2.0 |
| libhwp (accforaus) | Kotlin | O | O | X | - |
| pyhwp (mete0r) | Python | O | X | X | AGPL v3 |
| hwp.js (hahnlee) | JS | O | X | HTML ~20% | MIT |
| hwpers (Indosaram) | Rust | O | Claimed | SVG claimed | - |
| hwp-rs (hahnlee) | Rust | O | X | X | Apache 2.0 |

**LibreOffice**: The built-in hwpfilter only supports **HWP 3.0 and earlier**; HWP 5.0 can only be read via third-party extensions (H2Orestart) by converting to ODT first

**Key Conclusions**:
- Only **Hancom and a small number of commercial web offices** have fully implemented HWP 5.0 binary "read+edit+save+rendering (typesetting)"
- S Co. is strong in viewing/conversion but a programmatic HWP binary generation API is not confirmed
- Among publicly available open-source projects we have reviewed, **few cover editing + re-typesetting + rendering together**
- This product joins the small set of full implementations
- **Within the scope we have reviewed, this appears to be the first publicly available WebAssembly-based client-only (no server required) HWP save implementation** -- Hancom depends on servers; some commercial web offices offer WASM editing but a programmatic generation API for AI agents is not confirmed
- **Publicly available HWP generation tools targeting AI Agents are scarce** (commercial or open source) -- a differentiated position
- This represents both a high technical entry barrier and a source of competitive advantage
- For detailed vision and roadmap, see [project_vision.md](../../tech/archive/project_vision.md)

### 1.4 Business Model
- **License Type**: Annual subscription model (SaaS/On-premise)
- Product intended for **commercial sale**
- **Product Name**: HWP 5.0 Compatible Module for Rust (HWP Document Compatibility Tool)
- **Target Customers**: Government agencies and enterprises currently using Hancom WebGian

### 1.5 Ultimate Vision -- AI Agent HWP Report Generation

The fundamental reason for choosing a Rust clean-room design from the start:

**"The ultimate goal is to enable AI agents to automatically generate HWP reports"**

| Stage | Goal | Status |
|-------|------|--------|
| Stage 1 | Rust HWP Core Engine (parsing/rendering/editing/saving) | In progress |
| Stage 2 | WebAssembly-based web editor (Hancom replacement) | In progress |
| Stage 3 | HWP document generation API for AI agents | **Foundation technology secured** |

**Stage 3 Foundation Technology Status**:
- HTML table -> HWP table conversion: **Implemented** via OLE 2.0 clipboard pasteHtml
- HWP binary saving: Modified document serialization to .hwp **implemented**
- This means the pipeline for AI agents to write RAG-based expert reports in HTML -> convert/save to HWP via rhwp MCP tool is **technically feasible**
- In addition to existing AI report workflows (HTML -> PDF), a new **HTML -> HWP** path is now available

**Why Rust Clean-Room?**
- **Programmable engine**: A library-form engine that AI agents can directly create/edit HWP documents through APIs
- **No server required**: WASM-based independent execution on client/edge -> used as a lightweight tool for AI agents
- **Hancom dependency removal**: HWP generation without Hancom SDK/server, freely integrable into AI workflows
- **Government report automation**: A market is opening for AI to automatically author standardized HWP report forms

**Market Opportunity**:
- Government agencies are mandated to produce reports in HWP format -> latent demand for AI report automation
- Currently, publicly available tools for AI to directly generate HWP are **scarce** (even Hancom's SDK is server-dependent)
- Rust library + WASM is a **rare** combination that AI agents can call directly for HWP generation

**AI Report Generation Pipeline (Currently Feasible)**:
```
AI Agent (Claude/GPT)
    |  RAG-based expert report writing
    v
HTML Report (with tables, formatting)
    |
    |-- Existing path: html_to_pdf() -> PDF file
    |
    +-- New path: rhwp MCP Tool -> pasteHtml() -> HWP save -> .hwp file
```

### 1.6 Rendering Quality Comparison (2026-02-16)

Rendering quality was compared across major domestic commercial web viewers using `BookReview.hwp`, a default example file provided with Hancom HWP.

**Test subject**: Page 2 -- multiple page-anchored text boxes + paragraph text flow + Master Page text box

| Product | Result | Notes |
|---------|--------|-------|
| Hancom HWP (Desktop) | Normal | Reference baseline |
| **rhwp (This product)** | **Normal** | Identical result to Hancom HWP (both SVG/Canvas) |
| Hancom Docs (Web) | Processing failed | Text box + paragraph flow not handled |
| A commercial web office | Processing failed | Text box + paragraph flow not handled |
| S Co. (Web) | Processing failed | Text box + paragraph flow not handled |

Additional comparison using `KTX.hwp`, another Hancom HWP default example file.

**Test subject**: Landscape 1 page -- gradients/arrow markers/numerous shapes + precise layout

| Product | Quality | Notes |
|---------|---------|-------|
| Hancom HWP (Desktop) | 100% | Reference baseline |
| **rhwp (This product)** | **95%** | |
| A commercial web office | 90% | |
| Naver (Web) | 80% | |
| S Co. (Web) | 70% | |

**Key Implications**:
- Even Hancom's own default example files could not be processed by all 3 major domestic commercial web viewers
- This product achieves rendering results identical to or closest to Hancom HWP Desktop
- Precise rendering of text boxes + paragraph flow, shapes + gradients represents a high technical entry barrier

**Fundamental Difference in Development Approach**:
- Competitors could also achieve the same level of precise rendering by leveraging AI
- However, for AI to work accurately, accumulation of **HWP spec documentation, prompt systems, and codebase context** is essential
- The true competitive moat of this product is not the AI tool itself, but the **prompt engineering know-how and technical documentation assets** accumulated through 95 tasks
- Without this accumulation, adopting AI alone cannot achieve precise rendering like HWP text flow
- Result: Overwhelming advantage in customer feedback response speed and efficiency

## 2. Product Positioning

### 2.1 Core Value
- **HWP Native Compatibility** -- A rare engine that parses/renders/edits HWP binaries natively without a remote server, aside from Hancom
- **Drop-in Replacement** -- Replace existing customer environments with minimal changes
- **Performance Advantage** -- Overwhelmingly faster than JS-based systems with Rust + WebAssembly native engine
- **Price Competitiveness** -- Reasonable pricing compared to monopolistic subscription fees
- **Customer Responsiveness** -- Active feedback adoption and customized support

### 2.2 Technical Differentiators

| Item | Hancom WebGian | rhwp-studio |
|------|---------------|-------------|
| Core Engine | JavaScript | Rust + WebAssembly |
| Rendering | DOM-based | Canvas-based |
| File Parsing | JS parsing | Rust native parsing |
| Performance | Slow with large documents | Native-level performance (direct printing support) |
| Architecture | Monolithic JS | Rust core + TS frontend separation |
| **Printing** | **Browser/server-dependent printing** | **Direct PS/PCL generation printing** |

> [!TIP]
> For detailed architecture, refer to [direct_printing_guideline.md](file:///home/edward/vsworks/rhwp/mydocs/tech/archive/direct_printing_guideline.md).

## 3. Interoperability Strategy

### 3.1 Goal
Ensure API compatibility so that customers who have deployed Hancom WebGian can replace it **with almost no changes to their existing JS scripts**

### 3.2 API Compatibility Scope
- Complete analysis of 449 Hancom WebGian APIs in the feature specification (task_43_feature_def.md)
- 312 Actions and 137 SetID/Properties identified
- 7 categories: Document Management, View, Editing Environment, Cursor/Editing, Formatting, Objects, Tools

### 3.3 Compatibility Layer Architecture
```
Customer's Existing JS Scripts
        |
        v
+---------------------+
|  StudioExtensionAPI  |  <- Hancom WebGian API compatible interface
|  (Compatibility Layer)|
+---------------------+
|  CommandDispatcher    |  <- Internal command system
+---------------------+
|  WASM Bridge         |  <- TypeScript <-> Rust communication
+---------------------+
|  Rust Core Engine    |  <- HWP parsing/rendering/editing engine
+---------------------+
```

### 3.4 Replacement Scenario
1. Change the Hancom WebGian script reference on the customer's web page to rhwp-studio
2. Existing JS API call code is handled by the StudioExtensionAPI compatibility layer
3. Provide mapping/fallback for unsupported APIs as needed

## 4. Copyright/Legal Considerations

### 4.1 Protected Areas (Our Originality)
| Item | Status | Basis |
|------|--------|-------|
| Source Code | Fully independent implementation | Written from scratch in Rust, no Hancom code referenced/copied |
| Engine Architecture | Independent design | Rust+WASM based, completely different structure from Hancom |
| Rendering Method | Independent implementation | Canvas-based (Hancom uses DOM-based) |

### 4.2 Non-Copyrightable Areas
| Item | Basis |
|------|-------|
| HWP File Format | Parsing based on officially published Hancom spec (hwp_spec_5.0) |
| API Interface | APIs are ideas/functional elements and not subject to copyright protection |
| Functional UI/UX | Menu structure, shortcuts, feature layout are functional elements not subject to copyright |
| Interoperability | Interface compatibility for interoperability purposes is a copyright exception |

### 4.3 Case Law Basis
- **Oracle v. Google (2021, US Supreme Court)**: API interface reimplementation recognized as fair use. Ruled that reimplementing Java API declarations in Android is not copyright infringement.
- **EU Software Directive (2009/24/EC)**: Reverse engineering and interface compatibility for interoperability purposes is permitted
- **Korean Copyright Act Article 101-4**: Actions necessary to obtain information for program interoperability do not constitute copyright infringement

### 4.4 HWP Public Spec Copyright Clause (hwp_spec_5.0)

The copyright clause from Hancom's officially published "Hangul Document File Structure 5.0" specification:

**Permitted**: Anyone may freely view, copy, distribute, publish, and use

**Copyright of Derived Works**: "All copyrights of results developed by referencing this document shall belong to the individual or organization that developed the results"

**Required Notice**: The following notice must be included in the product's user interface, manual, help, and source code:
> "This product was developed with reference to Hancom's Hangul document file (.hwp) public specification."

**Restriction**: If attempting to exercise exclusive/monopolistic rights against Hancom based on spec-derived results, Hancom may actively exercise its rights

### 4.5 Unfair Competition Prevention Act (Critical)

A more challenging risk than copyright. Hancom could argue that their API ecosystem and user experience, built through significant investment, has been "free-ridden" upon to unlawfully benefit.

**Hancom's Expected Attack Logic**:
- Unfair Competition Prevention Act Article 2, Paragraph 1, Item Cha (General Clause): "Using another party's results created through substantial investment or effort in an unfair manner contrary to fair trade practices or competitive order for one's own business"
- Argument that Hancom's API design/ecosystem constitutes "results of substantial investment and effort"

**Defense Logic -- "Independent Engine + Compatibility Adapter" Structure**:
1. **The core product is an independent HWP editing engine**: Proprietary technology built from scratch with Rust+WASM, a completely different architecture from Hancom
2. **The compatibility layer is a supplementary convenience feature**: An optional "legacy code compatibility adapter" provided to protect customers' existing code assets
3. **Legitimacy of interoperability purpose**: A legitimate competitive purpose of resolving customer vendor lock-in
4. **APIs are merely interfaces**: Minimal interface mapping for functional compatibility, not replication of Hancom's internal implementation

**Product Structural Defense**:
```
+----------------------------------+
|   Independent Product (Core Value)|
|   - Rust HWP Engine (independent) |
|   - WASM Canvas Rendering (independent)|
|   - Original UI/UX (independent design) |
+----------------------------------+
|   Compatibility Adapter (Optional)|  <- Separable optional layer
|   - StudioExtensionAPI           |
|   - Legacy JS script compat mapping|
+----------------------------------+
```

**Execution Principles**:
- Compatibility adapter is implemented as a separate package from the core engine (physical separation)
- Marketing positions as "HWP document editor" not "Hancom replacement"
- Compatibility adapter is described as "migration support for customer convenience"

**Compatibility Adapter Implementation Strategy**:

The rhwp core designs its own API, and the compatibility adapter is separated into a distinct package that delegates via the adapter pattern.

```
[rhwp Independent API]           <- Self-designed, independently implemented
    ^                               exportHwp(), loadDocument(), etc.
[Compatibility Adapter Layer]    <- Separate package (rhwp-compat-hwpctrl/)
    ^                               Adapter pattern: signature conversion + delegation
[HwpCtrl Compatible Interface]   <- Called by customer's existing JS scripts
                                    SaveAs(), SaveDocument(), etc. 67 methods
```

- **Physical Separation**: Implemented as a separate `rhwp-compat-hwpctrl/` package. Not included in rhwp-studio core
- **Adapter Pattern**: Receives HwpCtrl method signatures and internally delegates by calling rhwp's independent API
  - e.g.: `SaveAs(fileName, format)` -> `wasm.exportHwp()` + Blob download
  - e.g.: `SetCharShape(key, value)` -> `wasm.applyCharFormat()`
- **Optional Application**: Adapter package is loaded additionally only when customers need existing JS script compatibility
- **Bulk Implementation**: All 67 HwpCtrl methods implemented at once, with all internal logic processed through rhwp independent API calls
- **Documentation**: Explicitly stated as "an adapter to help migration from existing HwpCtrl-based systems to rhwp"

This structure demonstrates at the architecture level that this is "providing API compatibility (interoperability)" not "API replication."

### 4.6 Precautions
| Item | Response |
|------|----------|
| **Spec Notice Text** | Insert Hancom's required notice text in product UI (About dialog, etc.), source code, and manuals |
| Trademark Usage | Prohibit use of Hancom trademarks ("Hangul", "Arae-a Hangul", "WebGian", "HWP") in product name/marketing |
| Icon Design | Use original designs rather than copying Hancom's proprietary icons (independently crafted SVG sprites) |
| Code Clean-Room | Maintain records of independent development without viewing/referencing Hancom source code |
| Development History | Independent development process provable via Git commit history |
| Non-Assertion Against Hancom | Prohibition of exercising exclusive/monopolistic rights against Hancom based on spec-derived results |

## 5. Development Status (As of 2026-02-13)

### 5.1 Progress
- Tasks 43-62 completed (20 tasks in 2 days)
- Rust core engine: HWP parsing, rendering, pagination, editing features
- rhwp-studio frontend: Canvas viewer, editing engine, menu/toolbar/status bar UI
- WASM build pipeline established

### 5.2 Feature Implementation Rate (Based on 312 Actions from Feature Specification)
- Implemented: approximately 50 (cursor movement 12, selection 8, text editing 10, formatting 8, view 6, file 4, other 2)
- vs Total: approximately 16%
- Target: Product-ready at 80% (approximately 250) completion
- Stage 1: February 28, 2026
  Estimated 80% development completion

## 6. Competitive Advantage Summary

| Item | Details |
|------|---------|
| Performance | Rust+WASM > JS (10x+) |
| Compatibility | API-compatible drop-in replacement |
| Cost | Saves over $28,000/year |
| Responsiveness | Active customer feedback adoption |
| Legal Safety | Independent implementation + interoperability basis |
| AI Extensibility | HWP generation API for agents |
| RAG Extensibility | Semantic chunking and domain classification HWP-SEMANTIC product integration |
