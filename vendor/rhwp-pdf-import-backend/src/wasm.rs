// SPDX-License-Identifier: MIT OR Apache-2.0
//! WebAssembly bindings for PDF Oxide.
//!
//! Provides a JavaScript/TypeScript API for PDF operations in browser
//! environments. Requires the `wasm` feature flag.
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import init, { WasmPdfDocument, WasmPdf } from 'pdf_oxide';
//!
//! await init();
//!
//! // Read an existing PDF
//! const response = await fetch('document.pdf');
//! const bytes = new Uint8Array(await response.arrayBuffer());
//! const doc = new WasmPdfDocument(bytes);
//! console.log(`Pages: ${doc.pageCount()}`);
//! console.log(doc.extractText(0));
//! console.log(doc.toMarkdown(0));
//!
//! // Create a new PDF from Markdown
//! const pdf = WasmPdf.fromMarkdown("# Hello\n\nWorld");
//! const pdfBytes = pdf.toBytes(); // Uint8Array
//!
//! // Edit a PDF
//! doc.setTitle("My Document");
//! doc.setPageRotation(0, 90);
//! const edited = doc.saveToBytes(); // Uint8Array
//! doc.free();
//! ```

use wasm_bindgen::prelude::*;

use crate::api::PdfBuilder;
use crate::converters::ConversionOptions;
use crate::document::PdfDocument;
use crate::editor::{
    DocumentEditor, EncryptionAlgorithm, EncryptionConfig, Permissions, SaveOptions,
};
use crate::search::{SearchOptions, TextSearcher};

// ============================================================================
// Logging
// ============================================================================

/// Set the maximum log level for pdf_oxide messages.
///
/// Accepts one of: `"off"`, `"error"`, `"warn"` / `"warning"`, `"info"`,
/// `"debug"`, `"trace"`. Case-insensitive. Default is `"off"` — the library
/// is silent unless explicitly enabled.
///
/// Logs are forwarded to the browser console (`console.log`, `console.warn`,
/// `console.error`, etc.). Fixes issue #280.
///
/// @example
/// ```javascript
/// import init, { setLogLevel } from "pdf-oxide-wasm";
/// await init();
/// setLogLevel("warn");
/// ```
#[wasm_bindgen(js_name = "setLogLevel")]
pub fn set_log_level(level: &str) -> Result<(), JsValue> {
    use log::{Level, LevelFilter};

    let (filter, console_level) = match level.to_ascii_lowercase().as_str() {
        "off" | "none" | "disabled" => (LevelFilter::Off, None),
        "error" => (LevelFilter::Error, Some(Level::Error)),
        "warn" | "warning" => (LevelFilter::Warn, Some(Level::Warn)),
        "info" => (LevelFilter::Info, Some(Level::Info)),
        "debug" => (LevelFilter::Debug, Some(Level::Debug)),
        "trace" => (LevelFilter::Trace, Some(Level::Trace)),
        other => {
            return Err(JsValue::from_str(&format!(
                "invalid log level '{}': expected off, error, warn, info, debug, or trace",
                other
            )));
        },
    };

    // console_log::init_with_level is idempotent on our side: we guard with
    // a static flag so repeated calls just update the max level.
    static INIT: std::sync::Once = std::sync::Once::new();
    if let Some(lvl) = console_level {
        INIT.call_once(|| {
            let _ = console_log::init_with_level(lvl);
        });
    }
    log::set_max_level(filter);
    Ok(())
}

/// Disable all pdf_oxide log output — convenience wrapper for
/// `setLogLevel("off")`.
#[wasm_bindgen(js_name = "disableLogging")]
pub fn disable_logging() {
    log::set_max_level(log::LevelFilter::Off);
}

// ============================================================================
// Standalone barcode SVG generation (no document needed)
// ============================================================================

