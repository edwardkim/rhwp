//! Comprehensive auto-extraction — public vocabulary (v0.3.51, #517).
//!
//! The configured-once [`AutoExtractor`] returns *all* recoverable text
//! from a PDF — native text **and** text embedded in images,
//! image-tables and figures — decided per-page/per-region, with a
//! machine-readable [`ReasonCode`] for every degraded result.
//!
//! This module is the **dependency root** and is *pure PDF inspection*:
//! it carries **no `#[cfg(feature = "ocr")]`** gate and is fully
//! testable on the no-`ocr` build (00-common-foundation §5/§9). The
//! classification signal model (T2/T3) and the [`AutoExtractor`]
//! pipeline (T4–T8) build on these types.
//!
//! Design contracts (api-design.md §3, README "Locked decisions"):
//! - **Strictly additive** — existing `extract_text`/`extract_spans`/…
//!   are byte-identical; this is a *new* surface.
//! - **Single mode knob** [`ExtractMode`] (Tika/unstructured pattern),
//!   not boolean soup.
//! - **Typed reason per region** — never a bare empty string; degraded
//!   results always say *why* (the #1 cross-tool user pain).
//! - **Graceful, never fail-loud** — OCR unavailable → warn + native
//!   fallback + [`ReasonCode::OcrRequestedButUnavailable`] (best-effort
//!   extraction degrades gracefully; only security ops fail-closed).
//! - All public types are `#[non_exhaustive]` + `serde` (the JSON
//!   C-ABI boundary, matching the shipped split-by-bookmarks idiom) so
//!   later tiers (T2/T3) are additive and non-breaking.

use serde::{Deserialize, Serialize};

/// How [`AutoExtractor`](crate::extractors::auto) decides text-vs-OCR.
///
/// One knob collapses off/auto/force (Tika `OCR_STRATEGY` /
/// unstructured `strategy` — the cleanest mode model; the multi-boolean
/// designs are the ones that caused upstream bugs). `#[non_exhaustive]`
/// → a future tier is a non-breaking addition (T2/T3 are deliberately
/// *not* present — `tier-model-strategy.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExtractMode {
    /// Native text layer only — never OCR (≈ today's `extract_text`).
    TextOnly,
    /// Per-region heuristic: text-layer vs OCR vs image-table recovery.
    /// The default (README locked decision 2).
    #[default]
    Auto,
    /// OCR everything, ignore any native text layer (the #460
    /// "suspect text layer" forced-OCR escape hatch).
    ForceOcr,
}

/// OCR recognition language (selects the per-language model + dict;
/// the detector is shared and script-agnostic). Default
/// [`English`](OcrLanguage::English). Only languages with an upstream
/// PaddleOCR ONNX model are listed — Hebrew is intentionally absent
/// (PaddleOCR publishes a Hebrew dict but **no** recognition model;
/// the loader is ready the instant such a pair is provided, but none
/// exists to fetch — a provisioning limit, not a code defect).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OcrLanguage {
    /// English / basic Latin (the default; legacy `rec.onnx`/`en_dict.txt`).
    #[default]
    English,
    /// Simplified Chinese (CJK).
    Chinese,
    /// Traditional Chinese.
    ChineseTraditional,
    /// Japanese.
    Japanese,
    /// Korean.
    Korean,
    /// Arabic (RTL).
    Arabic,
    /// Cyrillic (Russian and related).
    Cyrillic,
    /// Latin-script European languages (accented).
    Latin,
    /// Devanagari (Hindi/Marathi/…).
    Devanagari,
    /// Tamil.
    Tamil,
    /// Telugu.
    Telugu,
    /// Kannada.
    Kannada,
}

/// Resolved download/cache spec for one [`OcrLanguage`].
#[derive(Debug, Clone)]
pub struct OcrModelSpec {
    /// Recognition model filename in the cache dir.
    pub rec_file: String,
    /// Character dictionary filename in the cache dir.
    pub dict_file: String,
    /// Recognition model source URL.
    pub rec_url: String,
    /// Dictionary source URL.
    pub dict_url: String,
}

impl OcrLanguage {
    /// Shared, script-agnostic detector (PP-OCRv4) — one per cache dir.
    pub const DET_URL: &'static str =
        "https://huggingface.co/deepghs/paddleocr/resolve/main/det/ch_PP-OCRv4_det/model.onnx";

    /// Every supported language — for "provision everything" (the
    /// Docker/CI build case): `prefetch_models(OcrLanguage::ALL)` or
    /// `pdf-oxide models prefetch --all`. Hebrew is absent (no upstream
    /// PaddleOCR recognition model).
    pub const ALL: &'static [OcrLanguage] = &[
        OcrLanguage::English,
        OcrLanguage::Chinese,
        OcrLanguage::ChineseTraditional,
        OcrLanguage::Japanese,
        OcrLanguage::Korean,
        OcrLanguage::Arabic,
        OcrLanguage::Cyrillic,
        OcrLanguage::Latin,
        OcrLanguage::Devanagari,
        OcrLanguage::Tamil,
        OcrLanguage::Telugu,
        OcrLanguage::Kannada,
    ];

    /// Parse a free-form language code / alias (CLI, `ocr_languages`,
    /// auto-detection). Unknown → `None` (caller decides the fallback).
    #[must_use]
    pub fn from_code(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "" | "en" | "eng" | "english" | "latin-en" => Self::English,
            "zh" | "ch" | "chi" | "chinese" | "cjk" | "zh-cn" | "zh_hans" => Self::Chinese,
            "zh-tw" | "zh_hant" | "chinese_cht" | "cht" | "traditional" => Self::ChineseTraditional,
            "ja" | "jpn" | "japanese" | "japan" => Self::Japanese,
            "ko" | "kor" | "korean" => Self::Korean,
            "ar" | "ara" | "arabic" => Self::Arabic,
            "ru" | "rus" | "russian" | "cyrillic" | "uk" | "be" | "bg" | "sr" => Self::Cyrillic,
            "lat" | "latin" | "fr" | "de" | "es" | "it" | "pt" => Self::Latin,
            "hi" | "mr" | "ne" | "devanagari" => Self::Devanagari,
            "ta" | "tam" | "tamil" => Self::Tamil,
            "te" | "tel" | "telugu" => Self::Telugu,
            "kn" | "kan" | "ka" | "kannada" => Self::Kannada,
            _ => return None,
        })
    }

    /// PaddleOCR language code (also the cache-file stem for
    /// non-English).
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "english",
            Self::Chinese => "chinese",
            Self::ChineseTraditional => "chinese_cht",
            Self::Japanese => "japan",
            Self::Korean => "korean",
            Self::Arabic => "arabic",
            Self::Cyrillic => "cyrillic",
            Self::Latin => "latin",
            Self::Devanagari => "devanagari",
            Self::Tamil => "ta",
            Self::Telugu => "te",
            Self::Kannada => "ka",
        }
    }

    /// Cache filenames + source URLs for this language. English &
    /// Simplified-Chinese use the `monkt/paddleocr-onnx` PP-OCRv5
    /// packs (English keeps the legacy `rec.onnx`/`en_dict.txt`
    /// names); every other language uses the broader
    /// `deepghs/paddleocr` PP-OCRv3 rec ONNX + the PaddleOCR upstream
    /// dictionary (`rec_<code>.onnx` / `<code>_dict.txt`).
    #[must_use]
    pub fn spec(self) -> OcrModelSpec {
        let code = self.code();
        match self {
            Self::English => OcrModelSpec {
                rec_file: "rec.onnx".into(),
                dict_file: "en_dict.txt".into(),
                rec_url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/languages/english/rec.onnx".into(),
                dict_url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/languages/english/dict.txt".into(),
            },
            Self::Chinese => OcrModelSpec {
                rec_file: "rec_chinese.onnx".into(),
                dict_file: "chinese_dict.txt".into(),
                rec_url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/languages/chinese/rec.onnx".into(),
                dict_url: "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/languages/chinese/dict.txt".into(),
            },
            _ => OcrModelSpec {
                rec_file: format!("rec_{code}.onnx"),
                dict_file: format!("{code}_dict.txt"),
                rec_url: format!(
                    "https://huggingface.co/deepghs/paddleocr/resolve/main/rec/{code}_PP-OCRv3_rec/model.onnx"
                ),
                dict_url: format!(
                    "https://raw.githubusercontent.com/PaddlePaddle/PaddleOCR/main/ppocr/utils/dict/{code}_dict.txt"
                ),
            },
        }
    }
}

/// Options for [`AutoExtractor`](crate::extractors::auto).
///
/// Plain struct + `Default` + `#[non_exhaustive]` (house style:
/// `RedactionOptions`/`SplitByBookmarksOptions`); construct with
/// struct-update (`..Default::default()`), a preset
/// ([`fast`](Self::fast)/[`balanced`](Self::balanced)/
/// [`high_fidelity`](Self::high_fidelity)), or the
/// [`builder`](Self::builder) (mirrors the in-repo `OcrConfigBuilder`).
/// `serde` is the JSON config wire at the C-ABI.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
#[non_exhaustive]
pub struct AutoExtractOptions {
    /// Text-vs-OCR mode. Default [`ExtractMode::Auto`].
    pub mode: ExtractMode,
    /// Reconstruct image-tables into a structured [`TableData`] via the
    /// existing spatial detector over OCR spans (default `true`).
    pub reconstruct_image_tables: bool,
    /// Emit positioned `Figure`/`Table` placeholders in the text flow
    /// so reading order is never silently corrupted (default `true`).
    pub emit_placeholders: bool,
    /// OCR language hints (engine dict selection). Empty = auto/default.
    pub ocr_languages: Vec<String>,
    /// Auto-decision confidence threshold (`None` = preset/calibrated).
    pub min_text_confidence: Option<f32>,
    /// Image-table reconstruction confidence threshold (`None` =
    /// preset/calibrated).
    pub table_confidence: Option<f32>,
    /// Force OCR on these **0-based** page indices regardless of `mode`
    /// (additive on `Auto`; does not change [`ExtractMode`]). Empty =
    /// honour `mode` for every page.
    pub force_ocr_pages: Vec<usize>,
}

impl Default for AutoExtractOptions {
    fn default() -> Self {
        // `balanced()` is the documented default (README locked dec. 2;
        // api-design.md §3).
        Self::balanced()
    }
}

impl AutoExtractOptions {
    /// Text-layer biased, no layout/table work — the cheapest path.
    #[must_use]
    pub fn fast() -> Self {
        Self {
            mode: ExtractMode::TextOnly,
            reconstruct_image_tables: false,
            emit_placeholders: true,
            ocr_languages: Vec::new(),
            min_text_confidence: None,
            table_confidence: None,
            force_ocr_pages: Vec::new(),
        }
    }