/// Generate a 1D barcode as an SVG string.
///
/// `barcodeType`: 0=Code128, 1=Code39, 2=EAN13, 3=EAN8, 4=UPCA, 5=ITF, 6=Code93, 7=Codabar.
#[cfg(feature = "barcodes")]
#[wasm_bindgen(js_name = "generateBarcodeSvg")]
pub fn generate_barcode_svg(barcode_type: i32, data: String) -> Result<String, JsValue> {
    use crate::writer::{BarcodeGenerator, BarcodeOptions, BarcodeType};
    let bt = match barcode_type {
        0 => BarcodeType::Code128,
        1 => BarcodeType::Code39,
        2 => BarcodeType::Ean13,
        3 => BarcodeType::Ean8,
        4 => BarcodeType::UpcA,
        5 => BarcodeType::Itf,
        6 => BarcodeType::Code93,
        7 => BarcodeType::Codabar,
        _ => {
            return Err(JsValue::from_str(&format!(
                "unknown barcodeType {barcode_type}; valid values are 0–7"
            )))
        },
    };
    BarcodeGenerator::generate_1d_svg(bt, &data, &BarcodeOptions::default())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Generate a QR code as an SVG string.
///
/// `errorCorrection`: 0=Low, 1=Medium, 2=Quartile, 3=High. `size`: advisory pixel size.
#[cfg(feature = "barcodes")]
#[wasm_bindgen(js_name = "generateQrSvg")]
pub fn generate_qr_svg(data: String, error_correction: i32, size: u32) -> Result<String, JsValue> {
    use crate::writer::{BarcodeGenerator, QrCodeOptions, QrErrorCorrection};
    let ec = match error_correction {
        0 => QrErrorCorrection::Low,
        2 => QrErrorCorrection::Quartile,
        3 => QrErrorCorrection::High,
        _ => QrErrorCorrection::Medium,
    };
    let opts = QrCodeOptions::new().size(size).error_correction(ec);
    BarcodeGenerator::generate_qr_svg(&data, &opts).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ============================================================================
// split-by-bookmarks (#482) — free functions binding the Rust core
// ============================================================================

fn wasm_split_opts(
    title_prefix: Option<String>,
    ignore_case: bool,
    level: u32,
    include_front_matter: bool,
) -> crate::split_bookmarks::SplitByBookmarksOptions {
    crate::split_bookmarks::SplitByBookmarksOptions {
        title_prefix,
        ignore_case,
        level: crate::split_bookmarks::BookmarkLevel::from_u32(level),
        include_front_matter,
        ..Default::default()
    }
}

/// Plan a bookmark split without producing PDFs. Returns a JSON array
/// of segment objects (`index, startPage…` shape from
/// `BookmarkSegment`). `level`: 0 = all depths, 1 = top-level.
#[wasm_bindgen(js_name = "planSplitByBookmarks")]
pub fn plan_split_by_bookmarks(
    src_bytes: &[u8],
    title_prefix: Option<String>,
    ignore_case: bool,
    level: u32,
    include_front_matter: bool,
) -> Result<JsValue, JsValue> {
    let doc = crate::document::PdfDocument::from_bytes(src_bytes.to_vec())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let opts = wasm_split_opts(title_prefix, ignore_case, level, include_front_matter);
    let segs = crate::split_bookmarks::plan_split_by_bookmarks(&doc, &opts)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&segs).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Split at bookmark boundaries. Returns a JSON array of
/// `[segment, bytes]` pairs (bytes as a number array; source
/// unmodified).
#[wasm_bindgen(js_name = "splitByBookmarks")]
pub fn split_by_bookmarks(
    src_bytes: &[u8],
    title_prefix: Option<String>,
    ignore_case: bool,
    level: u32,
    include_front_matter: bool,
) -> Result<JsValue, JsValue> {
    let opts = wasm_split_opts(title_prefix, ignore_case, level, include_front_matter);
    let parts = crate::split_bookmarks::split_by_bookmarks_to_bytes(src_bytes, &opts)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&parts).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ============================================================================
// crypto governance policy (#230) — free functions binding the Rust core
// ============================================================================

/// Install the process-wide runtime crypto policy from its grammar
/// string (`"compat"|"strict"|"fips-strict"[;…]`). Fail-closed:
/// throws on an unparseable spec (policy NOT installed) or if a
/// policy is already set. Default (never set) is `compat`.
#[wasm_bindgen(js_name = "setCryptoPolicy")]
pub fn set_crypto_policy(spec: &str) -> Result<(), JsValue> {
    let policy: crate::crypto::SecurityPolicy = spec
        .parse()
        .map_err(|e: crate::crypto::PolicyParseError| JsValue::from_str(&e.to_string()))?;
    crate::crypto::set_policy(policy).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// The active crypto policy as its canonical grammar string.
#[wasm_bindgen(js_name = "cryptoPolicy")]
pub fn crypto_policy() -> String {
    crate::crypto::active_policy().to_string()
}

/// The cryptographic algorithm tokens exercised so far this process
/// (governance report), as a JSON string array.
#[wasm_bindgen(js_name = "cryptoInventory")]
pub fn crypto_inventory() -> Result<JsValue, JsValue> {
    let tokens: Vec<&'static str> = crate::crypto::inventory()
        .into_iter()
        .map(|a| a.token())
        .collect();
    serde_wasm_bindgen::to_value(&tokens).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// A CycloneDX 1.6 Cryptographic Bill of Materials (JSON string) of the
/// algorithms exercised so far this process (#230 Phase F).
#[wasm_bindgen(js_name = "cryptoCbom")]
pub fn crypto_cbom() -> String {
    crate::crypto::cbom_json()
}

/// #519: Air-gapped OCR model manifest — JSON (detector + every
/// supported language's cache filenames and source URLs).
///
/// WASM provisioning is **host-side**: browser/WASM has no filesystem
/// or network-to-disk, so a download-to-cache prefetch cannot run
/// here. This manifest is informational — it lets the JS host learn
/// which model files/URLs to fetch and bundle (or ship out of band)
/// before driving OCR. There is intentionally no `prefetchModels` in
/// the WASM surface (see `prefetchAvailable`, which always returns
/// `false`).
#[wasm_bindgen(js_name = "modelManifest")]
pub fn model_manifest() -> String {
    crate::extractors::auto::AutoExtractor::model_manifest()
}

/// #519: Whether this build can download OCR models to a local cache.
/// Always `false` in WASM — provisioning is host-side (see
/// `modelManifest`).
#[wasm_bindgen(js_name = "prefetchAvailable")]
pub fn prefetch_available() -> bool {
    false
}

// ============================================================================
// WasmPdfDocument — read, convert, search, extract, and edit PDFs
// ============================================================================

use std::sync::{Arc, Mutex};

/// A PDF document loaded from bytes for use in WebAssembly.
///
/// Create an instance by passing PDF file bytes to the constructor.
/// Call `.free()` when done to release memory.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmPdfDocument {
    inner: Arc<Mutex<PdfDocument>>,
    /// Raw bytes for editor initialization (kept for lazy editor creation)
    raw_bytes: Arc<Vec<u8>>,
    /// Lazy-initialized editor for mutation operations
    editor: Option<Arc<Mutex<DocumentEditor>>>,
}

#[wasm_bindgen]
impl WasmPdfDocument {
    /// Ensure the editor is initialized, creating it from the raw bytes if needed.
    fn ensure_editor(&mut self) -> Result<Arc<Mutex<DocumentEditor>>, JsValue> {
        if self.editor.is_none() {
            let editor = DocumentEditor::from_bytes(self.raw_bytes.to_vec())
                .map_err(|e| JsValue::from_str(&format!("Failed to open editor: {}", e)))?;
            self.editor = Some(Arc::new(Mutex::new(editor)));
        }
        Ok(self
            .editor
            .as_ref()
            .expect("editor just initialized")
            .clone())
    }
}

#[wasm_bindgen]
impl WasmPdfDocument {
    // ========================================================================
    // Constructor
    // ========================================================================

    /// Load a PDF document from raw bytes.
    ///
    /// @param data - PDF file contents as Uint8Array
    /// @param password - Optional password for encrypted PDFs
    /// @throws Error if the PDF is invalid or cannot be parsed
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8], password: Option<String>) -> Result<WasmPdfDocument, JsValue> {
        #[cfg(feature = "wasm")]
        console_error_panic_hook::set_once();

        let bytes = data.to_vec();
        let inner = PdfDocument::from_bytes(bytes.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to open PDF: {}", e)))?;

        if let Some(pw) = password {
            inner
                .authenticate(pw.as_bytes())
                .map_err(|e| JsValue::from_str(&format!("Authentication failed: {}", e)))?;
        }

        Ok(WasmPdfDocument {
            inner: Arc::new(Mutex::new(inner)),
            raw_bytes: Arc::new(bytes),
            editor: None,
        })
    }

    // ========================================================================
    // Group 1: Core Read-Only
    // ========================================================================

    /// Get the number of pages in the document.
    #[wasm_bindgen(js_name = "pageCount")]
    pub fn page_count(&mut self) -> Result<usize, JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .page_count()
            .map_err(|e| JsValue::from_str(&format!("Failed to get page count: {}", e)))
    }

    /// Count existing PDF signatures. Returns 0 when the document has
    /// no AcroForm or no signed signature fields.
    #[cfg(feature = "signatures")]
    #[wasm_bindgen(js_name = "signatureCount")]
    pub fn signature_count(&mut self) -> Result<usize, JsValue> {
        let mut doc = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        crate::signatures::count_signatures(&mut doc)
            .map_err(|e| JsValue::from_str(&format!("Failed to count signatures: {}", e)))
    }

    /// Enumerate existing PDF signatures. Each entry is a
    /// `WasmSignature` (inspection-only) mirroring the C# and Python
    /// Signature surfaces.
    #[cfg(feature = "signatures")]
    #[wasm_bindgen(js_name = "signatures")]
    pub fn signatures(&mut self) -> Result<Vec<WasmSignature>, JsValue> {
        let mut doc = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let list = crate::signatures::enumerate_signatures(&mut doc)
            .map_err(|e| JsValue::from_str(&format!("Failed to enumerate signatures: {}", e)))?;
        Ok(list
            .into_iter()
            .map(|info| WasmSignature { info })
            .collect())
    }

    /// The document's Document Security Store (`/DSS`) as a `Dss`, or
    /// `undefined` if absent. Mirrors Rust `signatures::read_dss`.
    #[cfg(feature = "signatures")]
    #[wasm_bindgen(js_name = "dss")]
    pub fn dss(&mut self) -> Result<Option<WasmDss>, JsValue> {
        let doc = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        crate::signatures::read_dss(&doc)
            .map(|opt| opt.map(|dss| WasmDss { dss }))
            .map_err(|e| JsValue::from_str(&format!("Failed to read DSS: {}", e)))
    }

    /// Get the PDF version as [major, minor].
    #[wasm_bindgen(js_name = "version")]
    pub fn version(&self) -> Result<Vec<u8>, JsValue> {
        let (major, minor) = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .version();
        Ok(vec![major, minor])
    }

    /// Authenticate with a password to decrypt an encrypted PDF.
    ///
    /// @param password - The password string
    /// @returns true if authentication succeeded
    #[wasm_bindgen(js_name = "authenticate")]
    pub fn authenticate(&mut self, password: &str) -> Result<bool, JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .authenticate(password.as_bytes())
            .map_err(|e| JsValue::from_str(&format!("Authentication failed: {}", e)))
    }

    /// Check if the document has a structure tree (Tagged PDF).
    #[wasm_bindgen(js_name = "hasStructureTree")]
    pub fn has_structure_tree(&mut self) -> Result<bool, JsValue> {
        Ok(matches!(
            self.inner
                .lock()
                .map_err(|_| JsValue::from_str("Mutex lock failed"))?
                .structure_tree(),
            Ok(Some(_))
        ))
    }

    // ========================================================================
    // Group 2: Text Extraction
    // ========================================================================

    /// Extract plain text from a single page.
    ///
    /// @param page_index - Zero-based page number
    /// @param region - Optional [x, y, width, height] to filter by
    #[wasm_bindgen(js_name = "extractText")]
    pub fn extract_text(
        &mut self,
        page_index: usize,
        region: JsValue, // Use JsValue to allow optional/undefined from JS
    ) -> Result<String, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;

        if !region.is_undefined() && !region.is_null() {
            let r: Vec<f32> = serde_wasm_bindgen::from_value(region)
                .map_err(|_| JsValue::from_str("Invalid region format. Expected [x, y, w, h]"))?;

            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            inner
                .extract_text_in_rect(
                    page_index,
                    crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
                    crate::layout::RectFilterMode::Intersects,
                )
                .map_err(|e| JsValue::from_str(&format!("Failed to extract text: {}", e)))
        } else {
            inner
                .extract_text(page_index)
                .map_err(|e| JsValue::from_str(&format!("Failed to extract text: {}", e)))
        }
    }

    /// Extract plain text from all pages, separated by form feed characters.
    #[wasm_bindgen(js_name = "extractAllText")]
    pub fn extract_all_text(&mut self) -> Result<String, JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .extract_all_text()
            .map_err(|e| JsValue::from_str(&format!("Failed to extract all text: {}", e)))
    }

    /// Extract a page as structured typed regions (issue #536), returned as a
    /// JSON string (a `StructuredPage`); parse with `JSON.parse` on the JS side.
    #[wasm_bindgen(js_name = "extractStructured")]
    pub fn extract_structured(&mut self, page_index: usize) -> Result<String, JsValue> {
        let structured = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .extract_structured(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to extract structured: {}", e)))?;
        serde_json::to_string(&structured)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    /// Identify and remove headers.
    ///
    /// Uses spec-compliant /Artifact tags when available (100% accuracy), or
    /// falls back to heuristic analysis of the top 15% of pages.
    ///
    /// @param threshold - Fraction of pages (0.0-1.0) where text must repeat (heuristic mode)
    #[wasm_bindgen(js_name = "removeHeaders")]
    pub fn remove_headers(&mut self, threshold: f32) -> Result<usize, JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .remove_headers(threshold)
            .map_err(|e| JsValue::from_str(&format!("Header removal failed: {}", e)))
    }

    /// Identify and remove footers.
    ///
    /// Uses spec-compliant /Artifact tags when available (100% accuracy), or
    /// falls back to heuristic analysis of the bottom 15% of pages.
    ///
    /// @param threshold - Fraction of pages (0.0-1.0) where text must repeat (heuristic mode)
    #[wasm_bindgen(js_name = "removeFooters")]
    pub fn remove_footers(&mut self, threshold: f32) -> Result<usize, JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .remove_footers(threshold)
            .map_err(|e| JsValue::from_str(&format!("Footer removal failed: {}", e)))
    }

    /// Identify and remove both headers and footers.
    ///
    /// Prioritizes ISO 32000 spec-compliant /Artifact tags, with a heuristic
    /// fallback for untagged PDFs.
    ///
    /// @param threshold - Fraction of pages (0.0-1.0) where text must repeat (heuristic mode)
    #[wasm_bindgen(js_name = "removeArtifacts")]
    pub fn remove_artifacts(&mut self, threshold: f32) -> Result<usize, JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .remove_artifacts(threshold)
            .map_err(|e| JsValue::from_str(&format!("Artifact removal failed: {}", e)))
    }

    /// Erase existing header content.
    ///
    /// Identifies existing text in the header area (top 15%) and marks it for erasure.
    ///
    /// @param page_index - Zero-based page number
    #[wasm_bindgen(js_name = "eraseHeader")]
    pub fn erase_header(&mut self, page_index: usize) -> Result<(), JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .erase_header(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to erase header: {}", e)))
    }

    /// Deprecated: Use eraseHeader instead.
    #[wasm_bindgen(js_name = "editHeader")]
    pub fn edit_header(&mut self, page_index: usize) -> Result<(), JsValue> {
        self.erase_header(page_index)
    }

    /// Erase existing footer content.
    ///
    /// Identifies existing text in the footer area (bottom 15%) and marks it for erasure.
    ///
    /// @param page_index - Zero-based page number
    #[wasm_bindgen(js_name = "eraseFooter")]
    pub fn erase_footer(&mut self, page_index: usize) -> Result<(), JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .erase_footer(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to erase footer: {}", e)))
    }

    /// Deprecated: Use eraseFooter instead.
    #[wasm_bindgen(js_name = "editFooter")]
    pub fn edit_footer(&mut self, page_index: usize) -> Result<(), JsValue> {
        self.erase_footer(page_index)
    }

    /// Erase both header and footer content.
    ///
    /// @param page_index - Zero-based page number
    #[wasm_bindgen(js_name = "eraseArtifacts")]
    pub fn erase_artifacts(&mut self, page_index: usize) -> Result<(), JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .erase_artifacts(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to erase artifacts: {}", e)))
    }

    /// Focus extraction on a specific rectangular region of a page (v0.3.14).
    ///
    /// @param page_index - Zero-based page number
    /// @param region - [x, y, width, height] in points
    #[wasm_bindgen(js_name = "within")]
    pub fn within(
        &self,
        page_index: usize,
        region: Vec<f32>,
    ) -> Result<WasmPdfPageRegion, JsValue> {
        if region.len() != 4 {
            return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
        }
        Ok(WasmPdfPageRegion {
            doc: self.clone(),
            page_index,
            region: crate::geometry::Rect::new(region[0], region[1], region[2], region[3]),
        })
    }

    /// Render a page to an image (PNG).
    ///
    /// Requires the `rendering` feature.
    ///
    /// @param page_index - Zero-based page number
    /// @param dpi - Dots per inch (default: 150)
    /// @returns Uint8Array containing the PNG image data
    #[cfg(feature = "rendering")]
    #[wasm_bindgen(js_name = "renderPage")]
    pub fn render_page(&mut self, page_index: usize, dpi: Option<u32>) -> Result<Vec<u8>, JsValue> {
        let opts = crate::rendering::RenderOptions::with_dpi(dpi.unwrap_or(150));
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let img = crate::rendering::render_page(&inner, page_index, &opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to render page: {}", e)))?;
        Ok(img.as_bytes().to_vec())
    }

    // ========================================================================
    // Group 3: Format Conversion
    // ========================================================================

    /// Convert a single page to Markdown.
    ///
    /// @param page_index - Zero-based page number
    /// @param detect_headings - Whether to detect headings (default: true)
    /// @param include_images - Whether to include images (default: false)
    #[wasm_bindgen(js_name = "toMarkdown")]
    pub fn to_markdown(
        &mut self,
        page_index: usize,
        detect_headings: Option<bool>,
        include_images: Option<bool>,
        include_form_fields: Option<bool>,
    ) -> Result<String, JsValue> {
        let mut opts = ConversionOptions::default();
        if let Some(dh) = detect_headings {
            opts.detect_headings = dh;
        }
        if let Some(ii) = include_images {
            opts.include_images = ii;
        }
        if let Some(iff) = include_form_fields {
            opts.include_form_fields = iff;
        }
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .to_markdown(page_index, &opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to markdown: {}", e)))
    }

    /// Convert all pages to Markdown.
    #[wasm_bindgen(js_name = "toMarkdownAll")]
    pub fn to_markdown_all(
        &mut self,
        detect_headings: Option<bool>,
        include_images: Option<bool>,
        include_form_fields: Option<bool>,
    ) -> Result<String, JsValue> {
        let mut opts = ConversionOptions::default();
        if let Some(dh) = detect_headings {
            opts.detect_headings = dh;
        }
        if let Some(ii) = include_images {
            opts.include_images = ii;
        }
        if let Some(iff) = include_form_fields {
            opts.include_form_fields = iff;
        }
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .to_markdown_all(&opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to markdown: {}", e)))
    }

    /// Convert a single page to HTML.
    ///
    /// @param page_index - Zero-based page number
    /// @param preserve_layout - Use CSS positioning to preserve layout (default: false)
    /// @param detect_headings - Whether to detect headings (default: true)
    #[wasm_bindgen(js_name = "toHtml")]
    pub fn to_html(
        &mut self,
        page_index: usize,
        preserve_layout: Option<bool>,
        detect_headings: Option<bool>,
        include_form_fields: Option<bool>,
    ) -> Result<String, JsValue> {
        let mut opts = ConversionOptions::default();
        if let Some(pl) = preserve_layout {
            opts.preserve_layout = pl;
        }
        if let Some(dh) = detect_headings {
            opts.detect_headings = dh;
        }
        if let Some(iff) = include_form_fields {
            opts.include_form_fields = iff;
        }
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .to_html(page_index, &opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to HTML: {}", e)))
    }

    /// Convert all pages to HTML.
    #[wasm_bindgen(js_name = "toHtmlAll")]
    pub fn to_html_all(
        &mut self,
        preserve_layout: Option<bool>,
        detect_headings: Option<bool>,
        include_form_fields: Option<bool>,
    ) -> Result<String, JsValue> {
        let mut opts = ConversionOptions::default();
        if let Some(pl) = preserve_layout {
            opts.preserve_layout = pl;
        }
        if let Some(dh) = detect_headings {
            opts.detect_headings = dh;
        }
        if let Some(iff) = include_form_fields {
            opts.include_form_fields = iff;
        }
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .to_html_all(&opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to HTML: {}", e)))
    }

    /// Convert a single page to plain text (with layout preservation options).
    #[wasm_bindgen(js_name = "toPlainText")]
    pub fn to_plain_text(&mut self, page_index: usize) -> Result<String, JsValue> {
        let opts = ConversionOptions::default();
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .to_plain_text(page_index, &opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to plain text: {}", e)))
    }

    /// Convert all pages to plain text.
    #[wasm_bindgen(js_name = "toPlainTextAll")]
    pub fn to_plain_text_all(&mut self) -> Result<String, JsValue> {
        let opts = ConversionOptions::default();
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .to_plain_text_all(&opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to plain text: {}", e)))
    }

    /// Convert the entire PDF to DOCX bytes (Uint8Array).
    #[wasm_bindgen(js_name = "toDocxBytes")]
    pub fn to_docx_bytes(&mut self) -> Result<Vec<u8>, JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .to_docx_bytes()
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to DOCX: {}", e)))
    }

    /// Convert the entire PDF to PPTX bytes (Uint8Array).
    #[wasm_bindgen(js_name = "toPptxBytes")]
    pub fn to_pptx_bytes(&mut self) -> Result<Vec<u8>, JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .to_pptx_bytes()
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to PPTX: {}", e)))
    }

    /// Convert the entire PDF to XLSX bytes (Uint8Array).
    #[wasm_bindgen(js_name = "toXlsxBytes")]
    pub fn to_xlsx_bytes(&mut self) -> Result<Vec<u8>, JsValue> {
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .to_xlsx_bytes()
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to XLSX: {}", e)))
    }

    /// Open a PDF from DOCX bytes.
    #[wasm_bindgen(js_name = "openFromDocxBytes")]
    pub fn open_from_docx_bytes(data: &[u8]) -> Result<WasmPdfDocument, JsValue> {
        let pdf_bytes = crate::converters::office::OfficeConverter::new()
            .convert_docx_bytes(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert DOCX: {}", e)))?;
        let inner = PdfDocument::from_bytes(pdf_bytes.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to open converted PDF: {}", e)))?;
        Ok(WasmPdfDocument {
            inner: std::sync::Arc::new(std::sync::Mutex::new(inner)),
            raw_bytes: std::sync::Arc::new(pdf_bytes),
            editor: None,
        })
    }

    /// Open a PDF from PPTX bytes.
    #[wasm_bindgen(js_name = "openFromPptxBytes")]
    pub fn open_from_pptx_bytes(data: &[u8]) -> Result<WasmPdfDocument, JsValue> {
        let pdf_bytes = crate::converters::office::OfficeConverter::new()
            .convert_pptx_bytes(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert PPTX: {}", e)))?;
        let inner = PdfDocument::from_bytes(pdf_bytes.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to open converted PDF: {}", e)))?;
        Ok(WasmPdfDocument {
            inner: std::sync::Arc::new(std::sync::Mutex::new(inner)),
            raw_bytes: std::sync::Arc::new(pdf_bytes),
            editor: None,
        })
    }

    /// Open a PDF from XLSX bytes.
    #[wasm_bindgen(js_name = "openFromXlsxBytes")]
    pub fn open_from_xlsx_bytes(data: &[u8]) -> Result<WasmPdfDocument, JsValue> {
        let pdf_bytes = crate::converters::office::OfficeConverter::new()
            .convert_xlsx_bytes(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert XLSX: {}", e)))?;
        let inner = PdfDocument::from_bytes(pdf_bytes.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to open converted PDF: {}", e)))?;
        Ok(WasmPdfDocument {
            inner: std::sync::Arc::new(std::sync::Mutex::new(inner)),
            raw_bytes: std::sync::Arc::new(pdf_bytes),
            editor: None,
        })
    }

    // ========================================================================
    // Group 4: Structured Extraction (returns JS objects via serde-wasm-bindgen)
    // ========================================================================

    /// Extract character-level data from a page.
    ///
    /// Returns an array of objects with: char, bbox {x, y, width, height},
    /// font_name, font_size, font_weight, is_italic, color {r, g, b}, etc.
    ///
    /// @param page_index - Zero-based page number
    /// @param region - Optional [x, y, width, height] to filter by
    #[wasm_bindgen(js_name = "extractChars")]
    pub fn extract_chars(
        &mut self,
        page_index: usize,
        region: Option<Vec<f32>>,
    ) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;

        let chars_result = if let Some(r) = region {
            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            inner.extract_chars_in_rect(
                page_index,
                crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
                crate::layout::RectFilterMode::Intersects,
            )
        } else {
            inner.extract_chars(page_index)
        };

        let chars = chars_result
            .map_err(|e| JsValue::from_str(&format!("Failed to extract chars: {}", e)))?;

        serde_wasm_bindgen::to_value(&chars)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract span-level data from a page.
    ///
    /// Returns an array of objects with: text, bbox, **page_bbox**, font_name,
    /// font_size, font_weight, is_italic, color, sequence, etc.
    ///
    /// `page_bbox` is the run's box in the displayed page frame. It differs
    /// from `bbox` on a rotated page or a mirrored run, and it cannot be
    /// derived from the other serialized fields because `mirrored` and
    /// `page_rotation_applied` are runtime-only.
    ///
    /// Optional `reading_order`: `"column_aware"` for XY-Cut column detection,
    /// `"structure"` to follow the tagged structure tree (table cells), or
    /// `"top_to_bottom"` (default).
    #[wasm_bindgen(js_name = "extractSpans")]
    pub fn extract_spans(
        &mut self,
        page_index: usize,
        region: Option<Vec<f32>>,
        reading_order: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let order = match reading_order.as_deref() {
            Some("column_aware") => crate::document::ReadingOrder::ColumnAware,
            Some("structure") => crate::document::ReadingOrder::Structure,
            Some("top_to_bottom") | None => crate::document::ReadingOrder::TopToBottom,
            Some(other) => {
                return Err(JsValue::from_str(&format!(
                    "Unknown reading_order '{}'. Expected 'top_to_bottom', 'column_aware', or 'structure'.",
                    other
                )));
            },
        };

        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let spans_result = if let Some(r) = region {
            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            inner.extract_spans_in_rect(
                page_index,
                crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
                crate::layout::RectFilterMode::Intersects,
            )
        } else {
            inner.extract_spans_with_reading_order(page_index, order)
        };

        let spans = spans_result
            .map_err(|e| JsValue::from_str(&format!("Failed to extract spans: {}", e)))?;

        // Carry `page_bbox` alongside `bbox`. It is a derived accessor rather
        // than a field, and the two inputs it needs — `mirrored` and
        // `page_rotation_applied` — are `#[serde(skip)]` runtime metadata, so a
        // JavaScript consumer could neither read it nor reconstruct it. On a
        // rotated page `bbox` is already in the displayed frame while
        // `rotation_degrees` keeps the raw content angle, so anything trying to
        // derive the run frame from the serialized fields alone double-counts
        // the rotation.
        let enriched: Vec<serde_json::Value> = spans
            .iter()
            .map(|s| {
                let mut v = serde_json::to_value(s).unwrap_or(serde_json::Value::Null);
                if let Some(obj) = v.as_object_mut() {
                    let pb = s.page_bbox();
                    obj.insert(
                        "page_bbox".to_string(),
                        serde_json::json!({
                            "x": pb.x, "y": pb.y, "width": pb.width, "height": pb.height
                        }),
                    );
                }
                v
            })
            .collect();
        serde_wasm_bindgen::to_value(&enriched)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract complete page text data in a single call.
    ///
    /// Returns `{ spans, chars, page_width, page_height }`.
    /// The `chars` are derived from spans using font-metric widths when available.
    ///
    /// Optional `reading_order`: `"column_aware"` for XY-Cut column detection,
    /// `"structure"` to follow the tagged structure tree (table cells), or
    /// `"top_to_bottom"` (default).
    #[wasm_bindgen(js_name = "extractPageText")]
    pub fn extract_page_text(
        &mut self,
        page_index: usize,
        reading_order: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let order = match reading_order.as_deref() {
            Some("column_aware") => crate::document::ReadingOrder::ColumnAware,
            Some("structure") => crate::document::ReadingOrder::Structure,
            Some("top_to_bottom") | None => crate::document::ReadingOrder::TopToBottom,
            Some(other) => {
                return Err(JsValue::from_str(&format!(
                    "Unknown reading_order '{}'. Expected 'top_to_bottom', 'column_aware', or 'structure'.",
                    other
                )));
            },
        };

        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;

        let page_text = inner
            .extract_page_text_with_options(page_index, order)
            .map_err(|e| JsValue::from_str(&format!("Failed to extract page text: {}", e)))?;

        serde_wasm_bindgen::to_value(&page_text)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract word-level data from a page.
    ///
    /// Returns an array of objects with: text, bbox, font_name, font_size,
    /// font_weight, is_italic, is_bold, sequence.
    ///
    /// `sequence` is the content-stream emission order of the originating
    /// span, letting callers tell words drawn consecutively in the content
    /// stream apart from ones that are merely spatially close.
    #[wasm_bindgen(js_name = "extractWords")]
    pub fn extract_words(
        &mut self,
        page_index: usize,
        region: Option<Vec<f32>>,
    ) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let words_result = if let Some(r) = region {
            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            inner.extract_words_in_rect(
                page_index,
                crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
                crate::layout::RectFilterMode::Intersects,
            )
        } else {
            inner.extract_words(page_index)
        };

        let words = words_result
            .map_err(|e| JsValue::from_str(&format!("Failed to extract words: {}", e)))?;
        serde_wasm_bindgen::to_value(&words)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract text lines from a page.
    ///
    /// Returns an array of objects with: text, bbox, words (array of Word objects).
    #[wasm_bindgen(js_name = "extractTextLines")]
    pub fn extract_text_lines(
        &mut self,
        page_index: usize,
        region: Option<Vec<f32>>,
    ) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let lines_result = if let Some(r) = region {
            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            inner.extract_text_lines_in_rect(
                page_index,
                crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
                crate::layout::RectFilterMode::Intersects,
            )
        } else {
            inner.extract_text_lines(page_index)
        };

        let lines = lines_result
            .map_err(|e| JsValue::from_str(&format!("Failed to extract lines: {}", e)))?;
        serde_wasm_bindgen::to_value(&lines)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract tables from a page (v0.3.14).
    ///
    /// @param page_index - Zero-based page number
    /// @param region - Optional [x, y, width, height] to filter by
    #[wasm_bindgen(js_name = "extractTables")]
    pub fn extract_tables(
        &mut self,
        page_index: usize,
        region: Option<Vec<f32>>,
    ) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let tables_result = if let Some(r) = region {
            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            inner.extract_tables_in_rect(
                page_index,
                crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
            )
        } else {
            inner.extract_tables(page_index)
        };

        let tables = tables_result
            .map_err(|e| JsValue::from_str(&format!("Failed to extract tables: {}", e)))?;

        // Convert tables to a simplified JSON-friendly format
        let json_tables: Vec<serde_json::Value> = tables
            .iter()
            .map(|t| {
                serde_json::json!({
                    "col_count": t.col_count,
                    "row_count": t.rows.len(),
                    "bbox": t.bbox.map(|b| serde_json::json!({"x": b.x, "y": b.y, "width": b.width, "height": b.height})),
                    "has_header": t.has_header,
                    "rows": t.rows.iter().map(|r| {
                        serde_json::json!({
                            "is_header": r.is_header,
                            "cells": r.cells.iter().map(|c| {
                                serde_json::json!({
                                    "text": c.text,
                                    "bbox": c.bbox.map(|b| serde_json::json!({"x": b.x, "y": b.y, "width": b.width, "height": b.height}))
                                })
                            }).collect::<Vec<_>>()
                        })
                    }).collect::<Vec<_>>()
                })
            })
            .collect();

        serde_wasm_bindgen::to_value(&json_tables)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    // ========================================================================
    // Group 5: Search
    // ========================================================================

    /// Search for text across all pages.
    ///
    /// @param pattern - Regex pattern or literal text to search for
    /// @param case_insensitive - Case insensitive search (default: false)
    /// @param literal - Treat pattern as literal text, not regex (default: false)
    /// @param whole_word - Match whole words only (default: false)
    /// @param max_results - Maximum results to return, 0 = unlimited (default: 0)
    ///
    /// Returns an array of {page, text, bbox, start_index, end_index, span_boxes}.
    #[wasm_bindgen(js_name = "search")]
    pub fn search(
        &mut self,
        pattern: &str,
        case_insensitive: Option<bool>,
        literal: Option<bool>,
        whole_word: Option<bool>,
        max_results: Option<usize>,
    ) -> Result<JsValue, JsValue> {
        let options = SearchOptions {
            case_insensitive: case_insensitive.unwrap_or(false),
            literal: literal.unwrap_or(false),
            whole_word: whole_word.unwrap_or(false),
            max_results: max_results.unwrap_or(0),
            page_range: None,
        };
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let results = TextSearcher::search(&inner, pattern, &options)
            .map_err(|e| JsValue::from_str(&format!("Search failed: {}", e)))?;
        serde_wasm_bindgen::to_value(&results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Search for text on a specific page.
    #[wasm_bindgen(js_name = "searchPage")]
    pub fn search_page(
        &mut self,
        page_index: usize,
        pattern: &str,
        case_insensitive: Option<bool>,
        literal: Option<bool>,
        whole_word: Option<bool>,
        max_results: Option<usize>,
    ) -> Result<JsValue, JsValue> {
        let options = SearchOptions {
            case_insensitive: case_insensitive.unwrap_or(false),
            literal: literal.unwrap_or(false),
            whole_word: whole_word.unwrap_or(false),
            max_results: max_results.unwrap_or(0),
            page_range: Some((page_index, page_index)),
        };
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let results = TextSearcher::search(&inner, pattern, &options)
            .map_err(|e| JsValue::from_str(&format!("Search failed: {}", e)))?;
        serde_wasm_bindgen::to_value(&results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Build the search index for every page up front, instead of the lazy
    /// per-page build `search()`/`searchPage()` otherwise do on first use.
    #[wasm_bindgen(js_name = "prepareSearch")]
    pub fn prepare_search(&mut self) -> Result<(), JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        inner
            .prepare_search()
            .map_err(|e| JsValue::from_str(&format!("prepare_search failed: {}", e)))
    }

    /// Drop the cached search index, if any, freeing its memory.
    /// `search()`/`searchPage()` rebuild it lazily on next use.
    #[wasm_bindgen(js_name = "clearSearchIndex")]
    pub fn clear_search_index(&mut self) -> Result<(), JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        inner.clear_search_index();
        Ok(())
    }

    // ========================================================================
    // Group 6: Image Info (read-only metadata)
    // ========================================================================

    /// Extract image metadata from a page.
    ///
    /// Returns an array of objects with: width, height, color_space,
    /// bits_per_component, bbox (if available). Does NOT return raw image bytes.
    ///
    /// @param page_index - Zero-based page number
    /// @param region - Optional [x, y, width, height] to filter by
    #[wasm_bindgen(js_name = "extractImages")]
    pub fn extract_images(
        &mut self,
        page_index: usize,
        region: Option<Vec<f32>>,
    ) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let images_result = if let Some(r) = region {
            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            inner.extract_images_in_rect(
                page_index,
                crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
            )
        } else {
            inner.extract_images(page_index)
        };

        let images = images_result
            .map_err(|e| JsValue::from_str(&format!("Failed to extract images: {}", e)))?;

        // Serialize image metadata (not raw bytes)
        let metadata: Vec<serde_json::Value> = images
            .iter()
            .map(|img| {
                let mut obj = serde_json::Map::new();
                obj.insert("width".into(), serde_json::Value::from(img.width()));
                obj.insert("height".into(), serde_json::Value::from(img.height()));
                obj.insert(
                    "color_space".into(),
                    serde_json::Value::from(format!("{:?}", img.color_space())),
                );
                obj.insert(
                    "bits_per_component".into(),
                    serde_json::Value::from(img.bits_per_component()),
                );
                if let Some(bbox) = img.bbox() {
                    let bbox_obj = serde_json::json!({
                        "x": bbox.x,
                        "y": bbox.y,
                        "width": bbox.width,
                        "height": bbox.height
                    });
                    obj.insert("bbox".into(), bbox_obj);
                }
                obj.insert("rotation".into(), serde_json::Value::from(img.rotation_degrees()));
                obj.insert("matrix".into(), serde_json::json!(img.matrix()));
                serde_json::Value::Object(obj)
            })
            .collect();

        serde_wasm_bindgen::to_value(&metadata)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    // ========================================================================
    // Group 6b: Document Structure (Outline, Annotations, Paths)
    // ========================================================================

    /// Get the document outline (bookmarks / table of contents).
    ///
    /// @returns Array of outline items or null if no outline exists.
    /// Each item has: { title, page (number|null), dest_name (string, optional), children (array) }
    #[wasm_bindgen(js_name = "getOutline")]
    pub fn get_outline(&mut self) -> Result<JsValue, JsValue> {
        let outline = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .get_outline()
            .map_err(|e| JsValue::from_str(&format!("Failed to get outline: {}", e)))?;

        match outline {
            None => Ok(JsValue::NULL),
            Some(items) => {
                let json = outline_to_json(&items);
                serde_wasm_bindgen::to_value(&json)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            },
        }
    }

    /// Get annotations from a page.
    ///
    /// @param page_index - Zero-based page number
    /// @returns Array of annotation objects with fields like subtype, rect, contents, etc.
    #[wasm_bindgen(js_name = "getAnnotations")]
    pub fn get_annotations(&mut self, page_index: usize) -> Result<JsValue, JsValue> {
        let annotations = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .get_annotations(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to get annotations: {}", e)))?;

        let result: Vec<serde_json::Value> = annotations
            .iter()
            .map(|ann| {
                let mut obj = serde_json::Map::new();

                if let Some(ref subtype) = ann.subtype {
                    obj.insert("subtype".into(), serde_json::Value::from(subtype.as_str()));
                }
                if let Some(ref contents) = ann.contents {
                    obj.insert("contents".into(), serde_json::Value::from(contents.as_str()));
                }
                if let Some(rect) = ann.rect {
                    obj.insert(
                        "rect".into(),
                        serde_json::json!([rect[0], rect[1], rect[2], rect[3]]),
                    );
                }
                if let Some(ref author) = ann.author {
                    obj.insert("author".into(), serde_json::Value::from(author.as_str()));
                }
                if let Some(ref date) = ann.creation_date {
                    obj.insert("creation_date".into(), serde_json::Value::from(date.as_str()));
                }
                if let Some(ref date) = ann.modification_date {
                    obj.insert("modification_date".into(), serde_json::Value::from(date.as_str()));
                }
                if let Some(ref subject) = ann.subject {
                    obj.insert("subject".into(), serde_json::Value::from(subject.as_str()));
                }
                if let Some(ref color) = ann.color {
                    if color.len() >= 3 {
                        obj.insert(
                            "color".into(),
                            serde_json::json!([color[0], color[1], color[2]]),
                        );
                    }
                }
                if let Some(opacity) = ann.opacity {
                    obj.insert("opacity".into(), serde_json::Value::from(opacity));
                }
                if let Some(ref ft) = ann.field_type {
                    obj.insert("field_type".into(), serde_json::Value::from(format!("{:?}", ft)));
                }
                if let Some(ref name) = ann.field_name {
                    obj.insert("field_name".into(), serde_json::Value::from(name.as_str()));
                }
                if let Some(ref val) = ann.field_value {
                    obj.insert("field_value".into(), serde_json::Value::from(val.as_str()));
                }
                if let Some(crate::annotations::LinkAction::Uri(ref uri)) = ann.action {
                    obj.insert("action_uri".into(), serde_json::Value::from(uri.as_str()));
                }

                serde_json::Value::Object(obj)
            })
            .collect();

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract vector paths (lines, curves, shapes) from a page.
    ///
    /// @param page_index - Zero-based page number
    /// @param region - Optional [x, y, width, height] to filter by
    /// @returns Array of path objects with bbox, stroke_color, fill_color, etc.
    #[wasm_bindgen(js_name = "extractPaths")]
    pub fn extract_paths(
        &mut self,
        page_index: usize,
        region: Option<Vec<f32>>,
    ) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;

        let paths_result = if let Some(r) = region {
            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            inner.extract_paths_in_rect(
                page_index,
                crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
            )
        } else {
            inner.extract_paths(page_index)
        };

        let paths = paths_result
            .map_err(|e| JsValue::from_str(&format!("Failed to extract paths: {}", e)))?;

        let result: Vec<serde_json::Value> = paths
            .iter()
            .map(|path| {
                let mut obj = serde_json::Map::new();

                obj.insert(
                    "bbox".into(),
                    serde_json::json!({
                        "x": path.bbox.x,
                        "y": path.bbox.y,
                        "width": path.bbox.width,
                        "height": path.bbox.height
                    }),
                );
                // Stroke-inflated extents — what the reader sees.
                let rendered = path.rendered_bbox();
                obj.insert(
                    "rendered_bbox".into(),
                    serde_json::json!({
                        "x": rendered.x,
                        "y": rendered.y,
                        "width": rendered.width,
                        "height": rendered.height
                    }),
                );
                obj.insert("stroke_width".into(), serde_json::Value::from(path.stroke_width));

                if let Some(ref color) = path.stroke_color {
                    obj.insert(
                        "stroke_color".into(),
                        serde_json::json!({"r": color.r, "g": color.g, "b": color.b}),
                    );
                }
                if let Some(ref color) = path.fill_color {
                    obj.insert(
                        "fill_color".into(),
                        serde_json::json!({"r": color.r, "g": color.g, "b": color.b}),
                    );
                }

                let cap_str = match path.line_cap {
                    crate::elements::LineCap::Butt => "butt",
                    crate::elements::LineCap::Round => "round",
                    crate::elements::LineCap::Square => "square",
                };
                obj.insert("line_cap".into(), serde_json::Value::from(cap_str));

                let join_str = match path.line_join {
                    crate::elements::LineJoin::Miter => "miter",
                    crate::elements::LineJoin::Round => "round",
                    crate::elements::LineJoin::Bevel => "bevel",
                };
                obj.insert("line_join".into(), serde_json::Value::from(join_str));

                obj.insert(
                    "operations_count".into(),
                    serde_json::Value::from(path.operations.len()),
                );

                let ops: Vec<serde_json::Value> = path
                    .operations
                    .iter()
                    .map(|op| match op {
                        crate::elements::PathOperation::MoveTo(x, y) => {
                            serde_json::json!({"op": "move_to", "x": x, "y": y})
                        }
                        crate::elements::PathOperation::LineTo(x, y) => {
                            serde_json::json!({"op": "line_to", "x": x, "y": y})
                        }
                        crate::elements::PathOperation::CurveTo(cx1, cy1, cx2, cy2, x, y) => {
                            serde_json::json!({"op": "curve_to", "cx1": cx1, "cy1": cy1, "cx2": cx2, "cy2": cy2, "x": x, "y": y})
                        }
                        crate::elements::PathOperation::Rectangle(x, y, w, h) => {
                            serde_json::json!({"op": "rectangle", "x": x, "y": y, "width": w, "height": h})
                        }
                        crate::elements::PathOperation::ClosePath => {
                            serde_json::json!({"op": "close_path"})
                        }
                    })
                    .collect();
                obj.insert("operations".into(), serde_json::Value::Array(ops));

                serde_json::Value::Object(obj)
            })
            .collect();

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract only rectangles from a page (v0.3.14).
    ///
    /// Identifies paths that form axis-aligned rectangles.
    ///
    /// @param page_index - Zero-based page number
    /// @param region - Optional [x, y, width, height] to filter by
    /// @returns Array of path objects
    #[wasm_bindgen(js_name = "extractRects")]
    pub fn extract_rects(
        &mut self,
        page_index: usize,
        region: Option<Vec<f32>>,
    ) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let rects_result = if let Some(r) = region {
            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            // Delegate to the rendered-extents region query so a
            // stroke-width-encoded rule matches a region over its drawn
            // bar, in parity with the Rust/Python entry points.
            inner.extract_rects_in_rect(
                page_index,
                crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
            )
        } else {
            inner.extract_rects(page_index)
        };

        let rects = rects_result
            .map_err(|e| JsValue::from_str(&format!("Failed to extract rects: {}", e)))?;
        serde_wasm_bindgen::to_value(&rects)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract only straight lines from a page (v0.3.14).
    ///
    /// Identifies paths that form a single straight line segment.
    ///
    /// @param page_index - Zero-based page number
    /// @param region - Optional [x, y, width, height] to filter by
    /// @returns Array of path objects
    #[wasm_bindgen(js_name = "extractLines")]
    pub fn extract_lines(
        &mut self,
        page_index: usize,
        region: Option<Vec<f32>>,
    ) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let lines_result = if let Some(r) = region {
            if r.len() != 4 {
                return Err(JsValue::from_str("Region must have exactly 4 elements [x, y, w, h]"));
            }
            // Rendered-extents region query, in parity with the
            // Rust/Python entry points.
            inner.extract_lines_in_rect(
                page_index,
                crate::geometry::Rect::new(r[0], r[1], r[2], r[3]),
            )
        } else {
            inner.extract_lines(page_index)
        };

        let lines = lines_result
            .map_err(|e| JsValue::from_str(&format!("Failed to extract lines: {}", e)))?;
        serde_wasm_bindgen::to_value(&lines)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

/// X.509 certificate parsed from a raw DER blob. Mirrors the C#,
/// Node, Python, and Go `Certificate` surfaces — `subject` / `issuer`
/// / `serial` / `validity` / `isValid` getters only.
#[cfg(feature = "signatures")]
#[wasm_bindgen]
pub struct WasmCertificate {
    creds: crate::signatures::SigningCredentials,
}

#[cfg(feature = "signatures")]
#[wasm_bindgen]
impl WasmCertificate {
    /// Load a certificate from a DER-encoded X.509 blob. Throws if
    /// the DER doesn't parse.
    #[wasm_bindgen(js_name = "load")]
    pub fn load(data: &[u8]) -> Result<WasmCertificate, JsValue> {
        if data.is_empty() {
            return Err(JsValue::from_str("Certificate data must not be empty"));
        }
        let creds = crate::signatures::SigningCredentials::from_der(data.to_vec())
            .map_err(|e| JsValue::from_str(&format!("Invalid certificate: {e}")))?;
        Ok(Self { creds })
    }

    /// Load a signer certificate + private key from PEM strings.
    /// `certPem` must begin `-----BEGIN CERTIFICATE-----`.
    /// `keyPem` must begin `-----BEGIN PRIVATE KEY-----` or `-----BEGIN RSA PRIVATE KEY-----`.
    #[wasm_bindgen(js_name = "loadPem")]
    pub fn load_pem(cert_pem: &str, key_pem: &str) -> Result<WasmCertificate, JsValue> {
        let creds = crate::signatures::SigningCredentials::from_pem(cert_pem, key_pem)
            .map_err(|e| JsValue::from_str(&format!("Failed to load PEM credentials: {e}")))?;
        Ok(Self { creds })
    }

    /// Load a signer certificate + private key from a PKCS#12 (.p12/.pfx) blob.
    /// `password` is the passphrase protecting the key bag.
    #[wasm_bindgen(js_name = "loadPkcs12")]
    pub fn load_pkcs12(data: &[u8], password: &str) -> Result<WasmCertificate, JsValue> {
        if data.is_empty() {
            return Err(JsValue::from_str("PKCS#12 data must not be empty"));
        }
        let creds = crate::signatures::SigningCredentials::from_pkcs12(data, password)
            .map_err(|e| JsValue::from_str(&format!("Failed to load PKCS#12: {e}")))?;
        Ok(Self { creds })
    }

    /// Subject distinguished name.
    #[wasm_bindgen(getter)]
    pub fn subject(&self) -> Result<String, JsValue> {
        self.creds
            .subject()
            .map_err(|e| JsValue::from_str(&format!("{e}")))
    }

    /// Issuer distinguished name.
    #[wasm_bindgen(getter)]
    pub fn issuer(&self) -> Result<String, JsValue> {
        self.creds
            .issuer()
            .map_err(|e| JsValue::from_str(&format!("{e}")))
    }

    /// Serial number as a hex string (no `0x` prefix).
    #[wasm_bindgen(getter)]
    pub fn serial(&self) -> Result<String, JsValue> {
        self.creds
            .serial()
            .map_err(|e| JsValue::from_str(&format!("{e}")))
    }

    /// Validity window as `[notBefore, notAfter]` Unix epoch seconds.
    /// JavaScript: `new Date(notBefore * 1000)` for a Date.
    #[wasm_bindgen(getter)]
    pub fn validity(&self) -> Result<Vec<i64>, JsValue> {
        let (nb, na) = self
            .creds
            .validity()
            .map_err(|e| JsValue::from_str(&format!("{e}")))?;
        Ok(vec![nb, na])
    }

    /// Whether the certificate is currently within its validity
    /// window. Does NOT verify chain, trust-root, or revocation.
    #[wasm_bindgen(getter, js_name = "isValid")]
    pub fn is_valid(&self) -> Result<bool, JsValue> {
        self.creds
            .is_valid()
            .map_err(|e| JsValue::from_str(&format!("{e}")))
    }
}

/// Sign raw PDF bytes and return the signed PDF as a `Uint8Array`.
///
/// `cert` must carry a private key (loaded via `Certificate.loadPem` or
/// `Certificate.loadPkcs12`).
#[cfg(feature = "signatures")]
#[wasm_bindgen(js_name = "signPdfBytes")]
pub fn wasm_sign_pdf_bytes(
    pdf_data: &[u8],
    cert: &WasmCertificate,
    reason: Option<String>,
    location: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    use crate::signatures::{sign_pdf_bytes, SignOptions};
    let opts = SignOptions {
        reason,
        location,
        ..Default::default()
    };
    sign_pdf_bytes(pdf_data, &cert.creds, opts)
        .map_err(|e| JsValue::from_str(&format!("signPdfBytes failed: {e}")))
}

// ─── PAdES LTV (#235) ───────────────────────────────────────────────────────

/// PAdES baseline level. Frozen integer mapping (BB=0, BT=1, BLt=2,
/// BLta=3) shared with the C ABI and every binding — never renumber.
#[cfg(feature = "signatures")]
#[wasm_bindgen(js_name = "PadesLevel")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WasmPadesLevel {
    /// B-B: signed attrs incl. the ESS signing-certificate-v2.
    BB = 0,
    /// B-T: B-B + an RFC 3161 signature-time-stamp unsigned attr.
    BT = 1,
    /// B-LT: B-T + a Document Security Store (DSS/VRI).
    BLt = 2,
    /// B-LTA: B-LT + a document-scoped `/DocTimeStamp`.
    BLta = 3,
}

#[cfg(feature = "signatures")]
impl WasmPadesLevel {
    fn to_core(self) -> crate::signatures::PadesLevel {
        crate::signatures::PadesLevel::from_code(self as i32)
            .expect("frozen PadesLevel code round-trips")
    }
    fn from_core(level: crate::signatures::PadesLevel) -> WasmPadesLevel {
        match level.code() {
            0 => WasmPadesLevel::BB,
            1 => WasmPadesLevel::BT,
            2 => WasmPadesLevel::BLt,
            _ => WasmPadesLevel::BLta,
        }
    }
}

/// Offline B-LT validation material (DER certs / CRLs / OCSP
/// responses). Build with `new()` then `addCert`/`addCrl`/`addOcsp`.
#[cfg(feature = "signatures")]
#[wasm_bindgen(js_name = "RevocationMaterial")]
#[derive(Default)]
pub struct WasmRevocationMaterial {
    certs: Vec<Vec<u8>>,
    crls: Vec<Vec<u8>>,
    ocsps: Vec<Vec<u8>>,
}

#[cfg(feature = "signatures")]
#[wasm_bindgen(js_class = "RevocationMaterial")]
impl WasmRevocationMaterial {
    /// Create an empty revocation-material set.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmRevocationMaterial {
        WasmRevocationMaterial::default()
    }
    /// Add a DER X.509 certificate.
    #[wasm_bindgen(js_name = "addCert")]
    pub fn add_cert(&mut self, der: &[u8]) {
        self.certs.push(der.to_vec());
    }
    /// Add a DER CRL.
    #[wasm_bindgen(js_name = "addCrl")]
    pub fn add_crl(&mut self, der: &[u8]) {
        self.crls.push(der.to_vec());
    }
    /// Add a DER OCSP response.
    #[wasm_bindgen(js_name = "addOcsp")]
    pub fn add_ocsp(&mut self, der: &[u8]) {
        self.ocsps.push(der.to_vec());
    }
}

/// A parsed Document Security Store (`/DSS`, ISO 32000-2 §12.8.4.3).
/// Count + index accessors mirror `WasmCertificate`'s flat shape
/// (wasm-bindgen cannot return `Uint8Array[]` directly).
#[cfg(feature = "signatures")]
#[wasm_bindgen(js_name = "Dss")]
pub struct WasmDss {
    dss: crate::signatures::DocumentSecurityStore,
}

#[cfg(feature = "signatures")]
#[wasm_bindgen(js_class = "Dss")]
impl WasmDss {
    /// Number of DER X.509 certificates in the DSS.
    #[wasm_bindgen(getter, js_name = "certCount")]
    pub fn cert_count(&self) -> usize {
        self.dss.certificates.len()
    }
    /// The `i`-th DER certificate, or `undefined` if out of range.
    #[wasm_bindgen(js_name = "getCert")]
    pub fn get_cert(&self, i: usize) -> Option<Vec<u8>> {
        self.dss.certificates.get(i).cloned()
    }
    /// Number of DER CRLs in the DSS.
    #[wasm_bindgen(getter, js_name = "crlCount")]
    pub fn crl_count(&self) -> usize {
        self.dss.crls.len()
    }
    /// The `i`-th DER CRL, or `undefined` if out of range.
    #[wasm_bindgen(js_name = "getCrl")]
    pub fn get_crl(&self, i: usize) -> Option<Vec<u8>> {
        self.dss.crls.get(i).cloned()
    }
    /// Number of DER OCSP responses in the DSS.
    #[wasm_bindgen(getter, js_name = "ocspCount")]
    pub fn ocsp_count(&self) -> usize {
        self.dss.ocsp_responses.len()
    }
    /// The `i`-th DER OCSP response, or `undefined` if out of range.
    #[wasm_bindgen(js_name = "getOcsp")]
    pub fn get_ocsp(&self, i: usize) -> Option<Vec<u8>> {
        self.dss.ocsp_responses.get(i).cloned()
    }
    /// Per-signature VRI keys (uppercase-hex SHA-1 of `/Contents`).
    #[wasm_bindgen(getter)]
    pub fn vri(&self) -> Vec<String> {
        self.dss
            .vri
            .iter()
            .map(|v| v.signature_digest.clone())
            .collect()
    }
}

/// Whether `pdf_data` carries a document-scoped RFC 3161
/// `/DocTimeStamp` archival timestamp (PAdES-B-LTA). This is the
/// document-level reader signal; a `WasmSignature`'s `padesLevel`
/// getter is signature-scoped and tops out at B-LT by design.
#[cfg(feature = "signatures")]
#[wasm_bindgen(js_name = "hasDocumentTimestamp")]
pub fn wasm_has_document_timestamp(pdf_data: &[u8]) -> bool {
    crate::signatures::has_document_timestamp(pdf_data)
}

/// Sign raw PDF bytes at a PAdES baseline level and return the signed
/// PDF as a `Uint8Array`.
///
/// `level` `BLTA` is reserved (→ error). For `BT`/`BLt` pass a
/// pre-fetched RFC 3161 `timestampToken` (DER): WASM intentionally
/// omits the online TSA client (same `ureq`-incompat carve-out as
/// v0.3.38) — without a token the core fail-closes with `Unsupported`.
/// `revocation` supplies the B-LT DSS material.
#[cfg(feature = "signatures")]
#[wasm_bindgen(js_name = "signPdfBytesPades")]
#[allow(clippy::too_many_arguments)]
pub fn wasm_sign_pdf_bytes_pades(
    pdf_data: &[u8],
    cert: &WasmCertificate,
    level: WasmPadesLevel,
    timestamp_token: Option<Vec<u8>>,
    revocation: Option<WasmRevocationMaterial>,
    reason: Option<String>,
    location: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    use crate::signatures::{sign_pdf_bytes_pades, RevocationMaterial, SignOptions};
    let opts = SignOptions {
        reason,
        location,
        ..Default::default()
    };
    let material = revocation
        .map(|r| RevocationMaterial {
            certificates: r.certs,
            crls: r.crls,
            ocsp_responses: r.ocsps,
            ..Default::default()
        })
        .unwrap_or_default();

    // The caller-supplied token is the timestamp source; the imprint is
    // computed by the core over the signature value, so the closure
    // simply returns the pre-fetched token verbatim.
    let ts_closure: Option<Box<dyn Fn(&[u8]) -> crate::error::Result<Vec<u8>>>> = timestamp_token
        .map(|tok| {
            Box::new(move |_sig: &[u8]| Ok(tok.clone()))
                as Box<dyn Fn(&[u8]) -> crate::error::Result<Vec<u8>>>
        });

    sign_pdf_bytes_pades(
        pdf_data,
        &cert.creds,
        opts,
        level.to_core(),
        ts_closure.as_deref(),
        &material,
    )
    .map_err(|e| JsValue::from_str(&format!("signPdfBytesPades failed: {e}")))
}

/// RFC 3161 timestamp parsed from a DER TimeStampToken or bare
/// TSTInfo. Mirrors the C#, Go, and Python `Timestamp` surfaces.
#[cfg(feature = "signatures")]
#[wasm_bindgen]
pub struct WasmTimestamp {
    inner: crate::signatures::Timestamp,
}

#[cfg(feature = "signatures")]
#[wasm_bindgen]
impl WasmTimestamp {
    /// Parse a DER blob that may be either a full TimeStampToken or
    /// the bare TSTInfo SEQUENCE.
    #[wasm_bindgen(js_name = "parse")]
    pub fn parse(data: &[u8]) -> Result<WasmTimestamp, JsValue> {
        if data.is_empty() {
            return Err(JsValue::from_str("Timestamp data must not be empty"));
        }
        let inner = crate::signatures::Timestamp::from_der(data)
            .map_err(|e| JsValue::from_str(&format!("Invalid timestamp: {e}")))?;
        Ok(Self { inner })
    }

    /// Generation time as Unix epoch seconds.
    #[wasm_bindgen(getter)]
    pub fn time(&self) -> i64 {
        self.inner.time()
    }

    /// Serial number as a hex string (no `0x` prefix).
    #[wasm_bindgen(getter)]
    pub fn serial(&self) -> String {
        self.inner.serial()
    }

    /// TSA policy OID in dotted-decimal form.
    #[wasm_bindgen(getter, js_name = "policyOid")]
    pub fn policy_oid(&self) -> String {
        self.inner.policy_oid()
    }

    /// TSA name from the token (may be empty).
    #[wasm_bindgen(getter, js_name = "tsaName")]
    pub fn tsa_name(&self) -> String {
        self.inner.tsa_name()
    }

    /// Hash algorithm enum value (1=SHA1, 2=SHA256, 3=SHA384,
    /// 4=SHA512, 0=unknown).
    #[wasm_bindgen(getter, js_name = "hashAlgorithm")]
    pub fn hash_algorithm(&self) -> i32 {
        self.inner.hash_algorithm() as i32
    }

    /// Raw message-imprint hash bytes.
    #[wasm_bindgen(getter, js_name = "messageImprint")]
    pub fn message_imprint(&self) -> Vec<u8> {
        self.inner.message_imprint()
    }

    /// Cryptographically verify this TimeStampToken.
    ///
    /// Returns `true` when the TSA's signature and `messageDigest` both pass.
    /// Returns `false` when a crypto check fails (tampered token or wrong key).
    /// Throws when the token is not CMS-wrapped or uses an unsupported algorithm.
    #[wasm_bindgen]
    pub fn verify(&self) -> Result<bool, JsValue> {
        self.inner
            .verify()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// A single existing PDF signature surfaced by
/// `WasmPdfDocument.signatures()`. `verify()` runs the signer-attributes
/// check; `verifyDetached()` adds the `messageDigest` content-hash check.
/// Supported: RSA-PKCS#1 v1.5, RSA-PSS, ECDSA P-256/P-384.
#[cfg(feature = "signatures")]
#[wasm_bindgen]
pub struct WasmSignature {
    info: crate::signatures::SignatureInfo,
}

#[cfg(feature = "signatures")]
#[wasm_bindgen]
impl WasmSignature {
    /// `/Name` entry from the signature dictionary, if present.
    #[wasm_bindgen(getter, js_name = "signerName")]
    pub fn signer_name(&self) -> Option<String> {
        self.info.signer_name.clone()
    }

    /// `/Reason` entry from the signature dictionary, if present.
    #[wasm_bindgen(getter)]
    pub fn reason(&self) -> Option<String> {
        self.info.reason.clone()
    }

    /// `/Location` entry from the signature dictionary, if present.
    #[wasm_bindgen(getter)]
    pub fn location(&self) -> Option<String> {
        self.info.location.clone()
    }

    /// `/ContactInfo` entry from the signature dictionary, if present.
    #[wasm_bindgen(getter, js_name = "contactInfo")]
    pub fn contact_info(&self) -> Option<String> {
        self.info.contact_info.clone()
    }

    /// Unix epoch (seconds). `None` if the `/M` entry is missing or
    /// unparseable.
    #[wasm_bindgen(getter, js_name = "signingTime")]
    pub fn signing_time(&self) -> Option<i64> {
        self.info
            .signing_time
            .as_deref()
            .and_then(crate::signatures::parse_pdf_date_to_epoch)
    }

    /// True iff `/ByteRange` is a 4-element array covering the whole
    /// document (i.e. the signature protects every byte of the file).
    #[wasm_bindgen(getter, js_name = "coversWholeDocument")]
    pub fn covers_whole_document(&self) -> bool {
        self.info.covers_whole_document
    }

    /// PAdES baseline level from this signature's CMS attributes alone
    /// (`BB` vs `BT`). `BLt` additionally needs the document `/DSS` —
    /// read it via `WasmPdfDocument.dss()` and re-classify there.
    #[wasm_bindgen(getter, js_name = "padesLevel")]
    pub fn pades_level(&self) -> WasmPadesLevel {
        WasmPadesLevel::from_core(crate::signatures::classify_pades_level(&self.info, None))
    }

    /// Run the RFC 5652 §5.4 signer-attributes crypto check. Today
    /// this covers RSA-PKCS#1 v1.5 over SHA-1/256/384/512 — the
    /// padding used by essentially every PDF signature.
    ///
    /// A `true` return proves the signer held the private key matching
    /// the embedded certificate and that the signed-attribute bundle
    /// is authentic. It does **not** verify the `messageDigest`
    /// attribute against the document's byte-range content hash —
    /// call `verifyDetached()` for that end-to-end check.
    ///
    /// Throws for RSA-PSS, ECDSA, unknown digest OIDs, or signatures
    /// without signed_attrs.
    #[wasm_bindgen(js_name = "verify")]
    pub fn verify(&self) -> Result<bool, JsValue> {
        let Some(contents) = self.info.contents() else {
            return Err(JsValue::from_str("Signature has no /Contents blob — nothing to verify"));
        };
        match crate::signatures::verify_signer(contents) {
            Ok(crate::signatures::SignerVerify::Valid) => Ok(true),
            Ok(crate::signatures::SignerVerify::Invalid) => Ok(false),
            Ok(crate::signatures::SignerVerify::Unknown) => Err(JsValue::from_str(
                "Signature.verify(): signer uses RSA-PSS, ECDSA, an unknown \
                 digest OID, or the CMS blob lacks signed_attrs",
            )),
            Err(e) => Err(JsValue::from_str(&format!(
                "Signature.verify(): failed to parse /Contents as CMS: {e}"
            ))),
        }
    }

    /// End-to-end detached-signature verification. Runs both the
    /// signer-attributes RSA-PKCS#1 v1.5 crypto check AND the RFC 5652
    /// §11.2 `messageDigest` check against the segment of `pdfData`
    /// this signature covers (extracted via `/ByteRange`).
    ///
    /// `pdfData` must be the full PDF file. A `true` result proves
    /// both the signer is authentic and that the document's byte-range
    /// content has not been altered since signing. `false` means one
    /// of the two checks failed (wrong key or tampered content).
    ///
    /// Throws for RSA-PSS, ECDSA, unknown digest OIDs, or CMS blobs
    /// missing `signed_attrs` / `messageDigest`.
    #[wasm_bindgen(js_name = "verifyDetached")]
    pub fn verify_detached(&self, pdf_data: &[u8]) -> Result<bool, JsValue> {
        let Some(contents) = self.info.contents() else {
            return Err(JsValue::from_str("Signature has no /Contents blob — nothing to verify"));
        };
        let br = self.info.byte_range();
        if br.len() != 4 {
            return Err(JsValue::from_str(
                "Signature has no /ByteRange — cannot extract signed bytes",
            ));
        }
        let byte_range: [i64; 4] = [br[0], br[1], br[2], br[3]];
        let signed_bytes =
            crate::signatures::ByteRangeCalculator::extract_signed_bytes(pdf_data, &byte_range)
                .map_err(|e| JsValue::from_str(&format!("Failed to extract signed bytes: {e}")))?;
        match crate::signatures::verify_signer_detached(contents, &signed_bytes) {
            Ok(crate::signatures::SignerVerify::Valid) => Ok(true),
            Ok(crate::signatures::SignerVerify::Invalid) => Ok(false),
            Ok(crate::signatures::SignerVerify::Unknown) => Err(JsValue::from_str(
                "Signature.verifyDetached(): signer uses RSA-PSS, ECDSA, an \
                 unknown digest, or the CMS blob lacks signed_attrs / messageDigest",
            )),
            Err(e) => Err(JsValue::from_str(&format!("Signature.verifyDetached(): {e}"))),
        }
    }
}

/// A focused view of a PDF page region for scoped extraction (v0.3.14).
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmPdfPageRegion {
    doc: WasmPdfDocument,
    page_index: usize,
    region: crate::geometry::Rect,
}

#[wasm_bindgen]
impl WasmPdfPageRegion {
    /// Extract text from this region.
    #[wasm_bindgen(js_name = "extractText")]
    pub fn extract_text(&mut self) -> Result<String, JsValue> {
        self.doc
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .extract_text_in_rect(
                self.page_index,
                self.region,
                crate::layout::RectFilterMode::Intersects,
            )
            .map_err(|e| JsValue::from_str(&format!("Failed to extract text: {}", e)))
    }

    /// Extract character-level data from this region.
    #[wasm_bindgen(js_name = "extractChars")]
    pub fn extract_chars(&mut self) -> Result<JsValue, JsValue> {
        self.doc.extract_chars(
            self.page_index,
            Some(vec![
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            ]),
        )
    }

    /// Extract words from this region.
    #[wasm_bindgen(js_name = "extractWords")]
    pub fn extract_words(&mut self) -> Result<JsValue, JsValue> {
        self.doc.extract_words(
            self.page_index,
            Some(vec![
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            ]),
        )
    }

    /// Extract text lines from this region.
    #[wasm_bindgen(js_name = "extractTextLines")]
    pub fn extract_text_lines(&mut self) -> Result<JsValue, JsValue> {
        self.doc.extract_text_lines(
            self.page_index,
            Some(vec![
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            ]),
        )
    }

    /// Extract tables from this region.
    #[wasm_bindgen(js_name = "extractTables")]
    pub fn extract_tables(&mut self) -> Result<JsValue, JsValue> {
        self.doc.extract_tables(
            self.page_index,
            Some(vec![
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            ]),
        )
    }

    /// Extract images from this region.
    #[wasm_bindgen(js_name = "extractImages")]
    pub fn extract_images(&mut self) -> Result<JsValue, JsValue> {
        self.doc.extract_images(
            self.page_index,
            Some(vec![
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            ]),
        )
    }

    /// Extract vector paths from this region.
    #[wasm_bindgen(js_name = "extractPaths")]
    pub fn extract_paths(&mut self) -> Result<JsValue, JsValue> {
        self.doc.extract_paths(
            self.page_index,
            Some(vec![
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            ]),
        )
    }

    /// Extract rectangles from this region.
    #[wasm_bindgen(js_name = "extractRects")]
    pub fn extract_rects(&mut self) -> Result<JsValue, JsValue> {
        self.doc.extract_rects(
            self.page_index,
            Some(vec![
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            ]),
        )
    }

    /// Extract straight lines from this region.
    #[wasm_bindgen(js_name = "extractLines")]
    pub fn extract_lines(&mut self) -> Result<JsValue, JsValue> {
        self.doc.extract_lines(
            self.page_index,
            Some(vec![
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height,
            ]),
        )
    }

    /// Extract text using OCR from this region.
    ///
    /// Region-scoped OCR is not wired yet; use the page-level
    /// `WasmPdfDocument.extractTextOcr(pageIndex, engine)` for now
    /// (#524 follow-up).
    #[wasm_bindgen(js_name = "extractTextOcr")]
    pub fn extract_text_ocr(&mut self, _engine: Option<WasmOcrEngine>) -> Result<String, JsValue> {
        Err(JsValue::from_str(
            "region-scoped OCR is not implemented; use \
             WasmPdfDocument.extractTextOcr(pageIndex, engine) for full-page OCR",
        ))
    }
}

/// OCR configuration for WebAssembly. (Currently a marker — the engine
/// uses tuned defaults; knobs are exposed as the WASM OCR surface
/// matures, #524.)
#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct WasmOcrConfig {}

#[wasm_bindgen]
impl WasmOcrConfig {
    /// Create a new OCR configuration.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

/// OCR engine for WebAssembly (#524).
///
/// OCR runs entirely in-WASM via the pure-Rust `tract` backend — no
/// native ONNX Runtime, no JS bridge. Model **delivery is host-side**:
/// the browser/Deno/edge host fetches the detector + recognizer ONNX
/// files and the char dictionary (see `modelManifest()` for the URLs)
/// — typically `fetch()` + the Cache API / IndexedDB for the
/// tens-of-MB models — then hands the bytes to the constructor. This
/// only works in the `wasm-ocr` build of `pdf-oxide`; the default
/// `pdf-oxide-wasm` has no OCR (the constructor returns an error
/// explaining this).
#[wasm_bindgen]
pub struct WasmOcrEngine {
    #[cfg(feature = "ocr-tract")]
    inner: std::rc::Rc<crate::ocr::OcrEngine>,
}

#[cfg(feature = "ocr-tract")]
#[wasm_bindgen]
impl WasmOcrEngine {
    /// Build an OCR engine from in-memory model bytes supplied by the
    /// host.
    ///
    /// @param detModel - DBNet detector ONNX bytes (`det.onnx`)
    /// @param recModel - SVTR recognizer ONNX bytes (e.g. `rec.onnx`)
    /// @param dict     - recognizer char dictionary, one char per line
    /// @param config   - reserved (tuned defaults are used)
    #[wasm_bindgen(constructor)]
    pub fn new(
        det_model: &[u8],
        rec_model: &[u8],
        dict: &str,
        _config: Option<WasmOcrConfig>,
    ) -> Result<WasmOcrEngine, JsValue> {
        let engine = crate::ocr::OcrEngine::from_bytes(
            det_model,
            rec_model,
            dict,
            crate::ocr::OcrConfig::default(),
        )
        .map_err(|e| JsValue::from_str(&format!("OCR engine init failed: {e}")))?;
        Ok(WasmOcrEngine {
            inner: std::rc::Rc::new(engine),
        })
    }

    /// Run OCR on a raw image (PNG / JPEG / TIFF bytes).
    ///
    /// Returns a JSON string:
    /// `{ "text": "...", "confidence": 0.0,
    ///    "spans": [ { "text": "...", "confidence": 0.0,
    ///                 "polygon": [[x,y],[x,y],[x,y],[x,y]] } ] }`
    #[wasm_bindgen(js_name = "ocrImage")]
    pub fn ocr_image(&self, image_bytes: &[u8]) -> Result<String, JsValue> {
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| JsValue::from_str(&format!("image decode failed: {e}")))?;
        let out = self
            .inner
            .ocr_image(&img)
            .map_err(|e| JsValue::from_str(&format!("OCR failed: {e}")))?;
        Ok(ocr_output_to_json(&out))
    }
}

#[cfg(not(feature = "ocr-tract"))]
#[wasm_bindgen]
impl WasmOcrEngine {
    /// Not available in this build. OCR needs the `wasm-ocr` build of
    /// `pdf-oxide` (the pure-Rust tract backend); the default
    /// `pdf-oxide-wasm` ships without it.
    #[wasm_bindgen(constructor)]
    pub fn new(
        _det_model: &[u8],
        _rec_model: &[u8],
        _dict: &str,
        _config: Option<WasmOcrConfig>,
    ) -> Result<WasmOcrEngine, JsValue> {
        Err(JsValue::from_str(
            "OCR is not available in this WASM build. Use the `wasm-ocr` build of \
             pdf-oxide (pure-Rust tract OCR); see modelManifest() for the model URLs.",
        ))
    }
}

/// Serialize an [`crate::ocr::OcrOutput`] to the documented JSON shape.
#[cfg(feature = "ocr-tract")]
fn ocr_output_to_json(out: &crate::ocr::OcrOutput) -> String {
    let spans: Vec<serde_json::Value> = out
        .spans
        .iter()
        .map(|s| {
            serde_json::json!({
                "text": s.text,
                "confidence": s.confidence,
                "polygon": s.polygon,
            })
        })
        .collect();
    serde_json::json!({
        "text": out.text_in_reading_order(),
        "confidence": out.total_confidence,
        "spans": spans,
    })
    .to_string()
}

#[cfg(feature = "ocr-tract")]
#[wasm_bindgen]
impl WasmPdfDocument {
    // =================================Group 6b: OCR========================================

    /// Extract text from a page using OCR.
    ///
    /// Renders/extracts the page's scanned image and runs the in-WASM
    /// tract OCR pipeline. Requires a [`WasmOcrEngine`] built from
    /// host-supplied model bytes. Returns the recognized text in
    /// reading order (falls back to any native page text if the page
    /// has no extractable image).
    #[wasm_bindgen(js_name = "extractTextOcr")]
    pub fn extract_text_ocr(
        &mut self,
        page_index: usize,
        engine: &WasmOcrEngine,
    ) -> Result<String, JsValue> {
        // Take `&WasmOcrEngine` by reference, not by value: passing an
        // exported class by value in wasm-bindgen consumes the JS
        // handle (its pointer is set to null after the call), which
        // would break engine reuse across pages. Borrowed handles let
        // callers build the engine once and call this method N times,
        // matching the "built once, reuse" recipe in OCR_GUIDE.md.
        // (#523 Copilot review.)
        let doc = self
            .inner
            .lock()
            .map_err(|e| JsValue::from_str(&format!("document lock poisoned: {e}")))?;
        crate::ocr::ocr_page(
            &doc,
            page_index,
            &engine.inner,
            &crate::ocr::OcrExtractOptions::default(),
        )
        .map_err(|e| JsValue::from_str(&format!("OCR failed: {e}")))
    }
}

#[cfg(not(feature = "ocr-tract"))]
#[wasm_bindgen]
impl WasmPdfDocument {
    // =================================Group 6b: OCR========================================

    /// Extract text using OCR. Not available in this build — OCR needs
    /// the `wasm-ocr` build of `pdf-oxide`.
    #[wasm_bindgen(js_name = "extractTextOcr")]
    pub fn extract_text_ocr(
        &mut self,
        _page_index: usize,
        _engine: &WasmOcrEngine,
    ) -> Result<String, JsValue> {
        Err(JsValue::from_str(
            "OCR is not available in this WASM build. Use the `wasm-ocr` build of \
             pdf-oxide (pure-Rust tract OCR); see modelManifest() for the model URLs.",
        ))
    }
}

#[wasm_bindgen]
impl WasmPdfDocument {
    // ========================================================================
    // Group 6c: Form Fields
    // ========================================================================

    /// Get all form fields from the document.
    ///
    /// Returns an array of form field objects, each with:
    /// - name: Full qualified field name
    /// - field_type: "text", "button", "choice", "signature", or "unknown"
    /// - value: string, boolean, array of strings, or null
    /// - tooltip: string or null
    /// - bounds: [x1, y1, x2, y2] or null
    /// - flags: number or null
    /// - max_length: number or null
    /// - is_readonly: boolean
    /// - is_required: boolean
    #[wasm_bindgen(js_name = "getFormFields")]
    pub fn get_form_fields(&mut self) -> Result<JsValue, JsValue> {
        use crate::extractors::forms::{field_flags, FieldType, FieldValue, FormExtractor};

        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let fields = FormExtractor::extract_fields(&inner)
            .map_err(|e| JsValue::from_str(&format!("Failed to extract form fields: {}", e)))?;

        let result: Vec<serde_json::Value> = fields
            .iter()
            .map(|field| {
                let mut obj = serde_json::Map::new();

                obj.insert("name".into(), serde_json::Value::from(field.full_name.as_str()));

                let ft_str = match &field.field_type {
                    FieldType::Text => "text",
                    FieldType::Button => "button",
                    FieldType::Choice => "choice",
                    FieldType::Signature => "signature",
                    FieldType::Unknown(_) => "unknown",
                };
                obj.insert("field_type".into(), serde_json::Value::from(ft_str));

                let value = match &field.value {
                    FieldValue::Text(s) => serde_json::Value::from(s.as_str()),
                    FieldValue::Name(s) => serde_json::Value::from(s.as_str()),
                    FieldValue::Boolean(b) => serde_json::Value::from(*b),
                    FieldValue::Array(v) => serde_json::Value::Array(
                        v.iter()
                            .map(|s| serde_json::Value::from(s.as_str()))
                            .collect(),
                    ),
                    FieldValue::None => serde_json::Value::Null,
                };
                obj.insert("value".into(), value);

                match &field.tooltip {
                    Some(t) => obj.insert("tooltip".into(), serde_json::Value::from(t.as_str())),
                    None => obj.insert("tooltip".into(), serde_json::Value::Null),
                };

                match &field.bounds {
                    Some(b) => {
                        obj.insert("bounds".into(), serde_json::json!([b[0], b[1], b[2], b[3]]))
                    },
                    None => obj.insert("bounds".into(), serde_json::Value::Null),
                };

                match field.flags {
                    Some(f) => {
                        obj.insert("flags".into(), serde_json::Value::from(f));
                        obj.insert(
                            "is_readonly".into(),
                            serde_json::Value::from(f & field_flags::READ_ONLY != 0),
                        );
                        obj.insert(
                            "is_required".into(),
                            serde_json::Value::from(f & field_flags::REQUIRED != 0),
                        );
                    },
                    None => {
                        obj.insert("flags".into(), serde_json::Value::Null);
                        obj.insert("is_readonly".into(), serde_json::Value::from(false));
                        obj.insert("is_required".into(), serde_json::Value::from(false));
                    },
                };

                match field.max_length {
                    Some(ml) => obj.insert("max_length".into(), serde_json::Value::from(ml)),
                    None => obj.insert("max_length".into(), serde_json::Value::Null),
                };

                serde_json::Value::Object(obj)
            })
            .collect();

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Check if the document contains XFA form data.
    ///
    /// @returns true if the document has XFA form data
    #[wasm_bindgen(js_name = "hasXfa")]
    pub fn has_xfa(&mut self) -> Result<bool, JsValue> {
        use crate::xfa::XfaExtractor;

        let mut inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        XfaExtractor::has_xfa(&mut inner)
            .map_err(|e| JsValue::from_str(&format!("Failed to check XFA: {}", e)))
    }

    /// Export form field data as FDF or XFDF bytes.
    ///
    /// @param format - "fdf" or "xfdf" (default: "fdf")
    /// @returns Uint8Array containing the exported form data
    #[wasm_bindgen(js_name = "exportFormData")]
    pub fn export_form_data(&mut self, format: Option<String>) -> Result<Vec<u8>, JsValue> {
        let fmt = format.as_deref().unwrap_or("fdf");

        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;

        // Write to a temporary in-memory buffer via a temp file path
        let tmp_path = "/tmp/pdf_oxide_form_export_wasm.tmp";
        match fmt {
            "fdf" => editor
                .export_form_data_fdf(tmp_path)
                .map_err(|e| JsValue::from_str(&format!("Failed to export FDF: {}", e)))?,
            "xfdf" => editor
                .export_form_data_xfdf(tmp_path)
                .map_err(|e| JsValue::from_str(&format!("Failed to export XFDF: {}", e)))?,
            _ => {
                return Err(JsValue::from_str(&format!(
                    "Unknown format '{}'. Use 'fdf' or 'xfdf'.",
                    fmt
                )))
            },
        }

        let bytes = std::fs::read(tmp_path)
            .map_err(|e| JsValue::from_str(&format!("Failed to read exported data: {}", e)))?;
        let _ = std::fs::remove_file(tmp_path);
        Ok(bytes)
    }

    // ========================================================================
    // Group 6d: Form Field Get/Set Values
    // ========================================================================

    /// Get the value of a specific form field by name.
    ///
    /// @param name - Full qualified field name (e.g., "name" or "topmostSubform[0].Page1[0].f1_01[0]")
    /// @returns The field value: string for text, boolean for checkbox, null if not found
    #[wasm_bindgen(js_name = "getFormFieldValue")]
    pub fn get_form_field_value(&mut self, name: &str) -> Result<JsValue, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let value = editor
            .get_form_field_value(name)
            .map_err(|e| JsValue::from_str(&format!("Failed to get field value: {}", e)))?;

        match value {
            Some(v) => wasm_form_field_value_to_js(&v),
            None => Ok(JsValue::NULL),
        }
    }

    /// Set the value of a form field.
    ///
    /// @param name - Full qualified field name
    /// @param value - New value: string for text fields, boolean for checkboxes
    #[wasm_bindgen(js_name = "setFormFieldValue")]
    pub fn set_form_field_value(&mut self, name: &str, value: JsValue) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let field_value = js_to_form_field_value(&value)?;
        editor
            .set_form_field_value(name, field_value)
            .map_err(|e| JsValue::from_str(&format!("Failed to set field value: {}", e)))
    }

    // ========================================================================
    // Group 6e: Image Bytes Extraction
    // ========================================================================

    /// Extract image bytes from a page as PNG data.
    ///
    /// Returns an array of objects with: width, height, data (Uint8Array of PNG bytes), format ("png").
    #[wasm_bindgen(js_name = "extractImageBytes")]
    pub fn extract_image_bytes(&mut self, page_index: usize) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let images = inner
            .extract_images(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to extract images: {}", e)))?;

        let arr = js_sys::Array::new();
        for img in &images {
            let png_data = img.to_png_bytes().map_err(|e| {
                JsValue::from_str(&format!("Failed to convert image to PNG: {}", e))
            })?;

            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &JsValue::from_str("width"), &JsValue::from(img.width()))?;
            js_sys::Reflect::set(&obj, &JsValue::from_str("height"), &JsValue::from(img.height()))?;
            js_sys::Reflect::set(&obj, &JsValue::from_str("format"), &JsValue::from_str("png"))?;
            let uint8_array = js_sys::Uint8Array::from(png_data.as_slice());
            js_sys::Reflect::set(&obj, &JsValue::from_str("data"), &uint8_array)?;
            arr.push(&obj);
        }
        Ok(arr.into())
    }

    // ========================================================================
    // Group 6f: Form Flattening
    // ========================================================================

    /// Flatten all form fields into page content.
    ///
    /// After flattening, form field values become static text and are no longer editable.
    #[wasm_bindgen(js_name = "flattenForms")]
    pub fn flatten_forms(&mut self) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .flatten_forms()
            .map_err(|e| JsValue::from_str(&format!("Failed to flatten forms: {}", e)))
    }

    /// Flatten form fields on a specific page.
    ///
    /// @param page_index - Zero-based page number
    #[wasm_bindgen(js_name = "flattenFormsOnPage")]
    pub fn flatten_forms_on_page(&mut self, page_index: usize) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .flatten_forms_on_page(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to flatten forms on page: {}", e)))
    }

    /// Return warnings collected during the last form-flattening save.
    ///
    /// Each entry names a widget field that had no `/AP` appearance stream;
    /// flattening such a field produces a blank rectangle.
    ///
    /// @returns Array of warning strings
    #[wasm_bindgen(js_name = "flattenWarnings")]
    pub fn flatten_warnings(&self) -> Vec<String> {
        let Ok(editor) = self
            .editor
            .as_ref()
            .ok_or(())
            .and_then(|arc| arc.lock().map_err(|_| ()))
        else {
            return Vec::new();
        };
        editor.flatten_warnings().to_vec()
    }

    // ========================================================================
    // Group 6g: PDF Merging
    // ========================================================================

    /// Merge another PDF (provided as bytes) into this document.
    ///
    /// @param data - The PDF file contents to merge as a Uint8Array
    /// @returns Number of pages merged
    #[wasm_bindgen(js_name = "mergeFrom")]
    pub fn merge_from(&mut self, data: &[u8]) -> Result<usize, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .merge_from_bytes(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to merge PDF: {}", e)))
    }

    // ========================================================================
    // Group 6h: File Embedding
    // ========================================================================

    /// Embed a file into the PDF document.
    ///
    /// @param name - Display name for the embedded file
    /// @param data - File contents as a Uint8Array
    #[wasm_bindgen(js_name = "embedFile")]
    pub fn embed_file(&mut self, name: &str, data: &[u8]) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .embed_file(name, data.to_vec())
            .map_err(|e| JsValue::from_str(&format!("Failed to embed file: {}", e)))
    }

    // ========================================================================
    // Group 6i: Page Labels
    // ========================================================================

    /// Get page label ranges from the document.
    ///
    /// @returns Array of {start_page, style, prefix, start_value} objects, or empty array
    #[wasm_bindgen(js_name = "pageLabels")]
    pub fn page_labels(&mut self) -> Result<JsValue, JsValue> {
        use crate::extractors::page_labels::PageLabelExtractor;

        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let labels = PageLabelExtractor::extract(&inner)
            .map_err(|e| JsValue::from_str(&format!("Failed to get page labels: {}", e)))?;

        let result: Vec<serde_json::Value> = labels
            .iter()
            .map(|label| {
                let mut obj = serde_json::Map::new();
                obj.insert("start_page".into(), serde_json::Value::from(label.start_page));
                obj.insert("style".into(), serde_json::Value::from(format!("{:?}", label.style)));
                match &label.prefix {
                    Some(p) => obj.insert("prefix".into(), serde_json::Value::from(p.as_str())),
                    None => obj.insert("prefix".into(), serde_json::Value::Null),
                };
                obj.insert("start_value".into(), serde_json::Value::from(label.start_value));
                serde_json::Value::Object(obj)
            })
            .collect();

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    // ========================================================================
    // Group 6j: XMP Metadata
    // ========================================================================

    /// Get XMP metadata from the document.
    ///
    /// @returns Object with XMP fields (dc_title, dc_creator, etc.) or null if no XMP
    #[wasm_bindgen(js_name = "xmpMetadata")]
    pub fn xmp_metadata(&mut self) -> Result<JsValue, JsValue> {
        use crate::extractors::xmp::XmpExtractor;

        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let metadata = XmpExtractor::extract(&inner)
            .map_err(|e| JsValue::from_str(&format!("Failed to get XMP metadata: {}", e)))?;

        match metadata {
            None => Ok(JsValue::NULL),
            Some(xmp) => {
                let mut obj = serde_json::Map::new();
                if let Some(ref title) = xmp.dc_title {
                    obj.insert("dc_title".into(), serde_json::Value::from(title.as_str()));
                }
                if !xmp.dc_creator.is_empty() {
                    obj.insert(
                        "dc_creator".into(),
                        serde_json::Value::Array(
                            xmp.dc_creator
                                .iter()
                                .map(|s| serde_json::Value::from(s.as_str()))
                                .collect(),
                        ),
                    );
                }
                if let Some(ref desc) = xmp.dc_description {
                    obj.insert("dc_description".into(), serde_json::Value::from(desc.as_str()));
                }
                if !xmp.dc_subject.is_empty() {
                    obj.insert(
                        "dc_subject".into(),
                        serde_json::Value::Array(
                            xmp.dc_subject
                                .iter()
                                .map(|s| serde_json::Value::from(s.as_str()))
                                .collect(),
                        ),
                    );
                }
                if let Some(ref lang) = xmp.dc_language {
                    obj.insert("dc_language".into(), serde_json::Value::from(lang.as_str()));
                }
                if let Some(ref tool) = xmp.xmp_creator_tool {
                    obj.insert("xmp_creator_tool".into(), serde_json::Value::from(tool.as_str()));
                }
                if let Some(ref date) = xmp.xmp_create_date {
                    obj.insert("xmp_create_date".into(), serde_json::Value::from(date.as_str()));
                }
                if let Some(ref date) = xmp.xmp_modify_date {
                    obj.insert("xmp_modify_date".into(), serde_json::Value::from(date.as_str()));
                }
                if let Some(ref producer) = xmp.pdf_producer {
                    obj.insert("pdf_producer".into(), serde_json::Value::from(producer.as_str()));
                }
                if let Some(ref keywords) = xmp.pdf_keywords {
                    obj.insert("pdf_keywords".into(), serde_json::Value::from(keywords.as_str()));
                }

                serde_wasm_bindgen::to_value(&serde_json::Value::Object(obj))
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            },
        }
    }

    // ========================================================================
    // Group 7: Editing — Metadata
    // ========================================================================

    /// Set the document title.
    #[wasm_bindgen(js_name = "setTitle")]
    pub fn set_title(&mut self, title: &str) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor.set_title(title);
        Ok(())
    }

    /// Set the document author.
    #[wasm_bindgen(js_name = "setAuthor")]
    pub fn set_author(&mut self, author: &str) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor.set_author(author);
        Ok(())
    }

    /// Set the document subject.
    #[wasm_bindgen(js_name = "setSubject")]
    pub fn set_subject(&mut self, subject: &str) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor.set_subject(subject);
        Ok(())
    }

    /// Set the document keywords.
    #[wasm_bindgen(js_name = "setKeywords")]
    pub fn set_keywords(&mut self, keywords: &str) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor.set_keywords(keywords);
        Ok(())
    }

    // ========================================================================
    // Group 7: Editing — Page Properties
    // ========================================================================

    /// Get the rotation of a page in degrees (0, 90, 180, 270).
    #[wasm_bindgen(js_name = "pageRotation")]
    pub fn page_rotation(&mut self, page_index: usize) -> Result<i32, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .get_page_rotation(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to get rotation: {}", e)))
    }

    /// Set the rotation of a page (0, 90, 180, or 270 degrees).
    #[wasm_bindgen(js_name = "setPageRotation")]
    pub fn set_page_rotation(&mut self, page_index: usize, degrees: i32) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .set_page_rotation(page_index, degrees)
            .map_err(|e| JsValue::from_str(&format!("Failed to set rotation: {}", e)))
    }

    /// Rotate a page by the given degrees (adds to current rotation).
    #[wasm_bindgen(js_name = "rotatePage")]
    pub fn rotate_page(&mut self, page_index: usize, degrees: i32) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .rotate_page_by(page_index, degrees)
            .map_err(|e| JsValue::from_str(&format!("Failed to rotate page: {}", e)))
    }

    /// Rotate all pages by the given degrees.
    #[wasm_bindgen(js_name = "rotateAllPages")]
    pub fn rotate_all_pages(&mut self, degrees: i32) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .rotate_all_pages(degrees)
            .map_err(|e| JsValue::from_str(&format!("Failed to rotate all pages: {}", e)))
    }

    /// Get the MediaBox of a page as [llx, lly, urx, ury].
    #[wasm_bindgen(js_name = "pageMediaBox")]
    pub fn page_media_box(&mut self, page_index: usize) -> Result<Vec<f32>, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let mbox = editor
            .get_page_media_box(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to get media box: {}", e)))?;
        Ok(mbox.to_vec())
    }

    /// Set the MediaBox of a page.
    #[wasm_bindgen(js_name = "setPageMediaBox")]
    pub fn set_page_media_box(
        &mut self,
        page_index: usize,
        llx: f32,
        lly: f32,
        urx: f32,
        ury: f32,
    ) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .set_page_media_box(page_index, [llx, lly, urx, ury])
            .map_err(|e| JsValue::from_str(&format!("Failed to set media box: {}", e)))
    }

    /// Get the CropBox of a page as [llx, lly, urx, ury], or null if not set.
    #[wasm_bindgen(js_name = "pageCropBox")]
    pub fn page_crop_box(&mut self, page_index: usize) -> Result<JsValue, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let cbox = editor
            .get_page_crop_box(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to get crop box: {}", e)))?;
        match cbox {
            Some(b) => serde_wasm_bindgen::to_value(&b.to_vec())
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e))),
            None => Ok(JsValue::NULL),
        }
    }

    /// Set the CropBox of a page.
    #[wasm_bindgen(js_name = "setPageCropBox")]
    pub fn set_page_crop_box(
        &mut self,
        page_index: usize,
        llx: f32,
        lly: f32,
        urx: f32,
        ury: f32,
    ) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .set_page_crop_box(page_index, [llx, lly, urx, ury])
            .map_err(|e| JsValue::from_str(&format!("Failed to set crop box: {}", e)))
    }

    /// Crop margins from all pages.
    #[wasm_bindgen(js_name = "cropMargins")]
    pub fn crop_margins(
        &mut self,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
    ) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .crop_margins(left, right, top, bottom)
            .map_err(|e| JsValue::from_str(&format!("Failed to crop margins: {}", e)))
    }

    // ========================================================================
    // Group 7: Editing — Erase / Whiteout
    // ========================================================================

    /// Erase (whiteout) a rectangular region on a page.
    #[wasm_bindgen(js_name = "eraseRegion")]
    pub fn erase_region(
        &mut self,
        page_index: usize,
        llx: f32,
        lly: f32,
        urx: f32,
        ury: f32,
    ) -> Result<(), JsValue> {
        // Mark in inner document for extraction filtering
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .erase_region(page_index, crate::geometry::Rect::new(llx, lly, urx - llx, ury - lly))
            .map_err(|e| JsValue::from_str(&format!("Failed to mark region: {}", e)))?;

        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .erase_region(page_index, [llx, lly, urx, ury])
            .map_err(|e| JsValue::from_str(&format!("Failed to erase region: {}", e)))
    }

    /// Erase multiple rectangular regions on a page.
    ///
    /// @param page_index - Zero-based page number
    /// @param rects - Flat array of coordinates [llx1,lly1,urx1,ury1, llx2,lly2,urx2,ury2, ...]
    #[wasm_bindgen(js_name = "eraseRegions")]
    pub fn erase_regions(&mut self, page_index: usize, rects: &[f32]) -> Result<(), JsValue> {
        if !rects.len().is_multiple_of(4) {
            return Err(JsValue::from_str("rects must have a length that is a multiple of 4"));
        }

        // Mark all regions in inner document
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        for chunk in rects.as_chunks::<4>().0.iter() {
            let (llx, lly, urx, ury) = (chunk[0], chunk[1], chunk[2], chunk[3]);
            inner
                .erase_region(
                    page_index,
                    crate::geometry::Rect::new(llx, lly, urx - llx, ury - lly),
                )
                .map_err(|e| JsValue::from_str(&format!("Failed to mark region: {}", e)))?;
        }
        drop(inner);

        let rect_arrays: Vec<[f32; 4]> = rects
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .erase_regions(page_index, &rect_arrays)
            .map_err(|e| JsValue::from_str(&format!("Failed to erase regions: {}", e)))
    }

    /// Clear all pending erase operations for a page.
    #[wasm_bindgen(js_name = "clearEraseRegions")]
    pub fn clear_erase_regions(&mut self, page_index: usize) -> Result<(), JsValue> {
        // Clear inner document regions
        self.inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?
            .clear_erase_regions(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to clear regions: {}", e)))?;

        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor.clear_erase_regions(page_index);
        Ok(())
    }

    // ========================================================================
    // Group 7: Editing — Annotations
    // ========================================================================

    /// Flatten annotations on a page into the page content.
    #[wasm_bindgen(js_name = "flattenPageAnnotations")]
    pub fn flatten_page_annotations(&mut self, page_index: usize) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .flatten_page_annotations(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to flatten annotations: {}", e)))
    }

    /// Flatten all annotations in the document into page content.
    #[wasm_bindgen(js_name = "flattenAllAnnotations")]
    pub fn flatten_all_annotations(&mut self) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .flatten_all_annotations()
            .map_err(|e| JsValue::from_str(&format!("Failed to flatten annotations: {}", e)))
    }

    // ========================================================================
    // Group 7: Editing — Redaction
    // ========================================================================

    /// Apply redactions on a page (removes redacted content permanently).
    #[wasm_bindgen(js_name = "applyPageRedactions")]
    pub fn apply_page_redactions(&mut self, page_index: usize) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .apply_page_redactions(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to apply redactions: {}", e)))
    }

    /// Apply all redactions in the document.
    #[wasm_bindgen(js_name = "applyAllRedactions")]
    pub fn apply_all_redactions(&mut self) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .apply_all_redactions()
            .map_err(|e| JsValue::from_str(&format!("Failed to apply redactions: {}", e)))
    }

    /// Queue an explicit destructive redaction rectangle on a page
    /// (page user space; `fill` is an optional DeviceRGB `[r,g,b]`).
    #[wasm_bindgen(js_name = "addRedaction")]
    pub fn add_redaction(
        &mut self,
        page: usize,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        fill: Option<Vec<f32>>,
    ) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let fill = fill.and_then(|v| {
            if v.len() == 3 {
                Some([v[0], v[1], v[2]])
            } else {
                None
            }
        });
        editor
            .add_redaction(page, [x0, y0, x1, y1], fill)
            .map_err(|e| JsValue::from_str(&format!("Failed to add redaction: {}", e)))
    }

    /// Number of redaction regions queued for `page`.
    #[wasm_bindgen(js_name = "redactionCount")]
    pub fn redaction_count(&mut self, page: usize) -> Result<usize, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .redaction_count(page)
            .map_err(|e| JsValue::from_str(&format!("Failed to count redactions: {}", e)))
    }

    /// Destructively apply all queued redactions (true content removal,
    /// ISO 32000-1:2008 §12.5.6.23). Returns a `RedactionReport` object.
    #[wasm_bindgen(js_name = "applyRedactionsDestructive")]
    pub fn apply_redactions_destructive(
        &mut self,
        scrub_metadata: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let opts = crate::redaction::RedactionOptions {
            scrub_metadata: scrub_metadata.unwrap_or(true),
            ..crate::redaction::RedactionOptions::default()
        };
        let report = editor
            .apply_redactions_destructive(opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to apply redactions: {}", e)))?;
        serde_wasm_bindgen::to_value(&report).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Standalone document sanitization (#231 T10): strip `/Info`,
    /// catalog XMP `/Metadata`, document JavaScript and embedded files
    /// without geometric redaction. Returns a `RedactionReport` object.
    #[wasm_bindgen(js_name = "sanitizeDocument")]
    pub fn sanitize_document(
        &mut self,
        scrub_metadata: Option<bool>,
        remove_javascript: Option<bool>,
        remove_embedded_files: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let opts = crate::redaction::RedactionOptions {
            scrub_metadata: scrub_metadata.unwrap_or(true),
            remove_javascript: remove_javascript.unwrap_or(true),
            remove_embedded_files: remove_embedded_files.unwrap_or(true),
            ..crate::redaction::RedactionOptions::default()
        };
        let report = editor
            .sanitize_document(opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to sanitize document: {}", e)))?;
        serde_wasm_bindgen::to_value(&report).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Style configuration for header/footer text.
#[wasm_bindgen(js_name = "ArtifactStyle")]
#[derive(Clone)]
pub struct WasmArtifactStyle {
    inner: crate::writer::ArtifactStyle,
}

impl Default for WasmArtifactStyle {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = "ArtifactStyle")]
impl WasmArtifactStyle {
    /// Create a new artifact style.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::writer::ArtifactStyle::new(),
        }
    }

    /// Set font for the artifact.
    pub fn font(mut self, name: &str, size: f32) -> Self {
        self.inner = self.inner.font(name, size);
        self
    }

    /// Set bold font for the artifact.
    pub fn bold(mut self) -> Self {
        self.inner = self.inner.bold();
        self
    }

    /// Set color for the artifact.
    pub fn color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.inner = self.inner.color(r, g, b);
        self
    }
}

/// A header or footer artifact definition.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmArtifact {
    inner: crate::writer::Artifact,
}

impl Default for WasmArtifact {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmArtifact {
    /// Create a new artifact.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::writer::Artifact::new(),
        }
    }

    /// Create a left-aligned artifact.
    #[wasm_bindgen(js_name = "left")]
    pub fn left(text: &str) -> WasmArtifact {
        WasmArtifact {
            inner: crate::writer::Artifact::left(text),
        }
    }

    /// Create a center-aligned artifact.
    #[wasm_bindgen(js_name = "center")]
    pub fn center(text: &str) -> WasmArtifact {
        WasmArtifact {
            inner: crate::writer::Artifact::center(text),
        }
    }

    /// Create a right-aligned artifact.
    #[wasm_bindgen(js_name = "right")]
    pub fn right(text: &str) -> WasmArtifact {
        WasmArtifact {
            inner: crate::writer::Artifact::right(text),
        }
    }

    /// Set style for the artifact.
    #[wasm_bindgen(js_name = "withStyle")]
    pub fn with_style(mut self, style: &WasmArtifactStyle) -> Self {
        self.inner = self.inner.with_style(style.inner.clone());
        self
    }

    /// Set vertical offset for the artifact.
    #[wasm_bindgen(js_name = "withOffset")]
    pub fn with_offset(mut self, offset: f32) -> Self {
        self.inner = self.inner.with_offset(offset);
        self
    }
}