    /// The default — `Auto` per-region routing with image-table
    /// reconstruction (calibrated thresholds).
    #[must_use]
    pub fn balanced() -> Self {
        Self {
            mode: ExtractMode::Auto,
            reconstruct_image_tables: true,
            emit_placeholders: true,
            ocr_languages: Vec::new(),
            min_text_confidence: None,
            table_confidence: None,
            force_ocr_pages: Vec::new(),
        }
    }

    /// Aggressive OCR + image-table recovery (still local-CPU T1; no
    /// VLM/cloud — `tier-model-strategy.md`).
    #[must_use]
    pub fn high_fidelity() -> Self {
        Self {
            mode: ExtractMode::Auto,
            reconstruct_image_tables: true,
            emit_placeholders: true,
            ocr_languages: Vec::new(),
            // Lower bars → escalate to OCR / table recovery sooner.
            min_text_confidence: Some(0.55),
            table_confidence: Some(0.45),
            force_ocr_pages: Vec::new(),
        }
    }

    /// Fluent builder (mirrors the in-repo `OcrConfigBuilder`).
    #[must_use]
    pub fn builder() -> AutoExtractOptionsBuilder {
        AutoExtractOptionsBuilder::new()
    }
}

/// Fluent builder for [`AutoExtractOptions`] — same shape as the
/// canonical in-repo `OcrConfigBuilder` (`mut self -> Self`, validating
/// setters, terminal `build`).
#[derive(Debug, Clone, Default)]
pub struct AutoExtractOptionsBuilder {
    opts: AutoExtractOptions,
}

impl AutoExtractOptionsBuilder {
    /// Start from the [`balanced`](AutoExtractOptions::balanced) preset.
    #[must_use]
    pub fn new() -> Self {
        Self {
            opts: AutoExtractOptions::balanced(),
        }
    }

    /// Set the text-vs-OCR [`ExtractMode`].
    #[must_use]
    pub fn mode(mut self, mode: ExtractMode) -> Self {
        self.opts.mode = mode;
        self
    }

    /// Toggle image-table reconstruction.
    #[must_use]
    pub fn reconstruct_image_tables(mut self, yes: bool) -> Self {
        self.opts.reconstruct_image_tables = yes;
        self
    }

    /// Toggle figure/table placeholders in the text flow.
    #[must_use]
    pub fn emit_placeholders(mut self, yes: bool) -> Self {
        self.opts.emit_placeholders = yes;
        self
    }

    /// Set OCR language hints.
    #[must_use]
    pub fn ocr_languages<I, S>(mut self, langs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.opts.ocr_languages = langs.into_iter().map(Into::into).collect();
        self
    }

    /// Auto-decision confidence threshold (clamped to `0.0..=1.0`).
    #[must_use]
    pub fn min_text_confidence(mut self, c: f32) -> Self {
        self.opts.min_text_confidence = Some(c.clamp(0.0, 1.0));
        self
    }

    /// Image-table confidence threshold (clamped to `0.0..=1.0`).
    #[must_use]
    pub fn table_confidence(mut self, c: f32) -> Self {
        self.opts.table_confidence = Some(c.clamp(0.0, 1.0));
        self
    }

    /// Force OCR on these 0-based page indices (additive on `mode`).
    #[must_use]
    pub fn force_ocr_pages<I: IntoIterator<Item = usize>>(mut self, pages: I) -> Self {
        self.opts.force_ocr_pages = pages.into_iter().collect();
        self
    }

    /// Finalise.
    #[must_use]
    pub fn build(self) -> AutoExtractOptions {
        self.opts
    }
}

/// Per-page classification outcome (T0/T0.5 — pure inspection).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PageKind {
    /// Usable native text dominates — extract the text layer.
    TextLayer,
    /// Image-dominated, no/garbled text — OCR the page.
    Scanned,
    /// Native text **and** image regions containing text — hybrid
    /// (native + region OCR).
    ImageText,
    /// Heterogeneous within the page (text + image-table/figure).
    Mixed,
    /// Blank/near-empty — neither extract nor OCR; not an error.
    Empty,
}

/// Machine-readable *why* for a region/page result — **the #1
/// cross-tool user-pain fix**. A non-degraded result is
/// [`Ok`](ReasonCode::Ok); any degraded one **must** name the cause.
/// `#[non_exhaustive]`, frozen `snake_case` wire tokens (append-only —
/// never renumber/rename, the `PadesLevel` lesson).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReasonCode {
    /// Extracted cleanly.
    Ok,
    /// Native text present and high-confidence.
    NativeTextHighConfidence,
    /// No text layer on the page at all.
    NoTextLayerPresent,
    /// Text layer present but below the usable-quality threshold.
    TextLayerBelowThreshold,
    /// Glyphs without usable ToUnicode/`(cid:NN)`/garbled mapping.
    GlyphMappingMissing,
    /// Encrypted and not authorised to extract.
    EncryptedNoExtractPermission,
    /// An image-table was reconstructed into [`TableData`].
    ImageTableReconstructed,
    /// A table region was detected but structure could not be
    /// recovered (graceful — heuristic fallback used, not fail-loud).
    ImageTableNoStructure,
    /// A chart/figure was detected; its internal data is **not**
    /// transcribed (honest scope boundary — placeholder emitted).
    ChartNotTranscribed,
    /// OCR was needed but unavailable (feature off / models absent /
    /// `mode = TextOnly`) → fell back to native text + warned.
    OcrRequestedButUnavailable,
    /// OCR ran but confidence was low → native used/merged.
    OcrLowConfidenceFallback,
    /// Region/page yielded no recoverable content.
    Empty,
}

/// Where a region's text came from. `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExtractSource {
    /// pdf_oxide native text-layer extraction.
    NativeText,
    /// OCR (region or full page).
    Ocr,
    /// Structured image-table reconstruction.
    ImageTableRecovery,
    /// Degraded fallback to native after an unavailable/low-confidence
    /// higher tier (paired with a non-`Ok` [`ReasonCode`]).
    Fallback,
}

/// Region classification. `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RegionKind {
    /// Body / inline text.
    Text,
    /// Heading (from structure tree or font-size heuristic).
    Heading,
    /// Tabular region (native or reconstructed image-table).
    Table,
    /// Figure/illustration (text recovered via region OCR if any).
    Figure,
    /// Chart/plot — internal data not transcribed (honest boundary).
    Chart,
}

/// A 4-point quadrilateral bounding box in PDF points (top-left,
/// top-right, bottom-right, bottom-left). A quad (not an AABB) so
/// skewed/rotated regions survive (cases I/J — Textract/Azure
/// convention). `serde` for the JSON C-ABI boundary.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Quad {
    /// `[tl, tr, br, bl]`, each `[x, y]` in PDF points.
    pub points: [[f32; 2]; 4],
}

impl Quad {
    /// Axis-aligned quad from `(x, y, w, h)` (PDF points, origin
    /// bottom-left).
    #[must_use]
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            points: [[x, y + h], [x + w, y + h], [x + w, y], [x, y]],
        }
    }
}

/// Structured table payload (serde-friendly — what crosses the JSON
/// C-ABI). Decoupled from the internal
/// `structure::table_extractor::Table` on purpose: T7 maps the spatial
/// detector's output into this; consumers get cells + markdown.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct TableData {
    /// Row-major cell text.
    pub rows: Vec<Vec<String>>,
    /// First row is a header.
    pub has_header: bool,
    /// GitHub-flavoured Markdown rendering of the table.
    pub markdown: String,
}

/// One classified, extracted region of a page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Region {
    /// 4-point bbox in PDF points.
    pub bbox: Quad,
    /// Region classification.
    pub kind: RegionKind,
    /// Recovered text (`""` if none — e.g. an un-transcribed chart;
    /// the bbox + reason still locate it for reading order).
    pub text: String,
    /// Reconstructed table (only for [`RegionKind::Table`]).
    pub table: Option<TableData>,
    /// Confidence `0.0..=1.0`, always present.
    pub confidence: f32,
    /// Where the text came from.
    pub source: ExtractSource,
    /// Why this source/outcome (typed; never opaque).
    pub reason: ReasonCode,
}

/// Document-level rollup status. `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExtractionStatus {
    /// Every region extracted cleanly.
    Complete,
    /// Some regions degraded (see per-region [`ReasonCode`]) but text
    /// was recovered — *not* an error (Docling `PARTIAL_SUCCESS`).
    PartialSuccess,
    /// No usable text recovered anywhere.
    NoTextRecovered,
}

/// Per-page extraction result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct PageExtraction {
    /// 0-based page index.
    pub page: usize,
    /// Per-page classification.
    pub kind: PageKind,
    /// Assembled, reading-ordered text for the page.
    pub text: String,
    /// Per-region results (bbox + reason always present, even when
    /// `text` is empty — pain #2/#8).
    pub regions: Vec<Region>,
    /// Page-level confidence `0.0..=1.0`.
    pub confidence: f32,
    /// Page-level rollup reason.
    pub reason: ReasonCode,
    /// Whether OCR actually ran for this page.
    pub ocr_used: bool,
    /// Page-level status.
    pub status: ExtractionStatus,
}

/// Whole-document extraction result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct DocumentExtraction {
    /// Per-page results (per-page decision — never one forced doc mode;
    /// case Q).
    pub pages: Vec<PageExtraction>,
    /// Document rollup status.
    pub status: ExtractionStatus,
    /// 0-based page indices a cheap preflight says need OCR.
    pub pages_needing_ocr: Vec<usize>,
}

/// Lightweight document classification (the cheap `classify_document`
/// preflight — no OCR, no rasterisation).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct DocumentClassification {
    /// Per-page kinds.
    pub pages: Vec<PageKind>,
    /// 0-based page indices needing OCR.
    pub pages_needing_ocr: Vec<usize>,
    /// Aggregate `mostly_text` / `mostly_scanned` / `mixed`.
    pub summary: DocumentSummary,
}

/// Aggregate document character. `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DocumentSummary {
    /// Predominantly born-digital text.
    MostlyText,
    /// Predominantly scanned/image.
    MostlyScanned,
    /// Heterogeneous (case Q).
    Mixed,
    /// No meaningful content.
    Empty,
}

// ───────────────────────── classifier (T2/T2.5/T3) ─────────────────────────
//
// Pure inspection of pdf_oxide *internals* — never the flattened output
// string (00-common-foundation §9). Lowest-level decision logic takes
// **injected primitives** so it is unit-testable without a
// `PdfDocument` (00-common-foundation §8 — the `sanitize_catalog`
// injected-resolver precedent). `PdfDocument::classify_page` /
// `classify_document` (in `document.rs`) gather the internal signals and
// delegate here. No `#[cfg(feature = "ocr")]`.