/// A header definition.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmHeader {
    #[allow(dead_code)] // retained for Clone semantics and future use
    inner: WasmArtifact,
}

impl Default for WasmHeader {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmHeader {
    /// Create a new empty header.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: WasmArtifact::new(),
        }
    }

    /// Create a left-aligned header.
    #[wasm_bindgen(js_name = "left")]
    pub fn left(text: &str) -> WasmHeader {
        WasmHeader {
            inner: WasmArtifact::left(text),
        }
    }

    /// Create a center-aligned header.
    #[wasm_bindgen(js_name = "center")]
    pub fn center(text: &str) -> WasmHeader {
        WasmHeader {
            inner: WasmArtifact::center(text),
        }
    }

    /// Create a right-aligned header.
    #[wasm_bindgen(js_name = "right")]
    pub fn right(text: &str) -> WasmHeader {
        WasmHeader {
            inner: WasmArtifact::right(text),
        }
    }
}

/// A footer definition.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmFooter {
    #[allow(dead_code)] // retained for Clone semantics and future use
    inner: WasmArtifact,
}

impl Default for WasmFooter {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmFooter {
    /// Create a new empty footer.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: WasmArtifact::new(),
        }
    }

    /// Create a left-aligned footer.
    #[wasm_bindgen(js_name = "left")]
    pub fn left(text: &str) -> WasmFooter {
        WasmFooter {
            inner: WasmArtifact::left(text),
        }
    }

    /// Create a center-aligned footer.
    #[wasm_bindgen(js_name = "center")]
    pub fn center(text: &str) -> WasmFooter {
        WasmFooter {
            inner: WasmArtifact::center(text),
        }
    }

    /// Create a right-aligned footer.
    #[wasm_bindgen(js_name = "right")]
    pub fn right(text: &str) -> WasmFooter {
        WasmFooter {
            inner: WasmArtifact::right(text),
        }
    }
}

/// A complete page template with header and footer.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmPageTemplate {
    inner: crate::writer::PageTemplate,
}

impl Default for WasmPageTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmPageTemplate {
    /// Create a new page template.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::writer::PageTemplate::new(),
        }
    }

    /// Set header artifact.
    pub fn header(mut self, header: &WasmArtifact) -> Self {
        self.inner = self.inner.header(header.inner.clone());
        self
    }

    /// Set footer artifact.
    pub fn footer(mut self, footer: &WasmArtifact) -> Self {
        self.inner = self.inner.footer(footer.inner.clone());
        self
    }

    /// Skip rendering template on the first page.
    #[wasm_bindgen(js_name = "skipFirstPage")]
    pub fn skip_first_page(mut self) -> Self {
        self.inner = self.inner.skip_first_page();
        self
    }
}

#[wasm_bindgen]
impl WasmPdfDocument {
    // ========================================================================
    // Group 7: Editing — Image Manipulation
    // ========================================================================

    /// Get information about images on a page.
    ///
    /// Returns an array of {name, bounds: [x, y, width, height], matrix: [a, b, c, d, e, f]}.
    #[wasm_bindgen(js_name = "pageImages")]
    pub fn page_images(&mut self, page_index: usize) -> Result<JsValue, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let images = editor
            .get_page_images(page_index)
            .map_err(|e| JsValue::from_str(&format!("Failed to get page images: {}", e)))?;
        serde_wasm_bindgen::to_value(&images)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Reposition an image on a page.
    #[wasm_bindgen(js_name = "repositionImage")]
    pub fn reposition_image(
        &mut self,
        page_index: usize,
        name: &str,
        x: f32,
        y: f32,
    ) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .reposition_image(page_index, name, x, y)
            .map_err(|e| JsValue::from_str(&format!("Failed to reposition image: {}", e)))
    }

    /// Resize an image on a page.
    #[wasm_bindgen(js_name = "resizeImage")]
    pub fn resize_image(
        &mut self,
        page_index: usize,
        name: &str,
        width: f32,
        height: f32,
    ) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .resize_image(page_index, name, width, height)
            .map_err(|e| JsValue::from_str(&format!("Failed to resize image: {}", e)))
    }

    /// Set the complete bounds of an image on a page.
    #[wasm_bindgen(js_name = "setImageBounds")]
    pub fn set_image_bounds(
        &mut self,
        page_index: usize,
        name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<(), JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .set_image_bounds(page_index, name, x, y, width, height)
            .map_err(|e| JsValue::from_str(&format!("Failed to set image bounds: {}", e)))
    }

    // ========================================================================
    // Group 7: Editing — Save
    // ========================================================================

    /// Save all edits and return the resulting PDF as bytes.
    ///
    /// @returns Uint8Array containing the modified PDF
    #[wasm_bindgen(js_name = "save")]
    pub fn save(&mut self) -> Result<Vec<u8>, JsValue> {
        self.save_to_bytes()
    }

    /// Save the modified PDF and return as bytes.
    /// `saveToBytes()` is the original method; `save()` is a convenience alias.
    ///
    /// @returns Uint8Array containing the modified PDF
    #[wasm_bindgen(js_name = "saveToBytes")]
    pub fn save_to_bytes(&mut self) -> Result<Vec<u8>, JsValue> {
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .save_to_bytes()
            .map_err(|e| JsValue::from_str(&format!("Failed to save PDF: {}", e)))
    }

    /// Save with options (compress, garbage_collect, linearize) and return bytes.
    ///
    /// @param {Object} [options] - Optional save options.
    /// @param {boolean} [options.compress=true] - Compress raw streams with FlateDecode.
    /// @param {boolean} [options.garbageCollect=true] - Remove unreachable objects.
    /// @param {boolean} [options.linearize=false] - Linearize (reserved, no-op).
    /// @returns Uint8Array containing the modified PDF
    #[wasm_bindgen(js_name = "saveWithOptions")]
    pub fn save_with_options_js(
        &mut self,
        compress: Option<bool>,
        garbage_collect: Option<bool>,
        linearize: Option<bool>,
    ) -> Result<Vec<u8>, JsValue> {
        let options = SaveOptions {
            compress: compress.unwrap_or(true),
            garbage_collect: garbage_collect.unwrap_or(true),
            linearize: linearize.unwrap_or(false),
            incremental: false,
            encryption: None,
        };
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .save_to_bytes_with_options(options)
            .map_err(|e| JsValue::from_str(&format!("Failed to save PDF: {}", e)))
    }

    /// Save with encryption and return the resulting PDF as bytes.
    #[wasm_bindgen(js_name = "saveEncryptedToBytes")]
    pub fn save_encrypted_to_bytes(
        &mut self,
        user_password: &str,
        owner_password: Option<String>,
        allow_print: Option<bool>,
        allow_copy: Option<bool>,
        allow_modify: Option<bool>,
        allow_annotate: Option<bool>,
    ) -> Result<Vec<u8>, JsValue> {
        let owner_pwd = owner_password.as_deref().unwrap_or(user_password);

        let permissions = Permissions {
            print: allow_print.unwrap_or(true),
            print_high_quality: allow_print.unwrap_or(true),
            modify: allow_modify.unwrap_or(true),
            copy: allow_copy.unwrap_or(true),
            annotate: allow_annotate.unwrap_or(true),
            fill_forms: allow_annotate.unwrap_or(true),
            accessibility: true,
            assemble: allow_modify.unwrap_or(true),
        };

        let config = EncryptionConfig::new(user_password, owner_pwd)
            .with_algorithm(EncryptionAlgorithm::Aes256)
            .with_permissions(permissions);

        let options = SaveOptions::with_encryption(config);
        let editor_arc = self.ensure_editor()?;
        let mut editor = editor_arc
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        editor
            .save_to_bytes_with_options(options)
            .map_err(|e| JsValue::from_str(&format!("Failed to save encrypted PDF: {}", e)))
    }

    // ========================================================================
    // Group 9: Validation — PDF/A, PDF/UA, PDF/X
    // ========================================================================

    /// Validate PDF/A compliance. Level: "1b", "2b", etc.
    #[wasm_bindgen(js_name = "validatePdfA")]
    pub fn validate_pdf_a(&mut self, level: &str) -> Result<JsValue, JsValue> {
        use crate::compliance::pdf_a::validate_pdf_a;
        use crate::compliance::types::PdfALevel;
        let pdf_level = match level {
            "1a" => PdfALevel::A1a,
            "1b" => PdfALevel::A1b,
            "2a" => PdfALevel::A2a,
            "2b" => PdfALevel::A2b,
            "2u" => PdfALevel::A2u,
            "3a" => PdfALevel::A3a,
            "3b" => PdfALevel::A3b,
            "3u" => PdfALevel::A3u,
            _ => return Err(JsValue::from_str(&format!("Unknown PDF/A level: {}", level))),
        };
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Lock failed"))?;
        let result =
            validate_pdf_a(&mut inner, pdf_level).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let errors: Vec<String> = result.errors.iter().map(|e| e.to_string()).collect();
        let warnings: Vec<String> = result.warnings.iter().map(|w| w.to_string()).collect();
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "valid": errors.is_empty(),
            "level": level,
            "errors": errors,
            "warnings": warnings,
        }))
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Convert the document to PDF/A compliance.
    ///
    /// Level must be one of: `"1a"`, `"1b"`, `"2a"`, `"2b"`, `"2u"`, `"3a"`, `"3b"`, `"3u"`.
    /// Returns a JS object with `success`, `level`, `actions`, and `errors` fields.
    #[wasm_bindgen(js_name = "convertToPdfA")]
    pub fn convert_to_pdf_a(&mut self, level: &str) -> Result<JsValue, JsValue> {
        use crate::compliance::convert_to_pdf_a;
        use crate::compliance::types::PdfALevel;
        let pdf_level = match level {
            "1a" => PdfALevel::A1a,
            "1b" => PdfALevel::A1b,
            "2a" => PdfALevel::A2a,
            "2b" => PdfALevel::A2b,
            "2u" => PdfALevel::A2u,
            "3a" => PdfALevel::A3a,
            "3b" => PdfALevel::A3b,
            "3u" => PdfALevel::A3u,
            _ => return Err(JsValue::from_str(&format!("Unknown PDF/A level: {}", level))),
        };
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Lock failed"))?;
        let result = convert_to_pdf_a(&mut inner, pdf_level)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let actions: Vec<String> = result
            .actions
            .iter()
            .map(|a| a.description.clone())
            .collect();
        let errors: Vec<String> = result.errors.iter().map(|e| e.reason.clone()).collect();
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "success": result.success,
            "level": level,
            "actions": actions,
            "errors": errors,
        }))
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Validate PDF/UA accessibility compliance.
    #[wasm_bindgen(js_name = "validatePdfUa")]
    pub fn validate_pdf_ua(&mut self, level: Option<String>) -> Result<JsValue, JsValue> {
        use crate::compliance::pdf_ua::{validate_pdf_ua, PdfUaLevel};
        let ua_level = match level.as_deref() {
            Some("2") => PdfUaLevel::Ua2,
            _ => PdfUaLevel::Ua1,
        };
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Lock failed"))?;
        let result =
            validate_pdf_ua(&mut inner, ua_level).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
        let warnings: Vec<String> = result.warnings.iter().map(|w| w.message.clone()).collect();
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "valid": result.is_compliant,
            "errors": errors,
            "warnings": warnings,
            "stats": {
                "structureElements": result.stats.structure_elements_checked,
                "images": result.stats.images_checked,
                "tables": result.stats.tables_checked,
                "pages": result.stats.pages_checked,
            }
        }))
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Validate PDF/X print production compliance.
    #[wasm_bindgen(js_name = "validatePdfX")]
    pub fn validate_pdf_x(&mut self, level: Option<String>) -> Result<JsValue, JsValue> {
        use crate::compliance::pdf_x::{validate_pdf_x, PdfXLevel};
        let x_level = match level.as_deref() {
            Some("1a") => PdfXLevel::X1a2001,
            Some("3") => PdfXLevel::X32002,
            Some("4") => PdfXLevel::X4,
            _ => PdfXLevel::X4,
        };
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Lock failed"))?;
        let result =
            validate_pdf_x(&mut inner, x_level).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "valid": result.is_compliant,
            "errors": errors,
        }))
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // ========================================================================
    // Group 10: Annotations
    // Note: add_link, add_highlight, add_note require editor API rework
    // to properly support PdfPage annotations. Tracked for future release.

    // ========================================================================
    // Group 11: Page Operations
    // ========================================================================

    /// Delete a page by index (0-based).
    #[wasm_bindgen(js_name = "deletePage")]
    pub fn delete_page(&mut self, index: usize) -> Result<(), JsValue> {
        use crate::editor::EditableDocument;
        let bytes = self.raw_bytes.to_vec();
        let mut editor = crate::editor::DocumentEditor::from_bytes(bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        editor
            .remove_page(index)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let new_bytes = editor
            .save_to_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let new_doc = crate::document::PdfDocument::from_bytes(new_bytes.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Lock failed"))?;
        *inner = new_doc;
        self.raw_bytes = Arc::new(new_bytes);
        Ok(())
    }

    /// Move a page within the document. Zero-based; `from_index` and
    /// `to_index` refer to positions **before** the move, matching the
    /// Python (`PyPdfDocument.move_page`) / Go (`DocumentEditor.MovePage`) /
    /// C# contracts.
    #[wasm_bindgen(js_name = "movePage")]
    pub fn move_page(&mut self, from_index: usize, to_index: usize) -> Result<(), JsValue> {
        use crate::editor::EditableDocument;
        let bytes = self.raw_bytes.to_vec();
        let mut editor = crate::editor::DocumentEditor::from_bytes(bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        editor
            .move_page(from_index, to_index)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let new_bytes = editor
            .save_to_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let new_doc = crate::document::PdfDocument::from_bytes(new_bytes.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Lock failed"))?;
        *inner = new_doc;
        self.raw_bytes = Arc::new(new_bytes);
        Ok(())
    }

    /// Extract specific pages to a new PDF (returns bytes).
    #[wasm_bindgen(js_name = "extractPages")]
    pub fn extract_pages(&mut self, pages: Vec<usize>) -> Result<Vec<u8>, JsValue> {
        use crate::editor::EditableDocument;
        let bytes = self.raw_bytes.to_vec();
        let mut editor = crate::editor::DocumentEditor::from_bytes(bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        // Keep only the requested pages by removing others in reverse order
        let page_count = editor
            .page_count()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        for i in (0..page_count).rev() {
            if !pages.contains(&i) {
                let _ = editor.remove_page(i);
            }
        }
        editor
            .save_to_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create a flattened PDF where each page is rendered as an image.
    /// Burns in all annotations, form fields, and overlays.
    /// Returns the flattened PDF as bytes.
    #[cfg(feature = "rendering")]
    #[wasm_bindgen(js_name = "flattenToImages")]
    pub fn flatten_to_images(&mut self, dpi: Option<u32>) -> Result<Vec<u8>, JsValue> {
        let dpi = dpi.unwrap_or(150);
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Lock failed"))?;
        crate::rendering::flatten_to_images(&inner, dpi)
            .map_err(|e| JsValue::from_str(&format!("Failed to flatten: {}", e)))
    }
}

// ============================================================================
// WasmPdf — PDF creation from content
// ============================================================================

/// Create new PDF documents from Markdown, HTML, or plain text.
///
/// ```javascript
/// const pdf = WasmPdf.fromMarkdown("# Hello\n\nWorld");
/// const bytes = pdf.toBytes(); // Uint8Array
/// console.log(`PDF size: ${pdf.size} bytes`);
/// ```
#[wasm_bindgen]
pub struct WasmPdf {
    bytes: Vec<u8>,
}

#[wasm_bindgen]
impl WasmPdf {
    /// Open an existing PDF from bytes for editing.
    ///
    /// @param data - PDF file contents as Uint8Array
    /// @returns WasmPdf for editing
    #[wasm_bindgen(js_name = "fromBytes")]
    pub fn from_bytes(data: &[u8]) -> Result<WasmPdf, JsValue> {
        let mut pdf = crate::api::Pdf::from_bytes(data.to_vec())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let bytes = pdf
            .save_to_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmPdf { bytes })
    }

    /// Merge multiple PDF byte arrays into a single PDF.
    ///
    /// @param pdfs - Array of Uint8Array, each containing a PDF
    /// @returns WasmPdf containing all pages
    #[wasm_bindgen(js_name = "merge")]
    pub fn merge(pdfs: Vec<js_sys::Uint8Array>) -> Result<WasmPdf, JsValue> {
        if pdfs.is_empty() {
            return Err(JsValue::from_str("No PDFs provided"));
        }
        let first_bytes = pdfs[0].to_vec();
        let first = crate::document::PdfDocument::from_bytes(first_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mut editor = crate::editor::DocumentEditor::from_document(first)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        for pdf_data in &pdfs[1..] {
            editor
                .merge_from_bytes(&pdf_data.to_vec())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }
        let bytes = editor
            .save_to_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmPdf { bytes })
    }

    /// Create a PDF from Markdown content.
    ///
    /// @param content - Markdown string
    /// @param title - Optional document title
    /// @param author - Optional document author
    #[wasm_bindgen(js_name = "fromMarkdown")]
    pub fn from_markdown(
        content: &str,
        title: Option<String>,
        author: Option<String>,
    ) -> Result<WasmPdf, JsValue> {
        let mut builder = PdfBuilder::new();
        if let Some(t) = title {
            builder = builder.title(t);
        }
        if let Some(a) = author {
            builder = builder.author(a);
        }
        let pdf = builder
            .from_markdown(content)
            .map_err(|e| JsValue::from_str(&format!("Failed to create PDF: {}", e)))?;
        Ok(WasmPdf {
            bytes: pdf.into_bytes(),
        })
    }

    /// Create a PDF from HTML content.
    ///
    /// @param content - HTML string
    /// @param title - Optional document title
    /// @param author - Optional document author
    #[wasm_bindgen(js_name = "fromHtml")]
    pub fn from_html(
        content: &str,
        title: Option<String>,
        author: Option<String>,
    ) -> Result<WasmPdf, JsValue> {
        let mut builder = PdfBuilder::new();
        if let Some(t) = title {
            builder = builder.title(t);
        }
        if let Some(a) = author {
            builder = builder.author(a);
        }
        let pdf = builder
            .from_html(content)
            .map_err(|e| JsValue::from_str(&format!("Failed to create PDF: {}", e)))?;
        Ok(WasmPdf {
            bytes: pdf.into_bytes(),
        })
    }

    /// Create a PDF from plain text.
    ///
    /// @param content - Plain text string
    /// @param title - Optional document title
    /// @param author - Optional document author
    #[wasm_bindgen(js_name = "fromText")]
    pub fn from_text(
        content: &str,
        title: Option<String>,
        author: Option<String>,
    ) -> Result<WasmPdf, JsValue> {
        let mut builder = PdfBuilder::new();
        if let Some(t) = title {
            builder = builder.title(t);
        }
        if let Some(a) = author {
            builder = builder.author(a);
        }
        let pdf = builder
            .from_text(content)
            .map_err(|e| JsValue::from_str(&format!("Failed to create PDF: {}", e)))?;
        Ok(WasmPdf {
            bytes: pdf.into_bytes(),
        })
    }

    /// Create a PDF from image bytes (PNG, JPEG, etc.).
    ///
    /// @param data - Image file contents as a Uint8Array
    #[wasm_bindgen(js_name = "fromImageBytes")]
    pub fn from_image_bytes(data: &[u8]) -> Result<WasmPdf, JsValue> {
        use crate::api::Pdf;
        let pdf = Pdf::from_image_bytes(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to create PDF from image: {}", e)))?;
        Ok(WasmPdf {
            bytes: pdf.into_bytes(),
        })
    }

    /// Create a PDF from multiple image byte arrays.
    ///
    /// Each image becomes a separate page. Pass an array of Uint8Arrays.
    ///
    /// @param images_array - Array of Uint8Arrays, each containing image file bytes (PNG/JPEG)
    #[wasm_bindgen(js_name = "fromMultipleImageBytes")]
    pub fn from_multiple_image_bytes(images_array: JsValue) -> Result<WasmPdf, JsValue> {
        use crate::writer::ImageData;

        let arr = js_sys::Array::from(&images_array);
        if arr.length() == 0 {
            return Err(JsValue::from_str("Empty image array"));
        }

        let mut images = Vec::new();
        for i in 0..arr.length() {
            let item = arr.get(i);
            let uint8 = js_sys::Uint8Array::new(&item);
            let bytes = uint8.to_vec();
            let image = ImageData::from_bytes(&bytes)
                .map_err(|e| JsValue::from_str(&format!("Failed to load image {}: {}", i, e)))?;
            images.push(image);
        }

        let pdf = PdfBuilder::new()
            .from_image_data_multiple(images)
            .map_err(|e| JsValue::from_str(&format!("Failed to create PDF from images: {}", e)))?;

        Ok(WasmPdf {
            bytes: pdf.into_bytes(),
        })
    }

    /// Get the PDF as a Uint8Array.
    #[wasm_bindgen(js_name = "toBytes")]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// Get the size of the PDF in bytes.
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    /// Render `html` with `css` applied, embedding `font_bytes` for the
    /// body text. The font must cover every codepoint used by `html` or
    /// unknown glyphs fall back to `.notdef`. See
    /// [`Self::from_html_css_with_fonts`] for a multi-font cascade.
    #[wasm_bindgen(js_name = "fromHtmlCss")]
    pub fn from_html_css(html: &str, css: &str, font_bytes: &[u8]) -> Result<WasmPdf, JsValue> {
        let pdf = crate::api::Pdf::from_html_css(html, css, font_bytes.to_vec())
            .map_err(|e| JsValue::from_str(&format!("fromHtmlCss failed: {e}")))?;
        Ok(WasmPdf {
            bytes: pdf.into_bytes(),
        })
    }

    /// Render `html` + `css` with a multi-font cascade. `families` and
    /// `fonts` are parallel arrays: `families[i]` names the CSS
    /// `font-family` that resolves to `fonts[i]` bytes. The first entry
    /// is the default used whenever a CSS `font-family` doesn't match a
    /// registered family.
    #[wasm_bindgen(js_name = "fromHtmlCssWithFonts")]
    pub fn from_html_css_with_fonts(
        html: &str,
        css: &str,
        families: Vec<String>,
        fonts: Vec<js_sys::Uint8Array>,
    ) -> Result<WasmPdf, JsValue> {
        if families.is_empty() {
            return Err(JsValue::from_str("at least one font must be provided"));
        }
        if families.len() != fonts.len() {
            return Err(JsValue::from_str("families and fonts arrays must be the same length"));
        }
        let font_vec: Vec<(String, Vec<u8>)> = families
            .into_iter()
            .zip(fonts.iter())
            .map(|(name, arr)| (name, arr.to_vec()))
            .collect();
        let pdf = crate::api::Pdf::from_html_css_with_fonts(html, css, font_vec)
            .map_err(|e| JsValue::from_str(&format!("fromHtmlCssWithFonts failed: {e}")))?;
        Ok(WasmPdf {
            bytes: pdf.into_bytes(),
        })
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert an editor FormFieldValue to a JsValue.
fn wasm_form_field_value_to_js(
    value: &crate::editor::form_fields::FormFieldValue,
) -> Result<JsValue, JsValue> {
    use crate::editor::form_fields::FormFieldValue;
    match value {
        FormFieldValue::Text(s) => Ok(JsValue::from_str(s)),
        FormFieldValue::Choice(s) => Ok(JsValue::from_str(s)),
        FormFieldValue::Boolean(b) => Ok(JsValue::from(*b)),
        FormFieldValue::MultiChoice(v) => {
            let arr = js_sys::Array::new();
            for s in v {
                arr.push(&JsValue::from_str(s));
            }
            Ok(arr.into())
        },
        FormFieldValue::None => Ok(JsValue::NULL),
    }
}

/// Convert a JsValue to an editor FormFieldValue.
fn js_to_form_field_value(
    value: &JsValue,
) -> Result<crate::editor::form_fields::FormFieldValue, JsValue> {
    use crate::editor::form_fields::FormFieldValue;

    if value.is_null() || value.is_undefined() {
        Ok(FormFieldValue::None)
    } else if let Some(b) = value.as_bool() {
        Ok(FormFieldValue::Boolean(b))
    } else if let Some(s) = value.as_string() {
        Ok(FormFieldValue::Text(s))
    } else if js_sys::Array::is_array(value) {
        let arr = js_sys::Array::from(value);
        let mut strings = Vec::new();
        for i in 0..arr.length() {
            let item = arr.get(i);
            strings.push(
                item.as_string()
                    .ok_or_else(|| JsValue::from_str("Array elements must be strings"))?,
            );
        }
        Ok(FormFieldValue::MultiChoice(strings))
    } else {
        Err(JsValue::from_str(
            "Value must be string, boolean, array of strings, null, or undefined",
        ))
    }
}

/// Convert OutlineItem tree to JSON for WASM serialization.
fn outline_to_json(items: &[crate::outline::OutlineItem]) -> Vec<serde_json::Value> {
    items
        .iter()
        .map(|item| {
            let mut obj = serde_json::Map::new();
            obj.insert("title".into(), serde_json::Value::from(item.title.as_str()));

            match &item.dest {
                Some(crate::outline::Destination::PageIndex(idx)) => {
                    obj.insert("page".into(), serde_json::Value::from(*idx));
                },
                Some(crate::outline::Destination::Named(name)) => {
                    obj.insert("page".into(), serde_json::Value::Null);
                    obj.insert("dest_name".into(), serde_json::Value::from(name.as_str()));
                },
                None => {
                    obj.insert("page".into(), serde_json::Value::Null);
                },
            }

            let children = outline_to_json(&item.children);
            obj.insert("children".into(), serde_json::Value::from(children));

            serde_json::Value::Object(obj)
        })
        .collect()
}

// ============================================================================
// Write-side API: DocumentBuilder / FluentPageBuilder / EmbeddedFont
// ============================================================================

/// Parse a stamp-type name into the Rust `StampType` enum. Unknown names
/// fall through to `Custom(String)`.
fn parse_wasm_stamp_type(name: &str) -> crate::writer::StampType {
    use crate::writer::StampType;
    match name {
        "Approved" => StampType::Approved,
        "Experimental" => StampType::Experimental,
        "NotApproved" => StampType::NotApproved,
        "AsIs" => StampType::AsIs,
        "Expired" => StampType::Expired,
        "NotForPublicRelease" => StampType::NotForPublicRelease,
        "Confidential" => StampType::Confidential,
        "Final" => StampType::Final,
        "Sold" => StampType::Sold,
        "Departmental" => StampType::Departmental,
        "ForComment" => StampType::ForComment,
        "TopSecret" => StampType::TopSecret,
        "Draft" => StampType::Draft,
        "ForPublicRelease" => StampType::ForPublicRelease,
        other => StampType::Custom(other.to_string()),
    }
}
//
// Mirrors the Python bindings' surface (see src/python.rs). Same buffering
// design: the Rust `FluentPageBuilder<'a>` borrows from `DocumentBuilder`
// and can't cross the wasm-bindgen boundary, so `WasmFluentPageBuilder`
// buffers operations and replays them against a real Rust builder inside
// `.done()`.
//
// Classes exported to JS (camelCase names via `js_name`):
//   * DocumentBuilder   — fluent top-level API
//   * FluentPageBuilder — per-page fluent API, committed by .done()
//   * EmbeddedFont      — TTF / OTF handle, consumed on registerEmbeddedFont

/// Horizontal-alignment enum shared by `textInRect`, buffered `table`, and
/// `streamingTable`. Maps 1:1 onto [`crate::writer::TextAlign`] /
/// [`crate::writer::CellAlign`]. Exported to JS as `Align` via `js_name`.
#[wasm_bindgen(js_name = "Align")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmAlign {
    /// Align to the left edge.
    Left = 0,
    /// Center horizontally.
    Center = 1,
    /// Align to the right edge.
    Right = 2,
}

impl From<WasmAlign> for crate::writer::TextAlign {
    fn from(a: WasmAlign) -> Self {
        match a {
            WasmAlign::Left => crate::writer::TextAlign::Left,
            WasmAlign::Center => crate::writer::TextAlign::Center,
            WasmAlign::Right => crate::writer::TextAlign::Right,
        }
    }
}

impl From<WasmAlign> for crate::writer::CellAlign {
    fn from(a: WasmAlign) -> Self {
        match a {
            WasmAlign::Left => crate::writer::CellAlign::Left,
            WasmAlign::Center => crate::writer::CellAlign::Center,
            WasmAlign::Right => crate::writer::CellAlign::Right,
        }
    }
}

impl WasmAlign {
    fn from_i32(v: i32) -> Self {
        match v {
            1 => WasmAlign::Center,
            2 => WasmAlign::Right,
            _ => WasmAlign::Left,
        }
    }
}

// Serde-deserializable view of a buffered table described by JS.
//
// Example JS:
//   page.table({
//     columns: [{ header: "SKU", width: 100, align: 0 }, ...],
//     rows: [["A-1","12"], ["B-2","3"]],
//     hasHeader: true,
//   });
#[derive(serde::Deserialize)]
struct WasmTableSpec {
    columns: Vec<WasmTableColumnSpec>,
    rows: Vec<Vec<String>>,
    #[serde(default, rename = "hasHeader", alias = "has_header")]
    has_header: bool,
}

#[derive(serde::Deserialize)]
struct WasmTableColumnSpec {
    header: Option<String>,
    width: Option<f32>,
    /// Accepts Align enum discriminant (0/1/2) or missing (defaults Left).
    #[serde(default)]
    align: Option<i32>,
}

#[derive(serde::Deserialize)]
struct WasmStreamingTableSpec {
    columns: Vec<WasmStreamingColumnSpec>,
    #[serde(default, rename = "repeatHeader", alias = "repeat_header")]
    repeat_header: bool,
    /// "fixed" | "sample" | "auto_all" (default "fixed")
    #[serde(default)]
    mode: Option<String>,
    #[serde(default, rename = "sampleRows", alias = "sample_rows")]
    sample_rows: Option<usize>,
    #[serde(default, rename = "minColWidthPt", alias = "min_col_width_pt")]
    min_col_width_pt: Option<f32>,
    #[serde(default, rename = "maxColWidthPt", alias = "max_col_width_pt")]
    max_col_width_pt: Option<f32>,
    /// Maximum rowspan. 0 or 1 = disabled (default).
    #[serde(default, rename = "maxRowspan", alias = "max_rowspan")]
    max_rowspan: Option<usize>,
    /// Maximum rows per batch before an automatic flush (default 256).
    #[serde(default, rename = "batchSize", alias = "batch_size")]
    batch_size: Option<usize>,
}

#[derive(serde::Deserialize)]
struct WasmStreamingColumnSpec {
    #[serde(default)]
    header: String,
    #[serde(default, rename = "width", alias = "widthPt", alias = "width_pt")]
    width: Option<f32>,
    #[serde(default)]
    align: Option<i32>,
}

/// Buffered operations applied to a real Rust `FluentPageBuilder` inside
/// `WasmFluentPageBuilder::done()`.
enum WasmPageOp {
    Font(String, f32),
    At(f32, f32),
    Text(String),
    Heading(u8, String),
    Paragraph(String),
    Space(f32),
    HorizontalRule,
    LinkUrl(String),
    LinkPage(usize),
    LinkNamed(String),
    Highlight(f32, f32, f32),
    Underline(f32, f32, f32),
    Strikeout(f32, f32, f32),
    Squiggly(f32, f32, f32),
    StickyNote(String),
    StickyNoteAt(f32, f32, String),
    Watermark(String),
    WatermarkConfidential,
    WatermarkDraft,
    Stamp(String),
    FreeText {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: String,
    },
    TextField {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        default_value: Option<String>,
    },
    Checkbox {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        checked: bool,
    },
    ComboBox {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        options: Vec<String>,
        selected: Option<String>,
    },
    RadioGroup {
        name: String,
        buttons: Vec<(String, f32, f32, f32, f32)>,
        selected: Option<String>,
    },
    PushButton {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        caption: String,
    },
    SignatureField {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    Footnote {
        ref_mark: String,
        note_text: String,
    },
    Columns {
        count: u32,
        gap_pt: f32,
        text: String,
    },
    Inline(String),
    InlineBold(String),
    InlineItalic(String),
    InlineColor {
        r: f32,
        g: f32,
        b: f32,
        text: String,
    },
    Newline,
    Rect(f32, f32, f32, f32),
    FilledRect(f32, f32, f32, f32, f32, f32, f32),
    Line(f32, f32, f32, f32),
    // v0.3.39 — issue #393 DocumentBuilder tables + primitives.
    TextInRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: String,
        align: WasmAlign,
    },
    StrokeRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
    },
    StrokeRectDashed {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
        dash: Vec<f32>,
        phase: f32,
    },
    StrokeLine {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
    },
    StrokeLineDashed {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
        dash: Vec<f32>,
        phase: f32,
    },
    NewPageSameSize,
    /// Buffered table: parsed columns + row strings are replayed by
    /// constructing a `crate::writer::Table` at commit time.
    BufferedTable {
        columns: Vec<(String, Option<f32>, WasmAlign)>,
        rows: Vec<Vec<String>>,
        has_header: bool,
    },
    /// Replay a recorded sequence of `StreamingTable` operations against
    /// the live `FluentPageBuilder` at commit time.
    StreamingTableBlock {
        config_columns: Vec<(String, f32, WasmAlign)>,
        repeat_header: bool,
        /// Each cell: (text, rowspan).
        rows: Vec<Vec<(String, usize)>>,
        mode: String,
        sample_rows: usize,
        min_col_width_pt: f32,
        max_col_width_pt: f32,
        max_rowspan: usize,
    },
    /// Pre-rendered barcode PNG bytes (generated at record time).
    BarcodeImage {
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    ImageWithAlt {
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        alt_text: String,
    },
    ImageArtifact {
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    LinkJavaScript(String),
    OnOpen(String),
    OnClose(String),
    FieldKeystroke(String),
    FieldFormat(String),
    FieldValidate(String),
    FieldCalculate(String),
}

/// Embedded TTF/OTF font usable by `WasmDocumentBuilder`. Single-use: once
/// passed to `registerEmbeddedFont`, the underlying Rust `EmbeddedFont` is
/// moved into the builder and this handle becomes empty.
#[wasm_bindgen]
pub struct WasmEmbeddedFont {
    inner: Option<crate::writer::EmbeddedFont>,
}

#[wasm_bindgen]
impl WasmEmbeddedFont {
    /// Load an embedded font from raw TTF/OTF bytes. Pass `name` to
    /// override the PostScript name baked into the font file.
    #[wasm_bindgen(js_name = "fromBytes")]
    pub fn from_bytes(data: &[u8], name: Option<String>) -> Result<WasmEmbeddedFont, JsValue> {
        crate::writer::EmbeddedFont::from_data(name, data.to_vec())
            .map(|inner| WasmEmbeddedFont { inner: Some(inner) })
            .map_err(|e| JsValue::from_str(&format!("fromBytes failed: {e}")))
    }

    /// The font's PostScript name (or the override). Empty once consumed.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner
            .as_ref()
            .map(|f| f.name.clone())
            .unwrap_or_default()
    }
}

/// WASM wrapper for [`crate::writer::DocumentBuilder`]. Fluent API for
/// programmatic multi-page PDF construction with embedded fonts and
/// annotations.
///
/// The terminal methods (`build`, `toBytesEncrypted`) **consume** the
/// builder; subsequent calls throw `Error: DocumentBuilder already
/// consumed`.
#[wasm_bindgen]
pub struct WasmDocumentBuilder {
    inner: Option<crate::writer::DocumentBuilder>,
}

impl WasmDocumentBuilder {
    fn take_inner(&mut self, ctx: &str) -> Result<crate::writer::DocumentBuilder, JsValue> {
        self.inner
            .take()
            .ok_or_else(|| JsValue::from_str(&format!("DocumentBuilder already consumed ({ctx})")))
    }

    fn with_inner<F>(&mut self, ctx: &str, f: F) -> Result<(), JsValue>
    where
        F: FnOnce(crate::writer::DocumentBuilder) -> crate::writer::DocumentBuilder,
    {
        let taken = self.take_inner(ctx)?;
        self.inner = Some(f(taken));
        Ok(())
    }
}

#[wasm_bindgen]
impl WasmDocumentBuilder {
    /// Create a new empty document builder. Equivalent to the Rust
    /// [`crate::writer::DocumentBuilder::new`] — every other method
    /// chains off the instance returned here.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmDocumentBuilder {
        WasmDocumentBuilder {
            inner: Some(crate::writer::DocumentBuilder::new()),
        }
    }

    /// Set document title.
    #[wasm_bindgen(js_name = "title")]
    pub fn title(&mut self, title: String) -> Result<(), JsValue> {
        self.with_inner("title", |b| b.title(title))
    }

    /// Set document author.
    #[wasm_bindgen(js_name = "author")]
    pub fn author(&mut self, author: String) -> Result<(), JsValue> {
        self.with_inner("author", |b| b.author(author))
    }

    /// Set document subject.
    #[wasm_bindgen(js_name = "subject")]
    pub fn subject(&mut self, subject: String) -> Result<(), JsValue> {
        self.with_inner("subject", |b| b.subject(subject))
    }

    /// Set document keywords (comma-separated per PDF convention).
    #[wasm_bindgen(js_name = "keywords")]
    pub fn keywords(&mut self, keywords: String) -> Result<(), JsValue> {
        self.with_inner("keywords", |b| b.keywords(keywords))
    }

    /// Set the creator application name recorded in the PDF.
    #[wasm_bindgen(js_name = "creator")]
    pub fn creator(&mut self, creator: String) -> Result<(), JsValue> {
        self.with_inner("creator", |b| b.creator(creator))
    }

    /// Run a JavaScript script when the document is opened (`/OpenAction`).
    #[wasm_bindgen(js_name = "onOpen")]
    pub fn on_open(&mut self, script: String) -> Result<(), JsValue> {
        self.with_inner("onOpen", |b| b.on_open(script))
    }

    /// Enable PDF/UA-1 tagged PDF mode.
    ///
    /// When enabled, `build()` emits `/MarkInfo`, `/StructTreeRoot`, `/Lang`,
    /// and `/ViewerPreferences` in the catalog. Opt-in — no effect unless called.
    #[wasm_bindgen(js_name = "taggedPdfUa1")]
    pub fn tagged_pdf_ua1(&mut self) -> Result<(), JsValue> {
        self.with_inner("taggedPdfUa1", |b| b.tagged_pdf_ua1())
    }

    /// Set the document's natural language tag (e.g. `"en-US"`).
    ///
    /// Emitted as `/Lang` in the catalog when `taggedPdfUa1()` is set.
    #[wasm_bindgen(js_name = "language")]
    pub fn language(&mut self, lang: String) -> Result<(), JsValue> {
        self.with_inner("language", |b| b.language(lang))
    }

    /// Add a role-map entry: custom structure type → standard PDF structure type.
    ///
    /// Emitted in `/RoleMap` inside the StructTreeRoot when `taggedPdfUa1()`
    /// is set. Multiple calls accumulate entries.
    #[wasm_bindgen(js_name = "roleMap")]
    pub fn role_map(&mut self, custom: String, standard: String) -> Result<(), JsValue> {
        self.with_inner("roleMap", |b| b.role_map(custom, standard))
    }

    /// Register a TTF / OTF font the pages can reference by name.
    /// **Consumes** `font` — reusing the `WasmEmbeddedFont` throws.
    #[wasm_bindgen(js_name = "registerEmbeddedFont")]
    pub fn register_embedded_font(
        &mut self,
        name: String,
        font: &mut WasmEmbeddedFont,
    ) -> Result<(), JsValue> {
        let embedded = font
            .inner
            .take()
            .ok_or_else(|| JsValue::from_str("EmbeddedFont already consumed"))?;
        self.with_inner("registerEmbeddedFont", |b| b.register_embedded_font(name, embedded))
    }

    /// Start a new A4 page. Returns a `FluentPageBuilder` that must be
    /// committed with `.done()` before calling another page method or a
    /// terminal (`build`, etc.).
    #[wasm_bindgen(js_name = "a4Page")]
    pub fn a4_page(&mut self) -> Result<WasmFluentPageBuilder, JsValue> {
        if self.inner.is_none() {
            return Err(JsValue::from_str("DocumentBuilder already consumed"));
        }
        Ok(WasmFluentPageBuilder::new_with_size(crate::writer::PageSize::A4))
    }

    /// Start a new US Letter page.
    #[wasm_bindgen(js_name = "letterPage")]
    pub fn letter_page(&mut self) -> Result<WasmFluentPageBuilder, JsValue> {
        if self.inner.is_none() {
            return Err(JsValue::from_str("DocumentBuilder already consumed"));
        }
        Ok(WasmFluentPageBuilder::new_with_size(crate::writer::PageSize::Letter))
    }

    /// Start a new page with custom dimensions in PDF points
    /// (72 pt = 1 inch).
    #[wasm_bindgen(js_name = "page")]
    pub fn page(&mut self, width: f32, height: f32) -> Result<WasmFluentPageBuilder, JsValue> {
        if self.inner.is_none() {
            return Err(JsValue::from_str("DocumentBuilder already consumed"));
        }
        Ok(WasmFluentPageBuilder::new_custom(width, height))
    }

    /// Commit a completed `FluentPageBuilder` back to this builder.
    /// Takes the place of the Rust `page.done()` re-parenting.
    ///
    /// JS users typically don't call this directly — the ergonomic
    /// pattern is `builder.a4Page();` for each page, then
    /// `builder.commitPage(page)` once ops are queued. For more fluent
    /// code, see the `FluentPageBuilder.done(builder)` helper which
    /// delegates to this method.
    #[wasm_bindgen(js_name = "commitPage")]
    pub fn commit_page(&mut self, page: &mut WasmFluentPageBuilder) -> Result<(), JsValue> {
        if page.done_called {
            return Err(JsValue::from_str("FluentPageBuilder.done() already called"));
        }
        page.done_called = true;

        let inner = self
            .inner
            .as_mut()
            .ok_or_else(|| JsValue::from_str("DocumentBuilder already consumed"))?;

        let page_size = page
            .page_size
            .unwrap_or(crate::writer::PageSize::Custom(page.custom_width, page.custom_height));
        let mut rust_page = inner.page(page_size);
        // Take ownership of the queued ops so any lingering `WasmStreamingTable`
        // Rc-clones become no-ops after the commit.
        let ops: Vec<WasmPageOp> = std::mem::take(&mut *page.ops.borrow_mut());
        for op in ops {
            rust_page = match op {
                WasmPageOp::Font(name, size) => rust_page.font(&name, size),
                WasmPageOp::At(x, y) => rust_page.at(x, y),
                WasmPageOp::Text(text) => rust_page.text(&text),
                WasmPageOp::Heading(level, text) => rust_page.heading(level, &text),
                WasmPageOp::Paragraph(text) => rust_page.paragraph(&text),
                WasmPageOp::Space(points) => rust_page.space(points),
                WasmPageOp::HorizontalRule => rust_page.horizontal_rule(),
                WasmPageOp::LinkUrl(url) => rust_page.link_url(&url),
                WasmPageOp::LinkPage(p) => rust_page.link_page(p),
                WasmPageOp::LinkNamed(dest) => rust_page.link_named(&dest),
                WasmPageOp::LinkJavaScript(script) => rust_page.link_javascript(&script),
                WasmPageOp::OnOpen(script) => rust_page.on_open(&script),
                WasmPageOp::OnClose(script) => rust_page.on_close(&script),
                WasmPageOp::FieldKeystroke(s) => rust_page.field_keystroke(&s),
                WasmPageOp::FieldFormat(s) => rust_page.field_format(&s),
                WasmPageOp::FieldValidate(s) => rust_page.field_validate(&s),
                WasmPageOp::FieldCalculate(s) => rust_page.field_calculate(&s),
                WasmPageOp::Highlight(r, g, b) => rust_page.highlight((r, g, b)),
                WasmPageOp::Underline(r, g, b) => rust_page.underline((r, g, b)),
                WasmPageOp::Strikeout(r, g, b) => rust_page.strikeout((r, g, b)),
                WasmPageOp::Squiggly(r, g, b) => rust_page.squiggly((r, g, b)),
                WasmPageOp::StickyNote(text) => rust_page.sticky_note(&text),
                WasmPageOp::StickyNoteAt(x, y, text) => rust_page.sticky_note_at(x, y, &text),
                WasmPageOp::Watermark(text) => rust_page.watermark(&text),
                WasmPageOp::WatermarkConfidential => rust_page.watermark_confidential(),
                WasmPageOp::WatermarkDraft => rust_page.watermark_draft(),
                WasmPageOp::Stamp(name) => rust_page.stamp(parse_wasm_stamp_type(&name)),
                WasmPageOp::FreeText { x, y, w, h, text } => {
                    rust_page.freetext(crate::geometry::Rect::new(x, y, w, h), &text)
                },
                WasmPageOp::TextField {
                    name,
                    x,
                    y,
                    w,
                    h,
                    default_value,
                } => rust_page.text_field(name, x, y, w, h, default_value),
                WasmPageOp::Checkbox {
                    name,
                    x,
                    y,
                    w,
                    h,
                    checked,
                } => rust_page.checkbox(name, x, y, w, h, checked),
                WasmPageOp::ComboBox {
                    name,
                    x,
                    y,
                    w,
                    h,
                    options,
                    selected,
                } => rust_page.combo_box(name, x, y, w, h, options, selected),
                WasmPageOp::RadioGroup {
                    name,
                    buttons,
                    selected,
                } => rust_page.radio_group(name, buttons, selected),
                WasmPageOp::PushButton {
                    name,
                    x,
                    y,
                    w,
                    h,
                    caption,
                } => rust_page.push_button(name, x, y, w, h, caption),
                WasmPageOp::SignatureField { name, x, y, w, h } => {
                    rust_page.signature_field(name, x, y, w, h)
                },
                WasmPageOp::Footnote {
                    ref_mark,
                    note_text,
                } => rust_page.footnote(&ref_mark, &note_text),
                WasmPageOp::Columns {
                    count,
                    gap_pt,
                    text,
                } => rust_page.columns(count, gap_pt, &text),
                WasmPageOp::Inline(text) => rust_page.inline(&text),
                WasmPageOp::InlineBold(text) => rust_page.inline_bold(&text),
                WasmPageOp::InlineItalic(text) => rust_page.inline_italic(&text),
                WasmPageOp::InlineColor { r, g, b, text } => rust_page.inline_color(r, g, b, &text),
                WasmPageOp::Newline => rust_page.newline(),
                WasmPageOp::Rect(x, y, w, h) => rust_page.rect(x, y, w, h),
                WasmPageOp::FilledRect(x, y, w, h, r, g, b) => {
                    rust_page.filled_rect(x, y, w, h, r, g, b)
                },
                WasmPageOp::Line(x1, y1, x2, y2) => rust_page.line(x1, y1, x2, y2),
                WasmPageOp::TextInRect {
                    x,
                    y,
                    w,
                    h,
                    text,
                    align,
                } => rust_page.text_in_rect(
                    crate::geometry::Rect::new(x, y, w, h),
                    &text,
                    align.into(),
                ),
                WasmPageOp::StrokeRect {
                    x,
                    y,
                    w,
                    h,
                    width,
                    r,
                    g,
                    b,
                } => {
                    rust_page.stroke_rect(x, y, w, h, crate::writer::LineStyle::new(width, r, g, b))
                },
                WasmPageOp::StrokeRectDashed {
                    x,
                    y,
                    w,
                    h,
                    width,
                    r,
                    g,
                    b,
                    dash,
                    phase,
                } => rust_page.stroke_rect(
                    x,
                    y,
                    w,
                    h,
                    crate::writer::LineStyle::new(width, r, g, b).with_dash(&dash, phase),
                ),
                WasmPageOp::StrokeLine {
                    x1,
                    y1,
                    x2,
                    y2,
                    width,
                    r,
                    g,
                    b,
                } => rust_page.stroke_line(
                    x1,
                    y1,
                    x2,
                    y2,
                    crate::writer::LineStyle::new(width, r, g, b),
                ),
                WasmPageOp::StrokeLineDashed {
                    x1,
                    y1,
                    x2,
                    y2,
                    width,
                    r,
                    g,
                    b,
                    dash,
                    phase,
                } => rust_page.stroke_line(
                    x1,
                    y1,
                    x2,
                    y2,
                    crate::writer::LineStyle::new(width, r, g, b).with_dash(&dash, phase),
                ),
                WasmPageOp::NewPageSameSize => rust_page.new_page_same_size(),
                WasmPageOp::BufferedTable {
                    columns,
                    rows,
                    has_header,
                } => {
                    // Build TableCell matrix from row strings.
                    let cell_rows: Vec<Vec<crate::writer::TableCell>> = rows
                        .into_iter()
                        .map(|r| {
                            r.into_iter()
                                .map(crate::writer::TableCell::text)
                                .collect::<Vec<_>>()
                        })
                        .collect();

                    // If hasHeader, prepend a synthetic header row built from column.header.
                    let cell_rows = if has_header {
                        let header: Vec<crate::writer::TableCell> = columns
                            .iter()
                            .map(|(h, _, _)| crate::writer::TableCell::text(h.clone()).bold())
                            .collect();
                        let mut out = Vec::with_capacity(cell_rows.len() + 1);
                        out.push(header);
                        out.extend(cell_rows);
                        out
                    } else {
                        cell_rows
                    };

                    let mut table = crate::writer::Table::new(cell_rows);
                    if has_header {
                        table = table.with_header_row();
                    }
                    let widths: Vec<crate::writer::ColumnWidth> = columns
                        .iter()
                        .map(|(_, w, _)| match w {
                            Some(pt) => crate::writer::ColumnWidth::Fixed(*pt),
                            None => crate::writer::ColumnWidth::Auto,
                        })
                        .collect();
                    let aligns: Vec<crate::writer::CellAlign> =
                        columns.iter().map(|(_, _, a)| (*a).into()).collect();
                    table = table.with_column_widths(widths).with_column_aligns(aligns);

                    rust_page.table(table)
                },
                WasmPageOp::BarcodeImage { bytes, x, y, w, h } => rust_page
                    .image_from_bytes(&bytes, crate::geometry::Rect::new(x, y, w, h))
                    .map_err(|e| JsValue::from_str(&e.to_string()))?,
                WasmPageOp::ImageWithAlt {
                    bytes,
                    x,
                    y,
                    w,
                    h,
                    alt_text,
                } => rust_page
                    .image_from_bytes_with_alt(
                        &bytes,
                        crate::geometry::Rect::new(x, y, w, h),
                        &alt_text,
                    )
                    .map_err(|e| JsValue::from_str(&e.to_string()))?,
                WasmPageOp::ImageArtifact { bytes, x, y, w, h } => rust_page
                    .image_from_bytes_as_artifact(&bytes, crate::geometry::Rect::new(x, y, w, h))
                    .map_err(|e| JsValue::from_str(&e.to_string()))?,
                WasmPageOp::StreamingTableBlock {
                    config_columns,
                    repeat_header,
                    rows,
                    mode,
                    sample_rows,
                    min_col_width_pt,
                    max_col_width_pt,
                    max_rowspan,
                } => {
                    let mut cfg = crate::writer::StreamingTableConfig::new()
                        .repeat_header(repeat_header)
                        .max_rowspan(max_rowspan);
                    cfg = match mode.as_str() {
                        "sample" => {
                            cfg.mode_sample(sample_rows, min_col_width_pt, max_col_width_pt)
                        },
                        "auto_all" => cfg.mode_auto_all(),
                        _ => cfg.mode_fixed(),
                    };
                    for (header, width, align) in config_columns {
                        cfg = cfg.column(
                            crate::writer::StreamingColumn::new(header)
                                .width_pt(width)
                                .align(align.into()),
                        );
                    }
                    let mut t = rust_page.streaming_table(cfg);
                    for row in rows {
                        let _ = t.push_row(|r| {
                            for (text, span) in &row {
                                if *span > 1 {
                                    r.span_cell(text.as_str(), *span);
                                } else {
                                    r.cell(text.as_str());
                                }
                            }
                        });
                    }
                    t.finish()
                },
            };
        }
        rust_page.done();
        Ok(())
    }

    /// Build the PDF and return it as a `Uint8Array`. **Consumes** the
    /// builder.
    #[wasm_bindgen(js_name = "build")]
    pub fn build(&mut self) -> Result<Vec<u8>, JsValue> {
        let inner = self.take_inner("build")?;
        inner
            .build()
            .map_err(|e| JsValue::from_str(&format!("build failed: {e}")))
    }

    /// Build the PDF with AES-256 encryption and return it as a
    /// `Uint8Array`. Granted permissions default to all. **Consumes**
    /// the builder.
    #[wasm_bindgen(js_name = "toBytesEncrypted")]
    pub fn to_bytes_encrypted(
        &mut self,
        user_password: &str,
        owner_password: &str,
    ) -> Result<Vec<u8>, JsValue> {
        let inner = self.take_inner("toBytesEncrypted")?;
        inner
            .to_bytes_encrypted(user_password, owner_password)
            .map_err(|e| JsValue::from_str(&format!("toBytesEncrypted failed: {e}")))
    }
}

impl Default for WasmDocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-page fluent builder. Buffers operations until `done(builder)` is
/// called, which commits them to the parent `WasmDocumentBuilder`. Each
/// instance is single-use — `done()` twice throws.
#[wasm_bindgen]
pub struct WasmFluentPageBuilder {
    page_size: Option<crate::writer::PageSize>,
    custom_width: f32,
    custom_height: f32,
    /// Shared so that any `WasmStreamingTable` spawned off this page can
    /// append its recorded block at `finish()` time without holding a
    /// Rust `&mut` borrow across the wasm-bindgen boundary.
    ops: std::rc::Rc<std::cell::RefCell<Vec<WasmPageOp>>>,
    done_called: bool,
    /// Tracked font name — mirrors the Rust `FluentPageBuilder`'s
    /// `text_config.font` so `measure()` can be served without round-tripping
    /// through a live builder.
    tracked_font: String,
    /// Tracked font size (points).
    tracked_font_size: f32,
    /// Tracked cursor y (points from page bottom, PDF convention). Needed so
    /// `remainingSpace()` can answer without committing the buffered ops.
    tracked_cursor_y: f32,
}

#[allow(missing_docs)] // docstrings on the Rust side (FluentPageBuilder::*) — methods here are thin op-buffers
#[wasm_bindgen]
impl WasmFluentPageBuilder {
    #[wasm_bindgen(js_name = "font")]
    pub fn font(&mut self, name: String, size: f32) -> Result<(), JsValue> {
        self.tracked_font = name.clone();
        self.tracked_font_size = size;
        self.push(WasmPageOp::Font(name, size))
    }

    #[wasm_bindgen(js_name = "at")]
    pub fn at(&mut self, x: f32, y: f32) -> Result<(), JsValue> {
        self.tracked_cursor_y = y;
        let _ = x;
        self.push(WasmPageOp::At(x, y))
    }

    #[wasm_bindgen(js_name = "text")]
    pub fn text(&mut self, text: String) -> Result<(), JsValue> {
        // Mirrors FluentPageBuilder::text — cursor drops by size * line_height
        // (default line_height 1.2).
        self.tracked_cursor_y -= self.tracked_font_size * 1.2;
        self.push(WasmPageOp::Text(text))
    }

    #[wasm_bindgen(js_name = "heading")]
    pub fn heading(&mut self, level: u8, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::Heading(level, text))
    }

    #[wasm_bindgen(js_name = "paragraph")]
    pub fn paragraph(&mut self, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::Paragraph(text))
    }

    #[wasm_bindgen(js_name = "space")]
    pub fn space(&mut self, points: f32) -> Result<(), JsValue> {
        self.tracked_cursor_y -= points;
        self.push(WasmPageOp::Space(points))
    }

    #[wasm_bindgen(js_name = "horizontalRule")]
    pub fn horizontal_rule(&mut self) -> Result<(), JsValue> {
        self.push(WasmPageOp::HorizontalRule)
    }

    // Annotations (Phase 3) ---------------------------------------------

    #[wasm_bindgen(js_name = "linkUrl")]
    pub fn link_url(&mut self, url: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::LinkUrl(url))
    }

    #[wasm_bindgen(js_name = "linkPage")]
    pub fn link_page(&mut self, page: usize) -> Result<(), JsValue> {
        self.push(WasmPageOp::LinkPage(page))
    }

    #[wasm_bindgen(js_name = "linkNamed")]
    pub fn link_named(&mut self, destination: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::LinkNamed(destination))
    }

    #[wasm_bindgen(js_name = "linkJavascript")]
    pub fn link_javascript(&mut self, script: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::LinkJavaScript(script))
    }

    #[wasm_bindgen(js_name = "onOpen")]
    pub fn on_open(&mut self, script: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::OnOpen(script))
    }

    #[wasm_bindgen(js_name = "onClose")]
    pub fn on_close(&mut self, script: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::OnClose(script))
    }

    #[wasm_bindgen(js_name = "fieldKeystroke")]
    pub fn field_keystroke(&mut self, script: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::FieldKeystroke(script))
    }

    #[wasm_bindgen(js_name = "fieldFormat")]
    pub fn field_format(&mut self, script: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::FieldFormat(script))
    }

    #[wasm_bindgen(js_name = "fieldValidate")]
    pub fn field_validate(&mut self, script: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::FieldValidate(script))
    }

    #[wasm_bindgen(js_name = "fieldCalculate")]
    pub fn field_calculate(&mut self, script: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::FieldCalculate(script))
    }

    #[wasm_bindgen(js_name = "highlight")]
    pub fn highlight(&mut self, r: f32, g: f32, b: f32) -> Result<(), JsValue> {
        self.push(WasmPageOp::Highlight(r, g, b))
    }

    #[wasm_bindgen(js_name = "underline")]
    pub fn underline(&mut self, r: f32, g: f32, b: f32) -> Result<(), JsValue> {
        self.push(WasmPageOp::Underline(r, g, b))
    }

    #[wasm_bindgen(js_name = "strikeout")]
    pub fn strikeout(&mut self, r: f32, g: f32, b: f32) -> Result<(), JsValue> {
        self.push(WasmPageOp::Strikeout(r, g, b))
    }

    #[wasm_bindgen(js_name = "squiggly")]
    pub fn squiggly(&mut self, r: f32, g: f32, b: f32) -> Result<(), JsValue> {
        self.push(WasmPageOp::Squiggly(r, g, b))
    }

    #[wasm_bindgen(js_name = "stickyNote")]
    pub fn sticky_note(&mut self, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::StickyNote(text))
    }

    #[wasm_bindgen(js_name = "stickyNoteAt")]
    pub fn sticky_note_at(&mut self, x: f32, y: f32, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::StickyNoteAt(x, y, text))
    }

    #[wasm_bindgen(js_name = "watermark")]
    pub fn watermark(&mut self, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::Watermark(text))
    }

    #[wasm_bindgen(js_name = "watermarkConfidential")]
    pub fn watermark_confidential(&mut self) -> Result<(), JsValue> {
        self.push(WasmPageOp::WatermarkConfidential)
    }

    #[wasm_bindgen(js_name = "watermarkDraft")]
    pub fn watermark_draft(&mut self) -> Result<(), JsValue> {
        self.push(WasmPageOp::WatermarkDraft)
    }

    #[wasm_bindgen(js_name = "stamp")]
    pub fn stamp(&mut self, name: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::Stamp(name))
    }

    #[wasm_bindgen(js_name = "freeText")]
    pub fn freetext(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: String,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::FreeText { x, y, w, h, text })
    }

    #[wasm_bindgen(js_name = "textField")]
    pub fn text_field(
        &mut self,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        default_value: Option<String>,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::TextField {
            name,
            x,
            y,
            w,
            h,
            default_value,
        })
    }

    #[wasm_bindgen(js_name = "checkbox")]
    pub fn checkbox(
        &mut self,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        checked: bool,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::Checkbox {
            name,
            x,
            y,
            w,
            h,
            checked,
        })
    }

    /// Add a dropdown combo-box.
    #[wasm_bindgen(js_name = "comboBox")]
    pub fn combo_box(
        &mut self,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        options: Vec<String>,
        selected: Option<String>,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::ComboBox {
            name,
            x,
            y,
            w,
            h,
            options,
            selected,
        })
    }

    /// Add a radio-button group. `values`, `xs`, `ys`, `ws`, `hs` are
    /// parallel arrays of length N describing each option's export
    /// value and rectangle. `selected` picks the initial value.
    #[wasm_bindgen(js_name = "radioGroup")]
    pub fn radio_group(
        &mut self,
        name: String,
        values: Vec<String>,
        xs: Vec<f32>,
        ys: Vec<f32>,
        ws: Vec<f32>,
        hs: Vec<f32>,
        selected: Option<String>,
    ) -> Result<(), JsValue> {
        let n = values.len();
        if xs.len() != n || ys.len() != n || ws.len() != n || hs.len() != n {
            return Err(JsValue::from_str(
                "radio_group: values/xs/ys/ws/hs must be equal-length arrays",
            ));
        }
        let buttons: Vec<(String, f32, f32, f32, f32)> = values
            .into_iter()
            .zip(xs)
            .zip(ys)
            .zip(ws)
            .zip(hs)
            .map(|((((v, x), y), w), h)| (v, x, y, w, h))
            .collect();
        self.push(WasmPageOp::RadioGroup {
            name,
            buttons,
            selected,
        })
    }

    /// Add a clickable push button with a visible caption.
    #[wasm_bindgen(js_name = "pushButton")]
    pub fn push_button(
        &mut self,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        caption: String,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::PushButton {
            name,
            x,
            y,
            w,
            h,
            caption,
        })
    }

    /// Add an unsigned signature placeholder field at the given bounds.
    #[wasm_bindgen(js_name = "signatureField")]
    pub fn signature_field(
        &mut self,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::SignatureField { name, x, y, w, h })
    }

    /// Add a footnote: inline `refMark` at the cursor and `noteText` body
    /// near the page bottom with a separator artifact line.
    #[wasm_bindgen(js_name = "footnote")]
    pub fn footnote(&mut self, ref_mark: String, note_text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::Footnote {
            ref_mark,
            note_text,
        })
    }

    /// Lay out `text` as balanced multi-column flow (`columnCount` columns,
    /// `gapPt` points between columns). Paragraphs in `text` are separated by `"\n\n"`.
    #[wasm_bindgen(js_name = "columns")]
    pub fn columns(&mut self, column_count: u32, gap_pt: f32, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::Columns {
            count: column_count,
            gap_pt,
            text,
        })
    }

    /// Emit `text` inline (advances cursorX only, not cursorY).
    #[wasm_bindgen(js_name = "inline")]
    pub fn inline(&mut self, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::Inline(text))
    }

    /// Inline bold run.
    #[wasm_bindgen(js_name = "inlineBold")]
    pub fn inline_bold(&mut self, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::InlineBold(text))
    }

    /// Inline italic run.
    #[wasm_bindgen(js_name = "inlineItalic")]
    pub fn inline_italic(&mut self, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::InlineItalic(text))
    }

    /// Inline colored run (RGB 0.0–1.0).
    #[wasm_bindgen(js_name = "inlineColor")]
    pub fn inline_color(&mut self, r: f32, g: f32, b: f32, text: String) -> Result<(), JsValue> {
        self.push(WasmPageOp::InlineColor { r, g, b, text })
    }

    /// Advance cursorY by one line-height and reset cursorX to 72 pt.
    #[wasm_bindgen(js_name = "newline")]
    pub fn newline(&mut self) -> Result<(), JsValue> {
        self.push(WasmPageOp::Newline)
    }

    /// Place a 1-D barcode image at `(x, y, w, h)` on the page.
    /// `barcodeType`: 0=Code128 1=Code39 2=EAN13 3=EAN8 4=UPCA 5=ITF
    /// 6=Code93 7=Codabar.
    #[wasm_bindgen(js_name = "barcode1d")]
    pub fn barcode_1d(
        &mut self,
        barcode_type: i32,
        data: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Result<(), JsValue> {
        let bt = match barcode_type {
            0 => crate::writer::BarcodeType::Code128,
            1 => crate::writer::BarcodeType::Code39,
            2 => crate::writer::BarcodeType::Ean13,
            3 => crate::writer::BarcodeType::Ean8,
            4 => crate::writer::BarcodeType::UpcA,
            5 => crate::writer::BarcodeType::Itf,
            6 => crate::writer::BarcodeType::Code93,
            7 => crate::writer::BarcodeType::Codabar,
            _ => {
                return Err(JsValue::from_str(&format!(
                    "unknown barcodeType {barcode_type}; valid values are 0–7"
                )))
            },
        };
        let opts = crate::writer::BarcodeOptions::new()
            .width(w as u32)
            .height(h as u32);
        let bytes = crate::writer::BarcodeGenerator::generate_1d(bt, &data, &opts)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.push(WasmPageOp::BarcodeImage { bytes, x, y, w, h })
    }

    /// Place a QR-code image at `(x, y, size, size)` on the page.
    #[wasm_bindgen(js_name = "barcodeQr")]
    pub fn barcode_qr(&mut self, data: String, x: f32, y: f32, size: f32) -> Result<(), JsValue> {
        let opts = crate::writer::QrCodeOptions::new().size(size as u32);
        let bytes = crate::writer::BarcodeGenerator::generate_qr(&data, &opts)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.push(WasmPageOp::BarcodeImage {
            bytes,
            x,
            y,
            w: size,
            h: size,
        })
    }

    /// Embed an image (JPEG/PNG/WebP bytes) with an accessibility alt text.
    #[wasm_bindgen(js_name = "imageWithAlt")]
    pub fn image_with_alt(
        &mut self,
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        alt_text: String,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::ImageWithAlt {
            bytes,
            x,
            y,
            w,
            h,
            alt_text,
        })
    }

    /// Embed a decorative image (JPEG/PNG/WebP bytes) as an /Artifact (no alt text).
    #[wasm_bindgen(js_name = "imageArtifact")]
    pub fn image_artifact(
        &mut self,
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::ImageArtifact { bytes, x, y, w, h })
    }

    /// Draw a stroked rectangle outline (1pt black).
    #[wasm_bindgen(js_name = "rect")]
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) -> Result<(), JsValue> {
        self.push(WasmPageOp::Rect(x, y, w, h))
    }

    /// Draw a filled rectangle. RGB channels in 0.0-1.0.
    #[wasm_bindgen(js_name = "filledRect")]
    pub fn filled_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        g: f32,
        b: f32,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::FilledRect(x, y, w, h, r, g, b))
    }

    /// Draw a line from (x1, y1) to (x2, y2) with 1pt black stroke.
    #[wasm_bindgen(js_name = "line")]
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> Result<(), JsValue> {
        self.push(WasmPageOp::Line(x1, y1, x2, y2))
    }

    // ──────────────────────────────────────────────────────────────────
    // v0.3.39 — issue #393 DocumentBuilder tables + primitives.
    // ──────────────────────────────────────────────────────────────────

    /// Measure the rendered width of `text` in the builder's current font
    /// and size, in PDF points. Pure query — does not mutate state.
    ///
    /// Thin JS view over [`crate::writer::FluentPageBuilder::measure`]. The
    /// WASM class tracks the current font/size independently of the
    /// buffered ops so this query is served without a live builder.
    #[wasm_bindgen(js_name = "measure")]
    pub fn measure(&self, text: &str) -> f32 {
        // Spin up a scratch DocumentBuilder + page + font() to delegate to
        // the real `FluentPageBuilder::measure`. This honours base-14 AFM
        // widths (embedded fonts aren't registered on the scratch builder;
        // callers measuring custom-font widths should measure after
        // committing or use base-14 metrics).
        let mut scratch = crate::writer::DocumentBuilder::new();
        let page = scratch
            .a4_page()
            .font(&self.tracked_font, self.tracked_font_size);
        page.measure(text)
    }

    /// Points remaining on the current page below the cursor (down to the
    /// 72 pt bottom margin). Mirrors
    /// [`crate::writer::FluentPageBuilder::remaining_space`] using the WASM
    /// class's independently-tracked cursor — accurate after `at`, `text`,
    /// `space`, `newPageSameSize`, `textInRect` and the table helpers.
    #[wasm_bindgen(js_name = "remainingSpace")]
    pub fn remaining_space(&self) -> f32 {
        const BOTTOM_MARGIN: f32 = 72.0;
        (self.tracked_cursor_y - BOTTOM_MARGIN).max(0.0)
    }

    /// Place wrapped text inside a rectangle with horizontal alignment.
    /// `align` is the `Align` enum (0 = Left, 1 = Center, 2 = Right) — also
    /// accepts a raw integer for JS callers that pre-date the enum import.
    #[wasm_bindgen(js_name = "textInRect")]
    pub fn text_in_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: String,
        align: i32,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::TextInRect {
            x,
            y,
            w,
            h,
            text,
            align: WasmAlign::from_i32(align),
        })
    }

    /// Draw a stroked rectangle with explicit stroke width and RGB colour.
    #[wasm_bindgen(js_name = "strokeRect")]
    #[allow(clippy::too_many_arguments)]
    pub fn stroke_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::StrokeRect {
            x,
            y,
            w,
            h,
            width,
            r,
            g,
            b,
        })
    }

    /// Draw a dashed rectangle border. `dash` is alternating on/off lengths in points; `phase` is the starting offset.
    #[wasm_bindgen(js_name = "strokeRectDashed")]
    #[allow(clippy::too_many_arguments)]
    pub fn stroke_rect_dashed(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
        dash: Vec<f32>,
        phase: f32,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::StrokeRectDashed {
            x,
            y,
            w,
            h,
            width,
            r,
            g,
            b,
            dash,
            phase,
        })
    }

    /// Draw a straight line with explicit stroke width and RGB colour.
    #[wasm_bindgen(js_name = "strokeLine")]
    #[allow(clippy::too_many_arguments)]
    pub fn stroke_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::StrokeLine {
            x1,
            y1,
            x2,
            y2,
            width,
            r,
            g,
            b,
        })
    }

    /// Draw a dashed line. `dash` is alternating on/off lengths in points; `phase` is the starting offset.
    #[wasm_bindgen(js_name = "strokeLineDashed")]
    #[allow(clippy::too_many_arguments)]
    pub fn stroke_line_dashed(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
        dash: Vec<f32>,
        phase: f32,
    ) -> Result<(), JsValue> {
        self.push(WasmPageOp::StrokeLineDashed {
            x1,
            y1,
            x2,
            y2,
            width,
            r,
            g,
            b,
            dash,
            phase,
        })
    }

    /// Finish the current page and start a new one with the same page
    /// size. Cursor resets to the top-left margin (72, height-72). The
    /// builder's font carries over.
    #[wasm_bindgen(js_name = "newPageSameSize")]
    pub fn new_page_same_size(&mut self) -> Result<(), JsValue> {
        // Reset tracked cursor to the top-left of a fresh page (same size).
        self.tracked_cursor_y = self.page_height() - 72.0;
        self.push(WasmPageOp::NewPageSameSize)
    }

    /// Render a buffered table from a JS object:
    ///
    /// ```javascript
    /// page.table({
    ///   columns: [
    ///     { header: "SKU", width: 100, align: Align.Left },
    ///     { header: "Qty", width: 60,  align: Align.Right },
    ///   ],
    ///   rows: [["A-1","12"], ["B-2","3"]],
    ///   hasHeader: true,
    /// });
    /// ```
    ///
    /// Uses `serde-wasm-bindgen` for deserialisation. Replays against the
    /// Rust `Table` builder at `done()` commit time.
    #[wasm_bindgen(js_name = "table")]
    pub fn table(&mut self, spec: JsValue) -> Result<(), JsValue> {
        let parsed: WasmTableSpec = serde_wasm_bindgen::from_value(spec)
            .map_err(|e| JsValue::from_str(&format!("table: invalid spec — {e}")))?;
        let columns: Vec<(String, Option<f32>, WasmAlign)> = parsed
            .columns
            .into_iter()
            .map(|c| {
                (c.header.unwrap_or_default(), c.width, WasmAlign::from_i32(c.align.unwrap_or(0)))
            })
            .collect();
        self.push(WasmPageOp::BufferedTable {
            columns,
            rows: parsed.rows,
            has_header: parsed.has_header,
        })
    }

    /// Open a streaming table. Returns a `StreamingTable` handle the caller
    /// pushes rows into; call `finish()` when done. The streamed rows are
    /// buffered per-table and replayed against the real Rust
    /// `StreamingTable` at `done()` commit time — avoiding the
    /// FluentPageBuilder-lifetime problem that otherwise can't cross the
    /// wasm-bindgen boundary.
    #[wasm_bindgen(js_name = "streamingTable")]
    pub fn streaming_table(&mut self, spec: JsValue) -> Result<WasmStreamingTable, JsValue> {
        if self.done_called {
            return Err(JsValue::from_str("FluentPageBuilder.done() already called"));
        }
        let parsed: WasmStreamingTableSpec = serde_wasm_bindgen::from_value(spec)
            .map_err(|e| JsValue::from_str(&format!("streamingTable: invalid spec — {e}")))?;
        let columns: Vec<(String, f32, WasmAlign)> = parsed
            .columns
            .into_iter()
            .map(|c| {
                (c.header, c.width.unwrap_or(100.0), WasmAlign::from_i32(c.align.unwrap_or(0)))
            })
            .collect();
        let batch_size = parsed.batch_size.unwrap_or(256).max(1);
        Ok(WasmStreamingTable {
            columns,
            repeat_header: parsed.repeat_header,
            current_batch: Vec::new(),
            completed_batches: Vec::new(),
            finished: false,
            mode: parsed.mode.unwrap_or_else(|| "fixed".to_string()),
            sample_rows: parsed.sample_rows.unwrap_or(50),
            min_col_width_pt: parsed.min_col_width_pt.unwrap_or(20.0),
            max_col_width_pt: parsed.max_col_width_pt.unwrap_or(400.0),
            max_rowspan: parsed.max_rowspan.unwrap_or(1),
            batch_size,
            page_ops: std::rc::Rc::clone(&self.ops),
        })
    }

    /// Convenience: commit this page's buffered ops to `builder`. Same
    /// as `builder.commitPage(this)` but lets JS users keep the
    /// chain-like flow:
    ///
    /// ```javascript
    /// const page = builder.a4Page();
    /// page.at(72, 720); page.text("Hi");
    /// page.done(builder);
    /// ```
    #[wasm_bindgen(js_name = "done")]
    pub fn done(&mut self, builder: &mut WasmDocumentBuilder) -> Result<(), JsValue> {
        builder.commit_page(self)
    }
}