/// Dominant raster codec on a page — a strong scan-vs-pictorial prior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageCodecClass {
    /// No raster images.
    None,
    /// CCITT Group 3/4 fax — 1-bit, almost always a scan.
    Ccitt,
    /// JBIG2 — 1-bit, almost always a scan.
    Jbig2,
    /// DCT/JPEG — photo or colour scan.
    Dct,
    /// Flate/other raster.
    Other,
}

/// Document-level scanner-vs-authoring prior (case P). Weak, never
/// decisive — only a tie-break nudge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ProducerPrior {
    /// Producer/Creator looks like scanner/OCR software.
    Scanner,
    /// Producer/Creator looks like authoring software.
    Authoring,
    /// Unknown / no Info.
    Unknown,
}

/// Per-page signals gathered from pdf_oxide internals (T2). All ratios
/// are `0.0..=1.0`. Serde so it can ride the explainable result.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct PageSignals {
    /// Non-artifact glyph count from the native text layer.
    pub text_glyph_count: usize,
    /// Σ text-box area ∩ page ÷ page area.
    pub text_area_ratio: f32,
    /// Σ CTM-transformed image-box area ∩ page ÷ page area (clamped;
    /// summed not max'd → multi-strip scans, case J, are caught).
    pub image_area_ratio: f32,
    /// Dominant raster codec.
    pub codec: ImageCodecClass,
    /// Tr-mode-3 (invisible) glyphs ÷ all glyphs — the OCR-sidecar /
    /// painted-over signal (cases C/C2).
    pub invisible_text_ratio: f32,
    /// (U+FFFD + control + replacement) ÷ chars — `(cid:NN)`/garbled.
    pub garbled_ratio: f32,
    /// Short-fragmented-"word" ratio (broken CMaps split every glyph).
    pub fragmented_word_ratio: f32,
    /// Consecutive/duplicated-token run ratio — 2-column born-digital
    /// scramble (the T0.5 addition, research §3a).
    pub consecutive_repeat_ratio: f32,
    /// Path-ish ops ÷ all content ops — outlined/vectorised text
    /// (case F).
    pub vector_path_density: f32,
    /// How many painted paths the page carries.
    ///
    /// The density beside it is `paths / (paths + glyphs + images)`, which
    /// saturates at 1.0 as soon as the page has no glyphs and no images —
    /// four paths and four thousand are indistinguishable by that ratio
    /// alone. Outlined text is the case the ratio exists to catch, and it is
    /// a case with *many* paths, so the count is what separates it from a
    /// diagram, a logo, or a pattern swatch.
    pub vector_path_count: usize,
    /// `MarkInfo.marked && !suspects` — high-precision digital prior.
    pub has_reliable_structure: bool,
    /// Scanner-vs-authoring producer prior (weak).
    pub producer_prior: ProducerPrior,
    /// No text, no significant image, no paths.
    pub page_is_empty: bool,
}

/// Cheap per-page classification (the `classify_page` preflight — no
/// OCR, no rasterisation): kind + confidence + typed reason + the raw
/// signals (explainable; our differentiator over downstream wrappers).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct PageClassification {
    /// 0-based page index.
    pub page: usize,
    /// Decided page kind.
    pub kind: PageKind,
    /// Confidence `0.0..=1.0`.
    pub confidence: f32,
    /// Typed reason for the decision.
    pub reason: ReasonCode,
    /// Raw signals (explainability).
    pub signals: PageSignals,
}

/// Whether `text` is CJK/Hangul-dominant: a majority of its non-whitespace
/// characters fall in the Han, Hiragana, Katakana, or Hangul Unicode
/// blocks. These scripts don't use inter-word spaces, so glyph-adjacency
/// word clustering naturally produces short (often 1-2 character) tokens —
/// a `frag`/`avg_word_len` signal calibrated for space-separated Latin
/// text misreads that as fragmentation. Used to skip the Latin-specific
/// checks in [`text_quality_gate`] for such text; script-agnostic signals
/// (garbled/repeat ratio) still apply normally.
#[must_use]
pub fn is_cjk_dominant_text(text: &str) -> bool {
    let mut total = 0usize;
    let mut cjk = 0usize;
    for c in text.chars() {
        if c.is_whitespace() {
            continue;
        }
        total += 1;
        let cp = c as u32;
        if (0x4E00..=0x9FFF).contains(&cp)   // CJK Unified Ideographs
            || (0x3400..=0x4DBF).contains(&cp) // CJK Extension A
            || (0x3040..=0x309F).contains(&cp) // Hiragana
            || (0x30A0..=0x30FF).contains(&cp) // Katakana
            || (0xAC00..=0xD7A3).contains(&cp)
        // Hangul Syllables
        {
            cjk += 1;
        }
    }
    total > 0 && (cjk as f32 / total as f32) > 0.5
}