impl WasmFluentPageBuilder {
    fn push(&mut self, op: WasmPageOp) -> Result<(), JsValue> {
        if self.done_called {
            return Err(JsValue::from_str("FluentPageBuilder.done() already called"));
        }
        self.ops.borrow_mut().push(op);
        Ok(())
    }

    fn new_with_size(size: crate::writer::PageSize) -> Self {
        let (_, height) = size.dimensions();
        Self {
            page_size: Some(size),
            custom_width: 0.0,
            custom_height: 0.0,
            ops: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            done_called: false,
            tracked_font: "Helvetica".to_string(),
            tracked_font_size: 12.0,
            // Mirrors `DocumentBuilder::page`: cursor starts at height - 72.
            tracked_cursor_y: height - 72.0,
        }
    }

    fn new_custom(width: f32, height: f32) -> Self {
        Self {
            page_size: None,
            custom_width: width,
            custom_height: height,
            ops: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            done_called: false,
            tracked_font: "Helvetica".to_string(),
            tracked_font_size: 12.0,
            tracked_cursor_y: height - 72.0,
        }
    }

    fn page_height(&self) -> f32 {
        match self.page_size {
            Some(s) => s.dimensions().1,
            None => self.custom_height,
        }
    }
}

/// WASM handle to a streaming-table building session. Created by
/// `FluentPageBuilder.streamingTable()`; rows are pushed via `pushRow`,
/// and the session is sealed with `finish()`.
///
/// Single-use: `finish()` twice throws, and `pushRow` after `finish()`
/// throws. The rows are buffered and replayed against the real Rust
/// `StreamingTable` at `WasmFluentPageBuilder.done()` commit time —
/// preserving the FluentPageBuilder borrow-lifetime invariant that can't
/// cross the wasm-bindgen boundary.
#[wasm_bindgen(js_name = "StreamingTable")]
pub struct WasmStreamingTable {
    columns: Vec<(String, f32, WasmAlign)>,
    repeat_header: bool,
    /// Rows accumulating in the current (not-yet-flushed) batch.
    current_batch: Vec<Vec<(String, usize)>>,
    /// Fully-completed batches waiting to be assembled at finish().
    completed_batches: Vec<Vec<Vec<(String, usize)>>>,
    finished: bool,
    mode: String,
    sample_rows: usize,
    min_col_width_pt: f32,
    max_col_width_pt: f32,
    max_rowspan: usize,
    /// Maximum rows per batch before an automatic flush (default 256).
    batch_size: usize,
    /// Shared handle to the parent page's op queue — used by `finish()`
    /// to thread the recorded block back onto the page without JS having
    /// to pass the page argument.
    page_ops: std::rc::Rc<std::cell::RefCell<Vec<WasmPageOp>>>,
}