/// T0.5 enriched text-quality gate (research §3a).
/// Pure: operates on the native-extracted text only. Returns the
/// degrading [`ReasonCode`] if the "born-digital" text is unusable, or
/// `None` if it is good.
///
/// Signals (thresholds are conservative defaults; presets/calibration
/// own tuning — we encode the signals, not 16 user knobs):
/// - U+FFFD / replacement / control ratio (`(cid:NN)`/garbled);
/// - **critical** short-fragmented-word ratio — a hard trigger that
///   overrides everything else;
/// - **consecutive-repeat / column-scramble** ratio;
/// - average word length & alphanumeric ratio;
/// - Unicode-block-aware script validity (no ASCII bias — case H).
#[must_use]
pub fn text_quality_gate(text: &str) -> Option<ReasonCode> {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    if total < 16 {
        // Too little to judge; let the coverage cascade decide.
        return None;
    }
    // Half-open `..` ranges are intentional: HT(0x09) LF(0x0A) VT(0x0B)
    // FF(0x0C) CR(0x0D) and SP(0x20) are legitimate whitespace and are
    // deliberately NOT counted as garbled (the range stops *before*
    // 0x09 and *before* 0x20) — #519 review.
    let bad = chars
        .iter()
        .filter(|&&c| {
            c == '\u{FFFD}'
                || ('\u{0}'..'\u{9}').contains(&c)
                || ('\u{E}'..'\u{20}').contains(&c)
                || ('\u{E000}'..='\u{F8FF}').contains(&c) // private-use (notdef/tofu)
        })
        .count();
    let garbled_ratio = bad as f32 / total as f32;
    if garbled_ratio > 0.20 {
        return Some(ReasonCode::GlyphMappingMissing);
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() >= 8 {
        let frag =
            words.iter().filter(|w| w.chars().count() <= 2).count() as f32 / words.len() as f32;
        // CJK/Hangul text has no inter-word spaces, so glyph-adjacency
        // clustering naturally produces short tokens — `frag` and
        // `avg_word_len` below are calibrated for space-separated Latin
        // text and would otherwise misread ordinary dense CJK prose as
        // fragmented. The repeat-ratio check (script-agnostic) still
        // applies normally.
        let cjk_dominant = is_cjk_dominant_text(text);
        // Critical hard-trigger (broken CMap splitting every glyph).
        if frag > 0.80 && !cjk_dominant {
            return Some(ReasonCode::GlyphMappingMissing);
        }
        // Consecutive-repeat / 2-column scramble: long runs of the
        // same token, or duplicated adjacent tokens.
        let mut repeats = 0usize;
        for w in words.windows(2) {
            if w[0] == w[1] {
                repeats += 1;
            }
        }
        let repeat_ratio = repeats as f32 / words.len() as f32;
        let avg_word_len =
            words.iter().map(|w| w.chars().count()).sum::<usize>() as f32 / words.len() as f32;
        if repeat_ratio > 0.30 || (frag > 0.55 && avg_word_len < 2.5 && !cjk_dominant) {
            return Some(ReasonCode::TextLayerBelowThreshold);
        }
    }

    // Unicode-block-aware validity: a page dominated by control /
    // replacement / private-use is unusable regardless of script.
    let alnum = chars.iter().filter(|c| c.is_alphanumeric()).count();
    if (alnum as f32 / total as f32) < 0.20 {
        return Some(ReasonCode::TextLayerBelowThreshold);
    }
    None
}

/// T3 decision cascade — pure, from gathered [`PageSignals`] + config.
/// Deterministic ordered guards then a coverage/density verdict (KISS:
/// a rule cascade, not ML). Returns `(kind, confidence, reason)`.
#[must_use]
pub fn classify_from_signals(
    s: &PageSignals,
    cfg: &AutoExtractOptions,
) -> (PageKind, f32, ReasonCode) {
    // Calibrated defaults (research §3/§4); presets/cfg override.
    let min_text_conf = cfg.min_text_confidence.unwrap_or(0.70);
    let scan_cover_min = 0.80_f32;
    let sparse_text_max = 0.10_f32;
    let min_glyphs = 24_usize;

    if s.page_is_empty {
        return (PageKind::Empty, 0.99, ReasonCode::Empty);
    }

    let usable_text = s.text_glyph_count >= min_glyphs
        && s.garbled_ratio <= 0.20
        && s.fragmented_word_ratio <= 0.80;

    // Outlined/vectorised text (case F): paths dominate, ~no text/raster.
    //
    // The path COUNT is load-bearing, not only the density. Text converted to
    // outlines produces one or more paths per glyph, so a page of it carries
    // paths in the hundreds or thousands; `min_glyphs` is already the bar for
    // "enough characters to be text", and outlined text cannot clear that bar
    // with fewer paths than glyphs it stands in for. Without the count a page
    // holding four vector drawings and nothing else scores a density of 1.0 —
    // the ratio saturates when there are no glyphs and no images to divide by
    // — and a diagram, a logo or a pattern swatch is reported as a scan that
    // OCR will recover. It is not a scan, there is no raster in it, and OCR
    // returns nothing from it.
    if !usable_text
        && s.vector_path_density > 0.60
        && s.vector_path_count >= min_glyphs
        && s.image_area_ratio < 0.20
    {
        return (PageKind::Scanned, 0.80, ReasonCode::NoTextLayerPresent);
    }

    // Scan dominates the page.
    if s.image_area_ratio >= scan_cover_min {
        // OCR sidecar already present and usable (cases C/C2) → keep it.
        if usable_text && s.invisible_text_ratio >= 0.50 {
            return (PageKind::TextLayer, 0.85, ReasonCode::NativeTextHighConfidence);
        }
        // Sparse text over a scan (case G) — the headline fix: text
        // *presence* is not enough; coverage must be relative.
        if s.text_area_ratio < sparse_text_max || !usable_text {
            let conf = if s.codec == ImageCodecClass::Ccitt || s.codec == ImageCodecClass::Jbig2 {
                0.95
            } else {
                0.85
            };
            // A full-bleed background image with a real, usable text
            // layer (a slide headline, a deck cover) is not "no text
            // layer" — the text is there, mapped correctly, and
            // extractable; it just doesn't cover enough of the page to
            // outweigh the scan-dominant image coverage. Report the
            // honest reason (coverage too low) rather than claiming no
            // text layer exists at all, which would tell a caller who
            // inspects `reason` there is nothing to extract.
            let reason = if usable_text {
                ReasonCode::TextLayerBelowThreshold
            } else {
                ReasonCode::NoTextLayerPresent
            };
            return (PageKind::Scanned, conf, reason);
        }
    }

    // Genuine hybrid: real text AND sub-page image region(s) (cases D/S).
    if usable_text && s.image_area_ratio > 0.05 && s.image_area_ratio < scan_cover_min {
        return (PageKind::ImageText, 0.75, ReasonCode::Ok);
    }

    // Usable native text dominates (case A, good sidecar).
    if usable_text {
        let mut conf = min_text_conf.max(0.80);
        if s.has_reliable_structure {
            conf = (conf + 0.10).min(0.99);
        }
        return (PageKind::TextLayer, conf, ReasonCode::NativeTextHighConfidence);
    }

    // Text present but below the high-confidence bar (< min_glyphs).
    if s.text_glyph_count > 0 {
        // Distinguish "few but CLEAN glyphs" from "garbled glyphs".
        // A short page of well-mapped, image-free text (e.g. a cover
        // line, a stub, a placeholder) is a *text* page — not a scan.
        // Only genuinely garbled glyphs (broken CMap/CID) or text
        // sitting over raster route to OCR. Without this guard a tiny
        // valid text PDF was misclassified `Scanned` and wrongly
        // listed in `pages_needing_ocr` (#519 cross-binding smoke).
        let clean = s.garbled_ratio <= 0.20 && s.fragmented_word_ratio <= 0.80;
        if clean && s.image_area_ratio < 0.05 {
            return (PageKind::TextLayer, 0.60, ReasonCode::NativeTextHighConfidence);
        }
        return (PageKind::Scanned, 0.80, ReasonCode::GlyphMappingMissing);
    }

    // Nothing usable, some raster → OCR; tie-break with producer prior.
    //
    // "Some raster" was the stated premise and was never tested. A page with
    // no text and no image — vector artwork, a chart, a rule-only frame —
    // reached here and was reported as a scan, so a caller was told OCR would
    // recover text from a page holding nothing OCR can read. `Empty` is the
    // honest answer: its contract is "neither extract nor OCR; not an error",
    // which is exactly the instruction such a page warrants. The page need not
    // be visually blank for that to hold — `PageKind` classifies what text
    // extraction should do, not whether the page has ink.
    //
    // The test keys on the CODEC, not the measured area: an image whose
    // placement rectangle cannot be resolved contributes 0 to
    // `image_area_ratio` while still being a raster the page draws, and gating
    // on area alone would report a genuine scan as needing no OCR.
    if s.codec == ImageCodecClass::None && s.image_area_ratio <= 0.0 {
        return (PageKind::Empty, 0.80, ReasonCode::Empty);
    }
    let conf = match s.producer_prior {
        ProducerPrior::Scanner => 0.85,
        _ => 0.70,
    };
    (PageKind::Scanned, conf, ReasonCode::NoTextLayerPresent)
}

/// Roll per-page kinds into a [`DocumentSummary`] (case Q — never a
/// forced single doc mode; this is an *aggregate* only).
#[must_use]
pub fn summarise(pages: &[PageKind]) -> DocumentSummary {
    if pages.is_empty() || pages.iter().all(|k| *k == PageKind::Empty) {
        return DocumentSummary::Empty;
    }
    let non_empty: Vec<&PageKind> = pages.iter().filter(|k| **k != PageKind::Empty).collect();
    let text = non_empty
        .iter()
        .filter(|k| matches!(k, PageKind::TextLayer))
        .count();
    let scanned = non_empty
        .iter()
        .filter(|k| matches!(k, PageKind::Scanned))
        .count();
    let n = non_empty.len();
    if text * 100 >= n * 80 {
        DocumentSummary::MostlyText
    } else if scanned * 100 >= n * 80 {
        DocumentSummary::MostlyScanned
    } else {
        DocumentSummary::Mixed
    }
}

// ─────────────────────────── AutoExtractor (T4–T8) ─────────────────────────
//
// The configured-once, reusable object (api-design.md §3 — rejected the
// document-bolted sketch). Construction is cheap & infallible (no I/O,
// never downloads — #513); model provisioning is the separate
// build-time `prefetch_models()`. The native/markdown/html/classify
// surface needs **no** `ocr` feature; OCR enrichment is `#[cfg(feature
// = "ocr")]` and **always** degrades gracefully (warn + native
// fallback + typed reason — never fail-loud; only security ops
// fail-closed — `feedback_extraction_graceful_fallback`).

use crate::converters::ConversionOptions;
use crate::document::PdfDocument;

/// Comprehensive auto-extractor — configure once, reuse across pages
/// and documents (#517).
#[derive(Debug, Clone)]
pub struct AutoExtractor {
    opts: AutoExtractOptions,
}

impl Default for AutoExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoExtractor {
    /// Default extractor (`balanced` preset, `ExtractMode::Auto`).
    /// Infallible, no I/O, never downloads (#513).
    #[must_use]
    pub fn new() -> Self {
        Self {
            opts: AutoExtractOptions::balanced(),
        }
    }

    /// Never touches OCR/models at all (`ExtractMode::TextOnly`) — the
    /// `new(DisableOcr)` shape.
    #[must_use]
    pub fn text_only() -> Self {
        Self {
            opts: AutoExtractOptions::fast(),
        }
    }

    /// Construct with explicit options.
    #[must_use]
    pub fn with(opts: AutoExtractOptions) -> Self {
        Self { opts }
    }

    /// The active options.
    #[must_use]
    pub fn options(&self) -> &AutoExtractOptions {
        &self.opts
    }

    /// Model cache directory (dependency-free, cross-platform):
    /// `$PDF_OXIDE_MODEL_DIR` if set; else the platform cache base —
    /// Windows `%LOCALAPPDATA%` (or `%USERPROFILE%`), elsewhere
    /// `$XDG_CACHE_HOME` (or `$HOME/.cache`) — joined with
    /// `pdf_oxide/models`. Falls back to a relative `.cache/...` only
    /// when no base env var exists at all (#519 review: `HOME` is
    /// typically unset on Windows).
    #[must_use]
    pub fn model_cache_dir() -> std::path::PathBuf {
        use std::path::PathBuf;
        if let Some(d) = std::env::var_os("PDF_OXIDE_MODEL_DIR") {
            return PathBuf::from(d);
        }
        #[cfg(windows)]
        let base = std::env::var_os("LOCALAPPDATA")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from);
        #[cfg(not(windows))]
        let base = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")));
        base.unwrap_or_else(|| PathBuf::from(".cache"))
            .join("pdf_oxide")
            .join("models")
    }

    /// **BUILD-TIME** model provisioning (`pdf-oxide models prefetch`,
    /// the Dockerfile `RUN`). Ensures the model cache dir exists and
    /// returns it. Idempotent, **never** errors on a missing network
    /// and is a no-op without the `ocr` feature (prefetch-only, no
    /// in-wheel bundling — #513; graceful). With `ocr` the ONNX
    /// det+rec+dict (+ T1 SLANet/PP-DocLayout) fetch is wired by the
    /// T-models task; the build-time contract + cache layout exist now.
    pub fn prefetch_models(langs: &[OcrLanguage]) -> crate::Result<std::path::PathBuf> {
        let dir = Self::model_cache_dir();
        std::fs::create_dir_all(&dir).map_err(crate::error::Error::Io)?;
        #[cfg(feature = "ocr")]
        {
            let mut want: Vec<OcrLanguage> = langs.to_vec();
            if want.is_empty() {
                want.push(OcrLanguage::English);
            }
            // Shared, script-agnostic detector (downloaded once).
            Self::http_fetch(OcrLanguage::DET_URL, &dir.join("det.onnx"))?;
            for l in want {
                let s = l.spec();
                Self::http_fetch(&s.rec_url, &dir.join(&s.rec_file))?;
                let dp = dir.join(&s.dict_file);
                Self::http_fetch(&s.dict_url, &dp)?;
                // PaddleOCR emits the space class last; ensure the dict
                // ends with a lone-space line (mirrors setup_ocr_models.sh).
                if let Ok(c) = std::fs::read_to_string(&dp) {
                    if c.lines().last() != Some(" ") {
                        let _ = std::fs::write(&dp, format!("{}\n ", c.trim_end_matches('\n')));
                    }
                }
            }
        }
        #[cfg(not(feature = "ocr"))]
        {
            let _ = langs; // OCR models are only usable with the `ocr` feature
        }
        Ok(dir)
    }

    /// One-shot English provisioning (back-compat / the common case) —
    /// equivalent to `prefetch_models(&[OcrLanguage::English])`.
    pub fn prefetch_models_default() -> crate::Result<std::path::PathBuf> {
        Self::prefetch_models(&[OcrLanguage::English])
    }

    /// Whether this build can actually **download** models — i.e. it
    /// was compiled with the `ocr` feature (which pulls the `ureq`
    /// fetcher). When `false`, [`prefetch_models`](Self::prefetch_models)
    /// only ensures the cache dir exists (no network). Callers (CLI,
    /// bindings) should warn instead of reporting a misleading success.
    #[must_use]
    pub fn prefetch_available() -> bool {
        cfg!(feature = "ocr")
    }

    /// Provision exactly the models **this** extractor's configured
    /// `ocr_languages` need — the languages flow from one config object
    /// into both download *and* extraction (no re-specifying).
    /// Unrecognised codes are skipped; empty → English.
    ///
    /// ```no_run
    /// # use pdf_oxide::extractors::{AutoExtractOptions, AutoExtractor};
    /// # fn f(doc: &pdf_oxide::document::PdfDocument) -> pdf_oxide::Result<()> {
    /// let opts = AutoExtractOptions::builder().ocr_languages(["chinese"]).build();
    /// let ae = AutoExtractor::with(opts);
    /// ae.prefetch()?;                 // downloads the Chinese pack (from opts)
    /// let page = ae.extract_page(doc, 0)?;
    /// # let _ = page; Ok(()) }
    /// ```
    pub fn prefetch(&self) -> crate::Result<std::path::PathBuf> {
        let mut langs: Vec<OcrLanguage> = self
            .opts
            .ocr_languages
            .iter()
            .filter_map(|s| OcrLanguage::from_code(s))
            .collect();
        if langs.is_empty() {
            langs.push(OcrLanguage::English);
        }
        Self::prefetch_models(&langs)
    }

    /// Idempotent HTTP GET → file (atomic via a `.part` temp). Skips
    /// when the destination already exists. `#[cfg(feature = "ocr")]`
    /// (download needs the `ureq` dep the `ocr` feature pulls in).
    #[cfg(feature = "ocr")]
    fn http_fetch(url: &str, dest: &std::path::Path) -> crate::Result<()> {
        use std::io::Read;
        if dest.is_file() {
            return Ok(());
        }
        let ioerr = |m: String| crate::error::Error::Io(std::io::Error::other(m));
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(180)))
            .build()
            .new_agent();
        let mut resp = agent
            .get(url)
            .call()
            .map_err(|e| ioerr(format!("prefetch GET {url}: {e}")))?;
        let mut buf = Vec::new();
        resp.body_mut()
            .as_reader()
            .read_to_end(&mut buf)
            .map_err(|e| ioerr(format!("prefetch read {url}: {e}")))?;
        if buf.len() < 256 {
            return Err(ioerr(format!(
                "prefetch {url}: response too small ({} bytes) — likely an error page",
                buf.len()
            )));
        }
        let tmp = dest.with_extension("part");
        std::fs::write(&tmp, &buf).map_err(crate::error::Error::Io)?;
        std::fs::rename(&tmp, dest).map_err(crate::error::Error::Io)?;
        Ok(())
    }

    /// Air-gapped model manifest — real JSON
    /// `{det, languages:[{language,rec_file,dict_file,rec_url,dict_url}]}`
    /// for every supported [`OcrLanguage`] (via `pdf-oxide models
    /// manifest`). Static & network-free; the canonical source of the
    /// download URLs + cache-file layout for mirroring/verification.
    #[must_use]
    pub fn model_manifest() -> String {
        let langs = [
            OcrLanguage::English,
            OcrLanguage::Chinese,
            OcrLanguage::ChineseTraditional,
            OcrLanguage::Japanese,
            OcrLanguage::Korean,
            OcrLanguage::Arabic,
            OcrLanguage::Cyrillic,
            OcrLanguage::Latin,
            OcrLanguage::Devanagari,
            OcrLanguage::Tamil,
            OcrLanguage::Telugu,
            OcrLanguage::Kannada,
        ];
        let entries: Vec<serde_json::Value> = langs
            .iter()
            .map(|l| {
                let s = l.spec();
                serde_json::json!({
                    "language": l.code(),
                    "rec_file": s.rec_file,
                    "dict_file": s.dict_file,
                    "rec_url": s.rec_url,
                    "dict_url": s.dict_url,
                })
            })
            .collect();
        serde_json::json!({
            "detector": { "file": "det.onnx", "url": OcrLanguage::DET_URL },
            "languages": entries,
            "note": "Hebrew has no upstream PaddleOCR recognition model; \
                     the loader is ready if one is provided.",
        })
        .to_string()
    }

    /// Cheap whole-document classification preflight (no OCR).
    pub fn classify(&self, doc: &PdfDocument) -> crate::Result<DocumentClassification> {
        doc.classify_document()
    }

    // ── tier-1 simple: text / markdown / html ──

    /// Per-page text. `Auto`/`ForceOcr` route via classification; OCR is
    /// attempted only with the `ocr` feature and **always** falls back
    /// to native text + a `log::warn` if unavailable/failed (never
    /// crashes, never silent-empty — the reason is observable via
    /// [`extract_page`](Self::extract_page)).
    pub fn extract_text(&self, doc: &PdfDocument, page: usize) -> crate::Result<String> {
        // TextOnly is the cheapest path and must NOT classify —
        // classification pulls spans/images, defeating the stated
        // "fast" contract (#519 review). `doc.extract_text` itself
        // fails closed on an encrypted-unauthenticated PDF (case L),
        // so the security invariant holds without classifying here.
        if matches!(self.opts.mode, ExtractMode::TextOnly) {
            return doc.extract_text(page);
        }
        // Auto/ForceOcr: classify (fails closed on encrypted, case L)
        // then route. `route` owns the OCR-vs-native decision and the
        // observable provenance.
        let cls = doc.classify_page(page)?;
        Ok(self.route(doc, page, &cls)?.0)
    }

    /// Resolve an OCR language code to its `(rec, dict)` filenames in
    /// the model cache dir. The detector (`det.onnx`) is shared and
    /// script-agnostic. English keeps the legacy `rec.onnx`/
    /// `en_dict.txt` names (back-compat); every other language uses
    /// `rec_<lang>.onnx` / `<lang>_dict.txt` — the layout
    /// `scripts/setup_ocr_models.sh <lang>…` produces. Aliases are
    /// normalised; unknown codes pass through verbatim.
    #[cfg(feature = "ocr")]
    fn ocr_lang_files(lang: &str) -> (String, String) {
        // Single source of truth: the same [`OcrLanguage`] spec that
        // `prefetch_models` / `model_manifest` / the setup script use,
        // so the loader filenames always match what was provisioned
        // (e.g. Japanese → `rec_japan.onnx`, not `rec_japanese.onnx`).
        // Unknown codes pass through verbatim for forward-compat.
        match OcrLanguage::from_code(lang) {
            Some(l) => {
                let s = l.spec();
                (s.rec_file, s.dict_file)
            },
            None => {
                let other = lang.trim().to_ascii_lowercase();
                (format!("rec_{other}.onnx"), format!("{other}_dict.txt"))
            },
        }
    }

    /// Build an [`OcrEngine`](crate::ocr::OcrEngine) for the requested
    /// `ocr_languages` from the documented model cache dir
    /// ([`model_cache_dir`](Self::model_cache_dir):
    /// `$PDF_OXIDE_MODEL_DIR` / the platform cache; layout per
    /// `prefetch_models()` / `scripts/setup_ocr_models.sh`). PaddleOCR
    /// uses one recognition model per pass, so the first requested
    /// language whose models are present wins; English is the final
    /// fallback. `None` when no usable models exist → callers degrade
    /// to native text gracefully (never fail-loud — only security ops
    /// fail-closed). Non-English languages require the matching
    /// per-language models to be provisioned; scripts without a
    /// PaddleOCR ONNX model (e.g. Hebrew) are a provisioning limit,
    /// not a code defect — the loader is ready the moment a
    /// `rec_<lang>.onnx`/`<lang>_dict.txt` pair is dropped in.
    /// Simple OCR-language heuristic from the source PDF (#519): when
    /// `ocr_languages` is unset, sample the document's own native text
    /// (this page, else page 0) and pick the dominant non-Latin script
    /// — so a scanned Chinese/Arabic/Cyrillic/Devanagari PDF whose
    /// other pages (or sparse layer) carry that script is OCR'd with
    /// the right model instead of the English default. `None` → English
    /// fallback (no clear non-Latin signal). Deliberately cheap and
    /// conservative — a hint, not a language classifier.
    #[cfg(feature = "ocr")]
    #[must_use]
    fn detect_ocr_language(doc: &PdfDocument, page: usize) -> Option<OcrLanguage> {
        let mut s = doc.extract_text(page).unwrap_or_default();
        if s.trim().is_empty() {
            s = doc.extract_text(0).unwrap_or_default();
        }
        if s.trim().is_empty() {
            return None;
        }
        let (mut han, mut cyr, mut arab, mut deva, mut latin) = (0usize, 0, 0, 0, 0);
        for c in s.chars().take(8000) {
            match c {
                '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' => han += 1,
                '\u{0400}'..='\u{04FF}' => cyr += 1,
                '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}' => arab += 1,
                '\u{0900}'..='\u{097F}' => deva += 1,
                'A'..='Z' | 'a'..='z' => latin += 1,
                _ => {},
            }
        }
        let (n, lang) = [
            (han, OcrLanguage::Chinese),
            (cyr, OcrLanguage::Cyrillic),
            (arab, OcrLanguage::Arabic),
            (deva, OcrLanguage::Devanagari),
        ]
        .into_iter()
        .max_by_key(|(n, _)| *n)?;
        // Require a real, non-incidental non-Latin presence.
        (n >= 4 && n * 2 >= latin).then_some(lang)
    }

    /// Build an OCR engine for `page` using this extractor's configured
    /// languages, or — when unset — a cheap script heuristic on the
    /// document's own native text (so a Chinese/Arabic/Cyrillic scan is
    /// not OCR'd with the English model; empty → the loader's English
    /// fallback). The single source of truth for the engine the
    /// Auto/ForceOcr router AND `extract_page`'s per-region split use
    /// (DRY — no divergent language selection).
    #[cfg(feature = "ocr")]
    #[must_use]
    fn build_ocr_engine(&self, doc: &PdfDocument, page: usize) -> Option<crate::ocr::OcrEngine> {
        let req: Vec<String> = if !self.opts.ocr_languages.is_empty() {
            self.opts.ocr_languages.clone()
        } else {
            Self::detect_ocr_language(doc, page)
                .map(|l| vec![l.code().to_string()])
                .unwrap_or_default()
        };
        Self::load_ocr_engine(&req)
    }

    #[cfg(feature = "ocr")]
    #[must_use]
    fn load_ocr_engine(langs: &[String]) -> Option<crate::ocr::OcrEngine> {
        let dir = Self::model_cache_dir();
        let det = dir.join("det.onnx");
        if !det.is_file() {
            return None;
        }
        let mut tries: Vec<String> = langs
            .iter()
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .collect();
        tries.push("english".to_string());
        for lang in tries {
            let (recf, dictf) = Self::ocr_lang_files(&lang);
            let rec = dir.join(&recf);
            let dict = dir.join(&dictf);
            if rec.is_file() && dict.is_file() {
                if let Ok(e) =
                    crate::ocr::OcrEngine::new(&det, &rec, &dict, crate::ocr::OcrConfig::default())
                {
                    return Some(e);
                }
            }
        }
        None
    }

    /// The single source of truth for the Auto/ForceOcr routing
    /// decision **and** its resulting provenance: returns the text plus
    /// the *actual* `(ExtractSource, ReasonCode)`. A native fallback
    /// after a failed / empty / not-compiled OCR is reported as
    /// [`ExtractSource::Fallback`], never mislabelled as
    /// [`ExtractSource::Ocr`] (#519 review). Always falls back to
    /// native text with a precise `log::warn`; never crashes, never
    /// silent-empty. `cls` is the already-computed classification (the
    /// caller classifies once — no double work for the rich path).
    fn route(
        &self,
        doc: &PdfDocument,
        page: usize,
        cls: &PageClassification,
    ) -> crate::Result<(String, ExtractSource, ReasonCode)> {
        let force = matches!(self.opts.mode, ExtractMode::ForceOcr)
            || self.opts.force_ocr_pages.contains(&page);
        let needs_ocr =
            force || matches!(cls.kind, PageKind::Scanned | PageKind::ImageText | PageKind::Mixed);
        if !needs_ocr {
            return Ok((doc.extract_text(page)?, ExtractSource::NativeText, cls.reason));
        }
        // Distinguish "OCR ran but produced nothing / errored" from
        // "OCR was unavailable (feature off / models absent)". Only the
        // former is `OcrLowConfidenceFallback`; the latter is
        // `OcrRequestedButUnavailable` per the `ReasonCode` docs
        // (PR #519 review — the reason code was previously always
        // `OcrRequestedButUnavailable` even when OCR was attempted).
        #[allow(unused_mut)]
        let mut ocr_attempted = false;
        #[cfg(feature = "ocr")]
        {
            // The page needs OCR. Build an engine from the documented
            // model cache dir (`AutoExtractor::model_cache_dir()` —
            // `$PDF_OXIDE_MODEL_DIR` or the platform cache, populated by
            // `prefetch_models()` / `scripts/setup_ocr_models.sh`).
            // Language selection: explicit `ocr_languages` wins;
            // otherwise a cheap script heuristic on the document's own
            // native text picks the model (so a Chinese/Arabic/Cyrillic
            // scan is not OCR'd with the English model). Empty → the
            // loader's English fallback.
            //
            // A page that carries BOTH a usable native text layer AND
            // image region(s) (ImageText / Mixed) must be the UNION of
            // the two disjoint sources: the native layer (higher
            // fidelity, already reading-ordered) PLUS any text recovered
            // from the image — never the native text *replaced* by a
            // full-page OCR pass that re-renders and garbles the clean
            // vector glyphs. Only a genuinely scanned page (no usable
            // native layer) is fully OCR-driven. `ForceOcr` keeps its
            // "ignore native" contract and never merges.
            let native_layer = doc.extract_text(page).unwrap_or_default();
            let native_is_good =
                !native_layer.trim().is_empty() && text_quality_gate(&native_layer).is_none();
            let hybrid = !force
                && native_is_good
                && matches!(cls.kind, PageKind::ImageText | PageKind::Mixed);
            match self.build_ocr_engine(doc, page) {
                Some(engine) => match crate::ocr::ocr_page(
                    doc,
                    page,
                    &engine,
                    &crate::ocr::OcrExtractOptions::default(),
                ) {
                    Ok(o) if !o.trim().is_empty() => {
                        if hybrid {
                            // Native preserved verbatim; the OCR'd image text is
                            // positioned at the image's spatial location (so a
                            // figure/chart caption reads in its correct place,
                            // not appended after the page). Falls back to a plain
                            // append-merge when the image bbox is unavailable.
                            let placed =
                                self.place_image_ocr_in_reading_order(doc, page, &native_layer, &o);
                            return Ok((placed, ExtractSource::Ocr, ReasonCode::Ok));
                        }
                        return Ok((o, ExtractSource::Ocr, ReasonCode::Ok));
                    },
                    // OCR ran but yielded nothing / errored: warn
                    // precisely (attempted — NOT "unavailable").
                    Ok(_) => {
                        ocr_attempted = true;
                        log::warn!(
                            "auto-extract: OCR produced no text for page \
                             {page} (kind={:?}); falling back to native text",
                            cls.kind
                        )
                    },
                    Err(e) => {
                        ocr_attempted = true;
                        log::warn!(
                            "auto-extract: OCR failed for page {page}: {e}; \
                             falling back to native text"
                        )
                    },
                },
                None => log::warn!(
                    "auto-extract: page {page} (kind={:?}) needs OCR but \
                     no models in {} — run scripts/setup_ocr_models.sh or \
                     set PDF_OXIDE_MODEL_DIR; falling back to native text",
                    cls.kind,
                    Self::model_cache_dir().display()
                ),
            }
        }
        #[cfg(not(feature = "ocr"))]
        {
            log::warn!(
                "auto-extract: OCR unavailable (ocr feature not enabled) \
                 for page {page} (kind={:?}); falling back to native \
                 text (reason OcrRequestedButUnavailable)",
                cls.kind
            );
        }
        // The classifier wanted OCR but it is unavailable. Do NOT
        // blanket-label the native fallback "partial / OCR-unavailable":
        // if the native text we got is itself high-quality (passes the
        // T0.5 gate) the extraction is genuinely COMPLETE — trust the
        // gate over the routing guess. Only when the native fallback is
        // *also* poor is the result truly degraded. Without this a
        // misclassified text page reported `partial_success` /
        // `ocr_requested_but_unavailable` despite a perfect extraction
        // (#519 cross-binding smoke).
        let native = doc.extract_text(page)?;
        if !native.trim().is_empty() && text_quality_gate(&native).is_none() {
            Ok((native, ExtractSource::NativeText, ReasonCode::NativeTextHighConfidence))
        } else {
            let reason = if ocr_attempted {
                // OCR genuinely ran but failed / produced no usable text.
                ReasonCode::OcrLowConfidenceFallback
            } else {
                // OCR was never run: feature off, or models absent.
                ReasonCode::OcrRequestedButUnavailable
            };
            Ok((native, ExtractSource::Fallback, reason))
        }
    }

    /// Build a single synthetic [`TextSpan`](crate::layout::TextSpan) carrying
    /// the text OCR'd from a page's image region, positioned so the reading-order
    /// pass drops it into the figure's slot. `None` when there is nothing new to
    /// add (every OCR line already appears natively) or no image bbox to anchor
    /// to.
    ///
    /// A tagged page is assembled in marked-content (MCID) order, so a span
    /// carrying no MCID would land after the whole page; we borrow the MCID of
    /// the native line immediately *above* the figure so the span is emitted
    /// right after it. An untagged page is assembled geometrically, where the
    /// span's y-centre alone places it (the borrowed MCID is `None` and
    /// harmless). Shared by the text, Markdown and HTML auto paths so all three
    /// position recovered image text identically.
    #[cfg(feature = "ocr")]
    fn build_image_ocr_span(
        &self,
        doc: &PdfDocument,
        page: usize,
        native: &str,
        ocr: &str,
    ) -> Option<crate::layout::TextSpan> {
        // Drop OCR lines already represented in the native layer (a sparse
        // invisible-text sidecar), mirroring `merge_native_and_ocr`.
        let norm = |s: &str| {
            s.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        };
        let native_norm = norm(native);
        let mut extra_lines: Vec<&str> = Vec::new();
        for line in ocr.lines() {
            let lt = line.trim();
            if lt.is_empty() {
                continue;
            }
            let ln = norm(lt);
            if ln.is_empty()
                || native_norm.contains(&ln)
                || extra_lines.iter().any(|e| norm(e) == ln)
            {
                continue;
            }
            extra_lines.push(lt);
        }
        if extra_lines.is_empty() {
            return None;
        }
        let extra_text = extra_lines.join(" ");

        // Page-space bbox of the largest image (the figure carrying the text).
        let b = doc
            .extract_images(page)
            .ok()
            .and_then(|imgs| {
                imgs.into_iter()
                    .max_by_key(|i| (i.width() as u64) * (i.height() as u64))
            })
            .and_then(|img| img.bbox().copied())?;

        let fig_center_y = b.y + b.height * 0.5;
        let anchor_mcid = doc
            .extract_spans(page)
            .unwrap_or_default()
            .iter()
            .filter(|s| !s.text.trim().is_empty() && s.bbox.y > fig_center_y)
            .min_by(|a, c| crate::utils::safe_float_cmp(a.bbox.y, c.bbox.y))
            .and_then(|s| s.mcid);

        Some(crate::layout::TextSpan {
            text: extra_text,
            bbox: crate::geometry::Rect::new(b.x, fig_center_y, b.width.max(1.0), 12.0),
            font_size: 12.0,
            mcid: anchor_mcid,
            ..Default::default()
        })
    }

    /// Merge native text with OCR'd image text, placing the recovered text at
    /// the image's **spatial** position in the page's reading order. Degrades to
    /// the plain append-merge when the text cannot be positioned (no image bbox),
    /// so the recovered text is never lost.
    #[cfg(feature = "ocr")]
    fn place_image_ocr_in_reading_order(
        &self,
        doc: &PdfDocument,
        page: usize,
        native: &str,
        ocr: &str,
    ) -> String {
        let Some(span) = self.build_image_ocr_span(doc, page, native, ocr) else {
            // Nothing new to add, or no geometry to place it — append-merge
            // (returns native verbatim when there is no extra text).
            return crate::ocr::merge_native_and_ocr(native, ocr);
        };
        // Match `extract_text`'s options (tables on) so the native portion is
        // byte-for-byte what the plain path produces.
        let opts = ConversionOptions {
            extract_tables: true,
            ..Default::default()
        };
        match doc.extract_text_with_extra_spans(page, vec![span], &opts) {
            Ok(t) if !t.trim().is_empty() => t,
            // Assembler hiccup — never lose the recovered text.
            _ => crate::ocr::merge_native_and_ocr(native, ocr),
        }
    }

    /// For a hybrid page (usable native text layer **and** an image region that
    /// carries text), OCR the image and return a positioned synthetic span ready
    /// to merge into any output format. `None` for non-hybrid pages, when OCR is
    /// unavailable/empty, or when nothing new is recovered — callers then emit
    /// pure native output. Gives the Markdown/HTML auto paths the same
    /// native-plus-positioned-image-text behaviour as the text path.
    #[cfg(feature = "ocr")]
    fn hybrid_image_ocr_span(
        &self,
        doc: &PdfDocument,
        page: usize,
    ) -> Option<crate::layout::TextSpan> {
        if matches!(self.opts.mode, ExtractMode::TextOnly) {
            return None;
        }
        let cls = doc.classify_page(page).ok()?;
        if !matches!(cls.kind, PageKind::ImageText | PageKind::Mixed) {
            return None;
        }
        let native = doc.extract_text(page).ok()?;
        if native.trim().is_empty() || text_quality_gate(&native).is_some() {
            return None;
        }
        let engine = self.build_ocr_engine(doc, page)?;
        let ocr =
            crate::ocr::ocr_page(doc, page, &engine, &crate::ocr::OcrExtractOptions::default())
                .ok()?;
        if ocr.trim().is_empty() {
            return None;
        }
        self.build_image_ocr_span(doc, page, &native, &ocr)
    }

    /// Per-page Markdown (reuses the existing converter — DRY; no
    /// forked renderer). Mirrors the CLI `markdown` subcommand.
    pub fn extract_markdown(&self, doc: &PdfDocument, page: usize) -> crate::Result<String> {
        // Auto mode: same contract as text — native Markdown PLUS text recovered
        // from image regions, positioned in reading order (never replacing the
        // native layer). Non-hybrid pages and the no-`ocr` build emit pure native.
        #[cfg(feature = "ocr")]
        if let Some(span) = self.hybrid_image_ocr_span(doc, page) {
            return doc.to_markdown_with_extra_spans(page, &[span], &ConversionOptions::default());
        }
        doc.to_markdown(page, &ConversionOptions::default())
    }

    /// Per-page HTML (reuses the existing converter — DRY).
    pub fn extract_html(&self, doc: &PdfDocument, page: usize) -> crate::Result<String> {
        // Auto mode: native HTML PLUS positioned image-region OCR (see
        // extract_markdown). Non-hybrid pages and the no-`ocr` build emit native.
        #[cfg(feature = "ocr")]
        if let Some(span) = self.hybrid_image_ocr_span(doc, page) {
            return doc.to_html_with_extra_spans(page, &[span], &ConversionOptions::default());
        }
        doc.to_html(page, &ConversionOptions::default())
    }

    /// Whole-document text (the common LLM/RAG case), reading-ordered
    /// per page.
    pub fn extract_document_text(&self, doc: &PdfDocument) -> crate::Result<String> {
        let n = doc.page_count()?;
        let mut out = String::new();
        for p in 0..n {
            if p > 0 {
                out.push_str("\n\n");
            }
            out.push_str(&self.extract_text(doc, p)?);
        }
        Ok(out)
    }

    /// Whole-document Markdown.
    pub fn extract_document_markdown(&self, doc: &PdfDocument) -> crate::Result<String> {
        let n = doc.page_count()?;
        let mut out = String::new();
        for p in 0..n {
            if p > 0 {
                out.push_str("\n\n");
            }
            out.push_str(&self.extract_markdown(doc, p)?);
        }
        Ok(out)
    }

    // ── tier-2 rich: per-region + typed reasons ──

    /// Rich per-page extraction: classified text + per-region results,
    /// each with a 4-point bbox and a typed [`ReasonCode`] — **never a
    /// bare empty string** (the #1 user-pain fix). Reading-order region
    /// granularity (per-image-region OCR / image-table reconstruction)
    /// is layered by the OCR pipeline tasks without changing this
    /// surface.
    pub fn extract_page(&self, doc: &PdfDocument, page: usize) -> crate::Result<PageExtraction> {
        let cls = doc.classify_page(page)?;

        // Hybrid page (#517 `PageKind::ImageText`/`Mixed` = a native
        // text layer AND an image that may carry its own text): emit
        // TRUTHFUL per-source regions — a `NativeText` region for the
        // text layer plus an `Ocr` region for text recovered from the
        // image — and assemble `text` as their merge. The generic path
        // below emits ONE region whose `source` is *inferred from
        // `cls.kind`*, which mislabelled the native text as `Ocr` and
        // (via the old HybridPage either/or in `extract_text_with_ocr`)
        // silently dropped the in-image text — you could not "extract
        // both". Fully `ocr`-gated; non-hybrid pages are unchanged.
        #[cfg(feature = "ocr")]
        {
            if matches!(cls.kind, PageKind::ImageText | PageKind::Mixed)
                && !matches!(self.opts.mode, ExtractMode::TextOnly)
            {
                let native = doc.extract_text(page).unwrap_or_default();
                let ocr: Option<String> = self
                    .build_ocr_engine(doc, page)
                    .and_then(|e| {
                        crate::ocr::ocr_page(
                            doc,
                            page,
                            &e,
                            &crate::ocr::OcrExtractOptions::default(),
                        )
                        .ok()
                    })
                    .filter(|t| !t.trim().is_empty());
                let bbox = doc
                    .get_page_media_box(page)
                    .map(|(x0, y0, x1, y1)| {
                        Quad::from_xywh(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs())
                    })
                    .unwrap_or(Quad::from_xywh(0.0, 0.0, 0.0, 0.0));
                let mut regions = Vec::new();
                if !native.trim().is_empty() {
                    let nr = if text_quality_gate(&native).is_none() {
                        ReasonCode::NativeTextHighConfidence
                    } else {
                        ReasonCode::Ok
                    };
                    regions.push(Region {
                        bbox,
                        kind: RegionKind::Text,
                        text: native.clone(),
                        table: None,
                        confidence: cls.confidence,
                        source: ExtractSource::NativeText,
                        reason: nr,
                    });
                }
                if let Some(o) = ocr.as_ref() {
                    regions.push(Region {
                        bbox,
                        kind: RegionKind::Figure,
                        text: o.clone(),
                        table: None,
                        confidence: cls.confidence,
                        source: ExtractSource::Ocr,
                        reason: ReasonCode::Ok,
                    });
                }
                if !regions.is_empty() {
                    let text = match ocr.as_ref() {
                        Some(o) if !native.trim().is_empty() => {
                            crate::ocr::merge_native_and_ocr(&native, o)
                        },
                        Some(o) => o.clone(),
                        None => native.clone(),
                    };
                    let ocr_used = ocr.is_some();
                    let status = if text.trim().is_empty() {
                        ExtractionStatus::NoTextRecovered
                    } else {
                        ExtractionStatus::Complete
                    };
                    return Ok(PageExtraction {
                        page,
                        kind: cls.kind,
                        text,
                        regions,
                        confidence: cls.confidence,
                        reason: ReasonCode::Ok,
                        ocr_used,
                        status,
                    });
                }
                // No content recovered → fall through to the generic
                // (precise Fallback / NoTextRecovered) path.
            }
        }

        // Use `route()`'s ACTUAL provenance instead of re-deriving
        // source/reason from `cls.kind` + `cfg!(ocr)` + text-emptiness.
        // The old heuristic mislabelled an OCR-attempted-but-failed
        // native fallback as `source=Ocr, reason=Ok` and computed
        // `ocr_used` from the feature flag rather than real usage
        // (PR #519 review). `route()` is the single source of truth and
        // already returns the truthful `(text, source, reason)`;
        // `TextOnly` keeps its classify-free fast path (mirrors
        // `extract_text`). `ocr_used` is now a fact: OCR text was used.
        let (text, source, reason) = if matches!(self.opts.mode, ExtractMode::TextOnly) {
            (doc.extract_text(page)?, ExtractSource::NativeText, cls.reason)
        } else {
            self.route(doc, page, &cls)?
        };
        let ocr_used = source == ExtractSource::Ocr;
        let bbox = doc
            .get_page_media_box(page)
            .map(|(x0, y0, x1, y1)| {
                Quad::from_xywh(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs())
            })
            .unwrap_or(Quad::from_xywh(0.0, 0.0, 0.0, 0.0));
        // A high-confidence native text result is COMPLETE, not partial
        // — `NativeTextHighConfidence` is a success reason, same as `Ok`
        // (#519: do not report `partial_success` for a full extraction).
        let status = if text.trim().is_empty() {
            ExtractionStatus::NoTextRecovered
        } else if matches!(reason, ReasonCode::Ok | ReasonCode::NativeTextHighConfidence) {
            ExtractionStatus::Complete
        } else {
            ExtractionStatus::PartialSuccess
        };
        let region = Region {
            bbox,
            kind: RegionKind::Text,
            text: text.clone(),
            table: None,
            confidence: cls.confidence,
            source,
            reason,
        };
        Ok(PageExtraction {
            page,
            kind: cls.kind,
            text,
            regions: vec![region],
            confidence: cls.confidence,
            reason,
            ocr_used,
            status,
        })
    }

    /// Rich whole-document extraction (per-page — never a forced doc
    /// mode, case Q) + aggregate status + `pages_needing_ocr`.
    pub fn extract_document(&self, doc: &PdfDocument) -> crate::Result<DocumentExtraction> {
        let n = doc.page_count()?;
        let mut pages = Vec::with_capacity(n);
        let mut need = Vec::new();
        for p in 0..n {
            let pe = self.extract_page(doc, p)?;
            if matches!(pe.kind, PageKind::Scanned | PageKind::ImageText | PageKind::Mixed) {
                need.push(p);
            }
            pages.push(pe);
        }
        let any_text = pages.iter().any(|p| !p.text.trim().is_empty());
        let all_ok = pages
            .iter()
            .all(|p| matches!(p.reason, ReasonCode::Ok | ReasonCode::NativeTextHighConfidence));
        let status = if !any_text {
            ExtractionStatus::NoTextRecovered
        } else if all_ok {
            ExtractionStatus::Complete
        } else {
            ExtractionStatus::PartialSuccess
        };
        Ok(DocumentExtraction {
            pages,
            status,
            pages_needing_ocr: need,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_mode_default_is_auto() {
        // README locked decision 2.
        assert_eq!(ExtractMode::default(), ExtractMode::Auto);
        assert_eq!(AutoExtractOptions::default().mode, ExtractMode::Auto);
    }

    #[test]
    fn presets_have_expected_shape() {
        assert_eq!(AutoExtractOptions::fast().mode, ExtractMode::TextOnly);
        assert!(!AutoExtractOptions::fast().reconstruct_image_tables);
        assert_eq!(AutoExtractOptions::balanced().mode, ExtractMode::Auto);
        assert!(AutoExtractOptions::balanced().reconstruct_image_tables);
        assert_eq!(AutoExtractOptions::high_fidelity().mode, ExtractMode::Auto);
        assert!(AutoExtractOptions::high_fidelity()
            .min_text_confidence
            .is_some());
        // Default == balanced.
        assert_eq!(AutoExtractOptions::default(), AutoExtractOptions::balanced());
    }

    #[test]
    fn builder_mirrors_ocrconfigbuilder_shape() {
        let o = AutoExtractOptions::builder()
            .mode(ExtractMode::ForceOcr)
            .reconstruct_image_tables(false)
            .ocr_languages(["en", "de"])
            .min_text_confidence(2.0) // clamps
            .force_ocr_pages([0, 2])
            .build();
        assert_eq!(o.mode, ExtractMode::ForceOcr);
        assert!(!o.reconstruct_image_tables);
        assert_eq!(o.ocr_languages, vec!["en".to_string(), "de".to_string()]);
        assert_eq!(o.min_text_confidence, Some(1.0)); // clamped
        assert_eq!(o.force_ocr_pages, vec![0, 2]);
    }

    #[test]
    fn options_json_roundtrip_is_stable() {
        // The JSON wire is the C-ABI boundary (matches split-by-bookmarks).
        let o = AutoExtractOptions::high_fidelity();
        let js = serde_json::to_string(&o).expect("serialize");
        assert!(js.contains("\"mode\":\"auto\""));
        let back: AutoExtractOptions = serde_json::from_str(&js).expect("deserialize");
        assert_eq!(o, back);
        // Partial JSON fills via #[serde(default)] (forward-compat).
        let partial: AutoExtractOptions =
            serde_json::from_str(r#"{"mode":"force_ocr"}"#).expect("partial");
        assert_eq!(partial.mode, ExtractMode::ForceOcr);
        assert!(partial.reconstruct_image_tables); // from Default(=balanced)
    }

    #[test]
    fn reason_and_enum_wire_tokens_are_snake_case_frozen() {
        // Frozen append-only wire tokens (PadesLevel lesson).
        assert_eq!(
            serde_json::to_string(&ReasonCode::OcrRequestedButUnavailable).unwrap(),
            "\"ocr_requested_but_unavailable\""
        );
        assert_eq!(
            serde_json::to_string(&ExtractSource::ImageTableRecovery).unwrap(),
            "\"image_table_recovery\""
        );
        assert_eq!(serde_json::to_string(&PageKind::ImageText).unwrap(), "\"image_text\"");
    }

    #[test]
    fn quad_from_xywh_is_tl_tr_br_bl() {
        let q = Quad::from_xywh(10.0, 20.0, 30.0, 40.0);
        assert_eq!(q.points[0], [10.0, 60.0]); // tl
        assert_eq!(q.points[2], [40.0, 20.0]); // br
    }

    // ── classifier (pure, injected primitives — no PdfDocument) ──

    fn sig() -> PageSignals {
        PageSignals {
            text_glyph_count: 0,
            text_area_ratio: 0.0,
            image_area_ratio: 0.0,
            codec: ImageCodecClass::None,
            invisible_text_ratio: 0.0,
            garbled_ratio: 0.0,
            fragmented_word_ratio: 0.0,
            consecutive_repeat_ratio: 0.0,
            vector_path_density: 0.0,
            vector_path_count: 0,
            has_reliable_structure: false,
            producer_prior: ProducerPrior::Unknown,
            page_is_empty: false,
        }
    }

    #[test]
    fn quality_gate_flags_cid_garbage_and_passes_clean() {
        let garbage: String = "\u{FFFD}".repeat(40);
        assert_eq!(text_quality_gate(&garbage), Some(ReasonCode::GlyphMappingMissing));
        assert_eq!(
            text_quality_gate("The quick brown fox jumps over the lazy dog repeatedly."),
            None
        );
    }

    #[test]
    fn quality_gate_catches_column_scramble_and_fragmentation() {
        // Critical fragmentation hard-trigger (every glyph split).
        let frag = "a b c d e f g h i j k l m n o p q r s t";
        assert_eq!(text_quality_gate(frag), Some(ReasonCode::GlyphMappingMissing));
        // Consecutive-repeat / 2-column scramble.
        let scramble = "alpha alpha beta beta gamma gamma delta delta epsilon epsilon zeta zeta";
        assert_eq!(text_quality_gate(scramble), Some(ReasonCode::TextLayerBelowThreshold));
    }

    #[test]
    fn quality_gate_does_not_flag_dense_cjk_prose_as_fragmented() {
        // Real Japanese sentence about cats (no inter-word spaces — this
        // script never has them). Naturally clusters into short 1-3
        // character "words" once split at whatever boundary a caller
        // uses; that must not read as glyph-per-span CMap breakage the
        // way it legitimately would for Latin text.
        let ja = "ネコ 猫 は 狭義 に は 食肉目 ネコ科 ネコ属 に 分類 される \
                  リビア ヤマネコ が 家畜 化 された イエネコ に 対する 通称 である";
        assert_eq!(
            text_quality_gate(ja),
            None,
            "dense CJK prose must not trigger the text-quality gate"
        );
        assert!(is_cjk_dominant_text(ja));
        assert!(!is_cjk_dominant_text("The quick brown fox jumps over the lazy dog."));
    }

    #[test]
    fn cascade_empty_scanned_sparse_over_scan_hybrid_textlayer() {
        // Empty.
        let mut s = sig();
        s.page_is_empty = true;
        assert_eq!(classify_from_signals(&s, &AutoExtractOptions::balanced()).0, PageKind::Empty);

        // Pure scan (CCITT) → high-confidence Scanned.
        let mut s = sig();
        s.image_area_ratio = 0.97;
        s.codec = ImageCodecClass::Ccitt;
        let (k, c, _) = classify_from_signals(&s, &AutoExtractOptions::balanced());
        assert_eq!(k, PageKind::Scanned);
        assert!(c >= 0.95);

        // Sparse text over a scan (case G — the headline fix): a tiny
        // header must NOT classify as TextLayer.
        let mut s = sig();
        s.image_area_ratio = 0.95;
        s.text_glyph_count = 60; // a Bates/header line
        s.text_area_ratio = 0.02; // but covers ~nothing
        assert_eq!(classify_from_signals(&s, &AutoExtractOptions::balanced()).0, PageKind::Scanned);

        // Hybrid: real text + sub-page image (cases D/S).
        let mut s = sig();
        s.text_glyph_count = 800;
        s.text_area_ratio = 0.5;
        s.image_area_ratio = 0.25;
        assert_eq!(
            classify_from_signals(&s, &AutoExtractOptions::balanced()).0,
            PageKind::ImageText
        );

        // Clean born-digital → TextLayer; structure boosts confidence.
        let mut s = sig();
        s.text_glyph_count = 1200;
        s.text_area_ratio = 0.6;
        s.has_reliable_structure = true;
        let (k, c, r) = classify_from_signals(&s, &AutoExtractOptions::balanced());
        assert_eq!(k, PageKind::TextLayer);
        assert_eq!(r, ReasonCode::NativeTextHighConfidence);
        assert!(c >= 0.90);
    }

    #[test]
    fn cascade_keeps_good_ocr_sidecar_over_scan() {
        // Case C/C2: scan + usable invisible OCR text → keep the text.
        let mut s = sig();
        s.image_area_ratio = 0.96;
        s.text_glyph_count = 1500;
        s.text_area_ratio = 0.55;
        s.invisible_text_ratio = 0.95;
        assert_eq!(
            classify_from_signals(&s, &AutoExtractOptions::balanced()).0,
            PageKind::TextLayer
        );
    }

    #[test]
    fn summary_is_aggregate_only_never_forced_mode() {
        use PageKind::*;
        assert_eq!(summarise(&[]), DocumentSummary::Empty);
        assert_eq!(summarise(&[Empty, Empty]), DocumentSummary::Empty);
        assert_eq!(
            summarise(&[TextLayer, TextLayer, TextLayer, TextLayer, Empty]),
            DocumentSummary::MostlyText
        );
        assert_eq!(
            summarise(&[Scanned, Scanned, Scanned, Scanned]),
            DocumentSummary::MostlyScanned
        );
        // Heterogeneous doc (case Q) stays Mixed — not forced.
        assert_eq!(summarise(&[TextLayer, Scanned, ImageText, Scanned]), DocumentSummary::Mixed);
    }

    #[test]
    fn auto_extractor_construction_is_cheap_and_infallible() {
        // No I/O, never downloads (#513).
        assert_eq!(AutoExtractor::new().options().mode, ExtractMode::Auto);
        assert_eq!(AutoExtractor::default().options().mode, ExtractMode::Auto);
        assert_eq!(AutoExtractor::text_only().options().mode, ExtractMode::TextOnly);
        let ae = AutoExtractor::with(AutoExtractOptions::high_fidelity());
        assert!(ae.options().min_text_confidence.is_some());
        // Manifest is static + network-free; cache dir is pure.
        // (The v0.3.51 manifest is JSON listing the shared detector
        // `det.onnx` + per-language recognition models; it contains no
        // plural "models" token. Assert the canonical cross-binding
        // invariant the Node/C# parity tests also use — the stale
        // `.contains("models")` was always false, a pre-existing FIPS
        // CI red from the v0.3.51 manifest rewrite.)
        let mm = AutoExtractor::model_manifest();
        assert!(mm.contains("det.onnx") && mm.contains("english"));
        assert!(AutoExtractor::model_cache_dir()
            .to_string_lossy()
            .contains("pdf_oxide"));
    }
}

#[cfg(test)]
mod vector_artwork_is_not_a_scan_tests {
    use super::*;

    fn base() -> PageSignals {
        PageSignals {
            text_glyph_count: 0,
            text_area_ratio: 0.0,
            image_area_ratio: 0.0,
            codec: ImageCodecClass::None,
            invisible_text_ratio: 0.0,
            garbled_ratio: 0.0,
            fragmented_word_ratio: 0.0,
            consecutive_repeat_ratio: 0.0,
            vector_path_density: 0.0,
            vector_path_count: 0,
            has_reliable_structure: false,
            producer_prior: ProducerPrior::Unknown,
            page_is_empty: false,
        }
    }

    /// The outlined-text branch keys on path *density*, which is
    /// `paths / (paths + glyphs + images)` and so saturates at 1.0 the moment a
    /// page has no glyphs and no images — four drawings score exactly what four
    /// thousand outlined glyphs would. The count is what separates them.
    #[test]
    fn test_handful_of_paths_is_artwork_not_outlined_text() {
        let mut s = base();
        s.vector_path_density = 1.0;
        s.vector_path_count = 4;
        let (kind, _, _) = classify_from_signals(&s, &AutoExtractOptions::default());
        assert_ne!(
            kind,
            PageKind::Scanned,
            "four vector paths and no raster is artwork; OCR recovers nothing"
        );
    }

    /// The control: outlined text still routes to OCR, so the fix cannot pass
    /// by disabling the branch. Text converted to outlines emits at least one
    /// path per glyph, so its count is high.
    #[test]
    fn outlined_text_is_still_an_ocr_candidate() {
        let mut s = base();
        s.vector_path_density = 1.0;
        s.vector_path_count = 2400;
        let (kind, _, reason) = classify_from_signals(&s, &AutoExtractOptions::default());
        assert_eq!(kind, PageKind::Scanned, "a page of outlined glyphs still needs OCR");
        assert_eq!(reason, ReasonCode::NoTextLayerPresent);
    }

    /// The terminal fallback claimed "some raster" and never tested it.
    #[test]
    fn no_text_and_no_raster_is_not_a_scan() {
        let s = base();
        let (kind, _, _) = classify_from_signals(&s, &AutoExtractOptions::default());
        assert_eq!(
            kind,
            PageKind::Empty,
            "no text and no image: nothing to extract and nothing for OCR to read"
        );
    }

    /// But an image whose placement cannot be measured is still a raster.
    #[test]
    fn test_unmeasurable_image_is_still_a_scan() {
        let mut s = base();
        s.codec = ImageCodecClass::Dct;
        s.image_area_ratio = 0.0;
        let (kind, _, _) = classify_from_signals(&s, &AutoExtractOptions::default());
        assert_eq!(
            kind,
            PageKind::Scanned,
            "gating on measured area alone would report a real scan as needing no OCR"
        );
    }
}