#[wasm_bindgen(js_class = "StreamingTable")]
impl WasmStreamingTable {
    /// Number of columns configured on this streaming table.
    #[wasm_bindgen(js_name = "columnCount")]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Number of rows in the current (not-yet-flushed) batch.
    #[wasm_bindgen(js_name = "pendingRowCount")]
    pub fn pending_row_count(&self) -> usize {
        self.current_batch.len()
    }

    /// Number of fully-completed batches waiting for finish().
    #[wasm_bindgen(js_name = "batchCount")]
    pub fn batch_count(&self) -> usize {
        self.completed_batches.len()
    }

    /// Push one row as an array of cell strings (all rowspan=1). Returns an
    /// error if the table has already been finished or if the row's cell count
    /// does not match the column count. Auto-flushes the batch when full.
    #[wasm_bindgen(js_name = "pushRow")]
    pub fn push_row(&mut self, cells: Vec<String>) -> Result<(), JsValue> {
        if self.finished {
            return Err(JsValue::from_str("StreamingTable already finished"));
        }
        if cells.len() != self.columns.len() {
            return Err(JsValue::from_str(&format!(
                "streamingTable: row has {} cells, expected {}",
                cells.len(),
                self.columns.len()
            )));
        }
        self.current_batch
            .push(cells.into_iter().map(|s| (s, 1usize)).collect());
        if self.current_batch.len() >= self.batch_size {
            self.flush_batch();
        }
        Ok(())
    }

    /// Push one row with per-cell rowspan values. `cells` is a JS array of
    /// `[text, rowspan]` two-element arrays. `rowspan == 1` is a normal cell.
    /// Auto-flushes the batch when full.
    #[wasm_bindgen(js_name = "pushRowSpan")]
    pub fn push_row_span(&mut self, cells: JsValue) -> Result<(), JsValue> {
        if self.finished {
            return Err(JsValue::from_str("StreamingTable already finished"));
        }
        let parsed: Vec<(String, usize)> = serde_wasm_bindgen::from_value(cells)
            .map_err(|e| JsValue::from_str(&format!("pushRowSpan: invalid cells — {e}")))?;
        if parsed.len() != self.columns.len() {
            return Err(JsValue::from_str(&format!(
                "streamingTable: row has {} cells, expected {}",
                parsed.len(),
                self.columns.len()
            )));
        }
        self.current_batch.push(parsed);
        if self.current_batch.len() >= self.batch_size {
            self.flush_batch();
        }
        Ok(())
    }

    /// Explicitly flush the current batch to `completed_batches`.
    /// Called automatically when `batch_size` rows accumulate.
    #[wasm_bindgen(js_name = "flush")]
    pub fn flush(&mut self) {
        self.flush_batch();
    }

    /// Seal the streaming table — the buffered rows are flushed onto the
    /// parent page's op queue, to be replayed against the real Rust
    /// `StreamingTable` at `done()` commit time. Calling `finish()` twice
    /// throws.
    #[wasm_bindgen(js_name = "finish")]
    pub fn finish(&mut self) -> Result<(), JsValue> {
        if self.finished {
            return Err(JsValue::from_str("StreamingTable already finished"));
        }
        self.flush_batch();
        self.finished = true;
        let rows: Vec<Vec<(String, usize)>> = self.completed_batches.drain(..).flatten().collect();
        let op = WasmPageOp::StreamingTableBlock {
            config_columns: std::mem::take(&mut self.columns),
            repeat_header: self.repeat_header,
            rows,
            mode: self.mode.clone(),
            sample_rows: self.sample_rows,
            min_col_width_pt: self.min_col_width_pt,
            max_col_width_pt: self.max_col_width_pt,
            max_rowspan: self.max_rowspan,
        };
        self.page_ops.borrow_mut().push(op);
        Ok(())
    }
}

impl WasmStreamingTable {
    fn flush_batch(&mut self) {
        if !self.current_batch.is_empty() {
            let batch = std::mem::take(&mut self.current_batch);
            self.completed_batches.push(batch);
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================
//
// JsValue is not functional on non-wasm32 targets (wasm-bindgen stubs abort).
// Tests are split into two groups:
//   1. Native-safe: methods returning Rust types on the happy path (no JsValue at runtime)
//   2. Wasm-only: methods that return JsValue or whose error paths create JsValue
//
// Run native tests:  cargo test --lib --features wasm -- wasm::tests
// Run wasm tests:    wasm-pack test --headless --node --features wasm

// ─── Comprehensive auto extraction (#517) ──────────────────────────────────
//
// Frozen JSON-string envelope, identical to the C-ABI / Python / other
// bindings (api-design.md §4 parity contract — JS callers `JSON.parse`).
// Strictly additive; existing methods byte-identical. WASM ships
// classify + reasons + native fallback (OCR is a documented WASM stub —
// `extractTextOcr` — so the auto path gracefully returns native text +
// an `ocr_requested_but_unavailable` reason, never an opaque error).
#[wasm_bindgen]
impl WasmPdfDocument {
    /// The document's structured diagnostics, as a real JS array of objects.
    ///
    /// Each entry carries `category` (a stable snake_case token), `page`,
    /// `message` and `spec_section`. **Tolerate categories you do not know** —
    /// they are added in minor releases.
    ///
    /// This is where the library reports *about* extraction. Nothing is
    /// written into the extracted content: a page with no text extracts as
    /// nothing and says so here, with its page index.
    ///
    /// Returned as an object rather than a JSON string, unlike the C ABI —
    /// parity of the *data model* is what matters across surfaces, and making
    /// a browser caller run `JSON.parse` on our output is a wart, not parity.
    /// In the browser this is the whole channel: `src/ffi.rs` is compiled out
    /// on wasm32, so the C accessor does not exist here.
    #[wasm_bindgen(js_name = "structuredWarnings")]
    pub fn structured_warnings(&mut self) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        serde_wasm_bindgen::to_value(&inner.structured_warnings())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// As `structuredWarnings`, but drains: the returned entries are removed.
    #[wasm_bindgen(js_name = "takeStructuredWarnings")]
    pub fn take_structured_warnings(&mut self) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        serde_wasm_bindgen::to_value(&inner.take_structured_warnings())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Cheap per-page text-vs-OCR classification → JSON
    /// `DocumentClassification`.
    #[wasm_bindgen(js_name = "classifyDocument")]
    pub fn classify_document(&mut self) -> Result<String, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let cls = inner
            .classify_document()
            .map_err(|e| JsValue::from_str(&format!("classify failed: {e}")))?;
        serde_json::to_string(&cls).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Cheap per-page classification → JSON `PageClassification`.
    #[wasm_bindgen(js_name = "classifyPage")]
    pub fn classify_page(&mut self, page_index: usize) -> Result<String, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let c = inner
            .classify_page(page_index)
            .map_err(|e| JsValue::from_str(&format!("classify failed: {e}")))?;
        serde_json::to_string(&c).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// One-shot auto text extraction — graceful native fallback (never
    /// the opaque OCR error #513).
    #[wasm_bindgen(js_name = "extractTextAuto")]
    pub fn extract_text_auto(&mut self, page_index: usize) -> Result<String, JsValue> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        inner
            .extract_text_auto(page_index)
            .map_err(|e| JsValue::from_str(&format!("auto extraction failed: {e}")))
    }

    /// Rich per-page extraction → JSON `PageExtraction` (per-region
    /// bbox + typed reason). `optionsJson` is `{}`-tolerant
    /// `AutoExtractOptions`; undefined/empty → defaults.
    #[wasm_bindgen(js_name = "extractPageAuto")]
    pub fn extract_page_auto(
        &mut self,
        page_index: usize,
        options_json: Option<String>,
    ) -> Result<String, JsValue> {
        let opts = match options_json {
            Some(ref s) if !s.trim().is_empty() => serde_json::from_str(s)
                .map_err(|e| JsValue::from_str(&format!("invalid optionsJson: {e}")))?,
            _ => crate::extractors::auto::AutoExtractOptions::default(),
        };
        let inner = self
            .inner
            .lock()
            .map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        let pe = crate::extractors::auto::AutoExtractor::with(opts)
            .extract_page(&inner, page_index)
            .map_err(|e| JsValue::from_str(&format!("auto extraction failed: {e}")))?;
        serde_json::to_string(&pe).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Test Helpers
    // ========================================================================

    fn make_text_pdf(text: &str) -> Vec<u8> {
        crate::api::Pdf::from_text(text).unwrap().into_bytes()
    }

    fn doc_from_text(text: &str) -> WasmPdfDocument {
        WasmPdfDocument::new(&make_text_pdf(text), None).unwrap()
    }

    fn make_markdown_pdf(md: &str) -> Vec<u8> {
        crate::api::PdfBuilder::new()
            .from_markdown(md)
            .unwrap()
            .into_bytes()
    }

    // ========================================================================
    // Group: Constructor
    // ========================================================================

    #[test]
    fn test_new_valid_pdf() {
        let bytes = make_text_pdf("Hello world");
        let result = WasmPdfDocument::new(&bytes, None);
        assert!(result.is_ok());
    }

    // Error-path tests require JsValue::from_str() which only works on wasm32
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_new_invalid_bytes() {
        let result = WasmPdfDocument::new(b"not a pdf at all", None);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_new_empty_bytes() {
        let result = WasmPdfDocument::new(b"", None);
        assert!(result.is_err());
    }

    // ========================================================================
    // Group: Core Read-Only
    // ========================================================================

    #[test]
    fn test_page_count() {
        let mut doc = doc_from_text("Hello");
        let count = doc.page_count().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_version() {
        let doc = doc_from_text("Hello");
        let ver = doc.version().unwrap();
        assert_eq!(ver.len(), 2);
        assert!(ver[0] >= 1, "major version should be at least 1");
    }

    #[test]
    fn test_authenticate_unencrypted() {
        let mut doc = doc_from_text("Hello");
        let result = doc.authenticate("password");
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_structure_tree_false() {
        let mut doc = doc_from_text("Hello");
        assert!(!doc.has_structure_tree().unwrap_or(false));
    }

    #[test]
    fn test_page_count_from_markdown() {
        let bytes = make_markdown_pdf("# Title\n\nSome content");
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        assert!(doc.page_count().unwrap() >= 1);
    }

    // ========================================================================
    // Group: Text Extraction
    // ========================================================================

    #[test]
    fn test_extract_text() {
        let mut doc = doc_from_text("Hello world");
        let text = doc.extract_text(0, JsValue::UNDEFINED).unwrap();
        assert!(
            text.contains("Hello") || text.contains("world"),
            "extracted text should contain source content, got: {}",
            text
        );
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_extract_text_invalid_page() {
        let mut doc = doc_from_text("Hello");
        let result = doc.extract_text(999, JsValue::UNDEFINED);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_all_text() {
        let mut doc = doc_from_text("Hello world");
        let text = doc.extract_all_text().unwrap();
        assert!(!text.is_empty(), "extract_all_text should return non-empty");
    }

    #[test]
    fn test_extract_text_preserves_content() {
        let mut doc = doc_from_text("Test content 12345");
        let text = doc.extract_text(0, JsValue::UNDEFINED).unwrap();
        assert!(text.contains("12345"), "should preserve numeric content, got: {}", text);
    }

    // ========================================================================
    // Group: Format Conversion
    // ========================================================================

    #[test]
    fn test_to_markdown() {
        let mut doc = doc_from_text("Hello markdown");
        let md = doc.to_markdown(0, None, None, None).unwrap();
        assert!(!md.is_empty());
    }

    #[test]
    fn test_to_markdown_all() {
        let mut doc = doc_from_text("Hello markdown");
        let md = doc.to_markdown_all(None, None, None).unwrap();
        assert!(!md.is_empty());
    }

    #[test]
    fn test_to_html() {
        let mut doc = doc_from_text("Hello html");
        let html = doc.to_html(0, None, None, None).unwrap();
        assert!(!html.is_empty());
    }

    #[test]
    fn test_to_html_all() {
        let mut doc = doc_from_text("Hello html");
        let html = doc.to_html_all(None, None, None).unwrap();
        assert!(!html.is_empty());
    }

    #[test]
    fn test_to_plain_text() {
        let mut doc = doc_from_text("Hello plain");
        let text = doc.to_plain_text(0).unwrap();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_to_plain_text_all() {
        let mut doc = doc_from_text("Hello plain");
        let text = doc.to_plain_text_all().unwrap();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_to_markdown_with_options() {
        let mut doc = doc_from_text("Hello options");
        let md = doc.to_markdown(0, Some(false), Some(false), None).unwrap();
        assert!(!md.is_empty());
    }

    #[test]
    fn test_to_html_with_options() {
        let mut doc = doc_from_text("Hello options");
        let html = doc.to_html(0, Some(true), Some(false), None).unwrap();
        assert!(!html.is_empty());
    }

    // ========================================================================
    // Group: Structured Extraction (serde_wasm_bindgen — wasm32 only)
    // ========================================================================

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_extract_chars_ok() {
        let mut doc = doc_from_text("ABC");
        let result = doc.extract_chars(0);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_extract_spans_ok() {
        let mut doc = doc_from_text("Hello spans");
        let result = doc.extract_spans(0);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_extract_chars_invalid_page() {
        let mut doc = doc_from_text("ABC");
        let result = doc.extract_chars(999);
        assert!(result.is_err());
    }

    // ========================================================================
    // Group: Search (serde_wasm_bindgen — wasm32 only)
    // ========================================================================

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_search_found() {
        let mut doc = doc_from_text("Hello world test search");
        let result = doc.search("Hello", None, Some(true), None, None);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_search_not_found() {
        let mut doc = doc_from_text("Hello world");
        let result = doc.search("ZZZZZ_NONEXISTENT", None, Some(true), None, None);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_search_page_found() {
        let mut doc = doc_from_text("Hello searchable content");
        let result = doc.search_page(0, "Hello", None, Some(true), None, None);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_search_page_invalid() {
        let mut doc = doc_from_text("Hello");
        let result = doc.search_page(999, "Hello", None, Some(true), None, None);
        let _ = result;
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_prepare_search_then_search_finds_term() {
        let mut doc = doc_from_text("Hello prepared searchable term");
        assert!(doc.prepare_search().is_ok());
        let result = doc.search("prepared", None, Some(true), None, None);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_clear_search_index_then_search_still_finds_term() {
        let mut doc = doc_from_text("Hello cleared searchable term");
        assert!(doc.prepare_search().is_ok());
        assert!(doc.clear_search_index().is_ok());
        let result = doc.search("cleared", None, Some(true), None, None);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Group: Image Info (serde_wasm_bindgen — wasm32 only)
    // ========================================================================

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_extract_images_ok() {
        let mut doc = doc_from_text("No images here");
        let result = doc.extract_images(0);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_extract_images_invalid_page() {
        let mut doc = doc_from_text("Hello");
        let result = doc.extract_images(999);
        assert!(result.is_err());
    }

    // ========================================================================
    // Group: Document Structure (serde_wasm_bindgen — wasm32 only)
    // ========================================================================

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_get_outline_ok() {
        let mut doc = doc_from_text("No outline here");
        let result = doc.get_outline();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_get_annotations_ok() {
        let mut doc = doc_from_text("No annotations here");
        let result = doc.get_annotations(0);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_get_annotations_invalid_page() {
        let mut doc = doc_from_text("Hello");
        let result = doc.get_annotations(999);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_extract_paths_ok() {
        let mut doc = doc_from_text("No paths here");
        let result = doc.extract_paths(0);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_extract_paths_invalid_page() {
        let mut doc = doc_from_text("Hello");
        let result = doc.extract_paths(999);
        assert!(result.is_err());
    }

    // ========================================================================
    // Group: Metadata Editing
    // ========================================================================

    #[test]
    fn test_set_title() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.set_title("My Title").is_ok());
    }

    #[test]
    fn test_set_author() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.set_author("Author Name").is_ok());
    }

    #[test]
    fn test_set_subject() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.set_subject("Subject Line").is_ok());
    }

    #[test]
    fn test_set_keywords() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.set_keywords("pdf, test, rust").is_ok());
    }

    // ========================================================================
    // Group: Page Properties
    // ========================================================================

    #[test]
    fn test_page_rotation() {
        let mut doc = doc_from_text("Hello");
        let rotation = doc.page_rotation(0).unwrap();
        assert_eq!(rotation, 0);
    }

    #[test]
    fn test_set_page_rotation() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.set_page_rotation(0, 90).is_ok());
        let rotation = doc.page_rotation(0).unwrap();
        assert_eq!(rotation, 90);
    }

    #[test]
    fn test_rotate_page() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.rotate_page(0, 90).is_ok());
    }

    #[test]
    fn test_rotate_all_pages() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.rotate_all_pages(180).is_ok());
    }

    #[test]
    fn test_page_media_box() {
        let mut doc = doc_from_text("Hello");
        let mbox = doc.page_media_box(0).unwrap();
        assert_eq!(mbox.len(), 4, "media box should have 4 coordinates");
        assert!(mbox[2] > mbox[0], "urx should be greater than llx");
        assert!(mbox[3] > mbox[1], "ury should be greater than lly");
    }

    #[test]
    fn test_set_page_media_box() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.set_page_media_box(0, 0.0, 0.0, 612.0, 792.0).is_ok());
    }

    // page_crop_box returns JsValue via serde_wasm_bindgen — wasm32 only
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_page_crop_box_unset() {
        let mut doc = doc_from_text("Hello");
        let result = doc.page_crop_box(0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_page_crop_box() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.set_page_crop_box(0, 10.0, 10.0, 600.0, 780.0).is_ok());
    }

    #[test]
    fn test_crop_margins() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.crop_margins(10.0, 10.0, 10.0, 10.0).is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_page_rotation_invalid_page() {
        let mut doc = doc_from_text("Hello");
        let result = doc.page_rotation(999);
        assert!(result.is_err());
    }

    // ========================================================================
    // Group: Erase / Whiteout
    // ========================================================================

    #[test]
    fn test_erase_region() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.erase_region(0, 0.0, 0.0, 100.0, 100.0).is_ok());
    }

    #[test]
    fn test_erase_regions_valid() {
        let mut doc = doc_from_text("Hello");
        let rects = [0.0, 0.0, 100.0, 100.0, 200.0, 200.0, 300.0, 300.0];
        assert!(doc.erase_regions(0, &rects).is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_erase_regions_invalid_length() {
        let mut doc = doc_from_text("Hello");
        let rects = [0.0, 0.0, 100.0]; // Not a multiple of 4
        let result = doc.erase_regions(0, &rects);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_erase_regions() {
        let mut doc = doc_from_text("Hello");
        doc.erase_region(0, 0.0, 0.0, 100.0, 100.0).unwrap();
        assert!(doc.clear_erase_regions(0).is_ok());
    }

    // ========================================================================
    // Group: Annotations
    // ========================================================================

    #[test]
    fn test_flatten_page_annotations() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.flatten_page_annotations(0).is_ok());
    }

    #[test]
    fn test_flatten_all_annotations() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.flatten_all_annotations().is_ok());
    }

    // ========================================================================
    // Group: Redaction
    // ========================================================================

    #[test]
    fn test_apply_page_redactions() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.apply_page_redactions(0).is_ok());
    }

    #[test]
    fn test_apply_all_redactions() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.apply_all_redactions().is_ok());
    }

    // ========================================================================
    // Group: Form Fields
    // ========================================================================

    fn make_form_pdf() -> Vec<u8> {
        use crate::geometry::Rect;
        use crate::writer::{CheckboxWidget, ComboBoxWidget, PdfWriter, TextFieldWidget};

        let mut writer = PdfWriter::new();
        {
            let mut page = writer.add_page(612.0, 792.0);
            page.add_text_field(
                TextFieldWidget::new("name", Rect::new(72.0, 700.0, 200.0, 20.0))
                    .with_value("Alice"),
            );
            page.add_checkbox(
                CheckboxWidget::new("agree", Rect::new(72.0, 650.0, 15.0, 15.0)).checked(),
            );
            page.add_combo_box(
                ComboBoxWidget::new("color", Rect::new(72.0, 600.0, 150.0, 20.0))
                    .with_options(vec!["Red", "Blue", "Green"])
                    .with_value("Blue"),
            );
        }
        writer.finish().expect("Failed to create form PDF")
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_get_form_fields_returns_array() {
        let bytes = make_form_pdf();
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        let result = doc.get_form_fields().unwrap();
        assert!(js_sys::Array::is_array(&result));
        let arr = js_sys::Array::from(&result);
        assert!(arr.length() >= 3, "Should have at least 3 fields, got {}", arr.length());
    }

    #[test]
    fn test_has_xfa_on_plain_pdf() {
        let mut doc = doc_from_text("No XFA");
        assert!(!doc.has_xfa().unwrap(), "Plain text PDF should not have XFA");
    }

    #[test]
    fn test_has_xfa_on_form_pdf() {
        let bytes = make_form_pdf();
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        assert!(!doc.has_xfa().unwrap(), "PdfWriter form should not have XFA");
    }

    // ========================================================================
    // Group: Image Manipulation (serde_wasm_bindgen — wasm32 only)
    // ========================================================================

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_page_images() {
        let mut doc = doc_from_text("Hello");
        let result = doc.page_images(0);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Group: Save
    // ========================================================================

    #[test]
    fn test_save_to_bytes() {
        let mut doc = doc_from_text("Hello save");
        let bytes = doc.save_to_bytes().unwrap();
        assert!(!bytes.is_empty(), "saved bytes should not be empty");
    }

    #[test]
    fn test_save_to_bytes_pdf_header() {
        let mut doc = doc_from_text("Hello header");
        let bytes = doc.save_to_bytes().unwrap();
        assert!(bytes.starts_with(b"%PDF"), "saved bytes should start with PDF header");
    }

    #[test]
    fn test_save_encrypted_to_bytes() {
        let mut doc = doc_from_text("Hello encrypted");
        let bytes = doc
            .save_encrypted_to_bytes("pass", None, None, None, None, None)
            .unwrap();
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_save_roundtrip() {
        let mut doc = doc_from_text("Roundtrip test");
        doc.set_title("Roundtrip Title").unwrap();
        let bytes = doc.save_to_bytes().unwrap();

        let mut doc2 = WasmPdfDocument::new(&bytes, None).unwrap();
        let text = doc2.extract_text(0, JsValue::UNDEFINED).unwrap();
        assert!(text.contains("Roundtrip"), "roundtrip should preserve text, got: {}", text);
    }

    // ========================================================================
    // Group: WasmPdf Creation
    // ========================================================================

    #[test]
    fn test_wasm_pdf_from_markdown() {
        let result = WasmPdf::from_markdown("# Hello\n\nWorld", None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wasm_pdf_from_html() {
        let result = WasmPdf::from_html("<h1>Hello</h1><p>World</p>", None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wasm_pdf_from_text() {
        let result = WasmPdf::from_text("Hello world", None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wasm_pdf_to_bytes() {
        let pdf = WasmPdf::from_text("Hello bytes", None, None).unwrap();
        let bytes = pdf.to_bytes();
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_wasm_pdf_size() {
        let pdf = WasmPdf::from_text("Hello size", None, None).unwrap();
        assert!(pdf.size() > 0, "PDF size should be positive");
    }

    #[test]
    fn test_wasm_pdf_with_metadata() {
        let pdf = WasmPdf::from_markdown(
            "# Test",
            Some("Test Title".to_string()),
            Some("Test Author".to_string()),
        )
        .unwrap();
        assert!(pdf.size() > 0);
        let mut doc = WasmPdfDocument::new(&pdf.to_bytes(), None).unwrap();
        assert_eq!(doc.page_count().unwrap(), 1);
    }

    // ========================================================================
    // Group: Lazy Editor Init
    // ========================================================================

    #[test]
    fn test_editor_lazy_init() {
        let doc = doc_from_text("Hello");
        assert!(doc.editor.is_none());
    }

    #[test]
    fn test_editor_initialized_on_edit() {
        let mut doc = doc_from_text("Hello");
        assert!(doc.editor.is_none());
        doc.set_title("Title").unwrap();
        assert!(doc.editor.is_some());
    }

    // ========================================================================
    // Group: Form Field Get/Set Values
    // ========================================================================

    #[test]
    fn test_get_form_field_value_text() {
        let bytes = make_form_pdf();
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        // get_form_field_value returns JsValue which aborts on non-wasm32,
        // so test the underlying Rust API directly here.
        let editor_mutex = doc.ensure_editor().unwrap();
        let mut editor = editor_mutex.lock().unwrap();
        let value = editor.get_form_field_value("name");
        assert!(value.is_ok(), "field 'name' should have a value");
    }

    #[test]
    fn test_set_form_field_value_text() {
        let bytes = make_form_pdf();
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        // set_form_field_value with a string JsValue
        // On native, JsValue operations are stubbed, so we test via the Rust API
        // instead — just verify the method exists and the type signatures match
        let editor_mutex = doc.ensure_editor().unwrap();
        let mut editor = editor_mutex.lock().unwrap();
        let result = editor.set_form_field_value(
            "name",
            crate::editor::form_fields::FormFieldValue::Text("Bob".to_string()),
        );
        assert!(result.is_ok(), "set_form_field_value should succeed");
    }

    // ========================================================================
    // Group: Image Bytes Extraction (native-safe: no JsValue in happy path)
    // ========================================================================

    #[test]
    fn test_extract_image_bytes_empty_on_text_pdf() {
        // Text-only PDF has no images — should not error
        let doc = doc_from_text("No images here");
        // extract_image_bytes returns JsValue, tested in wasm_bindgen_tests
        // For native, test that the underlying API works
        let images = doc.inner.lock().unwrap().extract_images(0).unwrap();
        assert_eq!(images.len(), 0);
    }

    // ========================================================================
    // Group: Form Flattening
    // ========================================================================

    #[test]
    fn test_flatten_forms() {
        let bytes = make_form_pdf();
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        let editor_mutex = doc.ensure_editor().unwrap();
        let mut editor = editor_mutex.lock().unwrap();
        let result = editor.flatten_forms();
        assert!(result.is_ok(), "flatten_forms should succeed");
    }

    #[test]
    fn test_flatten_forms_on_page() {
        let bytes = make_form_pdf();
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        let editor_mutex = doc.ensure_editor().unwrap();
        let mut editor = editor_mutex.lock().unwrap();
        let result = editor.flatten_forms_on_page(0);
        assert!(result.is_ok(), "flatten_forms_on_page should succeed");
    }

    // ========================================================================
    // Group: PDF Merging
    // ========================================================================

    #[test]
    fn test_merge_from_bytes() {
        let bytes1 = make_text_pdf("Page 1");
        let bytes2 = make_text_pdf("Page 2");
        let mut doc = WasmPdfDocument::new(&bytes1, None).unwrap();
        let editor_mutex = doc.ensure_editor().unwrap();
        let mut editor = editor_mutex.lock().unwrap();
        let count = editor.merge_from_bytes(&bytes2).unwrap();
        assert_eq!(count, 1, "should merge 1 page");
    }

    // ========================================================================
    // Group: File Embedding
    // ========================================================================

    #[test]
    fn test_embed_file() {
        let bytes = make_text_pdf("Hello");
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        let editor_mutex = doc.ensure_editor().unwrap();
        let mut editor = editor_mutex.lock().unwrap();
        let result = editor.embed_file("readme.txt", b"Hello World".to_vec());
        assert!(result.is_ok(), "embed_file should succeed");
    }

    // ========================================================================
    // Group: Page Labels
    // ========================================================================

    #[test]
    fn test_page_labels_empty() {
        let doc = doc_from_text("Hello");
        let labels = crate::extractors::page_labels::PageLabelExtractor::extract(
            &mut doc.inner.lock().unwrap(),
        );
        // Simple generated PDFs typically have no page labels
        assert!(labels.is_ok());
    }

    // ========================================================================
    // Group: XMP Metadata
    // ========================================================================

    #[test]
    fn test_xmp_metadata_none_for_simple_pdf() {
        let doc = doc_from_text("Hello");
        let metadata =
            crate::extractors::xmp::XmpExtractor::extract(&mut doc.inner.lock().unwrap());
        assert!(metadata.is_ok());
        // Simple generated PDFs may or may not have XMP
    }

    // ========================================================================
    // Group: PDF from Images
    // ========================================================================

    #[test]
    fn test_from_image_bytes() {
        // WasmPdf::from_image_bytes uses JsValue in error path, so test the
        // underlying Rust API directly on non-wasm32 targets.
        use crate::api::Pdf;
        let jpeg_data = create_minimal_jpeg();
        let result = Pdf::from_image_bytes(&jpeg_data);
        assert!(result.is_ok(), "Pdf::from_image_bytes should succeed: {:?}", result.err());
        let pdf = result.unwrap();
        assert!(!pdf.into_bytes().is_empty());
    }

    /// Create a minimal valid 1x1 white JPEG image (known-good bytes).
    fn create_minimal_jpeg() -> Vec<u8> {
        vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00,
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06,
            0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D,
            0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12, 0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D,
            0x1A, 0x1C, 0x1C, 0x20, 0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28,
            0x37, 0x29, 0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32,
            0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01,
            0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x1F, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02,
            0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0xFF, 0xC4, 0x00, 0xB5, 0x10,
            0x00, 0x02, 0x01, 0x03, 0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00,
            0x01, 0x7D, 0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06,
            0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42,
            0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16,
            0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x34, 0x35, 0x36, 0x37,
            0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55,
            0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73,
            0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
            0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5,
            0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA,
            0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6,
            0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA,
            0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFF, 0xDA, 0x00, 0x08,
            0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, 0xFB, 0xD5, 0xDB, 0x20, 0xA8, 0xF9, 0xFF, 0xD9,
        ]
    }

    // ========================================================================
    // Tests for new binding methods (v0.3.18)
    // ========================================================================

    #[test]
    fn test_validate_pdf_a() {
        let mut doc = doc_from_text("Hello World");
        let result = doc.validate_pdf_a("1b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pdf_a_invalid_level() {
        let mut doc = doc_from_text("Hello");
        let result = doc.validate_pdf_a("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_page() {
        // Create a 2-page PDF
        let bytes = make_markdown_pdf("# Page 1\n\n---\n\n# Page 2");
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        let initial_count = doc.page_count().unwrap();
        if initial_count >= 2 {
            assert!(doc.delete_page(0).is_ok());
        }
    }

    #[test]
    fn test_extract_pages() {
        let mut doc = doc_from_text("Extract me");
        let result = doc.extract_pages(vec![0]);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        // Verify the extracted PDF is valid
        let extracted = WasmPdfDocument::new(&bytes, None);
        assert!(extracted.is_ok());
    }

    // ========================================================================
    // Group: v0.3.39 DocumentBuilder — tables + primitives (#393 step 6b)
    // ========================================================================

    #[test]
    fn test_measure_nonzero_for_nonempty_text() {
        let mut b = WasmDocumentBuilder::new();
        let mut p = b.a4_page().unwrap();
        p.font("Helvetica".to_string(), 12.0).unwrap();
        let w = p.measure("Hello");
        assert!(w > 0.0, "measure should return a positive width, got {w}");
    }

    #[test]
    fn test_measure_zero_for_empty_text() {
        let b = WasmDocumentBuilder::new();
        let p = WasmFluentPageBuilder::new_with_size(crate::writer::PageSize::A4);
        let _ = b; // silence unused
        assert_eq!(p.measure(""), 0.0, "empty string should measure 0");
    }

    #[test]
    fn test_remaining_space_starts_at_page_height_minus_top_minus_bottom_margin() {
        let p = WasmFluentPageBuilder::new_with_size(crate::writer::PageSize::Letter);
        // Letter is 612 x 792. Top margin 72, bottom margin 72 → remaining = 792 - 72 - 72 = 648.
        assert!((p.remaining_space() - 648.0).abs() < 0.01);
    }

    #[test]
    fn test_remaining_space_after_at() {
        let mut p = WasmFluentPageBuilder::new_with_size(crate::writer::PageSize::Letter);
        p.at(72.0, 500.0).unwrap();
        // cursor_y = 500, bottom margin = 72 → 428.
        assert!((p.remaining_space() - 428.0).abs() < 0.01);
    }

    #[test]
    fn test_remaining_space_clamped_at_zero() {
        let mut p = WasmFluentPageBuilder::new_with_size(crate::writer::PageSize::Letter);
        p.at(72.0, 50.0).unwrap(); // below bottom margin
        assert_eq!(p.remaining_space(), 0.0);
    }

    #[test]
    fn test_new_page_same_size_resets_cursor() {
        let mut p = WasmFluentPageBuilder::new_with_size(crate::writer::PageSize::Letter);
        p.at(72.0, 100.0).unwrap();
        p.new_page_same_size().unwrap();
        // Should reset to height - 72 = 720.
        assert!((p.remaining_space() - 648.0).abs() < 0.01);
    }

    #[test]
    fn test_text_in_rect_round_trips_through_builder() {
        let mut b = WasmDocumentBuilder::new();
        let mut p = b.letter_page().unwrap();
        p.font("Helvetica".to_string(), 10.0).unwrap();
        p.text_in_rect(72.0, 720.0, 200.0, 100.0, "wrap me".to_string(), 1)
            .unwrap();
        p.done(&mut b).unwrap();
        let bytes = b.build().unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_stroke_rect_and_stroke_line_commit() {
        let mut b = WasmDocumentBuilder::new();
        let mut p = b.letter_page().unwrap();
        p.stroke_rect(50.0, 50.0, 200.0, 100.0, 2.0, 0.5, 0.5, 0.5)
            .unwrap();
        p.stroke_line(50.0, 50.0, 250.0, 50.0, 1.0, 0.2, 0.2, 0.2)
            .unwrap();
        p.done(&mut b).unwrap();
        let bytes = b.build().unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_stroke_rect_dashed_and_stroke_line_dashed_commit() {
        let mut b = WasmDocumentBuilder::new();
        let mut p = b.letter_page().unwrap();
        p.stroke_rect_dashed(50.0, 50.0, 200.0, 100.0, 2.0, 0.0, 0.0, 0.8, vec![3.0, 2.0], 0.0)
            .unwrap();
        p.stroke_line_dashed(50.0, 50.0, 250.0, 50.0, 1.0, 0.8, 0.0, 0.0, vec![5.0, 3.0], 1.0)
            .unwrap();
        p.done(&mut b).unwrap();
        let bytes = b.build().unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        // Dash operator must be present
        assert!(bytes.windows(3).any(|w| w == b" d\n") || bytes.windows(3).any(|w| w == b" d "));
    }

    #[test]
    fn test_buffered_table_replays_to_rust_table() {
        // Exercise the native path: construct a WasmPageOp::BufferedTable
        // directly and commit through commit_page to prove the replay logic
        // is wired. Skips the JsValue deserialisation that requires wasm32.
        let mut b = WasmDocumentBuilder::new();
        let mut p = b.letter_page().unwrap();
        p.font("Helvetica".to_string(), 10.0).unwrap();
        p.at(72.0, 720.0).unwrap();
        p.ops.borrow_mut().push(WasmPageOp::BufferedTable {
            columns: vec![
                ("SKU".into(), Some(100.0), WasmAlign::Left),
                ("Qty".into(), Some(60.0), WasmAlign::Right),
            ],
            rows: vec![
                vec!["A-1".into(), "12".into()],
                vec!["B-2".into(), "3".into()],
            ],
            has_header: true,
        });
        p.done(&mut b).unwrap();
        let bytes = b.build().unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.len() > 512, "table should produce a meaningful PDF");
    }

    #[test]
    fn test_streaming_table_block_replays_rows_to_rust_streaming_table() {
        let mut b = WasmDocumentBuilder::new();
        let mut p = b.letter_page().unwrap();
        p.font("Helvetica".to_string(), 10.0).unwrap();
        p.at(72.0, 720.0).unwrap();
        p.ops.borrow_mut().push(WasmPageOp::StreamingTableBlock {
            config_columns: vec![
                ("SKU".into(), 72.0, WasmAlign::Left),
                ("Item".into(), 200.0, WasmAlign::Left),
                ("Qty".into(), 48.0, WasmAlign::Right),
            ],
            repeat_header: true,
            rows: (0..5)
                .map(|i| {
                    vec![
                        (format!("A-{i}"), 1usize),
                        ("Widget".into(), 1),
                        ((i * 10).to_string(), 1),
                    ]
                })
                .collect(),
            mode: "fixed".to_string(),
            sample_rows: 50,
            min_col_width_pt: 20.0,
            max_col_width_pt: 400.0,
            max_rowspan: 1,
        });
        p.done(&mut b).unwrap();
        let bytes = b.build().unwrap();
        assert!(bytes.starts_with(b"%PDF-"));

        // Round-trip: re-open and extract text; at minimum the headers
        // should be present in the content stream.
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        let text = doc.extract_all_text().unwrap();
        assert!(
            text.contains("SKU") || text.contains("Item") || text.contains("Qty"),
            "streaming-table output should contain at least one header cell, got: {text:?}",
        );
    }

    #[test]
    fn test_streaming_table_bounded_batch_accumulates_and_flushes() {
        // Verify that push_row auto-flushes when batch_size is reached and that
        // the resulting PDF contains all rows regardless of batching.
        let mut b = WasmDocumentBuilder::new();
        let mut p = b.letter_page().unwrap();
        p.font("Helvetica".to_string(), 9.0).unwrap();
        p.at(72.0, 720.0).unwrap();

        let mut st = WasmStreamingTable {
            columns: vec![
                ("ID".into(), 60.0, WasmAlign::Left),
                ("Value".into(), 120.0, WasmAlign::Left),
            ],
            repeat_header: false,
            current_batch: Vec::new(),
            completed_batches: Vec::new(),
            finished: false,
            mode: "fixed".into(),
            sample_rows: 50,
            min_col_width_pt: 20.0,
            max_col_width_pt: 400.0,
            max_rowspan: 1,
            batch_size: 3,
            page_ops: std::rc::Rc::clone(&p.ops),
        };

        // Push 7 rows with batch_size=3: expect 2 full batches + 1 partial.
        for i in 0..7usize {
            st.push_row(vec![i.to_string(), format!("row-{i}")])
                .unwrap();
        }
        assert_eq!(st.batch_count(), 2, "expected 2 completed batches");
        assert_eq!(st.pending_row_count(), 1, "expected 1 pending row");

        st.finish().unwrap();
        p.done(&mut b).unwrap();
        let bytes = b.build().unwrap();
        assert!(bytes.starts_with(b"%PDF-"));

        // Re-open and verify row content survived.
        let mut doc = WasmPdfDocument::new(&bytes, None).unwrap();
        let text = doc.extract_all_text().unwrap();
        assert!(text.contains("row-0"), "first row missing");
        assert!(text.contains("row-6"), "last row missing");
    }

    #[test]
    fn test_align_enum_discriminants() {
        assert_eq!(WasmAlign::Left as i32, 0);
        assert_eq!(WasmAlign::Center as i32, 1);
        assert_eq!(WasmAlign::Right as i32, 2);
        assert_eq!(WasmAlign::from_i32(0), WasmAlign::Left);
        assert_eq!(WasmAlign::from_i32(1), WasmAlign::Center);
        assert_eq!(WasmAlign::from_i32(2), WasmAlign::Right);
        // Out-of-range falls back to Left.
        assert_eq!(WasmAlign::from_i32(99), WasmAlign::Left);
    }

    // Issue #401 regression — WasmDocumentBuilder.to_bytes_encrypted preserves embedded font.
    #[test]
    fn test_to_bytes_encrypted_embedded_font_content_preserved() {
        let font_path = std::path::Path::new("tests/fixtures/fonts/DejaVuSans.ttf");
        if !font_path.exists() {
            return; // skip if fixture missing
        }
        let font_bytes = std::fs::read(font_path).unwrap();
        let mut font = WasmEmbeddedFont::from_bytes(&font_bytes, None).unwrap();

        let mut builder = WasmDocumentBuilder::new();
        builder
            .register_embedded_font("DejaVu".to_string(), &mut font)
            .unwrap();
        let mut page = builder.a4_page().unwrap();
        page.font("DejaVu".to_string(), 12.0).unwrap();
        page.at(72.0, 720.0).unwrap();
        page.text("Hello embedded font".to_string()).unwrap();
        page.done(&mut builder).unwrap();

        let bytes = builder.to_bytes_encrypted("u", "o").unwrap();
        assert!(
            bytes.len() > 15_000,
            "issue #401: WasmDocumentBuilder.to_bytes_encrypted embedded-font result \
             ({} B) is too small; font sub-objects likely missing",
            bytes.len()
        );
        let has_encrypt = bytes.windows(8).any(|w| w == b"/Encrypt");
        assert!(has_encrypt, "encrypted PDF must contain /Encrypt");
    }

    // ========================================================================
    // Group: saveWithOptions
    // ========================================================================

    #[test]
    fn test_save_with_options_compress_true() {
        let mut doc = doc_from_text("Compress test");
        let bytes = doc.save_with_options_js(Some(true), None, None).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_save_with_options_compress_false() {
        let mut doc = doc_from_text("No compress test");
        let bytes = doc.save_with_options_js(Some(false), None, None).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_save_with_options_gc_true() {
        let mut doc = doc_from_text("GC test");
        let bytes = doc.save_with_options_js(None, Some(true), None).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_save_with_options_gc_false() {
        let mut doc = doc_from_text("No GC test");
        let bytes = doc.save_with_options_js(None, Some(false), None).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_save_with_options_all_defaults() {
        let mut doc = doc_from_text("All defaults");
        let bytes = doc.save_with_options_js(None, None, None).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_save_with_options_compress_smaller_or_equal() {
        let mut doc1 = doc_from_text("Compress size comparison test for WASM");
        let mut doc2 = doc_from_text("Compress size comparison test for WASM");
        let uncompressed = doc1
            .save_with_options_js(Some(false), Some(false), None)
            .unwrap();
        let compressed = doc2
            .save_with_options_js(Some(true), Some(false), None)
            .unwrap();
        assert!(
            compressed.len() <= uncompressed.len(),
            "compressed ({}) should be <= uncompressed ({})",
            compressed.len(),
            uncompressed.len()
        );
    }

    #[test]
    fn test_save_with_options_round_trips() {
        let mut doc = doc_from_text("Round-trip via saveWithOptions");
        let bytes = doc
            .save_with_options_js(Some(true), Some(true), Some(false))
            .unwrap();
        // Load the saved bytes back into a new document
        let mut doc2 = WasmPdfDocument::new(&bytes, None).unwrap();
        let pages = doc2.page_count().unwrap();
        assert!(pages >= 1);
    }
}
