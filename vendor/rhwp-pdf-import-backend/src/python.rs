//! Python bindings via PyO3.
//!
//! This module provides Python bindings for the PDF library, exposing the core functionality
//! through a Python-friendly API with proper error handling and type hints.

use std::collections::HashSet;
use std::path::PathBuf;

use pyo3::exceptions::{PyIOError, PyNotImplementedError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyBytes;

// Register module-level variable for .pyi (pyo3-stub-gen); matches m.add("VERSION", ...) below.
use crate::api::PdfBuilder as RustPdfBuilder;
#[cfg(feature = "python")]
use crate::converters::ConversionOptions as RustConversionOptions;
use crate::document::PdfDocument as RustPdfDocument;
use crate::extractors::forms::{
    field_flags, FieldType as RustFieldType, FieldValue as RustFieldValue,
    FormField as RustFormField,
};
use crate::layout::{Color as RustColor, TextChar as RustTextChar};
use crate::writer::{BlendMode as RustBlendMode, LineCap as RustLineCap, LineJoin as RustLineJoin};

/// Python wrapper for PdfDocument.
///
/// Provides PDF parsing, text extraction, and format conversion capabilities.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "PdfDocument")]
pub struct PyPdfDocument {
    pub(crate) inner: RustPdfDocument,
    pub(crate) path: Option<String>,
    pub(crate) raw_bytes: Option<Vec<u8>>,
    pub(crate) editor: Option<crate::editor::DocumentEditor>,
}

impl PyPdfDocument {
    /// Ensure the editor is initialized for DOM access.
    fn ensure_editor(&mut self) -> PyResult<()> {
        if self.editor.is_none() {
            let editor = if let Some(ref path) = self.path {
                crate::editor::DocumentEditor::open(path)
            } else if let Some(ref bytes) = self.raw_bytes {
                crate::editor::DocumentEditor::from_bytes(bytes.clone())
            } else {
                return Err(PyRuntimeError::new_err("No document source available"));
            };
            self.editor =
                Some(editor.map_err(|e| {
                    PyRuntimeError::new_err(format!("Failed to open editor: {}", e))
                })?);
        }
        Ok(())
    }
}

#[pymethods]
impl PyPdfDocument {
    /// Context manager support.
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager support.
    fn __exit__(
        &mut self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_val: Option<&Bound<'_, PyAny>>,
        _exc_tb: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        Ok(false)
    }

    /// Get PDF version.
    fn version(&self) -> (u8, u8) {
        self.inner.version()
    }

    /// Get number of pages.
    ///
    /// works as both an attribute (the v0.3.6 shape)
    /// AND as a method call (the shape). Returns a `_PageCount`
    /// wrapper that is callable (`doc.page_count()` returns the int),
    /// indexable (`range(doc.page_count)` works via `__index__`),
    /// comparable with ints (`doc.page_count == 5`). The method-call
    /// form is deprecated; new code should use the attribute form.
    #[getter(page_count)]
    fn page_count(&mut self) -> PyResult<PyPageCount> {
        let value = if let Some(ref mut editor) = self.editor {
            use crate::editor::EditableDocument;
            editor
                .page_count()
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get page count: {}", e)))?
        } else {
            self.inner
                .page_count()
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get page count: {}", e)))?
        };
        Ok(PyPageCount { value })
    }

    /// List all Optional Content Group (OCG) layer names in the document.
    ///
    /// Returns:
    ///     list[str]: Layer names from /OCProperties. Empty if no layers.
    ///
    /// Example:
    ///     layers = doc.get_layers()
    ///     # ['Dieline', 'Varnish', 'Text', 'Barcode']
    ///     text = doc.extract_text(0, exclude_layers=['Dieline', 'Varnish'])
    fn get_layers(&mut self) -> PyResult<Vec<String>> {
        self.inner
            .get_layers()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to get layers: {}", e)))
    }

    /// List ink / separation names used on a specific page.
    ///
    /// Args:
    ///     page (int): Page index (0-based)
    ///
    /// Returns:
    ///     list[str]: Ink names from Separation/DeviceN color spaces.
    ///
    /// Example:
    ///     inks = doc.get_page_inks(0)
    ///     # ['PANTONE 186 C', 'Spot Varnish', 'Die Cut']
    ///     text = doc.extract_text(0, exclude_inks=['Spot Varnish', 'Die Cut'])
    fn get_page_inks(&mut self, page: usize) -> PyResult<Vec<String>> {
        self.inner
            .get_page_inks(page)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to get page inks: {}", e)))
    }

    /// List ink / separation names declared on a page, INCLUDING those
    /// declared inside Form XObjects reached through the page's content
    /// stream `Do` operators.
    ///
    /// Unlike :py:meth:`get_page_inks`, which only walks the page's own
    /// ``/Resources/ColorSpace`` dictionary, this method recurses into
    /// each invoked Form XObject and accumulates ink names from every
    /// visited resource tree. This is the method the separation renderer
    /// uses internally to allocate plates, so its result matches the
    /// plate list returned by :py:meth:`render_separations`.
    ///
    /// Args:
    ///     page (int): Page index (0-based)
    ///
    /// Returns:
    ///     list[str]: Ink names from Separation/DeviceN color spaces in
    ///         the page resources and every nested Form XObject reached
    ///         via Do operators. Recursion is bounded; cycles are
    ///         deduplicated by object reference.
    ///
    /// Example:
    ///     inks = doc.get_page_inks_deep(0)
    ///     # ['Cut', 'PANTONE 186 C', 'yellow fluorescent']
    fn get_page_inks_deep(&mut self, page: usize) -> PyResult<Vec<String>> {
        self.inner
            .get_page_inks_deep(page)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to get page inks (deep): {}", e)))
    }

    /// Enumerate existing PDF signatures. Returns a list of
    /// `Signature` objects — empty list when the document has no
    /// AcroForm or no signed signature fields.
    ///
    /// Mirrors Rust `signatures::enumerate_signatures` and the C#
    /// `PdfDocument.Signatures` surface.
    fn signatures(&mut self) -> PyResult<Vec<PySignature>> {
        let list = crate::signatures::enumerate_signatures(&mut self.inner).map_err(|e| {
            PyRuntimeError::new_err(format!("Failed to enumerate signatures: {}", e))
        })?;
        Ok(list.into_iter().map(|info| PySignature { info }).collect())
    }

    /// Count existing PDF signatures without materialising them.
    fn signature_count(&mut self) -> PyResult<usize> {
        crate::signatures::count_signatures(&mut self.inner)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to count signatures: {}", e)))
    }

    /// The document's Document Security Store (`/DSS`) as a :class:`Dss`,
    /// or ``None`` if the PDF has no DSS. Mirrors Rust
    /// `signatures::read_dss`.
    fn dss(&self) -> PyResult<Option<PyDss>> {
        #[cfg(feature = "signatures")]
        {
            crate::signatures::read_dss(&self.inner)
                .map(|opt| opt.map(|dss| PyDss { dss }))
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to read DSS: {}", e)))
        }
        #[cfg(not(feature = "signatures"))]
        {
            Ok(None)
        }
    }

    /// Extract text from a page.
    ///
    /// Args:
    ///     page (int): Page index (0-based)
    ///     region (tuple, optional): Bounding box (x, y, width, height) to restrict extraction
    ///     exclude_layers (list[str], optional): OCG layer names to exclude from extraction
    ///     exclude_inks (list[str], optional): Separation/DeviceN ink names to exclude
    ///     extract_tables (bool): When ``False``, skip the table-detection sweep
    ///         for faster speed-first raw-text extraction (default ``True``).
    ///         Applies to **whole-page** extraction only — i.e. when neither
    ///         ``region`` nor ``exclude_layers`` / ``exclude_inks`` is given.
    ///         The scoped (``region``) and filtered (``exclude_*``) paths run
    ///         their normal pipeline and ignore this flag, since those surfaces
    ///         are already scoped and lack a table-detection hot spot.
    ///     include_artifacts (bool, optional): Include content tagged
    ///         `/Artifact` (running headers/footers, page numbers,
    ///         watermarks; ISO 32000-1:2008 §14.8.2.2.1). Default **True**,
    ///         matching `extract_words`/`extract_text_lines` — flipping the
    ///         default would surface as a content regression on PDFs whose
    ///         running-artifact heuristic over-triggers on real content
    ///         (e.g. a repeated footer that carries a section identifier).
    ///         Pass `False` to get the spec-correct behavior. Like
    ///         `extract_tables`, applies to whole-page extraction only.
    ///
    /// Note:
    ///     When ``exclude_layers`` or ``exclude_inks`` are specified, the same
    ///     full text assembly pipeline is used (structure-tree ordering, table
    ///     detection, column detection) — excluded content is simply removed
    ///     before assembly.
    #[pyo3(signature = (page, region=None, exclude_layers=None, exclude_inks=None, extract_tables=true, include_artifacts=true))]
    fn extract_text(
        &mut self,
        page: usize,
        region: Option<(f32, f32, f32, f32)>,
        exclude_layers: Option<Vec<String>>,
        exclude_inks: Option<Vec<String>>,
        extract_tables: bool,
        include_artifacts: bool,
    ) -> PyResult<String> {
        let has_filters = exclude_layers.is_some() || exclude_inks.is_some();
        let layers: HashSet<String> = exclude_layers.unwrap_or_default().into_iter().collect();
        let inks: HashSet<String> = exclude_inks.unwrap_or_default().into_iter().collect();

        if let Some((x, y, w, h)) = region {
            if has_filters {
                self.inner
                    .extract_text_filtered_in_rect(
                        page,
                        layers,
                        inks,
                        crate::geometry::Rect::new(x, y, w, h),
                        crate::layout::RectFilterMode::Intersects,
                    )
                    .map_err(|e| {
                        PyRuntimeError::new_err(format!("Failed to extract filtered text: {}", e))
                    })
            } else {
                self.inner
                    .extract_text_in_rect(
                        page,
                        crate::geometry::Rect::new(x, y, w, h),
                        crate::layout::RectFilterMode::Intersects,
                    )
                    .map_err(|e| {
                        PyRuntimeError::new_err(format!("Failed to extract text in region: {}", e))
                    })
            }
        } else if has_filters {
            self.inner
                .extract_text_filtered(page, layers, inks)
                .map_err(|e| {
                    PyRuntimeError::new_err(format!("Failed to extract filtered text: {}", e))
                })
        } else {
            let options = crate::converters::ConversionOptions {
                extract_tables,
                include_artifacts,
                ..Default::default()
            };
            self.inner
                .extract_text_with_options(page, &options)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to extract text: {}", e)))
        }
    }

    /// Identify and remove headers.
    #[pyo3(signature = (threshold=0.8))]
    fn remove_headers(&mut self, threshold: f32) -> PyResult<usize> {
        let count = self
            .inner
            .remove_headers(threshold)
            .map_err(|e| PyRuntimeError::new_err(format!("Header removal failed: {}", e)))?;

        self.sync_editor_erasures()?;
        Ok(count)
    }

    /// Identify and remove footers.
    #[pyo3(signature = (threshold=0.8))]
    fn remove_footers(&mut self, threshold: f32) -> PyResult<usize> {
        let count = self
            .inner
            .remove_footers(threshold)
            .map_err(|e| PyRuntimeError::new_err(format!("Footer removal failed: {}", e)))?;

        self.sync_editor_erasures()?;
        Ok(count)
    }

    /// Identify and remove both headers and footers.
    #[pyo3(signature = (threshold=0.8))]
    fn remove_artifacts(&mut self, threshold: f32) -> PyResult<usize> {
        let count = self
            .inner
            .remove_artifacts(threshold)
            .map_err(|e| PyRuntimeError::new_err(format!("Artifact removal failed: {}", e)))?;

        self.sync_editor_erasures()?;
        Ok(count)
    }

    /// Synchronize erasures to editor.
    fn sync_editor_erasures(&mut self) -> PyResult<()> {
        if let Some(ref mut editor) = self.editor {
            for (page, regions) in self.inner.erase_regions.lock().unwrap().iter() {
                editor.clear_erase_regions(*page);
                for rect in regions {
                    let _ = editor.erase_region(
                        *page,
                        [rect.x, rect.y, rect.x + rect.width, rect.y + rect.height],
                    );
                }
            }
        }
        Ok(())
    }

    /// Erase header area.
    fn erase_header(&mut self, page: usize) -> PyResult<()> {
        self.ensure_editor()?;
        self.inner
            .erase_header(page)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to erase header: {}", e)))?;
        self.sync_editor_erasures()?;
        Ok(())
    }

    /// Deprecated: Use erase_header instead.
    fn edit_header(&mut self, page: usize) -> PyResult<()> {
        self.erase_header(page)
    }

    /// Erase footer area.
    fn erase_footer(&mut self, page: usize) -> PyResult<()> {
        self.ensure_editor()?;
        self.inner
            .erase_footer(page)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to erase footer: {}", e)))?;
        self.sync_editor_erasures()?;
        Ok(())
    }

    /// Deprecated: Use erase_footer instead.
    fn edit_footer(&mut self, page: usize) -> PyResult<()> {
        self.erase_footer(page)
    }

    /// Erase both header and footer content.
    fn erase_artifacts(&mut self, page: usize) -> PyResult<()> {
        self.ensure_editor()?;
        self.inner
            .erase_artifacts(page)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to erase artifacts: {}", e)))?;
        self.sync_editor_erasures()?;
        Ok(())
    }

    /// Focus extraction on a region.
    #[pyo3(signature = (page, bbox))]
    fn within(slf: Py<Self>, page: usize, bbox: (f32, f32, f32, f32)) -> PyResult<PyPdfPageRegion> {
        Ok(PyPdfPageRegion {
            doc: slf,
            page_index: page,
            region: crate::geometry::Rect::new(bbox.0, bbox.1, bbox.2, bbox.3),
        })
    }

    /// Render a page to image bytes.
    ///
    /// Parameters mirror Rust's `RenderOptions`
    /// (src/rendering/page_renderer.rs):
    /// - `dpi`: resolution (default 72 for Python-binding back-compat,
    ///   150 at the Rust level).
    /// - `format`: "png" (default) or "jpeg".
    /// - `background`: RGBA tuple (0.0..=1.0); omit to keep the default
    ///   white fill.
    /// - `transparent`: if True, drop the background fill entirely
    ///   (wins over `background`).
    /// - `render_annotations`: toggle for annotation rendering.
    /// - `jpeg_quality`: 1..=100, only applied when `format="jpeg"`.
    /// - `excluded_layers`: list of OCG layer names to hide during rendering.
    #[pyo3(signature = (
        page,
        dpi=None,
        format=None,
        background=None,
        transparent=false,
        render_annotations=None,
        jpeg_quality=None,
        excluded_layers=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn render_page(
        &mut self,
        page: usize,
        dpi: Option<u32>,
        format: Option<&str>,
        background: Option<(f32, f32, f32, f32)>,
        transparent: bool,
        render_annotations: Option<bool>,
        jpeg_quality: Option<u8>,
        excluded_layers: Option<Vec<String>>,
    ) -> PyResult<Vec<u8>> {
        #[cfg(feature = "rendering")]
        {
            use pyo3::exceptions::PyValueError;

            let quality = match jpeg_quality {
                Some(q) => {
                    if !(1..=100).contains(&q) {
                        return Err(PyValueError::new_err(format!(
                            "jpeg_quality must be 1-100, got {q}",
                        )));
                    }
                    q
                },
                None => 85,
            };

            let mut options = crate::rendering::RenderOptions::with_dpi(dpi.unwrap_or(72));
            if let Some(fmt) = format {
                if fmt.eq_ignore_ascii_case("jpeg") || fmt.eq_ignore_ascii_case("jpg") {
                    options = options.as_jpeg(quality);
                } else if fmt.eq_ignore_ascii_case("png") {
                    // default — no change
                } else {
                    return Err(PyValueError::new_err(format!(
                        "format must be 'png' or 'jpeg', got {fmt:?}",
                    )));
                }
            }
            if let Some((r, g, b, a)) = background {
                options.background = Some([r, g, b, a]);
            }
            if transparent {
                options.background = None;
            }
            if let Some(flag) = render_annotations {
                options.render_annotations = flag;
            }
            if let Some(layers) = excluded_layers {
                options.excluded_layers = layers.into_iter().collect();
            }

            crate::rendering::render_page(&self.inner, page, &options)
                .map(|img| img.data)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to render page: {e}")))
        }
        #[cfg(not(feature = "rendering"))]
        {
            let _ = (
                page,
                dpi,
                format,
                background,
                transparent,
                render_annotations,
                jpeg_quality,
                excluded_layers,
            );
            Err(PyRuntimeError::new_err("Rendering feature not enabled."))
        }
    }

    /// Render a page to fit inside a target pixel bounding box, preserving
    /// aspect ratio. Picks the largest DPI such that both rendered
    /// dimensions are ≤ the target box. Useful for thumbnails / fixed-size
    /// previews where the caller doesn't want to do DPI arithmetic.
    ///
    /// Args:
    ///     page (int): Zero-based page index.
    ///     width (int): Target box width in pixels (must be > 0).
    ///     height (int): Target box height in pixels (must be > 0).
    ///     format (str, optional): "png" (default) or "jpeg".
    ///     background (tuple[float, float, float, float], optional):
    ///         RGBA in 0..1. Default white.
    ///     transparent (bool, optional): If True, no background fill.
    ///     render_annotations (bool, optional): default True.
    ///     jpeg_quality (int, optional): 1-100, default 85.
    ///     excluded_layers (list[str], optional): OCG layer names to hide.
    ///
    /// Returns: bytes of the rendered image. Issue #441 / #448.
    #[pyo3(signature = (
        page, width, height, *,
        format=None, background=None, transparent=false,
        render_annotations=None, jpeg_quality=None,
        excluded_layers=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn render_page_fit(
        &mut self,
        page: usize,
        width: u32,
        height: u32,
        format: Option<&str>,
        background: Option<(f32, f32, f32, f32)>,
        transparent: bool,
        render_annotations: Option<bool>,
        jpeg_quality: Option<u8>,
        excluded_layers: Option<Vec<String>>,
    ) -> PyResult<Vec<u8>> {
        #[cfg(feature = "rendering")]
        {
            use pyo3::exceptions::PyValueError;

            if width == 0 || height == 0 {
                return Err(PyValueError::new_err("width and height must be > 0"));
            }
            let quality = match jpeg_quality {
                Some(q) => {
                    if !(1..=100).contains(&q) {
                        return Err(PyValueError::new_err(format!(
                            "jpeg_quality must be 1-100, got {q}",
                        )));
                    }
                    q
                },
                None => 85,
            };

            let mut options = crate::rendering::RenderOptions::default();
            if let Some(fmt) = format {
                if fmt.eq_ignore_ascii_case("jpeg") || fmt.eq_ignore_ascii_case("jpg") {
                    options = options.as_jpeg(quality);
                } else if fmt.eq_ignore_ascii_case("png") {
                    // default — no change
                } else {
                    return Err(PyValueError::new_err(format!(
                        "format must be 'png' or 'jpeg', got {fmt:?}",
                    )));
                }
            }
            if let Some((r, g, b, a)) = background {
                options.background = Some([r, g, b, a]);
            }
            if transparent {
                options.background = None;
            }
            if let Some(flag) = render_annotations {
                options.render_annotations = flag;
            }
            if let Some(layers) = excluded_layers {
                options.excluded_layers = layers.into_iter().collect();
            }

            crate::rendering::render_page_fit(&self.inner, page, width, height, &options)
                .map(|img| img.data)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to render page: {e}")))
        }
        #[cfg(not(feature = "rendering"))]
        {
            let _ = (
                page,
                width,
                height,
                format,
                background,
                transparent,
                render_annotations,
                jpeg_quality,
                excluded_layers,
            );
            Err(PyRuntimeError::new_err("Rendering feature not enabled."))
        }
    }

    /// Render a page as raw premultiplied RGBA8888 pixels.
    ///
    /// Returns a ``RenderedPixmap`` namedtuple with fields ``data`` (bytes),
    /// ``width`` (int), and ``height`` (int).
    /// ``len(data) == width * height * 4``.
    ///
    /// Alpha is premultiplied (PDF spec §11 transparency model).
    /// Use this instead of :meth:`render_page` when you want to hand pixel
    /// data directly to PIL (``Image.frombuffer("RGBa", ...)``) or numpy
    /// without a PNG/JPEG encode/decode roundtrip.
    /// Note: PIL mode ``"RGBa"`` (lower-case a) is for premultiplied RGBA;
    /// use ``"RGBA"`` only after un-premultiplying the data.
    ///
    /// Args:
    ///     page (int): Zero-based page index.
    ///     dpi (int, optional): Resolution (default 150).
    #[pyo3(signature = (page, dpi=None))]
    fn render_pixmap(
        &mut self,
        py: Python<'_>,
        page: usize,
        dpi: Option<u32>,
    ) -> PyResult<Py<pyo3::PyAny>> {
        #[cfg(feature = "rendering")]
        {
            let options = crate::rendering::RenderOptions::with_dpi(dpi.unwrap_or(150)).as_raw();
            let img = crate::rendering::render_page(&self.inner, page, &options).map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to render page: {e}"))
            })?;
            let pixmap_mod = py.import("pdf_oxide")?;
            let pixmap_cls = pixmap_mod.getattr("RenderedPixmap")?;
            let data = pyo3::types::PyBytes::new(py, &img.data);
            let result = pixmap_cls.call1((data, img.width as i64, img.height as i64))?;
            Ok(result.into())
        }
        #[cfg(not(feature = "rendering"))]
        {
            let _ = (py, page, dpi);
            Err(pyo3::exceptions::PyRuntimeError::new_err("Rendering feature not enabled."))
        }
    }

    /// Render all ink separation plates for a page.
    ///
    /// Each plate is a grayscale image where pixel intensity (0-255) = ink
    /// coverage (0 = no ink, 255 = full tint). Process inks (Cyan, Magenta,
    /// Yellow, Black) are included if the page uses DeviceCMYK content; spot
    /// colors are taken from Separation and DeviceN color spaces. To display
    /// a plate as black ink on white paper, invert: ``display = 255 - value``.
    ///
    /// Args:
    ///     page (int): Zero-based page index.
    ///     dpi (int, optional): Resolution in DPI (default 150 — the Rust API
    ///         has no default; Python defaults to 150 for convenience).
    ///
    /// Returns:
    ///     list[SeparationPlate]: One plate per ink. Each has ``ink_name``,
    ///     ``data`` (grayscale bytes), ``width``, and ``height``.
    ///
    /// Example:
    ///     plates = doc.render_separations(0, dpi=150)
    ///     for plate in plates:
    ///         print(f"{plate.ink_name}: {plate.width}x{plate.height}")
    #[pyo3(signature = (page, dpi=None))]
    fn render_separations(
        &mut self,
        py: Python<'_>,
        page: usize,
        dpi: Option<u32>,
    ) -> PyResult<Vec<Py<pyo3::PyAny>>> {
        #[cfg(feature = "rendering")]
        {
            let dpi_value = dpi.unwrap_or(150);
            let plates = crate::rendering::render_separations(&self.inner, page, dpi_value)
                .map_err(|e| {
                    PyRuntimeError::new_err(format!("Failed to render separations: {e}"))
                })?;
            let plate_mod = py.import("pdf_oxide")?;
            let plate_cls = plate_mod.getattr("SeparationPlate")?;
            let mut out = Vec::with_capacity(plates.len());
            for plate in plates {
                let data = pyo3::types::PyBytes::new(py, &plate.data);
                let result = plate_cls.call1((
                    plate.ink_name,
                    data,
                    plate.width as i64,
                    plate.height as i64,
                ))?;
                out.push(result.into());
            }
            Ok(out)
        }
        #[cfg(not(feature = "rendering"))]
        {
            let _ = (py, page, dpi);
            Err(PyRuntimeError::new_err("Rendering feature not enabled."))
        }
    }

    /// Render a single ink separation plate for a page.
    ///
    /// Returns a grayscale image where pixel intensity (0-255) = ink coverage
    /// of the named ink (0 = no ink, 255 = full tint). If the ink is not
    /// present on the page, the plate is all zeros. To display the plate as
    /// black ink on white paper, invert: ``display = 255 - value``.
    ///
    /// Args:
    ///     page (int): Zero-based page index.
    ///     ink_name (str): Ink name (e.g., "Cyan", "PANTONE 185 C", "Dieline").
    ///     dpi (int, optional): Resolution in DPI (default 150 — the Rust API
    ///         has no default; Python defaults to 150 for convenience).
    ///
    /// Returns:
    ///     SeparationPlate: The plate with ``ink_name``, ``data``, ``width``, ``height``.
    ///
    /// Example:
    ///     dieline = doc.render_separation(0, "Dieline", dpi=150)
    #[pyo3(signature = (page, ink_name, dpi=None))]
    fn render_separation(
        &mut self,
        py: Python<'_>,
        page: usize,
        ink_name: &str,
        dpi: Option<u32>,
    ) -> PyResult<Py<pyo3::PyAny>> {
        #[cfg(feature = "rendering")]
        {
            let dpi_value = dpi.unwrap_or(150);
            let plate = crate::rendering::render_separation(&self.inner, page, ink_name, dpi_value)
                .map_err(|e| {
                    PyRuntimeError::new_err(format!("Failed to render separation: {e}"))
                })?;
            let plate_mod = py.import("pdf_oxide")?;
            let plate_cls = plate_mod.getattr("SeparationPlate")?;
            let data = pyo3::types::PyBytes::new(py, &plate.data);
            let result =
                plate_cls.call1((plate.ink_name, data, plate.width as i64, plate.height as i64))?;
            Ok(result.into())
        }
        #[cfg(not(feature = "rendering"))]
        {
            let _ = (py, page, ink_name, dpi);
            Err(PyRuntimeError::new_err("Rendering feature not enabled."))
        }
    }

    /// Extract low-level characters.
    ///
    /// Args:
    ///     page (int): Page index (0-based)
    ///     region (tuple, optional): Bounding box (x, y, width, height) to restrict extraction
    ///     exclude_layers (list[str], optional): OCG layer names to exclude from extraction
    ///     exclude_inks (list[str], optional): Separation/DeviceN ink names to exclude
    #[pyo3(signature = (page, region=None, exclude_layers=None, exclude_inks=None))]
    fn extract_chars(
        &mut self,
        page: usize,
        region: Option<(f32, f32, f32, f32)>,
        exclude_layers: Option<Vec<String>>,
        exclude_inks: Option<Vec<String>>,
    ) -> PyResult<Vec<PyTextChar>> {
        let has_filters = exclude_layers.is_some() || exclude_inks.is_some();
        let layers: HashSet<String> = exclude_layers.unwrap_or_default().into_iter().collect();
        let inks: HashSet<String> = exclude_inks.unwrap_or_default().into_iter().collect();

        let chars_result = if has_filters {
            let chars = self
                .inner
                .extract_chars_filtered(page, layers, inks)
                .map_err(|e| {
                    PyRuntimeError::new_err(format!("Failed to extract characters: {}", e))
                })?;
            // Apply region filter on top if specified
            if let Some((x, y, w, h)) = region {
                use crate::layout::SpatialCollectionFiltering;
                Ok(chars.filter_by_rect(
                    &crate::geometry::Rect::new(x, y, w, h),
                    crate::layout::RectFilterMode::Intersects,
                ))
            } else {
                Ok(chars)
            }
        } else if let Some((x, y, w, h)) = region {
            self.inner.extract_chars_in_rect(
                page,
                crate::geometry::Rect::new(x, y, w, h),
                crate::layout::RectFilterMode::Intersects,
            )
        } else {
            self.inner.extract_chars(page)
        };

        chars_result
            .map(|chars| {
                chars
                    .into_iter()
                    .map(|ch| PyTextChar { inner: ch })
                    .collect()
            })
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to extract characters: {}", e)))
    }

    /// Extract words from a page.
    ///
    /// Args:
    ///     page (int): Page index (0-based)
    ///     include_artifacts (bool, optional): Include words tagged as
    ///         `/Artifact` (running headers/footers, page numbers,
    ///         watermarks; ISO 32000-1:2008 §14.8.2.2.1). Default
    ///         **True** for backward compatibility with 0.3.41 — the
    ///         pre-existing code path returned all spans regardless of
    ///         artifact tag and the cross-build regression sweep showed
    ///         flipping the default would surface as a content
    ///         regression on PDFs whose running-artifact heuristic
    ///         over-triggers on real content. Pass `False` to get the
    ///         spec-correct behavior (artifact-tagged spans excluded).
    ///     region, word_gap_threshold, profile (deprecated, optional):
    ///         Power-user overrides retained for backward compatibility.
    ///         Passing any of these emits a DeprecationWarning. They will
    ///         move to a separate `extract_words_advanced` method in a
    ///         future minor release.
    #[pyo3(signature = (page, *, include_artifacts=true, region=None, word_gap_threshold=None, profile=None))]
    fn extract_words(
        &mut self,
        py: Python<'_>,
        page: usize,
        include_artifacts: bool,
        region: Option<(f32, f32, f32, f32)>,
        word_gap_threshold: Option<f32>,
        profile: Option<PyExtractionProfile>,
    ) -> PyResult<Vec<PyWord>> {
        use crate::layout::{RectFilterMode, SpatialCollectionFiltering};

        if region.is_some() || word_gap_threshold.is_some() || profile.is_some() {
            warn_deprecated_kwargs(
                py,
                "extract_words",
                &["region", "word_gap_threshold", "profile"],
            );
        }

        // Default (`include_artifacts=False`, the new spec-correct path)
        // routes through the `_no_artifacts` variant. The legacy
        // include-artifacts path keeps the pre-0.3.42 output verbatim.
        let words = if include_artifacts {
            self.inner
                .extract_words_with_thresholds(page, word_gap_threshold, profile.map(|p| p.inner))
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to extract words: {}", e)))?
        } else {
            self.inner
                .extract_words_with_thresholds_no_artifacts(
                    page,
                    word_gap_threshold,
                    profile.map(|p| p.inner),
                )
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to extract words: {}", e)))?
        };

        let filtered = if let Some((x, y, w, h)) = region {
            let rect = crate::geometry::Rect::new(x, y, w, h);
            words.filter_by_rect(&rect, RectFilterMode::Intersects)
        } else {
            words
        };

        Ok(filtered.into_iter().map(|w| PyWord { inner: w }).collect())
    }

    /// Extract text lines from a page.
    ///
    /// Args:
    ///     page (int): Page index (0-based)
    ///     include_artifacts (bool, optional): Include lines whose words
    ///         are tagged as `/Artifact` (running headers/footers, page
    ///         numbers, watermarks; ISO 32000-1:2008 §14.8.2.2.1).
    ///         Default **True** for backward compatibility with 0.3.41.
    ///         Pass `False` to get the spec-correct behavior
    ///         (artifact-tagged spans excluded).
    ///     region, word_gap_threshold, line_gap_threshold, profile
    ///         (deprecated, optional): Power-user overrides retained for
    ///         backward compatibility. Passing any of these emits a
    ///         DeprecationWarning. They will move to a separate
    ///         `extract_text_lines_advanced` method in a future release.
    #[pyo3(signature = (page, *, include_artifacts=true, region=None, word_gap_threshold=None, line_gap_threshold=None, profile=None))]
    fn extract_text_lines(
        &mut self,
        py: Python<'_>,
        page: usize,
        include_artifacts: bool,
        region: Option<(f32, f32, f32, f32)>,
        word_gap_threshold: Option<f32>,
        line_gap_threshold: Option<f32>,
        profile: Option<PyExtractionProfile>,
    ) -> PyResult<Vec<PyTextLine>> {
        use crate::layout::{RectFilterMode, SpatialCollectionFiltering};

        if region.is_some()
            || word_gap_threshold.is_some()
            || line_gap_threshold.is_some()
            || profile.is_some()
        {
            warn_deprecated_kwargs(
                py,
                "extract_text_lines",
                &[
                    "region",
                    "word_gap_threshold",
                    "line_gap_threshold",
                    "profile",
                ],
            );
        }

        let lines = if include_artifacts {
            self.inner
                .extract_text_lines_with_thresholds(
                    page,
                    word_gap_threshold,
                    line_gap_threshold,
                    profile.map(|p| p.inner),
                )
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to extract lines: {}", e)))?
        } else {
            self.inner
                .extract_text_lines_with_thresholds_no_artifacts(
                    page,
                    word_gap_threshold,
                    line_gap_threshold,
                    profile.map(|p| p.inner),
                )
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to extract lines: {}", e)))?
        };

        let filtered = if let Some((x, y, w, h)) = region {
            let rect = crate::geometry::Rect::new(x, y, w, h);
            lines.filter_by_rect(&rect, RectFilterMode::Intersects)
        } else {
            lines
        };

        Ok(filtered
            .into_iter()
            .map(|l| PyTextLine { inner: l })
            .collect())
    }

    /// Get the computed adaptive layout parameters for a page.
    fn page_layout_params(&mut self, page: usize) -> PyResult<PyLayoutParams> {
        use crate::layout::{AdaptiveLayoutParams, DocumentProperties};

        let spans = self
            .inner
            .extract_spans(page)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to extract spans: {}", e)))?;

        let media_box = self
            .inner
            .get_page_media_box(page)
            .unwrap_or((0.0, 0.0, 612.0, 792.0));
        let page_bbox =
            crate::geometry::Rect::new(media_box.0, media_box.1, media_box.2, media_box.3);

        let all_chars: Vec<_> = spans.iter().flat_map(|s| s.to_chars()).collect();
        let props = DocumentProperties::analyze(&all_chars, page_bbox)
            .map_err(|e| PyRuntimeError::new_err(format!("Layout analysis failed: {}", e)))?;
        let params = AdaptiveLayoutParams::from_properties(&props);

        Ok(PyLayoutParams {
            word_gap_threshold: params.word_gap_threshold,
            line_gap_threshold: params.line_gap_threshold,
            median_char_width: props.median_char_width,
            median_font_size: props.median_font_size,
            median_line_spacing: props.median_line_spacing,
            column_count: props.column_count,
        })
    }

    /// Check if Tagged PDF.
    fn has_structure_tree(&mut self) -> bool {
        self.inner.structure_tree().ok().flatten().is_some()
    }

    /// Convert page to plain text.
    ///
    /// Args:
    ///     include_artifacts (bool, optional): Include content tagged
    ///         `/Artifact` (running headers/footers, page numbers,
    ///         watermarks; ISO 32000-1:2008 §14.8.2.2.1). Default **True**
    ///         — see `PdfDocument.extract_text` for the full rationale.
    #[pyo3(signature = (page, preserve_layout=false, detect_headings=true, include_images=false, image_output_dir=None, include_artifacts=true))]
    fn to_plain_text(
        &mut self,
        page: usize,
        preserve_layout: bool,
        detect_headings: bool,
        include_images: bool,
        image_output_dir: Option<String>,
        include_artifacts: bool,
    ) -> PyResult<String> {
        let options = RustConversionOptions {
            preserve_layout,
            detect_headings,
            extract_tables: true,
            include_images,
            image_output_dir,
            include_artifacts,
            ..Default::default()
        };

        self.inner
            .to_plain_text(page, &options)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert to plain text: {}", e)))
    }

    /// Convert all pages to plain text.
    ///
    /// Args:
    ///     include_artifacts (bool, optional): See `PdfDocument.extract_text`.
    ///         Default **True**.
    #[pyo3(signature = (preserve_layout=false, detect_headings=true, include_images=false, image_output_dir=None, include_artifacts=true))]
    fn to_plain_text_all(
        &mut self,
        preserve_layout: bool,
        detect_headings: bool,
        include_images: bool,
        image_output_dir: Option<String>,
        include_artifacts: bool,
    ) -> PyResult<String> {
        let options = RustConversionOptions {
            preserve_layout,
            detect_headings,
            extract_tables: true,
            include_images,
            image_output_dir,
            include_artifacts,
            ..Default::default()
        };

        self.inner.to_plain_text_all(&options).map_err(|e| {
            PyRuntimeError::new_err(format!("Failed to convert all pages to plain text: {}", e))
        })
    }

    /// Convert page to Markdown.
    ///
    /// Args:
    ///     include_artifacts (bool, optional): See `PdfDocument.extract_text`.
    ///         Default **True**.
    #[pyo3(signature = (page, preserve_layout=false, detect_headings=true, include_images=false, image_output_dir=None, embed_images=true, include_form_fields=true, include_artifacts=true))]
    fn to_markdown(
        &mut self,
        page: usize,
        preserve_layout: bool,
        detect_headings: bool,
        include_images: bool,
        image_output_dir: Option<String>,
        embed_images: bool,
        include_form_fields: bool,
        include_artifacts: bool,
    ) -> PyResult<String> {
        let options = RustConversionOptions {
            preserve_layout,
            detect_headings,
            extract_tables: true,
            include_images,
            image_output_dir,
            embed_images,
            include_form_fields,
            include_artifacts,
            ..Default::default()
        };

        self.inner
            .to_markdown(page, &options)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert to Markdown: {}", e)))
    }

    /// Convert page to HTML.
    ///
    /// Note: unlike `extract_text`/`to_markdown`/`to_plain_text`, this
    /// always includes content tagged `/Artifact` — there is no
    /// `include_artifacts` toggle here because there is nothing to
    /// toggle off.
    #[pyo3(signature = (page, preserve_layout=false, detect_headings=true, include_images=false, image_output_dir=None, embed_images=true, include_form_fields=true))]
    fn to_html(
        &mut self,
        page: usize,
        preserve_layout: bool,
        detect_headings: bool,
        include_images: bool,
        image_output_dir: Option<String>,
        embed_images: bool,
        include_form_fields: bool,
    ) -> PyResult<String> {
        let options = RustConversionOptions {
            preserve_layout,
            detect_headings,
            extract_tables: true,
            include_images,
            image_output_dir,
            embed_images,
            include_form_fields,
            ..Default::default()
        };

        self.inner
            .to_html(page, &options)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert to HTML: {}", e)))
    }

    /// Convert all pages to Markdown.
    ///
    /// Args:
    ///     include_artifacts (bool, optional): See `PdfDocument.extract_text`.
    ///         Default **True**.
    #[pyo3(signature = (preserve_layout=false, detect_headings=true, include_images=false, image_output_dir=None, embed_images=true, include_form_fields=true, include_artifacts=true))]
    fn to_markdown_all(
        &mut self,
        preserve_layout: bool,
        detect_headings: bool,
        include_images: bool,
        image_output_dir: Option<String>,
        embed_images: bool,
        include_form_fields: bool,
        include_artifacts: bool,
    ) -> PyResult<String> {
        let options = RustConversionOptions {
            preserve_layout,
            detect_headings,
            extract_tables: true,
            include_images,
            image_output_dir,
            embed_images,
            include_form_fields,
            include_artifacts,
            ..Default::default()
        };

        self.inner.to_markdown_all(&options).map_err(|e| {
            PyRuntimeError::new_err(format!("Failed to convert all pages to Markdown: {}", e))
        })
    }

    /// Convert all pages to HTML.
    #[pyo3(signature = (preserve_layout=false, detect_headings=true, include_images=false, image_output_dir=None, embed_images=true, include_form_fields=true))]
    fn to_html_all(
        &mut self,
        preserve_layout: bool,
        detect_headings: bool,
        include_images: bool,
        image_output_dir: Option<String>,
        embed_images: bool,
        include_form_fields: bool,
    ) -> PyResult<String> {
        let options = RustConversionOptions {
            preserve_layout,
            detect_headings,
            extract_tables: true,
            include_images,
            image_output_dir,
            embed_images,
            include_form_fields,
            ..Default::default()
        };

        self.inner.to_html_all(&options).map_err(|e| {
            PyRuntimeError::new_err(format!("Failed to convert all pages to HTML: {}", e))
        })
    }

    /// Convert the entire PDF to a DOCX file on disk.
    fn to_docx(&self, path: &str) -> PyResult<()> {
        self.inner
            .to_docx(path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert to DOCX: {}", e)))
    }

    /// Convert the entire PDF to DOCX bytes in memory.
    fn to_docx_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self
            .inner
            .to_docx_bytes()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert to DOCX: {}", e)))?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Convert the entire PDF to a PPTX file on disk.
    fn to_pptx(&self, path: &str) -> PyResult<()> {
        self.inner
            .to_pptx(path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert to PPTX: {}", e)))
    }

    /// Convert the entire PDF to PPTX bytes in memory.
    fn to_pptx_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self
            .inner
            .to_pptx_bytes()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert to PPTX: {}", e)))?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Convert the entire PDF to an XLSX file on disk.
    fn to_xlsx(&self, path: &str) -> PyResult<()> {
        self.inner
            .to_xlsx(path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert to XLSX: {}", e)))
    }

    /// Convert the entire PDF to XLSX bytes in memory.
    fn to_xlsx_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self
            .inner
            .to_xlsx_bytes()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert to XLSX: {}", e)))?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Get page object for DOM access.
    fn page(&mut self, index: usize) -> PyResult<PyPdfPage> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        let page = editor
            .get_page(index)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to get page: {}", e)))?;
        Ok(PyPdfPage { inner: page })
    }

    /// Save modification to page.
    fn save_page(&mut self, page: &PyPdfPage) -> PyResult<()> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        editor
            .save_page(page.inner.clone())
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to save page: {}", e)))
    }

    /// Save document to *path* with optional compression and garbage-collection.
    ///
    /// Args:
    ///     path (str): Destination file path.
    ///     compress (bool): Compress unfiltered streams with FlateDecode. Default ``True``.
    ///     garbage_collect (bool): Remove unreachable objects. Default ``True``.
    ///     linearize (bool): Linearize for fast web view (no-op, reserved). Default ``False``.
    #[pyo3(signature = (path, compress=true, garbage_collect=true, linearize=false))]
    fn save(
        &mut self,
        path: &str,
        compress: bool,
        garbage_collect: bool,
        linearize: bool,
    ) -> PyResult<()> {
        use crate::editor::{EditableDocument, SaveOptions};
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            let options = SaveOptions {
                compress,
                garbage_collect,
                linearize,
                incremental: false,
                encryption: None,
            };
            editor
                .save_with_options(path, options)
                .map_err(|e| PyIOError::new_err(format!("Failed to save PDF: {}", e)))
        } else {
            Err(PyRuntimeError::new_err("No editor initialized."))
        }
    }

    /// Save document to bytes with optional compression and garbage-collection.
    ///
    /// Returns:
    ///     bytes: The serialized PDF as a byte string.
    ///
    /// Args:
    ///     compress (bool): Compress unfiltered streams with FlateDecode. Default ``True``.
    ///     garbage_collect (bool): Remove unreachable objects. Default ``True``.
    ///     linearize (bool): Linearize for fast web view (no-op, reserved). Default ``False``.
    #[pyo3(signature = (compress=true, garbage_collect=true, linearize=false))]
    fn to_bytes<'py>(
        &mut self,
        py: Python<'py>,
        compress: bool,
        garbage_collect: bool,
        linearize: bool,
    ) -> PyResult<Py<PyBytes>> {
        use crate::editor::SaveOptions;
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            let options = SaveOptions {
                compress,
                garbage_collect,
                linearize,
                incremental: false,
                encryption: None,
            };
            let bytes = editor.save_to_bytes_with_options(options).map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to save PDF to bytes: {}", e))
            })?;
            Ok(PyBytes::new(py, &bytes).unbind())
        } else {
            Err(PyRuntimeError::new_err("No editor initialized."))
        }
    }

    /// Set document metadata title.
    fn set_title(&mut self, title: &str) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor.set_title(title);
        }
        Ok(())
    }

    /// Set document metadata author.
    fn set_author(&mut self, author: &str) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor.set_author(author);
        }
        Ok(())
    }

    /// Set document metadata subject.
    fn set_subject(&mut self, subject: &str) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor.set_subject(subject);
        }
        Ok(())
    }

    /// Set document metadata keywords.
    fn set_keywords(&mut self, keywords: &str) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor.set_keywords(keywords);
        }
        Ok(())
    }

    /// Get page rotation.
    fn page_rotation(&mut self, page: usize) -> PyResult<i32> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .get_page_rotation(page)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Err(PyRuntimeError::new_err("No editor initialized."))
        }
    }

    /// Set page rotation.
    fn set_page_rotation(&mut self, page: usize, degrees: i32) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .set_page_rotation(page, degrees)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Ok(())
        }
    }

    /// Rotate page by degrees.
    fn rotate_page(&mut self, page: usize, degrees: i32) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .rotate_page_by(page, degrees)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Ok(())
        }
    }

    /// Rotate all pages.
    fn rotate_all_pages(&mut self, degrees: i32) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .rotate_all_pages(degrees)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Ok(())
        }
    }

    /// Get page mediabox.
    fn page_media_box(&mut self, page: usize) -> PyResult<(f32, f32, f32, f32)> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            let b = editor
                .get_page_media_box(page)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            Ok((b[0], b[1], b[2], b[3]))
        } else {
            Err(PyRuntimeError::new_err("No editor initialized."))
        }
    }

    /// Set page mediabox.
    fn set_page_media_box(
        &mut self,
        page: usize,
        llx: f32,
        lly: f32,
        urx: f32,
        ury: f32,
    ) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .set_page_media_box(page, [llx, lly, urx, ury])
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Ok(())
        }
    }

    /// Get page cropbox.
    fn page_crop_box(&mut self, page: usize) -> PyResult<Option<(f32, f32, f32, f32)>> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            let b = editor
                .get_page_crop_box(page)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            Ok(b.map(|v| (v[0], v[1], v[2], v[3])))
        } else {
            Ok(None)
        }
    }

    /// Set page cropbox.
    fn set_page_crop_box(
        &mut self,
        page: usize,
        llx: f32,
        lly: f32,
        urx: f32,
        ury: f32,
    ) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .set_page_crop_box(page, [llx, lly, urx, ury])
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Ok(())
        }
    }

    /// Crop all pages margins.
    fn crop_margins(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .crop_margins(left, right, top, bottom)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Ok(())
        }
    }

    /// Erase rectangular region.
    fn erase_region(
        &mut self,
        page: usize,
        llx: f32,
        lly: f32,
        urx: f32,
        ury: f32,
    ) -> PyResult<()> {
        let rect = crate::geometry::Rect::new(llx, lly, urx - llx, ury - lly);
        self.inner
            .erase_region(page, rect)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .erase_region(page, [llx, lly, urx, ury])
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Erase multiple regions.
    fn erase_regions(&mut self, page: usize, rects: Vec<(f32, f32, f32, f32)>) -> PyResult<()> {
        for (llx, lly, urx, ury) in &rects {
            let rect = crate::geometry::Rect::new(*llx, *lly, *urx - *llx, *ury - *lly);
            self.inner
                .erase_region(page, rect)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            let arrays: Vec<[f32; 4]> = rects.iter().map(|r| [r.0, r.1, r.2, r.3]).collect();
            editor
                .erase_regions(page, &arrays)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Clear erase regions.
    fn clear_erase_regions(&mut self, page: usize) -> PyResult<()> {
        self.inner
            .clear_erase_regions(page)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        if let Some(ref mut editor) = self.editor {
            editor.clear_erase_regions(page);
        }
        Ok(())
    }

    /// Flatten page annotations.
    fn flatten_page_annotations(&mut self, page: usize) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .flatten_page_annotations(page)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Flatten all annotations.
    fn flatten_all_annotations(&mut self) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .flatten_all_annotations()
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Check if page marked for flatten.
    fn is_page_marked_for_flatten(&self, page: usize) -> bool {
        self.editor
            .as_ref()
            .is_some_and(|e| e.is_page_marked_for_flatten(page))
    }

    /// Unmark page for flatten.
    fn unmark_page_for_flatten(&mut self, page: usize) {
        if let Some(ref mut editor) = self.editor {
            editor.unmark_page_for_flatten(page);
        }
    }

    /// Apply page redactions.
    fn apply_page_redactions(&mut self, page: usize) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .apply_page_redactions(page)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Apply all redactions.
    fn apply_all_redactions(&mut self) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .apply_all_redactions()
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Check if page marked for redaction.
    fn is_page_marked_for_redaction(&self, page: usize) -> bool {
        self.editor
            .as_ref()
            .is_some_and(|e| e.is_page_marked_for_redaction(page))
    }

    /// Unmark page for redaction.
    fn unmark_page_for_redaction(&mut self, page: usize) {
        if let Some(ref mut editor) = self.editor {
            editor.unmark_page_for_redaction(page);
        }
    }

    /// Queue an explicit destructive redaction rectangle on a page.
    ///
    /// Args:
    ///     page (int): zero-based page index.
    ///     rect (tuple[float,float,float,float]): ``(x0, y0, x1, y1)`` in
    ///         page user space.
    ///     fill (tuple[float,float,float] | None): overlay DeviceRGB
    ///         colour; ``None`` uses the default.
    #[pyo3(signature = (page, rect, fill=None))]
    fn add_redaction(
        &mut self,
        page: usize,
        rect: (f32, f32, f32, f32),
        fill: Option<(f32, f32, f32)>,
    ) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .add_redaction(
                    page,
                    [rect.0, rect.1, rect.2, rect.3],
                    fill.map(|(r, g, b)| [r, g, b]),
                )
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Number of redaction regions queued for ``page`` (annotations +
    /// programmatic rectangles).
    fn redaction_count(&mut self, page: usize) -> PyResult<usize> {
        self.ensure_editor()?;
        match self.editor {
            Some(ref mut editor) => editor
                .redaction_count(page)
                .map_err(|e| PyValueError::new_err(e.to_string())),
            None => Err(PyRuntimeError::new_err("No document source available")),
        }
    }

    /// Destructively apply all queued redactions (true content removal,
    /// ISO 32000-1:2008 §12.5.6.23) and return a report dict.
    ///
    /// Raises ``RuntimeError`` if content uses a composite/Type0 font
    /// (refused rather than risk a silent under-redaction).
    #[pyo3(signature = (scrub_metadata=true, remove_javascript=true, remove_embedded_files=true, fill=(0.0, 0.0, 0.0)))]
    fn apply_redactions_destructive(
        &mut self,
        py: Python<'_>,
        scrub_metadata: bool,
        remove_javascript: bool,
        remove_embedded_files: bool,
        fill: (f32, f32, f32),
    ) -> PyResult<Py<pyo3::types::PyDict>> {
        self.ensure_editor()?;
        let opts = crate::redaction::RedactionOptions {
            scrub_metadata,
            remove_javascript,
            remove_embedded_files,
            default_fill: [fill.0, fill.1, fill.2],
            ..crate::redaction::RedactionOptions::default()
        };
        let report = match self.editor {
            Some(ref mut editor) => editor
                .apply_redactions_destructive(opts)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
            None => return Err(PyRuntimeError::new_err("No document source available")),
        };
        let d = pyo3::types::PyDict::new(py);
        d.set_item("regions", report.regions)?;
        d.set_item("glyphs_removed", report.glyphs_removed)?;
        d.set_item("images_modified", report.images_modified)?;
        d.set_item("images_removed", report.images_removed)?;
        d.set_item("paths_pruned", report.paths_pruned)?;
        d.set_item("xobjects_specialized", report.xobjects_specialized)?;
        d.set_item("annotations_removed", report.annotations_removed)?;
        d.set_item("fonts_scrubbed", report.fonts_scrubbed)?;
        d.set_item("bytes_removed", report.bytes_removed)?;
        Ok(d.unbind())
    }

    /// Standalone document sanitization (#231 T10): strip `/Info`,
    /// catalog XMP `/Metadata`, document JavaScript and embedded files
    /// without geometric redaction. Returns a report dict.
    #[pyo3(signature = (scrub_metadata=true, remove_javascript=true, remove_embedded_files=true))]
    fn sanitize_document(
        &mut self,
        py: Python<'_>,
        scrub_metadata: bool,
        remove_javascript: bool,
        remove_embedded_files: bool,
    ) -> PyResult<Py<pyo3::types::PyDict>> {
        self.ensure_editor()?;
        let opts = crate::redaction::RedactionOptions {
            scrub_metadata,
            remove_javascript,
            remove_embedded_files,
            ..crate::redaction::RedactionOptions::default()
        };
        let report = match self.editor {
            Some(ref mut editor) => editor
                .sanitize_document(opts)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
            None => return Err(PyRuntimeError::new_err("No document source available")),
        };
        let d = pyo3::types::PyDict::new(py);
        d.set_item("regions", report.regions)?;
        d.set_item("glyphs_removed", report.glyphs_removed)?;
        d.set_item("images_modified", report.images_modified)?;
        d.set_item("images_removed", report.images_removed)?;
        d.set_item("paths_pruned", report.paths_pruned)?;
        d.set_item("xobjects_specialized", report.xobjects_specialized)?;
        d.set_item("annotations_removed", report.annotations_removed)?;
        d.set_item("fonts_scrubbed", report.fonts_scrubbed)?;
        d.set_item("bytes_removed", report.bytes_removed)?;
        Ok(d.unbind())
    }

    /// Get page images info.
    fn page_images(&mut self, page: usize, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            let images = editor
                .get_page_images(page)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            let list = pyo3::types::PyList::empty(py);
            for img in images {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("name", &img.name)?;
                dict.set_item("x", img.bounds[0])?;
                dict.set_item("y", img.bounds[1])?;
                dict.set_item("width", img.bounds[2])?;
                dict.set_item("height", img.bounds[3])?;
                dict.set_item(
                    "matrix",
                    (
                        img.matrix[0],
                        img.matrix[1],
                        img.matrix[2],
                        img.matrix[3],
                        img.matrix[4],
                        img.matrix[5],
                    ),
                )?;
                list.append(dict)?;
            }
            Ok(list.into())
        } else {
            Err(PyRuntimeError::new_err("No editor initialized."))
        }
    }

    /// Reposition image.
    fn reposition_image(&mut self, page: usize, image_name: &str, x: f32, y: f32) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .reposition_image(page, image_name, x, y)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Resize image.
    fn resize_image(
        &mut self,
        page: usize,
        image_name: &str,
        width: f32,
        height: f32,
    ) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .resize_image(page, image_name, width, height)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Set image bounds.
    fn set_image_bounds(
        &mut self,
        page: usize,
        image_name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .set_image_bounds(page, image_name, x, y, width, height)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Clear image modifications.
    fn clear_image_modifications(&mut self, page: usize) {
        if let Some(ref mut editor) = self.editor {
            editor.clear_image_modifications(page);
        }
    }

    /// Has image modifications.
    fn has_image_modifications(&self, page: usize) -> bool {
        self.editor
            .as_ref()
            .is_some_and(|e| e.has_image_modifications(page))
    }

    /// Search text.
    #[pyo3(signature = (pattern, case_insensitive=false, literal=false, whole_word=false, max_results=0))]
    fn search(
        &mut self,
        py: Python<'_>,
        pattern: &str,
        case_insensitive: bool,
        literal: bool,
        whole_word: bool,
        max_results: usize,
    ) -> PyResult<Py<PyAny>> {
        use crate::search::{SearchOptions, TextSearcher};
        let opts = SearchOptions::new()
            .with_case_insensitive(case_insensitive)
            .with_literal(literal)
            .with_whole_word(whole_word)
            .with_max_results(max_results);
        let results = TextSearcher::search(&self.inner, pattern, &opts)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for r in results {
            let d = pyo3::types::PyDict::new(py);
            d.set_item("page", r.page)?;
            d.set_item("text", &r.text)?;
            d.set_item("x", r.bbox.x)?;
            d.set_item("y", r.bbox.y)?;
            d.set_item("width", r.bbox.width)?;
            d.set_item("height", r.bbox.height)?;
            list.append(d)?;
        }
        Ok(list.into())
    }

    /// Search page text.
    #[pyo3(signature = (page, pattern, case_insensitive=false, literal=false, whole_word=false, max_results=0))]
    fn search_page(
        &mut self,
        py: Python<'_>,
        page: usize,
        pattern: &str,
        case_insensitive: bool,
        literal: bool,
        whole_word: bool,
        max_results: usize,
    ) -> PyResult<Py<PyAny>> {
        use crate::search::{SearchOptions, TextSearcher};
        let opts = SearchOptions::new()
            .with_case_insensitive(case_insensitive)
            .with_literal(literal)
            .with_whole_word(whole_word)
            .with_max_results(max_results)
            .with_page_range(page, page);
        let results = TextSearcher::search(&self.inner, pattern, &opts)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for r in results {
            let d = pyo3::types::PyDict::new(py);
            d.set_item("page", r.page)?;
            d.set_item("text", &r.text)?;
            d.set_item("x", r.bbox.x)?;
            d.set_item("y", r.bbox.y)?;
            d.set_item("width", r.bbox.width)?;
            d.set_item("height", r.bbox.height)?;
            list.append(d)?;
        }
        Ok(list.into())
    }

    /// Build the search index for every page up front, instead of the lazy
    /// per-page build `search()`/`search_page()` otherwise do on first use.
    fn prepare_search(&self) -> PyResult<()> {
        self.inner
            .prepare_search()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Drop the cached search index, if any, freeing its memory.
    /// `search()`/`search_page()` rebuild it lazily on next use.
    fn clear_search_index(&self) {
        self.inner.clear_search_index();
    }

    /// Extract images metadata.
    #[pyo3(signature = (page, region=None))]
    fn extract_images(
        &mut self,
        py: Python<'_>,
        page: usize,
        region: Option<(f32, f32, f32, f32)>,
    ) -> PyResult<Py<PyAny>> {
        let res = if let Some(r) = region {
            self.inner
                .extract_images_in_rect(page, crate::geometry::Rect::new(r.0, r.1, r.2, r.3))
        } else {
            self.inner.extract_images(page)
        };
        let images = res.map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for img in &images {
            let d = pyo3::types::PyDict::new(py);
            d.set_item("width", img.width())?;
            d.set_item("height", img.height())?;
            d.set_item("color_space", format!("{:?}", img.color_space()))?;
            d.set_item("bits_per_component", img.bits_per_component())?;
            if let Some(b) = img.bbox() {
                d.set_item("bbox", (b.x, b.y, b.width, b.height))?;
            } else {
                d.set_item("bbox", py.None())?;
            }
            d.set_item("rotation", img.rotation_degrees())?;
            d.set_item("matrix", img.matrix())?;
            list.append(d)?;
        }
        Ok(list.into())
    }

    /// Extract tables.
    #[pyo3(signature = (page, region=None, table_settings=None))]
    fn extract_tables(
        &mut self,
        py: Python<'_>,
        page: usize,
        region: Option<(f32, f32, f32, f32)>,
        table_settings: Option<Bound<'_, pyo3::types::PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let config = table_settings_to_config(table_settings)?;
        let res = if let Some(r) = region {
            self.inner.extract_tables_in_rect_with_config(
                page,
                crate::geometry::Rect::new(r.0, r.1, r.2, r.3),
                config,
            )
        } else {
            self.inner.extract_tables_with_config(page, config)
        };
        let tables = res.map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for t in &tables {
            let d = pyo3::types::PyDict::new(py);
            d.set_item("col_count", t.col_count)?;
            d.set_item("row_count", t.rows.len())?;
            if let Some(b) = t.bbox {
                d.set_item("bbox", (b.x, b.y, b.width, b.height))?;
            } else {
                d.set_item("bbox", py.None())?;
            }
            d.set_item("has_header", t.has_header)?;
            let rows = pyo3::types::PyList::empty(py);
            for r in &t.rows {
                let rd = pyo3::types::PyDict::new(py);
                rd.set_item("is_header", r.is_header)?;
                let cells = pyo3::types::PyList::empty(py);
                for c in &r.cells {
                    let cd = pyo3::types::PyDict::new(py);
                    cd.set_item("text", &c.text)?;
                    if let Some(b) = c.bbox {
                        cd.set_item("bbox", (b.x, b.y, b.width, b.height))?;
                    }
                    cells.append(cd)?;
                }
                rd.set_item("cells", cells)?;
                rows.append(rd)?;
            }
            d.set_item("rows", rows)?;
            list.append(d)?;
        }
        Ok(list.into())
    }

    /// Extract text spans.
    ///
    /// Args:
    ///     page: Zero-based page index.
    ///     region: Optional (x, y, w, h) bounding box to restrict extraction.
    ///     reading_order: Optional reading order strategy. One of "top_to_bottom"
    ///         (default), "column_aware" (XY-Cut column detection), or "structure"
    ///         (follow the tagged structure tree for table cells).
    #[pyo3(signature = (page, region=None, reading_order=None))]
    fn extract_spans(
        &mut self,
        page: usize,
        region: Option<(f32, f32, f32, f32)>,
        reading_order: Option<&str>,
    ) -> PyResult<Vec<PyTextSpan>> {
        let order = match reading_order {
            Some("column_aware") => crate::document::ReadingOrder::ColumnAware,
            Some("structure") => crate::document::ReadingOrder::Structure,
            Some("top_to_bottom") | None => crate::document::ReadingOrder::TopToBottom,
            Some(other) => {
                return Err(PyRuntimeError::new_err(format!(
                    "Unknown reading_order '{}'. Expected 'top_to_bottom', 'column_aware', or 'structure'.",
                    other
                )));
            },
        };

        let res = if let Some(r) = region {
            self.inner.extract_spans_in_rect(
                page,
                crate::geometry::Rect::new(r.0, r.1, r.2, r.3),
                crate::layout::RectFilterMode::Intersects,
            )
        } else {
            self.inner.extract_spans_with_reading_order(page, order)
        };
        res.map(|spans| spans.into_iter().map(|s| PyTextSpan { inner: s }).collect())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Extract complete page text data in a single call.
    ///
    /// Returns a dict with spans, per-character data, and page dimensions.
    /// The chars are derived from spans using font-metric widths when available.
    ///
    /// Args:
    ///     page (int): Zero-based page index.
    ///     reading_order (str, optional): Reading order strategy. One of
    ///         "top_to_bottom" (default), "column_aware", or "structure".
    ///
    /// Returns:
    ///     dict: ``{"spans": [...], "chars": [...], "page_width": float, "page_height": float}``
    #[pyo3(signature = (page, reading_order=None))]
    fn extract_page_text(
        &mut self,
        py: Python<'_>,
        page: usize,
        reading_order: Option<&str>,
    ) -> PyResult<Py<PyAny>> {
        let order = match reading_order {
            Some("column_aware") => crate::document::ReadingOrder::ColumnAware,
            Some("structure") => crate::document::ReadingOrder::Structure,
            Some("top_to_bottom") | None => crate::document::ReadingOrder::TopToBottom,
            Some(other) => {
                return Err(PyRuntimeError::new_err(format!(
                    "Unknown reading_order '{}'. Expected 'top_to_bottom', 'column_aware', or 'structure'.",
                    other
                )));
            },
        };

        let page_text = self
            .inner
            .extract_page_text_with_options(page, order)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let dict = pyo3::types::PyDict::new(py);

        // Spans as list of PyTextSpan
        let spans_list: Vec<PyTextSpan> = page_text
            .spans
            .into_iter()
            .map(|s| PyTextSpan { inner: s })
            .collect();
        dict.set_item("spans", spans_list)?;

        // Chars as list of PyTextChar
        let chars_list: Vec<PyTextChar> = page_text
            .chars
            .into_iter()
            .map(|ch| PyTextChar { inner: ch })
            .collect();
        dict.set_item("chars", chars_list)?;

        dict.set_item("page_width", page_text.page_width)?;
        dict.set_item("page_height", page_text.page_height)?;

        Ok(dict.into())
    }

    /// Get document outline.
    fn get_outline(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let outline = self
            .inner
            .get_outline()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        match outline {
            Some(items) => Ok(Some(outline_items_to_py(py, &items)?)),
            None => Ok(None),
        }
    }

    /// Extract a page as structured typed regions (issue #536), returned as a
    /// JSON string (deserialize with `json.loads`): a `StructuredPage` whose
    /// `regions` each carry `kind`, `text`, `bbox`, `spans`, and `column_index`.
    fn extract_structured(&mut self, page: usize) -> PyResult<String> {
        let structured = self
            .inner
            .extract_structured(page)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        serde_json::to_string(&structured).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Get page annotations info.
    fn get_annotations(&mut self, py: Python<'_>, page: usize) -> PyResult<Py<PyAny>> {
        let annos = self
            .inner
            .get_annotations(page)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for a in &annos {
            let d = pyo3::types::PyDict::new(py);
            if let Some(ref s) = a.subtype {
                d.set_item("subtype", s)?;
            }
            if let Some(ref c) = a.contents {
                d.set_item("contents", c)?;
            }
            if let Some(r) = a.rect {
                d.set_item("rect", (r[0], r[1], r[2], r[3]))?;
            }
            if let Some(ref au) = a.author {
                d.set_item("author", au)?;
            }
            if let Some(ref d1) = a.creation_date {
                d.set_item("creation_date", d1)?;
            }
            if let Some(ref d2) = a.modification_date {
                d.set_item("modification_date", d2)?;
            }
            if let Some(ref s) = a.subject {
                d.set_item("subject", s)?;
            }
            if let Some(ref c) = a.color {
                if c.len() >= 3 {
                    d.set_item("color", (c[0], c[1], c[2]))?;
                }
            }
            if let Some(o) = a.opacity {
                d.set_item("opacity", o)?;
            }
            if let Some(ref f) = a.field_type {
                d.set_item("field_type", format!("{:?}", f))?;
            }
            if let Some(ref n) = a.field_name {
                d.set_item("field_name", n)?;
            }
            if let Some(ref v) = a.field_value {
                d.set_item("field_value", v)?;
            }
            if let Some(crate::annotations::LinkAction::Uri(ref u)) = a.action {
                d.set_item("action_uri", u)?;
            }
            list.append(d)?;
        }
        Ok(list.into())
    }

    /// Extract vector paths.
    #[pyo3(signature = (page, region=None))]
    fn extract_paths(
        &mut self,
        py: Python<'_>,
        page: usize,
        region: Option<(f32, f32, f32, f32)>,
    ) -> PyResult<Py<PyAny>> {
        let res = if let Some(r) = region {
            self.inner
                .extract_paths_in_rect(page, crate::geometry::Rect::new(r.0, r.1, r.2, r.3))
        } else {
            self.inner.extract_paths(page)
        };
        let paths = res.map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for p in &paths {
            list.append(path_to_py_dict(py, p)?)?;
        }
        Ok(list.into())
    }

    /// Extract rectangles.
    #[pyo3(signature = (page, region=None))]
    fn extract_rects(
        &mut self,
        py: Python<'_>,
        page: usize,
        region: Option<(f32, f32, f32, f32)>,
    ) -> PyResult<Py<PyAny>> {
        let res = if let Some(r) = region {
            self.inner
                .extract_rects_in_rect(page, crate::geometry::Rect::new(r.0, r.1, r.2, r.3))
        } else {
            self.inner.extract_rects(page)
        };
        let paths = res.map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for p in &paths {
            list.append(path_to_py_dict(py, p)?)?;
        }
        Ok(list.into())
    }

    /// Extract lines.
    #[pyo3(signature = (page, region=None))]
    fn extract_lines(
        &mut self,
        py: Python<'_>,
        page: usize,
        region: Option<(f32, f32, f32, f32)>,
    ) -> PyResult<Py<PyAny>> {
        let res = if let Some(r) = region {
            self.inner
                .extract_lines_in_rect(page, crate::geometry::Rect::new(r.0, r.1, r.2, r.3))
        } else {
            self.inner.extract_lines(page)
        };
        let paths = res.map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for p in &paths {
            list.append(path_to_py_dict(py, p)?)?;
        }
        Ok(list.into())
    }

    /// #517: cheap per-page text-vs-OCR classification → JSON
    /// `PageClassification` (the frozen cross-binding envelope —
    /// `json.loads` it). No OCR/rasterisation.
    fn classify_page(&mut self, page: usize) -> PyResult<String> {
        let c = self
            .inner
            .classify_page(page)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        serde_json::to_string(&c).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// #517: cheap whole-document classification → JSON
    /// `DocumentClassification` (per-page kinds + `pages_needing_ocr`).
    fn classify_document(&mut self) -> PyResult<String> {
        let c = self
            .inner
            .classify_document()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        serde_json::to_string(&c).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// #517: one-shot auto text extraction — auto-routes text-vs-OCR
    /// with graceful native fallback (never the opaque OCR error #513).
    fn extract_text_auto(&mut self, page: usize) -> PyResult<String> {
        self.inner
            .extract_text_auto(page)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// #517: rich per-page extraction → JSON `PageExtraction`
    /// (per-region bbox + typed reason; never bare-empty).
    /// `options_json` is `{}`-tolerant `AutoExtractOptions`; None/empty
    /// → defaults.
    #[pyo3(signature = (page, options_json=None))]
    fn extract_page_auto(&mut self, page: usize, options_json: Option<&str>) -> PyResult<String> {
        let opts = match options_json {
            Some(s) if !s.trim().is_empty() => serde_json::from_str(s)
                .map_err(|e| PyRuntimeError::new_err(format!("invalid options_json: {e}")))?,
            _ => crate::extractors::auto::AutoExtractOptions::default(),
        };
        let pe = crate::extractors::auto::AutoExtractor::with(opts)
            .extract_page(&self.inner, page)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        serde_json::to_string(&pe).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Extract text using OCR.
    #[pyo3(signature = (page, engine=None))]
    fn extract_text_ocr(
        &mut self,
        _py: Python<'_>,
        page: usize,
        engine: Option<Bound<'_, PyAny>>,
    ) -> PyResult<String> {
        #[cfg(feature = "ocr")]
        {
            let ocr_engine = if let Some(eng) = engine {
                Some(eng.extract::<PyRef<PyOcrEngine>>()?)
            } else {
                None
            };
            let engine_inner = ocr_engine.as_ref().map(|e| &e.inner);
            let options = crate::ocr::OcrExtractOptions::default();
            self.inner
                .extract_text_with_ocr(page, engine_inner, options)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        }
        #[cfg(not(feature = "ocr"))]
        {
            let _ = (engine, page);
            // #513: actionable, not opaque. The bare "OCR feature not
            // enabled." gave a stuck Windows user no path forward. Say
            // exactly how to fix it + point at the graceful auto path
            // that never errors.
            Err(PyRuntimeError::new_err(
                "OCR is unavailable: this pdf_oxide build was compiled \
                 without the `ocr` feature. Fix: install the OCR-enabled \
                 wheel with `pip install 'pdf_oxide[ocr]'`, then build an \
                 OcrEngine with detection/recognition/dictionary model \
                 paths and pass it as extract_text_ocr(page, engine) — see \
                 examples/python/09-new-features/ocr_scanned_pdf. For \
                 automatic text-vs-OCR routing that gracefully falls back \
                 to native text instead of raising, use \
                 extract_text_auto(page).",
            ))
        }
    }

    /// Python wrapper: returns True if the page has a
    /// text layer; False for image-only / genuinely-empty pages.
    /// Callers route image-only pages to OCR.
    fn has_text_layer(&self, page: usize) -> PyResult<bool> {
        self.inner
            .has_text_layer(page)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Python wrapper: returns the document's /P
    /// permission flags as a dict (None for unencrypted PDFs). Keys
    /// match the PdfPermissions struct fields. Per PDF spec §7.6.3.2
    /// the flags are advisory; pdf_oxide does not enforce them.
    fn permissions(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self.inner.permissions() {
            None => Ok(py.None()),
            Some(p) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("print_low_res", p.print_low_res)?;
                dict.set_item("modify", p.modify)?;
                dict.set_item("copy", p.copy)?;
                dict.set_item("annotate", p.annotate)?;
                dict.set_item("fill_forms", p.fill_forms)?;
                dict.set_item("accessibility", p.accessibility)?;
                dict.set_item("assemble", p.assemble)?;
                dict.set_item("print_high_res", p.print_high_res)?;
                dict.set_item("raw_p", p.raw_p)?;
                Ok(dict.into_any().unbind())
            },
        }
    }

    /// Python wrapper: returns the
    /// document's accumulated structured warnings as a list of dicts.
    /// Each entry has `category`, `page`, `message`, `spec_section`.
    /// Companion to the Python per-target log-level downgrade — gives
    /// callers a structured opt-in surface instead of stderr text.
    ///
    /// Named `structured_warnings` (not `flatten_warnings`) to avoid
    /// collision with the existing `DocumentEditor::flatten_warnings`
    /// accessor for form-flattening warnings; the Rust side was
    /// renamed in for the same reason.
    fn structured_warnings(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let warnings = self.inner.structured_warnings();
        let list = pyo3::types::PyList::empty(py);
        for w in warnings {
            let entry = pyo3::types::PyDict::new(py);
            entry.set_item("category", w.category.as_str())?;
            entry.set_item("page", w.page)?;
            entry.set_item("message", &w.message)?;
            entry.set_item("spec_section", w.spec_section)?;
            list.append(entry)?;
        }
        Ok(list.into_any().unbind())
    }

    /// Get form fields.
    fn get_form_fields(&mut self) -> PyResult<Vec<PyFormField>> {
        use crate::extractors::forms::FormExtractor;
        let fields = FormExtractor::extract_fields(&self.inner)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(fields
            .into_iter()
            .map(|f| PyFormField { inner: f })
            .collect())
    }

    /// Get specific form field value.
    fn get_form_field_value(&mut self, name: &str, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        let value = editor
            .get_form_field_value(name)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        match value {
            Some(v) => form_field_value_to_python(&v, py),
            None => Ok(py.None()),
        }
    }

    /// Set form field value.
    fn set_form_field_value(&mut self, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        let field_value = python_to_form_field_value(value)?;
        editor
            .set_form_field_value(name, field_value)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Has XFA.
    fn has_xfa(&mut self) -> PyResult<bool> {
        use crate::xfa::XfaExtractor;
        XfaExtractor::has_xfa(&mut self.inner).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Export form data.
    #[pyo3(signature = (path, format="fdf"))]
    fn export_form_data(&mut self, path: &str, format: &str) -> PyResult<()> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        match format {
            "fdf" => editor
                .export_form_data_fdf(path)
                .map_err(|e| PyRuntimeError::new_err(e.to_string())),
            "xfdf" => editor
                .export_form_data_xfdf(path)
                .map_err(|e| PyRuntimeError::new_err(e.to_string())),
            _ => Err(PyRuntimeError::new_err("Unknown format.")),
        }
    }

    /// Extract image bytes as PNG.
    fn extract_image_bytes(&mut self, py: Python<'_>, page: usize) -> PyResult<Py<PyAny>> {
        let images = self
            .inner
            .extract_images(page)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for img in &images {
            let png_data = img
                .to_png_bytes()
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            let d = pyo3::types::PyDict::new(py);
            d.set_item("width", img.width())?;
            d.set_item("height", img.height())?;
            d.set_item("format", "png")?;
            d.set_item("data", pyo3::types::PyBytes::new(py, &png_data))?;
            list.append(d)?;
        }
        Ok(list.into())
    }

    /// Flatten forms.
    fn flatten_forms(&mut self) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .flatten_forms()
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Flatten page forms.
    fn flatten_forms_on_page(&mut self, page: usize) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .flatten_forms_on_page(page)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Return warnings collected during the last form-flattening save.
    ///
    /// Each entry names a widget field that had no ``/AP`` appearance stream;
    /// flattening such a field produces a blank rectangle.
    fn flatten_warnings(&self) -> Vec<String> {
        self.editor
            .as_ref()
            .map(|e| e.flatten_warnings().to_vec())
            .unwrap_or_default()
    }

    /// Merge from source.
    fn merge_from(&mut self, source: &Bound<'_, PyAny>) -> PyResult<usize> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        if let Ok(path) = source.extract::<String>() {
            editor
                .merge_from(&path)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else if let Ok(data) = source.extract::<Vec<u8>>() {
            editor
                .merge_from_bytes(&data)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        } else {
            Err(PyRuntimeError::new_err("Invalid source."))
        }
    }

    /// Embed file.
    fn embed_file(&mut self, name: &str, data: &Bound<'_, PyBytes>) -> PyResult<()> {
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            editor
                .embed_file(name, data.as_bytes().to_vec())
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Get page labels.
    fn page_labels(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        use crate::extractors::page_labels::PageLabelExtractor;
        let labels = PageLabelExtractor::extract(&self.inner)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for l in &labels {
            let d = pyo3::types::PyDict::new(py);
            d.set_item("start_page", l.start_page)?;
            d.set_item("style", format!("{:?}", l.style))?;
            if let Some(ref p) = l.prefix {
                d.set_item("prefix", p)?;
            } else {
                d.set_item("prefix", py.None())?;
            }
            d.set_item("start_value", l.start_value)?;
            list.append(d)?;
        }
        Ok(list.into())
    }

    /// Get XMP metadata.
    fn xmp_metadata(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        use crate::extractors::xmp::XmpExtractor;
        let meta = XmpExtractor::extract(&self.inner)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        match meta {
            Some(xmp) => {
                let d = pyo3::types::PyDict::new(py);
                if let Some(ref t) = xmp.dc_title {
                    d.set_item("dc_title", t)?;
                }
                if !xmp.dc_creator.is_empty() {
                    d.set_item("dc_creator", &xmp.dc_creator)?;
                }
                if let Some(ref desc) = xmp.dc_description {
                    d.set_item("dc_description", desc)?;
                }
                if !xmp.dc_subject.is_empty() {
                    d.set_item("dc_subject", &xmp.dc_subject)?;
                }
                if let Some(ref l) = xmp.dc_language {
                    d.set_item("dc_language", l)?;
                }
                if let Some(ref t) = xmp.xmp_creator_tool {
                    d.set_item("xmp_creator_tool", t)?;
                }
                if let Some(ref d1) = xmp.xmp_create_date {
                    d.set_item("xmp_create_date", d1)?;
                }
                if let Some(ref d2) = xmp.xmp_modify_date {
                    d.set_item("xmp_modify_date", d2)?;
                }
                if let Some(ref p) = xmp.pdf_producer {
                    d.set_item("pdf_producer", p)?;
                }
                if let Some(ref k) = xmp.pdf_keywords {
                    d.set_item("pdf_keywords", k)?;
                }
                Ok(d.into())
            },
            None => Ok(py.None()),
        }
    }

    // ==========================================
    // Validation — PDF/A, PDF/UA, PDF/X
    // ==========================================

    /// Validate PDF/A compliance.
    /// Returns a dict with 'valid', 'level', 'errors', 'warnings' keys.
    #[pyo3(signature = (level="1b"))]
    fn validate_pdf_a(&mut self, py: Python<'_>, level: &str) -> PyResult<Py<PyAny>> {
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
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown PDF/A level: '{}'. Use 1a, 1b, 2a, 2b, 2u, 3a, 3b, 3u",
                    level
                )))
            },
        };
        let result = validate_pdf_a(&mut self.inner, pdf_level)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let d = pyo3::types::PyDict::new(py);
        d.set_item("valid", result.errors.is_empty())?;
        d.set_item("level", level)?;
        let errors: Vec<String> = result.errors.iter().map(|e| e.to_string()).collect();
        let warnings: Vec<String> = result.warnings.iter().map(|w| w.to_string()).collect();
        d.set_item("errors", errors)?;
        d.set_item("warnings", warnings)?;
        Ok(d.into())
    }

    /// Convert document to PDF/A archival format in-place.
    /// `level` is one of: 1a, 1b, 2a, 2b, 2u, 3a, 3b, 3u.
    /// Returns a dict with 'success' (bool), 'actions' (list[str]), 'errors' (list[str]).
    #[pyo3(signature = (level="2b"))]
    fn convert_to_pdf_a(&mut self, py: Python<'_>, level: &str) -> PyResult<Py<PyAny>> {
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
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown PDF/A level: '{}'. Use 1a, 1b, 2a, 2b, 2u, 3a, 3b, 3u",
                    level
                )))
            },
        };
        let result = convert_to_pdf_a(&mut self.inner, pdf_level)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        // Sync raw_bytes so that to_bytes() sees the updated document,
        // drop any stale editor that was opened from the original bytes.
        self.raw_bytes = Some(self.inner.source_bytes.to_vec());
        self.path = None;
        self.editor = None;
        let d = pyo3::types::PyDict::new(py);
        d.set_item("success", result.success)?;
        d.set_item("level", level)?;
        let actions: Vec<String> = result
            .actions
            .iter()
            .map(|a| a.description.clone())
            .collect();
        let errors: Vec<String> = result.errors.iter().map(|e| e.reason.clone()).collect();
        d.set_item("actions", actions)?;
        d.set_item("errors", errors)?;
        Ok(d.into())
    }

    /// Validate PDF/UA accessibility compliance.
    /// Returns a dict with 'valid', 'errors', 'warnings' keys.
    fn validate_pdf_ua(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        use crate::compliance::pdf_ua::validate_pdf_ua;
        let result = validate_pdf_ua(&mut self.inner, crate::compliance::pdf_ua::PdfUaLevel::Ua1)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let d = pyo3::types::PyDict::new(py);
        d.set_item("valid", result.errors.is_empty())?;
        let errors: Vec<String> = result.errors.iter().map(|e| e.to_string()).collect();
        let warnings: Vec<String> = result.warnings.iter().map(|w| w.to_string()).collect();
        d.set_item("errors", errors)?;
        d.set_item("warnings", warnings)?;
        Ok(d.into())
    }

    /// Validate PDF/X print compliance.
    /// Returns a dict with 'valid', 'level', 'errors', 'warnings' keys.
    #[pyo3(signature = (level="1a_2001"))]
    fn validate_pdf_x(&mut self, py: Python<'_>, level: &str) -> PyResult<Py<PyAny>> {
        use crate::compliance::pdf_x::types::PdfXLevel;
        use crate::compliance::pdf_x::validator::validate_pdf_x;
        let pdf_level = match level {
            "1a_2001" => PdfXLevel::X1a2001,
            "3_2002" => PdfXLevel::X32002,
            "4" => PdfXLevel::X4,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown PDF/X level: '{}'. Use 1a_2001, 3_2002, 4",
                    level
                )))
            },
        };
        let result = validate_pdf_x(&mut self.inner, pdf_level)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let d = pyo3::types::PyDict::new(py);
        d.set_item("valid", result.errors.is_empty())?;
        d.set_item("level", level)?;
        let errors: Vec<String> = result.errors.iter().map(|e| e.to_string()).collect();
        let warnings: Vec<String> = result.warnings.iter().map(|w| w.to_string()).collect();
        d.set_item("errors", errors)?;
        d.set_item("warnings", warnings)?;
        Ok(d.into())
    }

    // ==========================================
    // Page Operations — extract, delete, move
    // ==========================================

    /// Extract page range to a new PDF file.
    /// `pages` is a list of 0-based page indices.
    fn extract_pages(&mut self, pages: Vec<usize>, output: &str) -> PyResult<()> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        editor
            .extract_pages(&pages, output)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Extract a subset of pages and return them as PDF bytes.
    /// `pages` is a list of 0-based indices to keep. The source document is not modified.
    ///
    /// # Example
    ///
    /// ```python
    /// from itertools import batched
    /// doc = PdfDocument.from_bytes(pdf_bytes)
    /// for chunk in batched(range(doc.page_count()), 50):
    ///     chunk_bytes = doc.extract_pages_to_bytes(list(chunk))
    /// ```
    fn extract_pages_to_bytes<'py>(
        &mut self,
        py: Python<'py>,
        pages: Vec<usize>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        let bytes = editor
            .extract_pages_to_bytes(&pages)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Extract several non-overlapping page ranges in one call. Each range is
    /// `(start, end)` interpreted as `[start, end)`. Returns a list of `bytes`
    /// objects, one per range, in the same order.
    ///
    /// # Example
    ///
    /// ```python
    /// doc = PdfDocument.from_bytes(pdf_bytes)
    /// n = doc.page_count()
    /// ranges = [(i, min(i + 3000, n)) for i in range(0, n, 3000)]
    /// chunks = doc.extract_page_ranges_to_bytes(ranges)
    /// ```
    fn extract_page_ranges_to_bytes<'py>(
        &mut self,
        py: Python<'py>,
        ranges: Vec<(usize, usize)>,
    ) -> PyResult<Vec<Bound<'py, PyBytes>>> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        let chunks = editor
            .extract_page_ranges_to_bytes(&ranges)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(chunks.into_iter().map(|b| PyBytes::new(py, &b)).collect())
    }

    /// Restrict this document to the listed pages, in the order given.
    /// Equivalent to PyMuPDF's `doc.select(page_list)`.
    /// Subsequent `save_to_bytes()` / `save()` produces a PDF containing only
    /// the selected pages, with garbage-collected resources.
    fn select_pages(&mut self, pages: Vec<usize>) -> PyResult<()> {
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        editor
            .select_pages(&pages)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Delete a page by index (0-based).
    fn delete_page(&mut self, index: usize) -> PyResult<()> {
        use crate::editor::EditableDocument;
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        editor
            .remove_page(index)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Move a page from one position to another (0-based indices).
    fn move_page(&mut self, from_index: usize, to_index: usize) -> PyResult<()> {
        use crate::editor::EditableDocument;
        self.ensure_editor()?;
        let editor = self.editor.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Internal error: editor missing after initialization")
        })?;
        editor
            .move_page(from_index, to_index)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Create a flattened PDF where each page is rendered as an image.
    /// This "burns in" all annotations, form fields, and overlays into
    /// a flat raster representation. Useful for redaction, archival,
    /// or ensuring consistent visual output across viewers.
    ///
    /// Returns the flattened PDF as bytes.
    #[pyo3(signature = (dpi=150))]
    fn flatten_to_images(&mut self, py: Python<'_>, dpi: u32) -> PyResult<Py<PyBytes>> {
        #[cfg(feature = "rendering")]
        {
            let bytes = crate::rendering::flatten_to_images(&self.inner, dpi)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            Ok(PyBytes::new(py, &bytes).unbind())
        }
        #[cfg(not(feature = "rendering"))]
        {
            Err(PyRuntimeError::new_err("Rendering feature not enabled"))
        }
    }

    fn __len__(&mut self) -> PyResult<usize> {
        // : `page_count` is now a #[getter] returning PyPageCount,
        // so call the inner Rust method directly here for the unsigned
        // int we need.
        self.inner
            .page_count()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to get page count: {}", e)))
    }

    fn __getitem__(slf: Py<Self>, py: Python<'_>, index: isize) -> PyResult<PyDocPage> {
        let count = slf
            .borrow_mut(py)
            .inner
            .page_count()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to get page count: {}", e)))?
            as isize;
        let idx = if index < 0 { count + index } else { index };
        if idx < 0 || idx >= count {
            return Err(pyo3::exceptions::PyIndexError::new_err("page index out of range"));
        }
        Ok(PyDocPage {
            doc: slf,
            page_index: idx as usize,
        })
    }

    fn __iter__(slf: Py<Self>, py: Python<'_>) -> PyResult<PyDocPageIter> {
        let count = slf
            .borrow_mut(py)
            .inner
            .page_count()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to get page count: {}", e)))?;
        Ok(PyDocPageIter {
            doc: slf,
            index: 0,
            count,
        })
    }

    /// Iterable view of all pages in this document. Equivalent to
    /// `iter(doc)` but explicitly named for discoverability — issue
    /// #447 — and matches the C# `doc.Pages` / Go `doc.Pages()`
    /// surface that already exists in those bindings. Each iteration
    /// yields a `Page` object that exposes per-page extraction APIs.
    ///
    /// Example:
    ///     for page in doc.pages:
    ///         print(page.text[:80])
    #[getter]
    fn pages(slf: Py<Self>, py: Python<'_>) -> PyResult<PyDocPageIter> {
        let count = slf
            .borrow_mut(py)
            .inner
            .page_count()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to get page count: {}", e)))?;
        Ok(PyDocPageIter {
            doc: slf,
            index: 0,
            count,
        })
    }

    fn __repr__(&self) -> String {
        format!("PdfDocument(version={}.{})", self.inner.version().0, self.inner.version().1)
    }
}

/// The credential-taking entry points live in their own `#[pymethods]` block.
///
/// Static analysis reads a `#[pymethods]` block's argument extraction as one
/// macro expansion, so a `password` argument here would taint every other
/// method's arguments in the same block — a page index handed to the library
/// and written to a debug log then reads as cleartext logging of a secret
/// (30 findings of that shape on one release). Keeping the constructors and
/// `authenticate` apart makes the analysis precise: only what a password
/// actually reaches is checked against it.
#[pymethods]
impl PyPdfDocument {
    /// Open a PDF file.
    ///
    /// Open a PDF file, optionally with a password for encrypted documents.
    ///
    /// Args:
    ///     path (str | pathlib.Path): Path to the PDF file
    ///     password (str, optional): Password for encrypted PDFs
    #[new]
    #[pyo3(signature = (path, password=None))]
    #[allow(unused_mut)]
    fn new(path: PathBuf, password: Option<&str>) -> PyResult<Self> {
        let mut doc = RustPdfDocument::open(&path)
            .map_err(|e| PyIOError::new_err(format!("Failed to open PDF: {}", e)))?;

        if let Some(pw) = password {
            let ok = doc
                .authenticate(pw.as_bytes())
                .map_err(|e| PyRuntimeError::new_err(format!("Authentication failed: {}", e)))?;
            if !ok {
                return Err(PyRuntimeError::new_err("Authentication failed: wrong password"));
            }
        }

        let path_str = path.to_string_lossy().into_owned();
        Ok(PyPdfDocument {
            inner: doc,
            path: Some(path_str),
            raw_bytes: None,
            editor: None,
        })
    }

    /// Open a PDF from bytes, optionally with a password.
    #[staticmethod]
    #[pyo3(signature = (data, password=None))]
    #[allow(unused_mut)]
    fn from_bytes(data: &Bound<'_, PyBytes>, password: Option<&str>) -> PyResult<Self> {
        let bytes = data.as_bytes().to_vec();
        let mut doc = RustPdfDocument::from_bytes(bytes.clone())
            .map_err(|e| PyIOError::new_err(format!("Failed to open PDF from bytes: {}", e)))?;

        if let Some(pw) = password {
            let ok = doc
                .authenticate(pw.as_bytes())
                .map_err(|e| PyRuntimeError::new_err(format!("Authentication failed: {}", e)))?;
            if !ok {
                return Err(PyRuntimeError::new_err("Authentication failed: wrong password"));
            }
        }

        Ok(PyPdfDocument {
            inner: doc,
            path: None,
            raw_bytes: Some(bytes),
            editor: None,
        })
    }

    /// Authenticate with a password.
    fn authenticate(&mut self, password: &str) -> PyResult<bool> {
        self.inner
            .authenticate(password.as_bytes())
            .map_err(|e| PyRuntimeError::new_err(format!("Authentication failed: {}", e)))
    }

    /// Save encrypted PDF.
    #[pyo3(signature = (path, user_password, owner_password=None, allow_print=true, allow_copy=true, allow_modify=true, allow_annotate=true))]
    fn save_encrypted(
        &mut self,
        path: &str,
        user_password: &str,
        owner_password: Option<&str>,
        allow_print: bool,
        allow_copy: bool,
        allow_modify: bool,
        allow_annotate: bool,
    ) -> PyResult<()> {
        use crate::editor::{
            EditableDocument, EncryptionAlgorithm, EncryptionConfig, Permissions, SaveOptions,
        };
        self.ensure_editor()?;
        if let Some(ref mut editor) = self.editor {
            let owner_pwd = owner_password.unwrap_or(user_password);
            let permissions = Permissions {
                print: allow_print,
                print_high_quality: allow_print,
                modify: allow_modify,
                copy: allow_copy,
                annotate: allow_annotate,
                fill_forms: allow_annotate,
                accessibility: true,
                assemble: allow_modify,
            };
            let config = EncryptionConfig::new(user_password, owner_pwd)
                .with_algorithm(EncryptionAlgorithm::Aes256)
                .with_permissions(permissions);
            let options = SaveOptions::with_encryption(config);
            editor
                .save_with_options(path, options)
                .map_err(|e| PyIOError::new_err(format!("Failed to save encrypted PDF: {}", e)))
        } else {
            Err(PyRuntimeError::new_err("No editor initialized."))
        }
    }

    /// Return the (possibly edited) document as encrypted bytes.
    ///
    /// Equivalent to `save_encrypted` but returns bytes instead of writing to disk.
    /// Useful for in-memory pipelines where writing a temporary file is undesirable.
    #[pyo3(signature = (user_password, owner_password=None, allow_print=true, allow_copy=true, allow_modify=true, allow_annotate=true))]
    fn to_bytes_encrypted<'py>(
        &mut self,
        py: Python<'py>,
        user_password: &str,
        owner_password: Option<&str>,
        allow_print: bool,
        allow_copy: bool,
        allow_modify: bool,
        allow_annotate: bool,
    ) -> PyResult<Bound<'py, PyBytes>> {
        use crate::editor::{EncryptionAlgorithm, EncryptionConfig, Permissions, SaveOptions};
        self.ensure_editor()?;
        let editor = self
            .editor
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("No editor initialized."))?;
        let owner_pwd = owner_password.unwrap_or(user_password);
        let permissions = Permissions {
            print: allow_print,
            print_high_quality: allow_print,
            modify: allow_modify,
            copy: allow_copy,
            annotate: allow_annotate,
            fill_forms: allow_annotate,
            accessibility: true,
            assemble: allow_modify,
        };
        let config = EncryptionConfig::new(user_password, owner_pwd)
            .with_algorithm(EncryptionAlgorithm::Aes256)
            .with_permissions(permissions);
        let options = SaveOptions::with_encryption(config);
        let bytes = editor
            .save_to_bytes_with_options(options)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to encrypt PDF: {}", e)))?;
        Ok(PyBytes::new(py, &bytes))
    }
}

/// Int-like callable wrapper returned by `PdfDocument.page_count` for
/// backward-compatibility. Behaves as an int via `__int__` /
/// `__index__` / comparison protocols. Also callable via `__call__` so
/// the method-call shape (`doc.page_count()`) still works.
///
/// New code should use the attribute form (`doc.page_count`); the
/// method-call form is deprecated and will be removed in v0.4.0.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "_PageCount")]
pub struct PyPageCount {
    value: usize,
}

#[pymethods]
impl PyPageCount {
    /// Method-call form (v0.3.54 shape): `doc.page_count()` returns
    /// the int. Deprecated; use the attribute form.
    fn __call__(&self) -> usize {
        self.value
    }

    /// `int(doc.page_count)` returns the int.
    fn __int__(&self) -> usize {
        self.value
    }

    /// `range(doc.page_count)` works via the index protocol. This is
    /// the v0.3.6 shape that broke in.
    fn __index__(&self) -> usize {
        self.value
    }

    fn __repr__(&self) -> String {
        format!("{}", self.value)
    }

    fn __str__(&self) -> String {
        format!("{}", self.value)
    }

    /// `doc.page_count == 5` works against `int`. Also compares with
    /// another `_PageCount` instance.
    fn __eq__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        if let Ok(n) = other.extract::<usize>() {
            return Ok(self.value == n);
        }
        if let Ok(n) = other.extract::<i64>() {
            return Ok(n >= 0 && self.value == n as usize);
        }
        if let Ok(other_pc) = other.extract::<PyRef<PyPageCount>>() {
            return Ok(self.value == other_pc.value);
        }
        Ok(false)
    }

    fn __ne__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        Ok(!self.__eq__(other)?)
    }

    fn __lt__(&self, other: usize) -> bool {
        self.value < other
    }
    fn __le__(&self, other: usize) -> bool {
        self.value <= other
    }
    fn __gt__(&self, other: usize) -> bool {
        self.value > other
    }
    fn __ge__(&self, other: usize) -> bool {
        self.value >= other
    }

    fn __hash__(&self) -> usize {
        self.value
    }

    /// Arithmetic compatibility — callers may write
    /// `doc.page_count - 1` to get the last page index.
    fn __sub__(&self, other: usize) -> usize {
        self.value.saturating_sub(other)
    }

    fn __add__(&self, other: usize) -> usize {
        self.value + other
    }

    fn __bool__(&self) -> bool {
        self.value != 0
    }
}

/// Iterator over pages of a PdfDocument.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "PdfDocumentIter")]
pub struct PyDocPageIter {
    doc: Py<PyPdfDocument>,
    index: usize,
    count: usize,
}

#[pymethods]
impl PyDocPageIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> Option<PyDocPage> {
        if self.index >= self.count {
            return None;
        }
        let page = PyDocPage {
            doc: self.doc.clone_ref(py),
            page_index: self.index,
        };
        self.index += 1;
        Some(page)
    }
}

/// A single page of a PdfDocument, providing lazy access to all page-level operations.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Page", subclass)]
pub struct PyDocPage {
    doc: Py<PyPdfDocument>,
    page_index: usize,
}

#[pymethods]
impl PyDocPage {
    #[getter]
    fn index(&self) -> usize {
        self.page_index
    }

    #[getter]
    fn bbox(&self, py: Python<'_>) -> PyResult<(f32, f32, f32, f32)> {
        self.doc
            .borrow_mut(py)
            .inner
            .get_page_media_box(self.page_index)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    #[getter]
    fn width(&self, py: Python<'_>) -> PyResult<f32> {
        self.bbox(py).map(|(llx, _, urx, _)| urx - llx)
    }

    #[getter]
    fn height(&self, py: Python<'_>) -> PyResult<f32> {
        self.bbox(py).map(|(_, lly, _, ury)| ury - lly)
    }

    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        self.doc
            .borrow_mut(py)
            .extract_text(self.page_index, None, None, None, true, true)
    }

    #[getter]
    fn chars(&self, py: Python<'_>) -> PyResult<Vec<PyTextChar>> {
        self.doc
            .borrow_mut(py)
            .extract_chars(self.page_index, None, None, None)
    }

    #[getter]
    fn words(&self, py: Python<'_>) -> PyResult<Vec<PyWord>> {
        // `include_artifacts=true` mirrors the public `PdfDocument.extract_words`
        // default. If that default is ever flipped (spec-correct exclude), update
        // this getter (and the `lines` getter below) to match — there's no shared
        // constant because pyo3 signatures need literal bools.
        self.doc
            .borrow_mut(py)
            .extract_words(py, self.page_index, true, None, None, None)
    }

    #[getter]
    fn lines(&self, py: Python<'_>) -> PyResult<Vec<PyTextLine>> {
        self.doc.borrow_mut(py).extract_text_lines(
            py,
            self.page_index,
            true,
            None,
            None,
            None,
            None,
        )
    }

    #[getter]
    fn spans(&self, py: Python<'_>) -> PyResult<Vec<PyTextSpan>> {
        self.doc
            .borrow_mut(py)
            .extract_spans(self.page_index, None, None)
    }

    #[getter]
    fn tables(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.doc
            .borrow_mut(py)
            .extract_tables(py, self.page_index, None, None)
    }

    #[getter]
    fn images(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.doc
            .borrow_mut(py)
            .extract_images(py, self.page_index, None)
    }

    #[getter]
    fn annotations(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.doc.borrow_mut(py).get_annotations(py, self.page_index)
    }

    #[getter]
    fn paths(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.doc
            .borrow_mut(py)
            .extract_paths(py, self.page_index, None)
    }

    #[pyo3(signature = (preserve_layout=false, detect_headings=true, include_images=false, image_output_dir=None, embed_images=true, include_form_fields=true, include_artifacts=true))]
    #[allow(clippy::too_many_arguments)]
    fn markdown(
        &self,
        py: Python<'_>,
        preserve_layout: bool,
        detect_headings: bool,
        include_images: bool,
        image_output_dir: Option<String>,
        embed_images: bool,
        include_form_fields: bool,
        include_artifacts: bool,
    ) -> PyResult<String> {
        self.doc.borrow_mut(py).to_markdown(
            self.page_index,
            preserve_layout,
            detect_headings,
            include_images,
            image_output_dir,
            embed_images,
            include_form_fields,
            include_artifacts,
        )
    }

    #[pyo3(signature = (preserve_layout=false, detect_headings=true, include_images=false, image_output_dir=None, include_artifacts=true))]
    fn plain_text(
        &self,
        py: Python<'_>,
        preserve_layout: bool,
        detect_headings: bool,
        include_images: bool,
        image_output_dir: Option<String>,
        include_artifacts: bool,
    ) -> PyResult<String> {
        self.doc.borrow_mut(py).to_plain_text(
            self.page_index,
            preserve_layout,
            detect_headings,
            include_images,
            image_output_dir,
            include_artifacts,
        )
    }

    #[pyo3(signature = (preserve_layout=false, detect_headings=true, include_images=false, image_output_dir=None, embed_images=true, include_form_fields=true))]
    fn html(
        &self,
        py: Python<'_>,
        preserve_layout: bool,
        detect_headings: bool,
        include_images: bool,
        image_output_dir: Option<String>,
        embed_images: bool,
        include_form_fields: bool,
    ) -> PyResult<String> {
        self.doc.borrow_mut(py).to_html(
            self.page_index,
            preserve_layout,
            detect_headings,
            include_images,
            image_output_dir,
            embed_images,
            include_form_fields,
        )
    }

    #[pyo3(signature = (
        dpi=None,
        format=None,
        background=None,
        transparent=false,
        render_annotations=None,
        jpeg_quality=None,
        excluded_layers=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        py: Python<'_>,
        dpi: Option<u32>,
        format: Option<&str>,
        background: Option<(f32, f32, f32, f32)>,
        transparent: bool,
        render_annotations: Option<bool>,
        jpeg_quality: Option<u8>,
        excluded_layers: Option<Vec<String>>,
    ) -> PyResult<Vec<u8>> {
        self.doc.borrow_mut(py).render_page(
            self.page_index,
            dpi,
            format,
            background,
            transparent,
            render_annotations,
            jpeg_quality,
            excluded_layers,
        )
    }

    /// Render this page as raw premultiplied RGBA8888 pixels.
    /// Delegates to :meth:`PdfDocument.render_pixmap`. See that method for
    /// full documentation.
    #[pyo3(signature = (dpi=None))]
    fn render_pixmap(&self, py: Python<'_>, dpi: Option<u32>) -> PyResult<Py<pyo3::PyAny>> {
        self.doc
            .borrow_mut(py)
            .render_pixmap(py, self.page_index, dpi)
    }

    #[pyo3(signature = (pattern, case_insensitive=false, literal=false, whole_word=false, max_results=100))]
    fn search(
        &self,
        py: Python<'_>,
        pattern: &str,
        case_insensitive: bool,
        literal: bool,
        whole_word: bool,
        max_results: usize,
    ) -> PyResult<Py<PyAny>> {
        self.doc.borrow_mut(py).search_page(
            py,
            self.page_index,
            pattern,
            case_insensitive,
            literal,
            whole_word,
            max_results,
        )
    }

    fn region(&self, py: Python<'_>, x: f32, y: f32, width: f32, height: f32) -> PyPdfPageRegion {
        PyPdfPageRegion {
            doc: self.doc.clone_ref(py),
            page_index: self.page_index,
            region: crate::geometry::Rect::new(x, y, width, height),
        }
    }

    fn __repr__(&self) -> String {
        format!("Page(index={})", self.page_index)
    }
}

/// A form field extracted from a PDF AcroForm.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "FormField")]
pub struct PyFormField {
    inner: RustFormField,
}

#[pymethods]
impl PyFormField {
    #[allow(clippy::misnamed_getters)]
    #[getter]
    fn name(&self) -> &str {
        &self.inner.full_name
    }
    #[getter]
    fn field_type(&self) -> &str {
        match &self.inner.field_type {
            RustFieldType::Text => "text",
            RustFieldType::Button => "button",
            RustFieldType::Choice => "choice",
            RustFieldType::Signature => "signature",
            RustFieldType::Unknown(_) => "unknown",
        }
    }
    #[getter]
    fn value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        field_value_to_python(&self.inner.value, py)
    }
    #[getter]
    fn tooltip(&self) -> Option<&str> {
        self.inner.tooltip.as_deref()
    }
    #[getter]
    fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        self.inner.bounds.map(|b| (b[0], b[1], b[2], b[3]))
    }
    #[getter]
    fn flags(&self) -> Option<u32> {
        self.inner.flags
    }
    #[getter]
    fn max_length(&self) -> Option<u32> {
        self.inner.max_length
    }
    #[getter]
    fn is_readonly(&self) -> bool {
        self.inner
            .flags
            .is_some_and(|f| f & field_flags::READ_ONLY != 0)
    }
    #[getter]
    fn is_required(&self) -> bool {
        self.inner
            .flags
            .is_some_and(|f| f & field_flags::REQUIRED != 0)
    }
    fn __repr__(&self) -> String {
        format!("FormField(name=\"{}\", type=\"{}\")", self.inner.full_name, self.field_type())
    }
}

/// Emit a `DeprecationWarning` when one of the named kwargs is supplied
/// to a method whose advanced surface is on the way out (issue #457
/// Step 5). Best effort: any error from the `warnings` module is
/// swallowed — the caller still gets the usable result with the
/// deprecated kwarg honored.
fn warn_deprecated_kwargs(py: Python<'_>, method: &str, kwargs: &[&str]) {
    let msg = format!(
        "{}() kwargs {:?} are deprecated and will move to a separate \
         {}_advanced method in a future release. The default API is now \
         knob-free; pass the kwarg only if you genuinely need to override \
         the spec-correct default.",
        method, kwargs, method,
    );
    let _: PyResult<()> = (|| {
        let warnings = py.import("warnings")?;
        let deprecation = py.import("builtins")?.getattr("DeprecationWarning")?;
        warnings.call_method1("warn", (msg, deprecation))?;
        Ok(())
    })();
}

fn field_value_to_python(value: &RustFieldValue, py: Python<'_>) -> PyResult<Py<PyAny>> {
    match value {
        RustFieldValue::Text(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        RustFieldValue::Name(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        RustFieldValue::Boolean(b) => Ok(b.into_pyobject(py)?.to_owned().into_any().unbind()),
        RustFieldValue::Array(v) => Ok(v.into_pyobject(py)?.into_any().unbind()),
        RustFieldValue::None => Ok(py.None()),
    }
}

fn form_field_value_to_python(
    value: &crate::editor::form_fields::FormFieldValue,
    py: Python<'_>,
) -> PyResult<Py<PyAny>> {
    use crate::editor::form_fields::FormFieldValue;
    match value {
        FormFieldValue::Text(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        FormFieldValue::Choice(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        FormFieldValue::Boolean(b) => Ok(b.into_pyobject(py)?.to_owned().into_any().unbind()),
        FormFieldValue::MultiChoice(v) => Ok(v.into_pyobject(py)?.into_any().unbind()),
        FormFieldValue::None => Ok(py.None()),
    }
}

fn python_to_form_field_value(
    value: &Bound<'_, PyAny>,
) -> PyResult<crate::editor::form_fields::FormFieldValue> {
    use crate::editor::form_fields::FormFieldValue;
    if let Ok(b) = value.extract::<bool>() {
        Ok(FormFieldValue::Boolean(b))
    } else if let Ok(s) = value.extract::<String>() {
        Ok(FormFieldValue::Text(s))
    } else if let Ok(v) = value.extract::<Vec<String>>() {
        Ok(FormFieldValue::MultiChoice(v))
    } else if value.is_none() {
        Ok(FormFieldValue::None)
    } else {
        Err(PyRuntimeError::new_err("Invalid value."))
    }
}

/// Python wrapper for PDF creation.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Pdf", skip_from_py_object)]
pub struct PyPdf {
    bytes: Vec<u8>,
}

#[pymethods]
impl PyPdf {
    #[staticmethod]
    #[pyo3(signature = (content, title=None, author=None))]
    fn from_markdown(content: &str, title: Option<&str>, author: Option<&str>) -> PyResult<Self> {
        let mut b = RustPdfBuilder::new();
        if let Some(t) = title {
            b = b.title(t);
        }
        if let Some(a) = author {
            b = b.author(a);
        }
        let pdf = b
            .from_markdown(content)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPdf {
            bytes: pdf.into_bytes(),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (content, title=None, author=None))]
    fn from_html(content: &str, title: Option<&str>, author: Option<&str>) -> PyResult<Self> {
        let mut b = RustPdfBuilder::new();
        if let Some(t) = title {
            b = b.title(t);
        }
        if let Some(a) = author {
            b = b.author(a);
        }
        let pdf = b
            .from_html(content)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPdf {
            bytes: pdf.into_bytes(),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (content, title=None, author=None))]
    fn from_text(content: &str, title: Option<&str>, author: Option<&str>) -> PyResult<Self> {
        let mut b = RustPdfBuilder::new();
        if let Some(t) = title {
            b = b.title(t);
        }
        if let Some(a) = author {
            b = b.author(a);
        }
        let pdf = b
            .from_text(content)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPdf {
            bytes: pdf.into_bytes(),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (content, template, title=None, author=None))]
    fn from_markdown_with_template(
        content: &str,
        template: &PyPageTemplate,
        title: Option<&str>,
        author: Option<&str>,
    ) -> PyResult<Self> {
        let mut b = RustPdfBuilder::new().template(template.inner.clone());
        if let Some(t) = title {
            b = b.title(t);
        }
        if let Some(a) = author {
            b = b.author(a);
        }
        let pdf = b
            .from_markdown(content)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPdf {
            bytes: pdf.into_bytes(),
        })
    }

    /// Build a PDF by rendering `html` with `css` applied, embedding a
    /// single font for the body text. The font must cover every
    /// codepoint used by `html`, or unknown glyphs fall back to
    /// `.notdef`. See `from_html_css_with_fonts` for a multi-font
    /// cascade.
    #[staticmethod]
    fn from_html_css(html: &str, css: &str, font_bytes: &Bound<'_, PyBytes>) -> PyResult<Self> {
        let bytes = font_bytes.as_bytes().to_vec();
        let pdf = crate::api::Pdf::from_html_css(html, css, bytes)
            .map_err(|e| PyRuntimeError::new_err(format!("from_html_css failed: {e}")))?;
        Ok(PyPdf {
            bytes: pdf.into_bytes(),
        })
    }

    /// Build a PDF from HTML+CSS with a multi-font cascade. `fonts` is
    /// a list of `(family_name, font_bytes)` tuples; the first entry is
    /// the default used whenever a CSS `font-family` doesn't match any
    /// registered family.
    #[staticmethod]
    fn from_html_css_with_fonts(
        html: &str,
        css: &str,
        fonts: Vec<(String, Bound<'_, PyBytes>)>,
    ) -> PyResult<Self> {
        if fonts.is_empty() {
            return Err(PyValueError::new_err("at least one font must be provided"));
        }
        let font_vec: Vec<(String, Vec<u8>)> = fonts
            .into_iter()
            .map(|(name, b)| (name, b.as_bytes().to_vec()))
            .collect();
        let pdf = crate::api::Pdf::from_html_css_with_fonts(html, css, font_vec).map_err(|e| {
            PyRuntimeError::new_err(format!("from_html_css_with_fonts failed: {e}"))
        })?;
        Ok(PyPdf {
            bytes: pdf.into_bytes(),
        })
    }

    fn save(&self, path: &str) -> PyResult<()> {
        std::fs::write(path, &self.bytes).map_err(|e| PyIOError::new_err(e.to_string()))
    }

    fn to_bytes<'py>(&self, py: Python<'py>) -> Py<PyBytes> {
        PyBytes::new(py, &self.bytes).unbind()
    }

    #[staticmethod]
    fn from_image(path: &str) -> PyResult<Self> {
        use crate::api::Pdf;
        let pdf = Pdf::from_image(path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPdf {
            bytes: pdf.into_bytes(),
        })
    }

    #[staticmethod]
    fn from_images(paths: Vec<String>) -> PyResult<Self> {
        use crate::api::Pdf;
        let pdf = Pdf::from_images(&paths).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPdf {
            bytes: pdf.into_bytes(),
        })
    }

    #[staticmethod]
    fn from_image_bytes(data: &Bound<'_, PyBytes>) -> PyResult<Self> {
        use crate::api::Pdf;
        let pdf = Pdf::from_image_bytes(data.as_bytes())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPdf {
            bytes: pdf.into_bytes(),
        })
    }

    /// Open an existing PDF from bytes.
    ///
    /// Args:
    ///     data (bytes): PDF file contents
    ///
    /// Returns:
    ///     Pdf: A Pdf object for editing
    #[staticmethod]
    fn from_bytes(data: &Bound<'_, PyBytes>) -> PyResult<Self> {
        let mut pdf = crate::api::Pdf::from_bytes(data.as_bytes().to_vec())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let bytes = pdf
            .save_to_bytes()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPdf { bytes })
    }

    /// Merge multiple PDF files into one.
    ///
    /// Args:
    ///     paths (list[str]): List of paths to PDF files to merge
    ///
    /// Returns:
    ///     Pdf: A new PDF containing all pages from the input files
    #[staticmethod]
    fn merge(paths: Vec<String>) -> PyResult<Self> {
        let bytes =
            crate::api::merge_pdfs(&paths).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPdf { bytes })
    }

    fn __len__(&self) -> usize {
        self.bytes.len()
    }
    fn __repr__(&self) -> String {
        format!("Pdf({} bytes)", self.bytes.len())
    }
}

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "OfficeConverter",
    skip_from_py_object
)]
pub struct PyOfficeConverter;

#[pymethods]
impl PyOfficeConverter {
    #[new]
    fn new() -> Self {
        PyOfficeConverter
    }
    #[staticmethod]
    fn from_docx(path: &str) -> PyResult<PyPdf> {
        crate::converters::office::OfficeConverter::new()
            .convert_docx(path)
            .map(|b| PyPdf { bytes: b })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    #[staticmethod]
    fn from_docx_bytes(data: &Bound<'_, PyBytes>) -> PyResult<PyPdf> {
        crate::converters::office::OfficeConverter::new()
            .convert_docx_bytes(data.as_bytes())
            .map(|b| PyPdf { bytes: b })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    #[staticmethod]
    fn from_xlsx(path: &str) -> PyResult<PyPdf> {
        crate::converters::office::OfficeConverter::new()
            .convert_xlsx(path)
            .map(|b| PyPdf { bytes: b })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    #[staticmethod]
    fn from_xlsx_bytes(data: &Bound<'_, PyBytes>) -> PyResult<PyPdf> {
        crate::converters::office::OfficeConverter::new()
            .convert_xlsx_bytes(data.as_bytes())
            .map(|b| PyPdf { bytes: b })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    #[staticmethod]
    fn from_pptx(path: &str) -> PyResult<PyPdf> {
        crate::converters::office::OfficeConverter::new()
            .convert_pptx(path)
            .map(|b| PyPdf { bytes: b })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    #[staticmethod]
    fn from_pptx_bytes(data: &Bound<'_, PyBytes>) -> PyResult<PyPdf> {
        crate::converters::office::OfficeConverter::new()
            .convert_pptx_bytes(data.as_bytes())
            .map(|b| PyPdf { bytes: b })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    #[staticmethod]
    fn convert(path: &str) -> PyResult<PyPdf> {
        crate::converters::office::OfficeConverter::new()
            .convert(path)
            .map(|b| PyPdf { bytes: b })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

use crate::editor::{ElementId, PdfElement, PdfPage as RustPdfPage, PdfText as RustPdfText};

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "PdfPageRegion",
    skip_from_py_object
)]
pub struct PyPdfPageRegion {
    pub doc: Py<PyPdfDocument>,
    pub page_index: usize,
    pub region: crate::geometry::Rect,
}

#[pymethods]
impl PyPdfPageRegion {
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        (self.region.x, self.region.y, self.region.width, self.region.height)
    }
    fn extract_text(&self, py: Python<'_>) -> PyResult<String> {
        let mut d = self.doc.bind(py).borrow_mut();
        d.extract_text(self.page_index, Some(self.bbox()), None, None, true, true)
    }
    fn extract_words(&self, py: Python<'_>) -> PyResult<Vec<PyWord>> {
        let mut d = self.doc.bind(py).borrow_mut();
        d.extract_words(py, self.page_index, true, Some(self.bbox()), None, None)
    }
    fn extract_text_lines(&self, py: Python<'_>) -> PyResult<Vec<PyTextLine>> {
        let mut d = self.doc.bind(py).borrow_mut();
        d.extract_text_lines(py, self.page_index, true, Some(self.bbox()), None, None, None)
    }
    #[pyo3(signature = (table_settings=None))]
    fn extract_tables(
        &self,
        py: Python<'_>,
        table_settings: Option<Bound<'_, pyo3::types::PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let mut d = self.doc.bind(py).borrow_mut();
        d.extract_tables(py, self.page_index, Some(self.bbox()), table_settings)
    }
    fn extract_images(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let mut d = self.doc.bind(py).borrow_mut();
        d.extract_images(py, self.page_index, Some(self.bbox()))
    }
    fn extract_paths(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let d = self.doc.bind(py).borrow_mut();
        let res = d
            .inner
            .extract_paths_in_rect(self.page_index, self.region)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let list = pyo3::types::PyList::empty(py);
        for p in &res {
            list.append(path_to_py_dict(py, p)?)?;
        }
        Ok(list.into())
    }
    fn __repr__(&self) -> String {
        format!("PdfPageRegion(page={}, bbox={:?})", self.page_index, self.region)
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "PdfPage")]
pub struct PyPdfPage {
    inner: RustPdfPage,
}

#[pymethods]
impl PyPdfPage {
    #[getter]
    fn index(&self) -> usize {
        self.inner.page_index
    }
    #[getter]
    fn width(&self) -> f32 {
        self.inner.width
    }
    #[getter]
    fn height(&self) -> f32 {
        self.inner.height
    }
    fn children(&self) -> Vec<PyPdfElement> {
        self.inner
            .children()
            .into_iter()
            .map(|e| PyPdfElement { inner: e })
            .collect()
    }
    fn find_text_containing(&self, needle: &str) -> Vec<PyPdfText> {
        self.inner
            .find_text_containing(needle)
            .into_iter()
            .map(|t| PyPdfText { inner: t })
            .collect()
    }
    fn find_images(&self) -> Vec<PyPdfImage> {
        self.inner
            .find_images()
            .into_iter()
            .map(|i| PyPdfImage { inner: i })
            .collect()
    }
    fn get_element(&self, _id: &str) -> Option<PyPdfElement> {
        None
    }
    fn set_text(&mut self, text_id: &PyPdfTextId, new_text: &str) -> PyResult<()> {
        self.inner
            .set_text(text_id.inner, new_text)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    fn annotations(&self) -> Vec<PyAnnotationWrapper> {
        self.inner
            .annotations()
            .iter()
            .map(|a| PyAnnotationWrapper { inner: a.clone() })
            .collect()
    }
    fn add_link(&mut self, x: f32, y: f32, width: f32, height: f32, url: &str) -> String {
        use crate::writer::LinkAnnotation;
        let l = LinkAnnotation::uri(crate::geometry::Rect::new(x, y, width, height), url);
        format!("{:?}", self.inner.add_annotation(l))
    }
    fn add_highlight(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: (f32, f32, f32),
    ) -> String {
        use crate::writer::TextMarkupAnnotation;
        use crate::TextMarkupType;
        let l = TextMarkupAnnotation::from_rect(
            TextMarkupType::Highlight,
            crate::geometry::Rect::new(x, y, width, height),
        )
        .with_color(color.0, color.1, color.2);
        format!("{:?}", self.inner.add_annotation(l))
    }
    fn add_note(&mut self, x: f32, y: f32, text: &str) -> String {
        use crate::writer::TextAnnotation;
        let l = TextAnnotation::new(crate::geometry::Rect::new(x, y, 24.0, 24.0), text);
        format!("{:?}", self.inner.add_annotation(l))
    }
    fn remove_annotation(&mut self, index: usize) -> bool {
        self.inner.remove_annotation(index).is_some()
    }
    #[pyo3(signature = (text, x, y, font_size=12.0))]
    fn add_text(&mut self, text: &str, x: f32, y: f32, font_size: f32) -> PyPdfTextId {
        use crate::elements::{FontSpec, TextContent, TextStyle};
        let c = TextContent {
            text: text.to_string(),
            bbox: crate::geometry::Rect::new(x, y, text.len() as f32 * font_size * 0.6, font_size),
            font: FontSpec {
                name: "Helvetica".to_string(),
                size: font_size,
            },
            style: TextStyle::default(),
            reading_order: None,
            artifact_type: None,
            origin: None,
            rotation_degrees: None,
            matrix: None,
        };
        PyPdfTextId {
            inner: self.inner.add_text(c),
        }
    }
    fn remove_element(&mut self, id: &PyPdfTextId) -> bool {
        self.inner.remove_element(id.inner)
    }
    fn __repr__(&self) -> String {
        format!(
            "PdfPage(index={}, width={:.1}, height={:.1})",
            self.inner.page_index, self.inner.width, self.inner.height
        )
    }
}

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "PdfTextId",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyPdfTextId {
    inner: ElementId,
}
#[pymethods]
impl PyPdfTextId {
    fn __repr__(&self) -> String {
        format!("PdfTextId({:?})", self.inner)
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "PdfText", skip_from_py_object)]
#[derive(Clone)]
pub struct PyPdfText {
    inner: RustPdfText,
}
#[pymethods]
impl PyPdfText {
    #[getter]
    fn id(&self) -> PyPdfTextId {
        PyPdfTextId {
            inner: self.inner.id(),
        }
    }
    #[getter]
    fn value(&self) -> String {
        self.inner.text().to_string()
    }
    #[getter]
    fn text(&self) -> String {
        self.value()
    }
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        let r = self.inner.bbox();
        (r.x, r.y, r.width, r.height)
    }
    #[getter]
    fn font_name(&self) -> String {
        self.inner.font_name().to_string()
    }
    #[getter]
    fn font_size(&self) -> f32 {
        self.inner.font_size()
    }
    #[getter]
    fn is_bold(&self) -> bool {
        self.inner.is_bold()
    }
    #[getter]
    fn is_italic(&self) -> bool {
        self.inner.is_italic()
    }
    fn contains(&self, n: &str) -> bool {
        self.inner.contains(n)
    }
    fn starts_with(&self, p: &str) -> bool {
        self.inner.starts_with(p)
    }
    fn ends_with(&self, s: &str) -> bool {
        self.inner.ends_with(s)
    }
    fn __repr__(&self) -> String {
        format!("PdfText({:?})", self.inner.text())
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "PdfImage", skip_from_py_object)]
#[derive(Clone)]
pub struct PyPdfImage {
    inner: crate::editor::PdfImage,
}
#[pymethods]
impl PyPdfImage {
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        let r = self.inner.bbox();
        (r.x, r.y, r.width, r.height)
    }
    #[getter]
    fn width(&self) -> u32 {
        self.inner.dimensions().0
    }
    #[getter]
    fn height(&self) -> u32 {
        self.inner.dimensions().1
    }
    #[getter]
    fn aspect_ratio(&self) -> f32 {
        self.inner.aspect_ratio()
    }
    fn __repr__(&self) -> String {
        let (w, h) = self.inner.dimensions();
        format!("PdfImage({}x{})", w, h)
    }
}

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "PdfAnnotation",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyAnnotationWrapper {
    inner: crate::editor::AnnotationWrapper,
}
#[pymethods]
impl PyAnnotationWrapper {
    #[getter]
    fn subtype(&self) -> String {
        format!("{:?}", self.inner.subtype())
    }
    #[getter]
    fn rect(&self) -> (f32, f32, f32, f32) {
        let r = self.inner.rect();
        (r.x, r.y, r.width, r.height)
    }
    #[getter]
    fn contents(&self) -> Option<String> {
        self.inner.contents().map(|s| s.to_string())
    }
    #[getter]
    fn color(&self) -> Option<(f32, f32, f32)> {
        self.inner.color()
    }
    #[getter]
    fn is_modified(&self) -> bool {
        self.inner.is_modified()
    }
    #[getter]
    fn is_new(&self) -> bool {
        self.inner.is_new()
    }
    fn __repr__(&self) -> String {
        format!("PdfAnnotation(subtype={:?})", self.inner.subtype())
    }
}

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "PdfElement",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyPdfElement {
    inner: PdfElement,
}
#[pymethods]
impl PyPdfElement {
    fn is_text(&self) -> bool {
        self.inner.is_text()
    }
    fn is_image(&self) -> bool {
        self.inner.is_image()
    }
    fn is_path(&self) -> bool {
        self.inner.is_path()
    }
    fn is_table(&self) -> bool {
        self.inner.is_table()
    }
    fn is_structure(&self) -> bool {
        self.inner.is_structure()
    }
    fn as_text(&self) -> Option<PyPdfText> {
        if let PdfElement::Text(t) = &self.inner {
            Some(PyPdfText { inner: t.clone() })
        } else {
            None
        }
    }
    fn as_image(&self) -> Option<PyPdfImage> {
        if let PdfElement::Image(i) = &self.inner {
            Some(PyPdfImage { inner: i.clone() })
        } else {
            None
        }
    }
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        let r = self.inner.bbox();
        (r.x, r.y, r.width, r.height)
    }
    fn __repr__(&self) -> String {
        "PdfElement".to_string()
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "TextChar", skip_from_py_object)]
#[derive(Clone)]
pub struct PyTextChar {
    inner: RustTextChar,
}
#[pymethods]
impl PyTextChar {
    #[getter]
    fn char(&self) -> char {
        self.inner.char
    }
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        (
            self.inner.bbox.x,
            self.inner.bbox.y,
            self.inner.bbox.width,
            self.inner.bbox.height,
        )
    }
    #[getter]
    fn font_name(&self) -> String {
        self.inner.font_name.clone()
    }
    #[getter]
    fn font_size(&self) -> f32 {
        self.inner.font_size
    }
    #[getter]
    fn font_weight(&self) -> String {
        format!("{:?}", self.inner.font_weight)
    }
    #[getter]
    fn is_italic(&self) -> bool {
        self.inner.is_italic
    }
    #[getter]
    fn is_monospace(&self) -> bool {
        self.inner.is_monospace
    }
    #[getter]
    fn color(&self) -> (f32, f32, f32) {
        (self.inner.color.r, self.inner.color.g, self.inner.color.b)
    }
    #[getter]
    fn rotation_degrees(&self) -> f32 {
        self.inner.rotation_degrees
    }
    #[getter]
    fn origin_x(&self) -> f32 {
        self.inner.origin_x
    }
    #[getter]
    fn origin_y(&self) -> f32 {
        self.inner.origin_y
    }
    #[getter]
    fn advance_width(&self) -> f32 {
        self.inner.advance_width
    }
    #[getter]
    fn mcid(&self) -> Option<u32> {
        self.inner.mcid
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "TextSpan", skip_from_py_object)]
#[derive(Clone)]
pub struct PyTextSpan {
    inner: crate::layout::TextSpan,
}
#[pymethods]
impl PyTextSpan {
    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        (
            self.inner.bbox.x,
            self.inner.bbox.y,
            self.inner.bbox.width,
            self.inner.bbox.height,
        )
    }
    /// Where the run physically sits on the page: `bbox` with any text-matrix
    /// rotation resolved into an axis-aligned page-space hull. Identical to
    /// `bbox` for upright runs.
    #[getter]
    fn page_bbox(&self) -> (f32, f32, f32, f32) {
        let b = self.inner.page_bbox();
        (b.x, b.y, b.width, b.height)
    }
    #[getter]
    fn font_name(&self) -> &str {
        &self.inner.font_name
    }
    #[getter]
    fn font_size(&self) -> f32 {
        self.inner.font_size
    }
    #[getter]
    fn is_bold(&self) -> bool {
        self.inner.font_weight as u16 >= 700
    }
    #[getter]
    fn is_italic(&self) -> bool {
        self.inner.is_italic
    }
    #[getter]
    fn is_monospace(&self) -> bool {
        self.inner.is_monospace
    }
    #[getter]
    fn char_widths(&self) -> Vec<f32> {
        self.inner.char_widths.clone()
    }
    #[getter]
    fn color(&self) -> (f32, f32, f32) {
        (self.inner.color.r, self.inner.color.g, self.inner.color.b)
    }
    /// Content-stream emission order: incremented as spans are emitted
    /// during the Tj/TJ walk, so two spans with adjacent `sequence`
    /// values were drawn consecutively, distinguishing that from spans
    /// that are merely spatially close.
    #[getter]
    fn sequence(&self) -> usize {
        self.inner.sequence
    }
    /// Which ISO 32000-1 §9.10.2 mapping tier the span's font offered:
    /// `"to_unicode"`, `"encoding"`, `"predefined_cmap"`, `"embedded_cmap"`,
    /// `"actual_text"`, or `"fallback"` — the last meaning the font carried no
    /// mapping resource, so the text is a fabricated glyph-index echo rather
    /// than read from the file. `None` when the font could not be resolved.
    #[getter]
    fn provenance(&self) -> Option<&'static str> {
        self.inner.provenance.map(|p| p.as_str())
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "TextWord", skip_from_py_object)]
#[derive(Clone)]
pub struct PyWord {
    inner: crate::layout::Word,
}
#[pymethods]
impl PyWord {
    #[getter]
    fn text(&self) -> String {
        self.inner.text.clone()
    }
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        (
            self.inner.bbox.x,
            self.inner.bbox.y,
            self.inner.bbox.width,
            self.inner.bbox.height,
        )
    }
    #[getter]
    fn font_name(&self) -> String {
        self.inner.dominant_font.clone()
    }
    #[getter]
    fn font_size(&self) -> f32 {
        self.inner.avg_font_size
    }
    #[getter]
    fn is_bold(&self) -> bool {
        self.inner.is_bold
    }
    #[getter]
    fn is_italic(&self) -> bool {
        self.inner.is_italic
    }
    #[getter]
    fn chars(&self) -> Vec<PyTextChar> {
        self.inner
            .chars
            .iter()
            .map(|c| PyTextChar { inner: c.clone() })
            .collect()
    }
    /// Content-stream emission order of the originating span — see
    /// `TextSpan.sequence`.
    #[getter]
    fn sequence(&self) -> usize {
        self.inner.sequence
    }
    /// Rotation of the word's glyph run in degrees, snapped to a quadrant
    /// (0 / 90 / 180 / -90) — see `Word.rotation_degrees`.
    #[getter]
    fn rotation_degrees(&self) -> f32 {
        self.inner.rotation_degrees
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "TextLine", skip_from_py_object)]
#[derive(Clone)]
pub struct PyTextLine {
    inner: crate::layout::TextLine,
}
#[pymethods]
impl PyTextLine {
    #[getter]
    fn text(&self) -> String {
        self.inner.text.clone()
    }
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        (
            self.inner.bbox.x,
            self.inner.bbox.y,
            self.inner.bbox.width,
            self.inner.bbox.height,
        )
    }
    #[getter]
    fn words(&self) -> Vec<PyWord> {
        self.inner
            .words
            .iter()
            .map(|w| PyWord { inner: w.clone() })
            .collect()
    }
    #[getter]
    fn chars(&self) -> Vec<PyTextChar> {
        self.inner
            .words
            .iter()
            .flat_map(|w| w.chars.iter().map(|c| PyTextChar { inner: c.clone() }))
            .collect()
    }
}

fn path_to_py_dict(py: Python<'_>, path: &crate::elements::PathContent) -> PyResult<Py<PyAny>> {
    let d = pyo3::types::PyDict::new(py);
    d.set_item("bbox", (path.bbox.x, path.bbox.y, path.bbox.width, path.bbox.height))?;
    // Stroke-inflated extents — what the reader sees.
    let rendered = path.rendered_bbox();
    d.set_item("rendered_bbox", (rendered.x, rendered.y, rendered.width, rendered.height))?;
    d.set_item("stroke_width", path.stroke_width)?;
    if let Some(ref c) = path.stroke_color {
        d.set_item("stroke_color", (c.r, c.g, c.b))?;
    } else {
        d.set_item("stroke_color", py.None())?;
    }
    if let Some(ref c) = path.fill_color {
        d.set_item("fill_color", (c.r, c.g, c.b))?;
    } else {
        d.set_item("fill_color", py.None())?;
    }
    d.set_item("operations_count", path.operations.len())?;

    // Optional Content Group (PDF "layer") name resolved from the
    // enclosing `BDC /OC … EMC` markers in the source content stream.
    // `None` when the path was emitted outside any /OC region or when
    // the PDF has no optional-content metadata. See PDF spec
    // ISO 32000-1:2008 §8.11 (Optional Content) and §14.6 (Marked Content).
    match path.layer {
        Some(ref name) => d.set_item("layer", name.as_str())?,
        None => d.set_item("layer", py.None())?,
    }

    // Expose path operations as list of dicts for vector extraction use cases
    let ops_list = pyo3::types::PyList::empty(py);
    for op in &path.operations {
        let op_dict = pyo3::types::PyDict::new(py);
        match op {
            crate::elements::PathOperation::MoveTo(x, y) => {
                op_dict.set_item("op", "move_to")?;
                op_dict.set_item("x", *x)?;
                op_dict.set_item("y", *y)?;
            },
            crate::elements::PathOperation::LineTo(x, y) => {
                op_dict.set_item("op", "line_to")?;
                op_dict.set_item("x", *x)?;
                op_dict.set_item("y", *y)?;
            },
            crate::elements::PathOperation::CurveTo(cx1, cy1, cx2, cy2, x, y) => {
                op_dict.set_item("op", "curve_to")?;
                op_dict.set_item("cx1", *cx1)?;
                op_dict.set_item("cy1", *cy1)?;
                op_dict.set_item("cx2", *cx2)?;
                op_dict.set_item("cy2", *cy2)?;
                op_dict.set_item("x", *x)?;
                op_dict.set_item("y", *y)?;
            },
            crate::elements::PathOperation::Rectangle(x, y, w, h) => {
                op_dict.set_item("op", "rectangle")?;
                op_dict.set_item("x", *x)?;
                op_dict.set_item("y", *y)?;
                op_dict.set_item("width", *w)?;
                op_dict.set_item("height", *h)?;
            },
            crate::elements::PathOperation::ClosePath => {
                op_dict.set_item("op", "close_path")?;
            },
        }
        ops_list.append(op_dict)?;
    }
    d.set_item("operations", ops_list)?;

    Ok(d.into())
}

fn table_settings_to_config(
    settings: Option<Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<crate::structure::spatial_table_detector::TableDetectionConfig> {
    use crate::structure::spatial_table_detector::{TableDetectionConfig, TableStrategy};
    let mut c = TableDetectionConfig::default();
    if let Some(d) = settings {
        if let Some(v) = d.get_item("horizontal_strategy")? {
            let s: String = v.extract()?;
            c.horizontal_strategy = match s.as_str() {
                "lines" => TableStrategy::Lines,
                "text" => TableStrategy::Text,
                "both" => TableStrategy::Both,
                _ => return Err(PyRuntimeError::new_err("Invalid strategy")),
            };
        }
        if let Some(v) = d.get_item("vertical_strategy")? {
            let s: String = v.extract()?;
            c.vertical_strategy = match s.as_str() {
                "lines" => TableStrategy::Lines,
                "text" => TableStrategy::Text,
                "both" => TableStrategy::Both,
                _ => return Err(PyRuntimeError::new_err("Invalid strategy")),
            };
        }
        if let Some(v) = d.get_item("column_tolerance")? {
            c.column_tolerance = v.extract()?;
        }
        if let Some(v) = d.get_item("row_tolerance")? {
            c.row_tolerance = v.extract()?;
        }
        if let Some(v) = d.get_item("min_table_cells")? {
            c.min_table_cells = v.extract()?;
        }
    }
    Ok(c)
}

fn outline_items_to_py(
    py: Python<'_>,
    items: &[crate::outline::OutlineItem],
) -> PyResult<Py<PyAny>> {
    let list = pyo3::types::PyList::empty(py);
    for i in items {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("title", &i.title)?;
        match &i.dest {
            Some(crate::outline::Destination::PageIndex(idx)) => {
                d.set_item("page", *idx)?;
            },
            _ => {
                d.set_item("page", py.None())?;
            },
        }
        d.set_item("children", outline_items_to_py(py, &i.children)?)?;
        list.append(d)?;
    }
    Ok(list.into())
}

#[cfg(feature = "ocr")]
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "OcrEngine", unsendable)]
pub struct PyOcrEngine {
    inner: crate::ocr::OcrEngine,
}
#[cfg(not(feature = "ocr"))]
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "OcrEngine", unsendable)]
pub struct PyOcrEngine {}
#[cfg(not(feature = "ocr"))]
#[pymethods]
impl PyOcrEngine {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(
        _args: &Bound<'_, pyo3::types::PyTuple>,
        _kwargs: Option<Bound<'_, pyo3::types::PyDict>>,
    ) -> PyResult<Self> {
        Err(PyRuntimeError::new_err("OCR not enabled."))
    }
}
#[cfg(feature = "ocr")]
#[pymethods]
impl PyOcrEngine {
    #[new]
    #[pyo3(signature = (det_model_path, rec_model_path, dict_path, config=None))]
    fn new(
        det_model_path: &str,
        rec_model_path: &str,
        dict_path: &str,
        config: Option<&PyOcrConfig>,
    ) -> PyResult<Self> {
        let c = config.map(|c| c.inner.clone()).unwrap_or_default();
        let e = crate::ocr::OcrEngine::new(det_model_path, rec_model_path, dict_path, c)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyOcrEngine { inner: e })
    }
}

#[cfg(feature = "ocr")]
#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "OcrConfig",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyOcrConfig {
    inner: crate::ocr::OcrConfig,
}
#[cfg(not(feature = "ocr"))]
#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "OcrConfig",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyOcrConfig {}
#[cfg(not(feature = "ocr"))]
#[pymethods]
impl PyOcrConfig {
    #[new]
    #[pyo3(signature = (**_kwargs))]
    fn new(_kwargs: Option<Bound<'_, pyo3::types::PyDict>>) -> PyResult<Self> {
        Err(PyRuntimeError::new_err("OCR not enabled."))
    }
}
#[cfg(feature = "ocr")]
#[pymethods]
impl PyOcrConfig {
    #[new]
    #[pyo3(signature = (det_threshold=None, rec_threshold=None, num_threads=None))]
    fn new(
        det_threshold: Option<f32>,
        rec_threshold: Option<f32>,
        num_threads: Option<usize>,
    ) -> Self {
        let mut c = crate::ocr::OcrConfig::default();
        if let Some(v) = det_threshold {
            c.det_threshold = v;
        }
        if let Some(v) = rec_threshold {
            c.rec_threshold = v;
        }
        if let Some(v) = num_threads {
            c.num_threads = v;
        }
        PyOcrConfig { inner: c }
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Color", skip_from_py_object)]
#[derive(Clone)]
pub struct PyColor {
    inner: RustColor,
}
#[pymethods]
impl PyColor {
    #[new]
    fn new(r: f32, g: f32, b: f32) -> Self {
        PyColor {
            inner: RustColor::new(r, g, b),
        }
    }
    #[staticmethod]
    fn black() -> Self {
        PyColor {
            inner: RustColor::black(),
        }
    }
    #[staticmethod]
    fn white() -> Self {
        PyColor {
            inner: RustColor::white(),
        }
    }
    #[staticmethod]
    fn red() -> Self {
        PyColor {
            inner: RustColor::new(1.0, 0.0, 0.0),
        }
    }
    #[staticmethod]
    fn green() -> Self {
        PyColor {
            inner: RustColor::new(0.0, 1.0, 0.0),
        }
    }
    #[staticmethod]
    fn blue() -> Self {
        PyColor {
            inner: RustColor::new(0.0, 0.0, 1.0),
        }
    }
    #[staticmethod]
    fn from_hex(hex: &str) -> PyResult<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(PyRuntimeError::new_err("Invalid hex color length"));
        }
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| PyRuntimeError::new_err("Invalid hex color"))? as f32
            / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| PyRuntimeError::new_err("Invalid hex color"))? as f32
            / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| PyRuntimeError::new_err("Invalid hex color"))? as f32
            / 255.0;
        Ok(PyColor {
            inner: RustColor::new(r, g, b),
        })
    }
    #[getter]
    fn r(&self) -> f32 {
        self.inner.r
    }
    #[getter]
    fn g(&self) -> f32 {
        self.inner.g
    }
    #[getter]
    fn b(&self) -> f32 {
        self.inner.b
    }
}

#[allow(dead_code)]
#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "BlendMode",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyBlendMode {
    inner: RustBlendMode,
}
#[pymethods]
impl PyBlendMode {
    #[staticmethod]
    #[allow(non_snake_case)]
    fn NORMAL() -> Self {
        PyBlendMode {
            inner: RustBlendMode::Normal,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn MULTIPLY() -> Self {
        PyBlendMode {
            inner: RustBlendMode::Multiply,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn SCREEN() -> Self {
        PyBlendMode {
            inner: RustBlendMode::Screen,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn OVERLAY() -> Self {
        PyBlendMode {
            inner: RustBlendMode::Overlay,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn DARKEN() -> Self {
        PyBlendMode {
            inner: RustBlendMode::Darken,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn LIGHTEN() -> Self {
        PyBlendMode {
            inner: RustBlendMode::Lighten,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn COLOR_DODGE() -> Self {
        PyBlendMode {
            inner: RustBlendMode::ColorDodge,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn COLOR_BURN() -> Self {
        PyBlendMode {
            inner: RustBlendMode::ColorBurn,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn HARD_LIGHT() -> Self {
        PyBlendMode {
            inner: RustBlendMode::HardLight,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn SOFT_LIGHT() -> Self {
        PyBlendMode {
            inner: RustBlendMode::SoftLight,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn DIFFERENCE() -> Self {
        PyBlendMode {
            inner: RustBlendMode::Difference,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn EXCLUSION() -> Self {
        PyBlendMode {
            inner: RustBlendMode::Exclusion,
        }
    }
}

#[allow(dead_code)]
#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "ExtGState",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyExtGState {
    fill_alpha: Option<f32>,
    stroke_alpha: Option<f32>,
    blend_mode: Option<RustBlendMode>,
}
#[pymethods]
impl PyExtGState {
    #[new]
    fn new() -> Self {
        PyExtGState {
            fill_alpha: None,
            stroke_alpha: None,
            blend_mode: None,
        }
    }
    fn alpha(&self, a: f32) -> Self {
        let v = Some(a.clamp(0.0, 1.0));
        PyExtGState {
            fill_alpha: v,
            stroke_alpha: v,
            blend_mode: self.blend_mode,
        }
    }
    fn fill_alpha(&self, a: f32) -> Self {
        PyExtGState {
            fill_alpha: Some(a.clamp(0.0, 1.0)),
            stroke_alpha: self.stroke_alpha,
            blend_mode: self.blend_mode,
        }
    }
    fn stroke_alpha(&self, a: f32) -> Self {
        PyExtGState {
            fill_alpha: self.fill_alpha,
            stroke_alpha: Some(a.clamp(0.0, 1.0)),
            blend_mode: self.blend_mode,
        }
    }
    fn blend_mode(&self, mode: &PyBlendMode) -> Self {
        PyExtGState {
            fill_alpha: self.fill_alpha,
            stroke_alpha: self.stroke_alpha,
            blend_mode: Some(mode.inner),
        }
    }
    #[staticmethod]
    fn semi_transparent() -> Self {
        PyExtGState {
            fill_alpha: Some(0.5),
            stroke_alpha: Some(0.5),
            blend_mode: None,
        }
    }
}

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "LinearGradient",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyLinearGradient {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    stops: Vec<(f32, RustColor)>,
}
#[pymethods]
impl PyLinearGradient {
    #[new]
    fn new() -> Self {
        PyLinearGradient {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            stops: Vec::new(),
        }
    }
    fn start(&self, x: f32, y: f32) -> Self {
        let mut slf = self.clone();
        slf.x1 = x;
        slf.y1 = y;
        slf
    }
    fn end(&self, x: f32, y: f32) -> Self {
        let mut slf = self.clone();
        slf.x2 = x;
        slf.y2 = y;
        slf
    }
    fn add_stop(&self, offset: f32, color: &PyColor) -> Self {
        let mut slf = self.clone();
        slf.stops.push((offset, color.inner));
        slf
    }
    #[staticmethod]
    fn horizontal(width: f32, start: &PyColor, end: &PyColor) -> Self {
        PyLinearGradient {
            x1: 0.0,
            y1: 0.0,
            x2: width,
            y2: 0.0,
            stops: vec![(0.0, start.inner), (1.0, end.inner)],
        }
    }
    #[staticmethod]
    fn vertical(height: f32, start: &PyColor, end: &PyColor) -> Self {
        PyLinearGradient {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: height,
            stops: vec![(0.0, start.inner), (1.0, end.inner)],
        }
    }
}

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "RadialGradient",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyRadialGradient {
    x1: f32,
    y1: f32,
    r1: f32,
    x2: f32,
    y2: f32,
    r2: f32,
    stops: Vec<(f32, RustColor)>,
}
#[pymethods]
impl PyRadialGradient {
    #[new]
    fn new() -> Self {
        PyRadialGradient {
            x1: 50.0,
            y1: 50.0,
            r1: 0.0,
            x2: 50.0,
            y2: 50.0,
            r2: 50.0,
            stops: Vec::new(),
        }
    }
    fn inner_circle(&self, x: f32, y: f32, r: f32) -> Self {
        let mut slf = self.clone();
        slf.x1 = x;
        slf.y1 = y;
        slf.r1 = r;
        slf
    }
    fn outer_circle(&self, x: f32, y: f32, r: f32) -> Self {
        let mut slf = self.clone();
        slf.x2 = x;
        slf.y2 = y;
        slf.r2 = r;
        slf
    }
    fn add_stop(&self, offset: f32, color: &PyColor) -> Self {
        let mut slf = self.clone();
        slf.stops.push((offset, color.inner));
        slf
    }
    #[staticmethod]
    fn centered(x: f32, y: f32, radius: f32) -> Self {
        PyRadialGradient {
            x1: x,
            y1: y,
            r1: 0.0,
            x2: x,
            y2: y,
            r2: radius,
            stops: Vec::new(),
        }
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "LineCap", skip_from_py_object)]
#[derive(Clone)]
pub struct PyLineCap {
    pub inner: RustLineCap,
}
#[pymethods]
impl PyLineCap {
    #[staticmethod]
    fn butt() -> Self {
        Self {
            inner: RustLineCap::Butt,
        }
    }
    #[staticmethod]
    fn round() -> Self {
        Self {
            inner: RustLineCap::Round,
        }
    }
    #[staticmethod]
    fn square() -> Self {
        Self {
            inner: RustLineCap::Square,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn BUTT() -> Self {
        Self::butt()
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn ROUND() -> Self {
        Self::round()
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn SQUARE() -> Self {
        Self::square()
    }
    /// `LineCap.Round`-style representation, so a value read back from the
    /// editor's settings can be told apart in a REPL.
    fn __repr__(&self) -> String {
        format!("LineCap.{:?}", self.inner)
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "LineJoin", skip_from_py_object)]
#[derive(Clone)]
pub struct PyLineJoin {
    pub inner: RustLineJoin,
}
#[pymethods]
impl PyLineJoin {
    #[staticmethod]
    fn miter() -> Self {
        Self {
            inner: RustLineJoin::Miter,
        }
    }
    #[staticmethod]
    fn round() -> Self {
        Self {
            inner: RustLineJoin::Round,
        }
    }
    #[staticmethod]
    fn bevel() -> Self {
        Self {
            inner: RustLineJoin::Bevel,
        }
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn MITER() -> Self {
        Self::miter()
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn ROUND() -> Self {
        Self::round()
    }
    #[staticmethod]
    #[allow(non_snake_case)]
    fn BEVEL() -> Self {
        Self::bevel()
    }
    /// `LineJoin.Round`-style representation, so a value read back from the
    /// editor's settings can be told apart in a REPL.
    fn __repr__(&self) -> String {
        format!("LineJoin.{:?}", self.inner)
    }
}

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "PatternPresets",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyPatternPresets;
#[pymethods]
impl PyPatternPresets {
    #[staticmethod]
    fn horizontal_stripes(width: f32, height: f32, stripe_height: f32, color: &PyColor) -> Vec<u8> {
        crate::writer::PatternPresets::horizontal_stripes(width, height, stripe_height, color.inner)
    }
    #[staticmethod]
    fn vertical_stripes(width: f32, height: f32, stripe_width: f32, color: &PyColor) -> Vec<u8> {
        crate::writer::PatternPresets::vertical_stripes(width, height, stripe_width, color.inner)
    }
    #[staticmethod]
    fn checkerboard(size: f32, color1: &PyColor, color2: &PyColor) -> Vec<u8> {
        crate::writer::PatternPresets::checkerboard(size, color1.inner, color2.inner)
    }
    #[staticmethod]
    fn dots(spacing: f32, radius: f32, color: &PyColor) -> Vec<u8> {
        crate::writer::PatternPresets::dots(spacing, radius, color.inner)
    }
    #[staticmethod]
    fn diagonal_lines(size: f32, line_width: f32, color: &PyColor) -> Vec<u8> {
        crate::writer::PatternPresets::diagonal_lines(size, line_width, color.inner)
    }
    #[staticmethod]
    fn crosshatch(size: f32, line_width: f32, color: &PyColor) -> Vec<u8> {
        crate::writer::PatternPresets::crosshatch(size, line_width, color.inner)
    }
}

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "ArtifactStyle",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyArtifactStyle {
    pub inner: crate::writer::ArtifactStyle,
}
#[pymethods]
impl PyArtifactStyle {
    #[new]
    fn new() -> Self {
        Self {
            inner: crate::writer::ArtifactStyle::default(),
        }
    }
    fn font<'a>(
        mut slf: PyRefMut<'a, Self>,
        name: &str,
        size: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.inner = slf.inner.clone().font(name, size);
        Ok(slf)
    }
    fn bold<'a>(mut slf: PyRefMut<'a, Self>) -> PyResult<PyRefMut<'a, Self>> {
        slf.inner = slf.inner.clone().bold();
        Ok(slf)
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Artifact", from_py_object)]
#[derive(Clone)]
pub struct PyArtifact {
    pub inner: crate::writer::Artifact,
}
#[pymethods]
impl PyArtifact {
    #[new]
    fn new() -> Self {
        Self {
            inner: crate::writer::Artifact::new(),
        }
    }
    #[staticmethod]
    fn center(t: &str) -> Self {
        Self {
            inner: crate::writer::Artifact::center(t),
        }
    }
    fn with_left<'a>(mut slf: PyRefMut<'a, Self>, t: &str) -> PyResult<PyRefMut<'a, Self>> {
        slf.inner = slf.inner.clone().with_left(t);
        Ok(slf)
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Header", from_py_object)]
#[derive(Clone)]
pub struct PyHeader {
    pub inner: PyArtifact,
}
#[pymethods]
impl PyHeader {
    #[new]
    fn new() -> Self {
        Self {
            inner: PyArtifact {
                inner: crate::writer::Artifact::new(),
            },
        }
    }
    #[staticmethod]
    fn center(t: &str) -> Self {
        Self {
            inner: PyArtifact {
                inner: crate::writer::Artifact::center(t),
            },
        }
    }
}

#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Footer", from_py_object)]
#[derive(Clone)]
pub struct PyFooter {
    pub inner: PyArtifact,
}
#[pymethods]
impl PyFooter {
    #[new]
    fn new() -> Self {
        Self {
            inner: PyArtifact {
                inner: crate::writer::Artifact::new(),
            },
        }
    }
    #[staticmethod]
    fn center(t: &str) -> Self {
        Self {
            inner: PyArtifact {
                inner: crate::writer::Artifact::center(t),
            },
        }
    }
}

/// Parse a stamp-type name into the Rust `StampType` enum. Unknown names
/// fall through to `StampType::Custom(String)` so callers can use any
/// text they like. Shared by the Python / WASM / FFI write-side bindings.
fn parse_stamp_type(name: &str) -> crate::writer::StampType {
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

// =============================================================================
// Write-side API: DocumentBuilder, FluentPageBuilder, EmbeddedFont
// =============================================================================
//
// These three pyclasses expose the Rust write-side fluent API to Python:
// register an embedded TTF, build a multi-page PDF with CJK / Cyrillic /
// Greek text, save to bytes or file, with optional AES-256 encryption.
//
// Architectural note: the Rust `FluentPageBuilder<'a>` carries a mutable
// borrow of `DocumentBuilder`, which pyo3 cannot represent across GIL
// boundaries. `PyFluentPageBuilder` therefore **buffers** its operations
// in a `Vec<PendingPageOp>` and applies them in one shot against a real
// Rust `FluentPageBuilder` inside `done()`. This also gives Python users
// a natural "build the page, then commit it" mental model.

/// Buffered operations that `PyFluentPageBuilder` replays against the real
/// Rust `FluentPageBuilder` inside `done()`. Each variant mirrors a method
/// on the Rust builder so the Python fluent chain maps 1:1 onto the Rust
/// fluent chain at commit time.
enum PendingPageOp {
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
    TextInRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: String,
        align: i32,
    },
    NewPageSameSize,
    Table {
        widths: Vec<f32>,
        aligns: Vec<i32>,
        rows: Vec<Vec<String>>,
        has_header: bool,
    },
    StreamingTable {
        headers: Vec<String>,
        widths: Vec<f32>,
        aligns: Vec<i32>,
        repeat_header: bool,
        /// Each cell is (text, rowspan); rowspan==1 is a normal cell.
        rows: Vec<Vec<(String, usize)>>,
        /// "fixed" | "sample" | "auto_all"
        mode: String,
        sample_rows: usize,
        min_col_width_pt: f32,
        max_col_width_pt: f32,
        max_rowspan: usize,
    },
    /// Pre-rendered barcode PNG (generated at record time so errors
    /// surface at the Python call site, not during replay).
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

fn parse_align_to_cell(i: i32) -> crate::writer::CellAlign {
    match i {
        1 => crate::writer::CellAlign::Center,
        2 => crate::writer::CellAlign::Right,
        _ => crate::writer::CellAlign::Left,
    }
}

fn parse_align_to_text(i: i32) -> crate::writer::TextAlign {
    match i {
        1 => crate::writer::TextAlign::Center,
        2 => crate::writer::TextAlign::Right,
        _ => crate::writer::TextAlign::Left,
    }
}

fn align_str_to_int(s: &str) -> PyResult<i32> {
    match s.to_ascii_lowercase().as_str() {
        "left" | "l" => Ok(0),
        "center" | "centre" | "c" => Ok(1),
        "right" | "r" => Ok(2),
        other => Err(PyValueError::new_err(format!(
            "invalid align '{}': expected 'left', 'center', or 'right'",
            other
        ))),
    }
}

fn extract_align(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<i32> {
    // Accept str, int, or the Align pyclass itself.
    if let Ok(s) = obj.extract::<String>() {
        return align_str_to_int(&s);
    }
    if let Ok(a) = obj.extract::<PyRef<PyAlign>>() {
        return Ok(*a as i32);
    }
    if let Ok(i) = obj.extract::<i32>() {
        if (0..=2).contains(&i) {
            return Ok(i);
        }
    }
    let _ = py;
    Err(PyValueError::new_err(
        "align must be 'left'/'center'/'right' or an Align enum value",
    ))
}

/// Python-side horizontal alignment enum. Maps 1:1 to `CellAlign` /
/// `TextAlign` in the Rust core. Values are plain ints so the class can
/// be used interchangeably with the string form ("left"/"center"/"right")
/// anywhere the Python bindings accept alignment.
#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "Align",
    eq,
    eq_int,
    skip_from_py_object
)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyAlign {
    Left = 0,
    Center = 1,
    Right = 2,
}

#[pymethods]
impl PyAlign {
    #[classattr]
    const LEFT: PyAlign = PyAlign::Left;
    #[classattr]
    const CENTER: PyAlign = PyAlign::Center;
    #[classattr]
    const RIGHT: PyAlign = PyAlign::Right;

    fn __int__(&self) -> i32 {
        *self as i32
    }

    fn __repr__(&self) -> &'static str {
        match self {
            PyAlign::Left => "Align.LEFT",
            PyAlign::Center => "Align.CENTER",
            PyAlign::Right => "Align.RIGHT",
        }
    }
}

/// Python-side column descriptor used by `Table`
/// `FluentPageBuilder.streaming_table`. Constructor matches the research
/// C shape: `Column(header, width=100.0, align=Align.LEFT)`.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Column", skip_from_py_object)]
#[derive(Clone)]
pub struct PyColumn {
    pub header: String,
    pub width: f32,
    pub align: i32,
}

#[pymethods]
impl PyColumn {
    #[new]
    #[pyo3(signature = (header, width=100.0, align=None))]
    fn new(header: String, width: f32, align: Option<Bound<'_, PyAny>>) -> PyResult<Self> {
        let align_i = match align {
            Some(obj) => extract_align(obj.py(), &obj)?,
            None => 0,
        };
        Ok(Self {
            header,
            width,
            align: align_i,
        })
    }

    #[getter]
    fn header(&self) -> &str {
        &self.header
    }
    #[getter]
    fn width(&self) -> f32 {
        self.width
    }
    #[getter]
    fn align(&self) -> i32 {
        self.align
    }

    fn __repr__(&self) -> String {
        format!("Column(header={:?}, width={}, align={})", self.header, self.width, self.align)
    }
}

/// Python-side buffered-table value object consumed by
/// `FluentPageBuilder.table`. Carries columns (with widths + alignments
/// + headers), rows of string cells, and a `has_header` flag that
/// promotes the first row to the header style.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Table", skip_from_py_object)]
#[derive(Clone)]
pub struct PyTable {
    pub columns: Vec<PyColumn>,
    pub rows: Vec<Vec<String>>,
    pub has_header: bool,
}

#[pymethods]
impl PyTable {
    #[new]
    #[pyo3(signature = (columns, rows, has_header=false))]
    fn new(
        columns: Vec<PyRef<'_, PyColumn>>,
        rows: Vec<Vec<String>>,
        has_header: bool,
    ) -> PyResult<Self> {
        let cols: Vec<PyColumn> = columns.into_iter().map(|c| (*c).clone()).collect();
        let n_cols = cols.len();
        if n_cols == 0 {
            return Err(PyValueError::new_err("Table requires at least one Column"));
        }
        for (i, row) in rows.iter().enumerate() {
            if row.len() != n_cols {
                return Err(PyValueError::new_err(format!(
                    "Table row {} has {} cells, expected {}",
                    i,
                    row.len(),
                    n_cols
                )));
            }
        }
        Ok(Self {
            columns: cols,
            rows,
            has_header,
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "Table(columns={}, rows={}, has_header={})",
            self.columns.len(),
            self.rows.len(),
            self.has_header
        )
    }
}

/// Python wrapper for an embedded TTF/OTF font usable by `DocumentBuilder`.
///
/// `EmbeddedFont` is a one-shot handle: once it is passed to
/// `DocumentBuilder.register_embedded_font`, the underlying Rust
/// `EmbeddedFont` is moved into the builder and this handle becomes
/// empty. Registering the same handle twice raises `RuntimeError`.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "EmbeddedFont")]
pub struct PyEmbeddedFont {
    pub(crate) inner: Option<crate::writer::EmbeddedFont>,
}

#[pymethods]
impl PyEmbeddedFont {
    /// Load an embedded TTF / OTF font from a file path. The PostScript
    /// name baked into the font face is used as the default PDF font
    /// name when registered without an override.
    #[staticmethod]
    fn from_file(path: &str) -> PyResult<Self> {
        crate::writer::EmbeddedFont::from_file(path)
            .map(|inner| Self { inner: Some(inner) })
            .map_err(|e| PyIOError::new_err(format!("failed to load font: {e}")))
    }

    /// Load an embedded font from a Python `bytes` / `bytearray`. Pass
    /// `name` to override the PostScript name the PDF will record.
    #[staticmethod]
    #[pyo3(signature = (data, name=None))]
    fn from_bytes(data: &Bound<'_, PyBytes>, name: Option<String>) -> PyResult<Self> {
        let bytes = data.as_bytes().to_vec();
        crate::writer::EmbeddedFont::from_data(name, bytes)
            .map(|inner| Self { inner: Some(inner) })
            .map_err(|e| PyValueError::new_err(format!("failed to parse font: {e}")))
    }

    /// The font's PostScript name (or the override passed to
    /// `from_bytes`). Empty string after the font has been consumed.
    #[getter]
    fn name(&self) -> &str {
        self.inner.as_ref().map(|f| f.name.as_str()).unwrap_or("")
    }

    fn __repr__(&self) -> String {
        match self.inner.as_ref() {
            Some(f) => format!("EmbeddedFont('{}')", f.name),
            None => "EmbeddedFont(<consumed>)".to_string(),
        }
    }
}

/// Python wrapper for `crate::writer::DocumentBuilder`, the high-level
/// fluent PDF-creation API.
///
/// Methods mutate `self` in place and return `self` so that Python can
/// use a fluent chain:
///
/// ```python
/// pdf_bytes = (
///     DocumentBuilder()
///     .title("Hello")
///     .register_embedded_font("DejaVu", EmbeddedFont.from_file("DejaVuSans.ttf"))
///     .a4_page()
///         .font("DejaVu", 12.0)
///         .at(72.0, 720.0).text("Привет, мир!")
///         .done()
///     .build()
/// )
/// ```
///
/// `build()`, `save()`, `save_encrypted()`, `to_bytes_encrypted()`,
/// `save_with_encryption()` **consume** the builder — subsequent calls
/// on the same instance raise `RuntimeError`.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "DocumentBuilder")]
pub struct PyDocumentBuilder {
    pub(crate) inner: Option<crate::writer::DocumentBuilder>,
}

impl PyDocumentBuilder {
    fn take_inner(&mut self, ctx: &str) -> PyResult<crate::writer::DocumentBuilder> {
        self.inner.take().ok_or_else(|| {
            PyRuntimeError::new_err(format!("DocumentBuilder already consumed ({ctx})"))
        })
    }

    fn with_inner<F>(&mut self, ctx: &str, f: F) -> PyResult<()>
    where
        F: FnOnce(crate::writer::DocumentBuilder) -> crate::writer::DocumentBuilder,
    {
        let taken = self.take_inner(ctx)?;
        self.inner = Some(f(taken));
        Ok(())
    }
}

#[pymethods]
impl PyDocumentBuilder {
    #[new]
    fn new() -> Self {
        Self {
            inner: Some(crate::writer::DocumentBuilder::new()),
        }
    }

    fn title<'a>(mut slf: PyRefMut<'a, Self>, title: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.with_inner("title", |b| b.title(title))?;
        Ok(slf)
    }

    fn author<'a>(mut slf: PyRefMut<'a, Self>, author: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.with_inner("author", |b| b.author(author))?;
        Ok(slf)
    }

    fn subject<'a>(mut slf: PyRefMut<'a, Self>, subject: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.with_inner("subject", |b| b.subject(subject))?;
        Ok(slf)
    }

    fn keywords<'a>(mut slf: PyRefMut<'a, Self>, keywords: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.with_inner("keywords", |b| b.keywords(keywords))?;
        Ok(slf)
    }

    fn creator<'a>(mut slf: PyRefMut<'a, Self>, creator: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.with_inner("creator", |b| b.creator(creator))?;
        Ok(slf)
    }

    fn on_open<'a>(mut slf: PyRefMut<'a, Self>, script: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.with_inner("on_open", |b| b.on_open(script))?;
        Ok(slf)
    }

    /// Enable PDF/UA-1 tagged PDF mode.
    ///
    /// When enabled, `build()` emits `/MarkInfo`, `/StructTreeRoot`, `/Lang`,
    /// and `/ViewerPreferences` in the catalog. Safe to ignore — has no effect
    /// on documents that don't call this method (strict opt-in). Bundle F-1/F-2.
    fn tagged_pdf_ua1<'a>(mut slf: PyRefMut<'a, Self>) -> PyResult<PyRefMut<'a, Self>> {
        slf.with_inner("tagged_pdf_ua1", |b| b.tagged_pdf_ua1())?;
        Ok(slf)
    }

    /// Set the document's natural language tag (e.g. `"en-US"`).
    ///
    /// Emitted as `/Lang` in the catalog when `tagged_pdf_ua1()` is set.
    fn language<'a>(mut slf: PyRefMut<'a, Self>, lang: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.with_inner("language", |b| b.language(lang))?;
        Ok(slf)
    }

    /// Add a role-map entry: custom structure type → standard PDF structure type.
    ///
    /// Emitted in `/RoleMap` inside the StructTreeRoot when `tagged_pdf_ua1()`
    /// is set. Multiple calls accumulate entries.
    fn role_map<'a>(
        mut slf: PyRefMut<'a, Self>,
        custom: String,
        standard: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.with_inner("role_map", |b| b.role_map(custom, standard))?;
        Ok(slf)
    }

    /// Register a TTF/OTF font the PDF pages can reference by name. The
    /// `EmbeddedFont` handle is **consumed** — reusing it raises
    /// `RuntimeError`.
    fn register_embedded_font<'a>(
        mut slf: PyRefMut<'a, Self>,
        name: String,
        font: &Bound<'_, PyEmbeddedFont>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        let embedded = font
            .borrow_mut()
            .inner
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("EmbeddedFont already consumed"))?;
        slf.with_inner("register_embedded_font", |b| b.register_embedded_font(name, embedded))?;
        Ok(slf)
    }

    /// Start a new A4 page and return a `FluentPageBuilder`. Call
    /// `.done()` on the returned builder to commit the page.
    #[pyo3(signature = ())]
    fn a4_page(slf_handle: Py<Self>) -> PyFluentPageBuilder {
        PyFluentPageBuilder {
            parent: slf_handle,
            page_size: Some(crate::writer::PageSize::A4),
            custom_width: 0.0,
            custom_height: 0.0,
            ops: Vec::new(),
            done_called: false,
            current_font: "Helvetica".to_string(),
            current_size: 12.0,
            last_y: None,
        }
    }

    #[pyo3(signature = ())]
    fn letter_page(slf_handle: Py<Self>) -> PyFluentPageBuilder {
        PyFluentPageBuilder {
            parent: slf_handle,
            page_size: Some(crate::writer::PageSize::Letter),
            custom_width: 0.0,
            custom_height: 0.0,
            ops: Vec::new(),
            done_called: false,
            current_font: "Helvetica".to_string(),
            current_size: 12.0,
            last_y: None,
        }
    }

    /// Start a new page with custom dimensions in PDF points
    /// (72 pt = 1 inch). Use for non-standard paper sizes.
    #[pyo3(signature = (width, height))]
    fn page(slf_handle: Py<Self>, width: f32, height: f32) -> PyFluentPageBuilder {
        PyFluentPageBuilder {
            parent: slf_handle,
            page_size: None,
            custom_width: width,
            custom_height: height,
            ops: Vec::new(),
            done_called: false,
            current_font: "Helvetica".to_string(),
            current_size: 12.0,
            last_y: None,
        }
    }

    /// Build the PDF and return it as `bytes`. **Consumes** the builder.
    fn build<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let inner = self.take_inner("build")?;
        let bytes = inner
            .build()
            .map_err(|e| PyRuntimeError::new_err(format!("build failed: {e}")))?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Build and save the PDF to `path`. **Consumes** the builder.
    fn save(&mut self, path: &str) -> PyResult<()> {
        let inner = self.take_inner("save")?;
        inner
            .save(path)
            .map_err(|e| PyIOError::new_err(format!("save failed: {e}")))
    }

    /// Build and save the PDF with AES-256 encryption. Default grants
    /// all permissions; pass a custom `EncryptionConfig` via
    /// `save_with_encryption` for fine-grained control. **Consumes**
    /// the builder.
    fn save_encrypted(
        &mut self,
        path: &str,
        user_password: &str,
        owner_password: &str,
    ) -> PyResult<()> {
        let inner = self.take_inner("save_encrypted")?;
        inner
            .save_encrypted(path, user_password, owner_password)
            .map_err(|e| PyIOError::new_err(format!("save_encrypted failed: {e}")))
    }

    /// Build and return the encrypted PDF as `bytes` using AES-256.
    /// **Consumes** the builder.
    fn to_bytes_encrypted<'py>(
        &mut self,
        py: Python<'py>,
        user_password: &str,
        owner_password: &str,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let inner = self.take_inner("to_bytes_encrypted")?;
        let bytes = inner
            .to_bytes_encrypted(user_password, owner_password)
            .map_err(|e| PyRuntimeError::new_err(format!("to_bytes_encrypted failed: {e}")))?;
        Ok(PyBytes::new(py, &bytes))
    }
}

/// Python wrapper that buffers page-level operations until `done()`.
///
/// See [`PyDocumentBuilder`] for the fluent pattern. This class holds a
/// reference back to its parent `DocumentBuilder` and is single-use:
/// once `done()` is called, subsequent method calls raise
/// `RuntimeError`.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "FluentPageBuilder")]
pub struct PyFluentPageBuilder {
    parent: Py<PyDocumentBuilder>,
    page_size: Option<crate::writer::PageSize>,
    custom_width: f32,
    custom_height: f32,
    ops: Vec<PendingPageOp>,
    done_called: bool,
    /// Best-effort current font tracking for `measure()` — updated on
    /// every buffered `font()` call. Pure client-side cache; the Rust
    /// builder still owns authoritative state after `done()`.
    current_font: String,
    current_size: f32,
    /// Last `at()` y coordinate, used for the client-side
    /// `remaining_space` estimate. `None` means "unknown — assume top
    /// margin of the page".
    last_y: Option<f32>,
}

impl PyFluentPageBuilder {
    fn push(&mut self, op: PendingPageOp) -> PyResult<()> {
        if self.done_called {
            return Err(PyRuntimeError::new_err("FluentPageBuilder.done() already called"));
        }
        self.ops.push(op);
        Ok(())
    }
}

#[pymethods]
impl PyFluentPageBuilder {
    fn font<'a>(
        mut slf: PyRefMut<'a, Self>,
        name: String,
        size: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.current_font = name.clone();
        slf.current_size = size;
        slf.push(PendingPageOp::Font(name, size))?;
        Ok(slf)
    }

    fn at<'a>(mut slf: PyRefMut<'a, Self>, x: f32, y: f32) -> PyResult<PyRefMut<'a, Self>> {
        slf.last_y = Some(y);
        slf.push(PendingPageOp::At(x, y))?;
        Ok(slf)
    }

    fn text<'a>(mut slf: PyRefMut<'a, Self>, text: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Text(text))?;
        Ok(slf)
    }

    fn heading<'a>(
        mut slf: PyRefMut<'a, Self>,
        level: u8,
        text: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Heading(level, text))?;
        Ok(slf)
    }

    fn paragraph<'a>(mut slf: PyRefMut<'a, Self>, text: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Paragraph(text))?;
        Ok(slf)
    }

    fn space<'a>(mut slf: PyRefMut<'a, Self>, points: f32) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Space(points))?;
        Ok(slf)
    }

    fn horizontal_rule<'a>(mut slf: PyRefMut<'a, Self>) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::HorizontalRule)?;
        Ok(slf)
    }

    // -----------------------------------------------------------------
    // Annotation methods — operate on the *previous* text element just
    // as in the Rust API.
    // -----------------------------------------------------------------

    fn link_url<'a>(mut slf: PyRefMut<'a, Self>, url: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::LinkUrl(url))?;
        Ok(slf)
    }

    fn link_page<'a>(mut slf: PyRefMut<'a, Self>, page: usize) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::LinkPage(page))?;
        Ok(slf)
    }

    fn link_named<'a>(
        mut slf: PyRefMut<'a, Self>,
        destination: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::LinkNamed(destination))?;
        Ok(slf)
    }

    fn link_javascript<'a>(
        mut slf: PyRefMut<'a, Self>,
        script: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::LinkJavaScript(script))?;
        Ok(slf)
    }

    fn on_open<'a>(mut slf: PyRefMut<'a, Self>, script: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::OnOpen(script))?;
        Ok(slf)
    }

    fn on_close<'a>(mut slf: PyRefMut<'a, Self>, script: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::OnClose(script))?;
        Ok(slf)
    }

    fn field_keystroke<'a>(
        mut slf: PyRefMut<'a, Self>,
        script: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::FieldKeystroke(script))?;
        Ok(slf)
    }

    fn field_format<'a>(
        mut slf: PyRefMut<'a, Self>,
        script: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::FieldFormat(script))?;
        Ok(slf)
    }

    fn field_validate<'a>(
        mut slf: PyRefMut<'a, Self>,
        script: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::FieldValidate(script))?;
        Ok(slf)
    }

    fn field_calculate<'a>(
        mut slf: PyRefMut<'a, Self>,
        script: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::FieldCalculate(script))?;
        Ok(slf)
    }

    fn highlight<'a>(
        mut slf: PyRefMut<'a, Self>,
        color: (f32, f32, f32),
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Highlight(color.0, color.1, color.2))?;
        Ok(slf)
    }

    fn underline<'a>(
        mut slf: PyRefMut<'a, Self>,
        color: (f32, f32, f32),
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Underline(color.0, color.1, color.2))?;
        Ok(slf)
    }

    fn strikeout<'a>(
        mut slf: PyRefMut<'a, Self>,
        color: (f32, f32, f32),
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Strikeout(color.0, color.1, color.2))?;
        Ok(slf)
    }

    fn squiggly<'a>(
        mut slf: PyRefMut<'a, Self>,
        color: (f32, f32, f32),
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Squiggly(color.0, color.1, color.2))?;
        Ok(slf)
    }

    fn sticky_note<'a>(mut slf: PyRefMut<'a, Self>, text: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::StickyNote(text))?;
        Ok(slf)
    }

    fn sticky_note_at<'a>(
        mut slf: PyRefMut<'a, Self>,
        x: f32,
        y: f32,
        text: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::StickyNoteAt(x, y, text))?;
        Ok(slf)
    }

    fn watermark<'a>(mut slf: PyRefMut<'a, Self>, text: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Watermark(text))?;
        Ok(slf)
    }

    fn watermark_confidential<'a>(mut slf: PyRefMut<'a, Self>) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::WatermarkConfidential)?;
        Ok(slf)
    }

    fn watermark_draft<'a>(mut slf: PyRefMut<'a, Self>) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::WatermarkDraft)?;
        Ok(slf)
    }

    /// Attach a standard stamp annotation at the current cursor position
    /// with the default 150×50 point box. Valid names are the PDF spec's
    /// standard stamp types ("Approved", "NotApproved", "Draft",
    /// "Confidential", "Final", "Experimental", "Expired",
    /// "ForPublicRelease", "NotForPublicRelease", "AsIs", "Sold",
    /// "Departmental", "ForComment", "TopSecret") — any other name is
    /// emitted verbatim as a custom stamp.
    fn stamp<'a>(mut slf: PyRefMut<'a, Self>, name: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Stamp(name))?;
        Ok(slf)
    }

    /// Place free-flowing text inside a rectangular annotation (no cursor
    /// advance; independent of the `at()`/`text()` flow).
    fn freetext<'a>(
        mut slf: PyRefMut<'a, Self>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::FreeText { x, y, w, h, text })?;
        Ok(slf)
    }

    /// Add a single-line text form field at the given rectangle.
    /// `default_value` is the initial text; pass `None` or an empty
    /// string for a blank field.
    #[pyo3(signature = (name, x, y, w, h, default_value=None))]
    fn text_field<'a>(
        mut slf: PyRefMut<'a, Self>,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        default_value: Option<String>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::TextField {
            name,
            x,
            y,
            w,
            h,
            default_value,
        })?;
        Ok(slf)
    }

    /// Add a checkbox form field at the given rectangle. `checked`
    /// sets the initial state.
    fn checkbox<'a>(
        mut slf: PyRefMut<'a, Self>,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        checked: bool,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Checkbox {
            name,
            x,
            y,
            w,
            h,
            checked,
        })?;
        Ok(slf)
    }

    /// Add a dropdown combo-box form field. `options` are the user-
    /// visible choices (also the submitted values); `selected` picks
    /// the initial value (or pass `None` to leave blank).
    #[pyo3(signature = (name, x, y, w, h, options, selected=None))]
    fn combo_box<'a>(
        mut slf: PyRefMut<'a, Self>,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        options: Vec<String>,
        selected: Option<String>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::ComboBox {
            name,
            x,
            y,
            w,
            h,
            options,
            selected,
        })?;
        Ok(slf)
    }

    /// Add a radio-button group. `buttons` is a list of
    /// `(export_value, x, y, w, h)` tuples, one per option. `selected`
    /// picks the initial value.
    #[pyo3(signature = (name, buttons, selected=None))]
    fn radio_group<'a>(
        mut slf: PyRefMut<'a, Self>,
        name: String,
        buttons: Vec<(String, f32, f32, f32, f32)>,
        selected: Option<String>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::RadioGroup {
            name,
            buttons,
            selected,
        })?;
        Ok(slf)
    }

    /// Add a clickable push button with a visible caption.
    fn push_button<'a>(
        mut slf: PyRefMut<'a, Self>,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        caption: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::PushButton {
            name,
            x,
            y,
            w,
            h,
            caption,
        })?;
        Ok(slf)
    }

    /// Add an unsigned signature placeholder field at the given bounds.
    fn signature_field<'a>(
        mut slf: PyRefMut<'a, Self>,
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::SignatureField { name, x, y, w, h })?;
        Ok(slf)
    }

    /// Add a footnote: inline `ref_mark` at the cursor + `note_text` body
    /// placed near the page bottom with a separator artifact line.
    fn footnote<'a>(
        mut slf: PyRefMut<'a, Self>,
        ref_mark: String,
        note_text: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Footnote {
            ref_mark,
            note_text,
        })?;
        Ok(slf)
    }

    /// Lay out `text` as balanced multi-column flow.
    /// `column_count` columns with `gap_pt` points between them.
    /// Paragraphs may be separated by `"\\n\\n"` in `text`.
    fn columns<'a>(
        mut slf: PyRefMut<'a, Self>,
        column_count: u32,
        gap_pt: f32,
        text: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Columns {
            count: column_count,
            gap_pt,
            text,
        })?;
        Ok(slf)
    }

    /// Emit `text` inline at the cursor (advances cursor_x, not cursor_y).
    fn inline<'a>(mut slf: PyRefMut<'a, Self>, text: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Inline(text))?;
        Ok(slf)
    }

    /// Inline bold run.
    fn inline_bold<'a>(mut slf: PyRefMut<'a, Self>, text: String) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::InlineBold(text))?;
        Ok(slf)
    }

    /// Inline italic run.
    fn inline_italic<'a>(
        mut slf: PyRefMut<'a, Self>,
        text: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::InlineItalic(text))?;
        Ok(slf)
    }

    /// Inline colored run (RGB 0.0–1.0).
    fn inline_color<'a>(
        mut slf: PyRefMut<'a, Self>,
        r: f32,
        g: f32,
        b: f32,
        text: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::InlineColor { r, g, b, text })?;
        Ok(slf)
    }

    /// Advance cursor_y by one line-height and reset cursor_x to 72 pt.
    fn newline<'a>(mut slf: PyRefMut<'a, Self>) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Newline)?;
        Ok(slf)
    }

    /// Place a 1-D barcode image at `(x, y, w, h)` on the page.
    /// `barcode_type`: 0=Code128 1=Code39 2=EAN13 3=EAN8 4=UPCA 5=ITF
    /// 6=Code93 7=Codabar. Errors surface here (at call time), not at
    /// `done()`.
    fn barcode_1d<'a>(
        mut slf: PyRefMut<'a, Self>,
        barcode_type: i32,
        data: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
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
                return Err(PyValueError::new_err(format!(
                    "unknown barcode_type {barcode_type}; valid values are 0–7"
                )))
            },
        };
        let opts = crate::writer::BarcodeOptions::new()
            .width(w as u32)
            .height(h as u32);
        let bytes = crate::writer::BarcodeGenerator::generate_1d(bt, &data, &opts)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        slf.push(PendingPageOp::BarcodeImage { bytes, x, y, w, h })?;
        Ok(slf)
    }

    /// Place a QR-code image at `(x, y, size, size)` on the page.
    /// Errors surface here (at call time), not at `done()`.
    fn barcode_qr<'a>(
        mut slf: PyRefMut<'a, Self>,
        data: String,
        x: f32,
        y: f32,
        size: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        let opts = crate::writer::QrCodeOptions::new().size(size as u32);
        let bytes = crate::writer::BarcodeGenerator::generate_qr(&data, &opts)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        slf.push(PendingPageOp::BarcodeImage {
            bytes,
            x,
            y,
            w: size,
            h: size,
        })?;
        Ok(slf)
    }

    /// Embed an image (JPEG/PNG/WebP bytes) with an accessibility alt text.
    fn image_with_alt<'a>(
        mut slf: PyRefMut<'a, Self>,
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        alt_text: String,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::ImageWithAlt {
            bytes,
            x,
            y,
            w,
            h,
            alt_text,
        })?;
        Ok(slf)
    }

    /// Embed a decorative image (JPEG/PNG/WebP bytes) as an /Artifact (no alt text).
    fn image_artifact<'a>(
        mut slf: PyRefMut<'a, Self>,
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::ImageArtifact { bytes, x, y, w, h })?;
        Ok(slf)
    }

    /// Draw a stroked rectangle outline (1pt black).
    fn rect<'a>(
        mut slf: PyRefMut<'a, Self>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Rect(x, y, w, h))?;
        Ok(slf)
    }

    /// Draw a filled rectangle. RGB channels in 0.0–1.0.
    fn filled_rect<'a>(
        mut slf: PyRefMut<'a, Self>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        g: f32,
        b: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::FilledRect(x, y, w, h, r, g, b))?;
        Ok(slf)
    }

    /// Draw a line from (x1, y1) to (x2, y2) with 1pt black stroke.
    fn line<'a>(
        mut slf: PyRefMut<'a, Self>,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::Line(x1, y1, x2, y2))?;
        Ok(slf)
    }

    // ── v0.3.39 primitives + tables (#393 step 6a) ──────────────────────

    /// Measure the rendered width of `text` in PDF points, using the
    /// most recently set `font()` / size (defaults: Helvetica, 12 pt).
    ///
    /// Implemented against a standalone base-14 `FontManager`, so any
    /// PostScript name resolvable to a base-14 face (Helvetica, Times-
    /// Roman, Courier, and their Bold/Italic variants) is measured
    /// accurately. Custom embedded fonts registered on the
    /// `DocumentBuilder` fall back to Helvetica metrics until a future
    /// release routes measurement through the parent builder's
    /// `FontManager`.
    fn measure(&self, text: &str) -> PyResult<f32> {
        let fm = crate::writer::FontManager::new();
        Ok(fm.text_width(text, &self.current_font, self.current_size))
    }

    /// Best-effort vertical space between the last known cursor y
    /// the bottom margin (72 pt). Because `PyFluentPageBuilder` buffers
    /// ops until `done()`, this is a client-side estimate: it returns
    /// `last_at_y - 72` when `at()` has been called, else `page_height
    /// - 144` (standard 1" top + bottom margins).
    ///
    /// For authoritative page-break decisions during streaming table
    /// rendering, prefer `streaming_table(..., repeat_header=True)` —
    /// the Rust core handles page breaks internally.
    fn remaining_space(&self) -> f32 {
        let page_height = match self.page_size {
            Some(crate::writer::PageSize::A4) => 842.0,
            Some(crate::writer::PageSize::Letter) => 792.0,
            Some(crate::writer::PageSize::Legal) => 1008.0,
            Some(crate::writer::PageSize::Custom(_, h)) => h,
            _ => self.custom_height,
        };
        let y = self.last_y.unwrap_or(page_height - 72.0);
        (y - 72.0).max(0.0)
    }

    /// Place wrapped text inside a rectangle with horizontal alignment.
    /// `align` accepts `"left"`, `"center"`, `"right"` (case-insensitive)
    /// or an `Align` enum value.
    #[pyo3(signature = (x, y, w, h, text, align=None))]
    fn text_in_rect<'a>(
        mut slf: PyRefMut<'a, Self>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: String,
        align: Option<Bound<'_, PyAny>>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        let align_i = match align {
            Some(obj) => extract_align(obj.py(), &obj)?,
            None => 0,
        };
        slf.push(PendingPageOp::TextInRect {
            x,
            y,
            w,
            h,
            text,
            align: align_i,
        })?;
        Ok(slf)
    }

    /// Draw a stroked rectangle with explicit width (points) and RGB
    /// colour (channels 0..1).
    #[pyo3(signature = (x, y, w, h, width=1.0, color=(0.0, 0.0, 0.0)))]
    fn stroke_rect<'a>(
        mut slf: PyRefMut<'a, Self>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        width: f32,
        color: (f32, f32, f32),
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::StrokeRect {
            x,
            y,
            w,
            h,
            width,
            r: color.0,
            g: color.1,
            b: color.2,
        })?;
        Ok(slf)
    }

    /// Draw a dashed rectangle border. `dash` is alternating on/off lengths in points.
    #[pyo3(signature = (x, y, w, h, dash, width=1.0, color=(0.0, 0.0, 0.0), phase=0.0))]
    fn stroke_rect_dashed<'a>(
        mut slf: PyRefMut<'a, Self>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        dash: Vec<f32>,
        width: f32,
        color: (f32, f32, f32),
        phase: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::StrokeRectDashed {
            x,
            y,
            w,
            h,
            width,
            r: color.0,
            g: color.1,
            b: color.2,
            dash,
            phase,
        })?;
        Ok(slf)
    }

    /// Draw a line with explicit width (points) and RGB colour.
    #[pyo3(signature = (x1, y1, x2, y2, width=1.0, color=(0.0, 0.0, 0.0)))]
    fn stroke_line<'a>(
        mut slf: PyRefMut<'a, Self>,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        color: (f32, f32, f32),
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::StrokeLine {
            x1,
            y1,
            x2,
            y2,
            width,
            r: color.0,
            g: color.1,
            b: color.2,
        })?;
        Ok(slf)
    }

    /// Draw a dashed line. `dash` is alternating on/off lengths in points.
    #[pyo3(signature = (x1, y1, x2, y2, dash, width=1.0, color=(0.0, 0.0, 0.0), phase=0.0))]
    fn stroke_line_dashed<'a>(
        mut slf: PyRefMut<'a, Self>,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        dash: Vec<f32>,
        width: f32,
        color: (f32, f32, f32),
        phase: f32,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.push(PendingPageOp::StrokeLineDashed {
            x1,
            y1,
            x2,
            y2,
            width,
            r: color.0,
            g: color.1,
            b: color.2,
            dash,
            phase,
        })?;
        Ok(slf)
    }

    /// Finish the current page and start a fresh one with the same
    /// dimensions. Subsequent buffered ops land on the new page. The
    /// current font / size tracking carries over.
    fn new_page_same_size<'a>(mut slf: PyRefMut<'a, Self>) -> PyResult<PyRefMut<'a, Self>> {
        slf.last_y = None;
        slf.push(PendingPageOp::NewPageSameSize)?;
        Ok(slf)
    }

    /// Emit a buffered `Table` — the whole row matrix is rendered in
    /// one pass at `done()` time. Column widths come from the
    /// `Column.width` on each `Column`; per-column alignment from
    /// `Column.align`.
    fn table<'a>(
        mut slf: PyRefMut<'a, Self>,
        table: &Bound<'_, PyTable>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        let t = table.borrow();
        let widths: Vec<f32> = t.columns.iter().map(|c| c.width).collect();
        let aligns: Vec<i32> = t.columns.iter().map(|c| c.align).collect();
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(t.rows.len() + 1);
        if t.has_header {
            // If has_header, the first user row isn't the header — the
            // columns' own .header strings are. Inject a synthetic header
            // row at the top so buffered Table::with_header_row works.
            rows.push(t.columns.iter().map(|c| c.header.clone()).collect());
        }
        rows.extend(t.rows.iter().cloned());
        slf.push(PendingPageOp::Table {
            widths,
            aligns,
            rows,
            has_header: t.has_header,
        })?;
        Ok(slf)
    }

    /// Open a `StreamingTable` bound to this page. Cells are pushed
    /// row-at-a-time via the returned handle; `finish()` returns this
    /// `FluentPageBuilder` for further chaining. Row emission is
    /// deferred until `done()` (all rows are collected first, then
    /// streamed through the Rust `StreamingTable` at commit time).
    ///
    /// `columns` is a list of `Column`; `repeat_header=True` redraws
    /// the header row at every page break.
    #[pyo3(signature = (columns, repeat_header=false, mode="fixed", sample_rows=50, min_col_width_pt=20.0, max_col_width_pt=400.0, max_rowspan=1, batch_size=256))]
    fn streaming_table(
        slf_handle: Py<Self>,
        py: Python<'_>,
        columns: Vec<PyRef<'_, PyColumn>>,
        repeat_header: bool,
        mode: &str,
        sample_rows: usize,
        min_col_width_pt: f32,
        max_col_width_pt: f32,
        max_rowspan: usize,
        batch_size: usize,
    ) -> PyResult<PyStreamingTable> {
        if columns.is_empty() {
            return Err(PyValueError::new_err("streaming_table requires at least one Column"));
        }
        if batch_size == 0 {
            return Err(PyValueError::new_err("batch_size must be >= 1"));
        }
        let cols: Vec<PyColumn> = columns.into_iter().map(|c| (*c).clone()).collect();
        let _ = py;
        Ok(PyStreamingTable {
            parent: slf_handle,
            columns: cols,
            repeat_header,
            current_batch: Vec::new(),
            completed_batches: Vec::new(),
            finished: false,
            mode: mode.to_string(),
            sample_rows,
            min_col_width_pt,
            max_col_width_pt,
            max_rowspan,
            batch_size,
        })
    }

    /// Commit the page's buffered operations to the parent
    /// `DocumentBuilder` and return the parent for further chaining.
    /// After `done()`, this `FluentPageBuilder` is spent.
    fn done(&mut self, py: Python) -> PyResult<Py<PyDocumentBuilder>> {
        if self.done_called {
            return Err(PyRuntimeError::new_err("FluentPageBuilder.done() already called"));
        }
        self.done_called = true;

        let parent_handle = self.parent.clone_ref(py);
        let mut parent_ref = parent_handle.borrow_mut(py);
        let inner = parent_ref
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("DocumentBuilder already consumed"))?;

        let page_size = self
            .page_size
            .unwrap_or(crate::writer::PageSize::Custom(self.custom_width, self.custom_height));

        let mut page = inner.page(page_size);
        for op in self.ops.drain(..) {
            page = match op {
                PendingPageOp::Font(name, size) => page.font(&name, size),
                PendingPageOp::At(x, y) => page.at(x, y),
                PendingPageOp::Text(text) => page.text(&text),
                PendingPageOp::Heading(level, text) => page.heading(level, &text),
                PendingPageOp::Paragraph(text) => page.paragraph(&text),
                PendingPageOp::Space(points) => page.space(points),
                PendingPageOp::HorizontalRule => page.horizontal_rule(),
                PendingPageOp::LinkUrl(url) => page.link_url(&url),
                PendingPageOp::LinkPage(p) => page.link_page(p),
                PendingPageOp::LinkNamed(dest) => page.link_named(&dest),
                PendingPageOp::LinkJavaScript(script) => page.link_javascript(&script),
                PendingPageOp::OnOpen(script) => page.on_open(&script),
                PendingPageOp::OnClose(script) => page.on_close(&script),
                PendingPageOp::FieldKeystroke(s) => page.field_keystroke(&s),
                PendingPageOp::FieldFormat(s) => page.field_format(&s),
                PendingPageOp::FieldValidate(s) => page.field_validate(&s),
                PendingPageOp::FieldCalculate(s) => page.field_calculate(&s),
                PendingPageOp::Highlight(r, g, b) => page.highlight((r, g, b)),
                PendingPageOp::Underline(r, g, b) => page.underline((r, g, b)),
                PendingPageOp::Strikeout(r, g, b) => page.strikeout((r, g, b)),
                PendingPageOp::Squiggly(r, g, b) => page.squiggly((r, g, b)),
                PendingPageOp::StickyNote(text) => page.sticky_note(&text),
                PendingPageOp::StickyNoteAt(x, y, text) => page.sticky_note_at(x, y, &text),
                PendingPageOp::Watermark(text) => page.watermark(&text),
                PendingPageOp::WatermarkConfidential => page.watermark_confidential(),
                PendingPageOp::WatermarkDraft => page.watermark_draft(),
                PendingPageOp::Stamp(name) => page.stamp(parse_stamp_type(&name)),
                PendingPageOp::FreeText { x, y, w, h, text } => {
                    page.freetext(crate::geometry::Rect::new(x, y, w, h), &text)
                },
                PendingPageOp::TextField {
                    name,
                    x,
                    y,
                    w,
                    h,
                    default_value,
                } => page.text_field(name, x, y, w, h, default_value),
                PendingPageOp::Checkbox {
                    name,
                    x,
                    y,
                    w,
                    h,
                    checked,
                } => page.checkbox(name, x, y, w, h, checked),
                PendingPageOp::ComboBox {
                    name,
                    x,
                    y,
                    w,
                    h,
                    options,
                    selected,
                } => page.combo_box(name, x, y, w, h, options, selected),
                PendingPageOp::RadioGroup {
                    name,
                    buttons,
                    selected,
                } => page.radio_group(name, buttons, selected),
                PendingPageOp::PushButton {
                    name,
                    x,
                    y,
                    w,
                    h,
                    caption,
                } => page.push_button(name, x, y, w, h, caption),
                PendingPageOp::SignatureField { name, x, y, w, h } => {
                    page.signature_field(name, x, y, w, h)
                },
                PendingPageOp::Footnote {
                    ref_mark,
                    note_text,
                } => page.footnote(&ref_mark, &note_text),
                PendingPageOp::Columns {
                    count,
                    gap_pt,
                    text,
                } => page.columns(count, gap_pt, &text),
                PendingPageOp::Inline(text) => page.inline(&text),
                PendingPageOp::InlineBold(text) => page.inline_bold(&text),
                PendingPageOp::InlineItalic(text) => page.inline_italic(&text),
                PendingPageOp::InlineColor { r, g, b, text } => page.inline_color(r, g, b, &text),
                PendingPageOp::Newline => page.newline(),
                PendingPageOp::Rect(x, y, w, h) => page.rect(x, y, w, h),
                PendingPageOp::FilledRect(x, y, w, h, r, g, b) => {
                    page.filled_rect(x, y, w, h, r, g, b)
                },
                PendingPageOp::Line(x1, y1, x2, y2) => page.line(x1, y1, x2, y2),
                PendingPageOp::StrokeRect {
                    x,
                    y,
                    w,
                    h,
                    width,
                    r,
                    g,
                    b,
                } => page.stroke_rect(x, y, w, h, crate::writer::LineStyle::new(width, r, g, b)),
                PendingPageOp::StrokeRectDashed {
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
                } => {
                    let style =
                        crate::writer::LineStyle::new(width, r, g, b).with_dash(&dash, phase);
                    page.stroke_rect(x, y, w, h, style)
                },
                PendingPageOp::StrokeLine {
                    x1,
                    y1,
                    x2,
                    y2,
                    width,
                    r,
                    g,
                    b,
                } => {
                    page.stroke_line(x1, y1, x2, y2, crate::writer::LineStyle::new(width, r, g, b))
                },
                PendingPageOp::StrokeLineDashed {
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
                } => {
                    let style =
                        crate::writer::LineStyle::new(width, r, g, b).with_dash(&dash, phase);
                    page.stroke_line(x1, y1, x2, y2, style)
                },
                PendingPageOp::TextInRect {
                    x,
                    y,
                    w,
                    h,
                    text,
                    align,
                } => page.text_in_rect(
                    crate::geometry::Rect::new(x, y, w, h),
                    &text,
                    parse_align_to_text(align),
                ),
                PendingPageOp::NewPageSameSize => page.new_page_same_size(),
                PendingPageOp::BarcodeImage { bytes, x, y, w, h } => page
                    .image_from_bytes(&bytes, crate::geometry::Rect::new(x, y, w, h))
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
                PendingPageOp::ImageWithAlt {
                    bytes,
                    x,
                    y,
                    w,
                    h,
                    alt_text,
                } => page
                    .image_from_bytes_with_alt(
                        &bytes,
                        crate::geometry::Rect::new(x, y, w, h),
                        &alt_text,
                    )
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
                PendingPageOp::ImageArtifact { bytes, x, y, w, h } => page
                    .image_from_bytes_as_artifact(&bytes, crate::geometry::Rect::new(x, y, w, h))
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
                PendingPageOp::Table {
                    widths,
                    aligns,
                    rows,
                    has_header,
                } => {
                    let cells: Vec<Vec<crate::writer::TableCell>> = rows
                        .into_iter()
                        .map(|row| {
                            row.into_iter()
                                .map(crate::writer::TableCell::text)
                                .collect()
                        })
                        .collect();
                    let mut tbl = crate::writer::Table::new(cells);
                    let col_widths: Vec<crate::writer::ColumnWidth> = widths
                        .iter()
                        .map(|&w| crate::writer::ColumnWidth::Fixed(w))
                        .collect();
                    tbl = tbl.with_column_widths(col_widths);
                    let col_aligns: Vec<crate::writer::CellAlign> =
                        aligns.iter().map(|&a| parse_align_to_cell(a)).collect();
                    tbl.column_aligns = col_aligns;
                    if has_header {
                        tbl = tbl.with_header_row();
                    }
                    page.table(tbl)
                },
                PendingPageOp::StreamingTable {
                    headers,
                    widths,
                    aligns,
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
                    for i in 0..headers.len() {
                        let col = crate::writer::StreamingColumn::new(headers[i].clone())
                            .width_pt(widths[i])
                            .align(parse_align_to_cell(aligns[i]));
                        cfg = cfg.column(col);
                    }
                    let mut st = page.streaming_table(cfg);
                    for row in rows {
                        let _ = st.push_row(|r| {
                            for (text, span) in row {
                                if span > 1 {
                                    r.span_cell(text, span);
                                } else {
                                    r.cell(text);
                                }
                            }
                        });
                    }
                    st.finish()
                },
            };
        }
        page.done();

        drop(parent_ref);
        Ok(parent_handle)
    }
}

/// Streaming-table handle: collects rows cell-by-cell and — at
/// `finish()` — attaches a `PendingPageOp::StreamingTable` to the
/// parent `PyFluentPageBuilder` so the Rust `StreamingTable` core runs
/// at `done()` time. Per the buffered architecture note on
/// `PyFluentPageBuilder`, we can't hold a live Rust `StreamingTable`
/// across GIL boundaries, so we buffer Python-side and replay.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "StreamingTable")]
pub struct PyStreamingTable {
    parent: Py<PyFluentPageBuilder>,
    columns: Vec<PyColumn>,
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
}

#[pymethods]
impl PyStreamingTable {
    /// Push a single row of string cells (all rowspan=1). Length must match
    /// the number of configured columns; otherwise `ValueError` is raised.
    /// Auto-flushes the current batch when `batch_size` is reached.
    fn push_row(&mut self, py: Python<'_>, cells: Vec<String>) -> PyResult<()> {
        if self.finished {
            return Err(PyRuntimeError::new_err("StreamingTable.finish() already called"));
        }
        if cells.len() != self.columns.len() {
            return Err(PyValueError::new_err(format!(
                "row has {} cells, expected {}",
                cells.len(),
                self.columns.len()
            )));
        }
        self.current_batch
            .push(cells.into_iter().map(|s| (s, 1usize)).collect());
        if self.current_batch.len() >= self.batch_size {
            self._flush_batch(py)?;
        }
        Ok(())
    }

    /// Push a row with per-cell rowspan values. Each element is a
    /// `(text, rowspan)` tuple; rowspan == 1 is a normal cell.
    /// Auto-flushes the current batch when `batch_size` is reached.
    fn push_row_span(&mut self, py: Python<'_>, cells: Vec<(String, usize)>) -> PyResult<()> {
        if self.finished {
            return Err(PyRuntimeError::new_err("StreamingTable.finish() already called"));
        }
        if cells.len() != self.columns.len() {
            return Err(PyValueError::new_err(format!(
                "row has {} cells, expected {}",
                cells.len(),
                self.columns.len()
            )));
        }
        self.current_batch.push(cells);
        if self.current_batch.len() >= self.batch_size {
            self._flush_batch(py)?;
        }
        Ok(())
    }

    /// Number of columns configured on this streaming table.
    fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Number of rows in the current (not-yet-flushed) batch.
    fn pending_row_count(&self) -> usize {
        self.current_batch.len()
    }

    /// Number of fully-completed batches waiting for finish().
    fn batch_count(&self) -> usize {
        self.completed_batches.len()
    }

    /// Explicitly flush the current batch to `completed_batches`.
    /// Called automatically when `batch_size` rows accumulate.
    fn flush(&mut self, py: Python<'_>) -> PyResult<()> {
        self._flush_batch(py)
    }

    /// Close the streaming table and return the parent
    /// `FluentPageBuilder` for further chaining.
    fn finish(&mut self, py: Python<'_>) -> PyResult<Py<PyFluentPageBuilder>> {
        if self.finished {
            return Err(PyRuntimeError::new_err("StreamingTable.finish() already called"));
        }
        self._flush_batch(py)?;
        self.finished = true;

        let parent_handle = self.parent.clone_ref(py);
        let mut parent_ref = parent_handle.borrow_mut(py);
        let headers: Vec<String> = self.columns.iter().map(|c| c.header.clone()).collect();
        let widths: Vec<f32> = self.columns.iter().map(|c| c.width).collect();
        let aligns: Vec<i32> = self.columns.iter().map(|c| c.align).collect();
        // Assemble all batches into a flat row list for the existing dispatch.
        let rows: Vec<Vec<(String, usize)>> = self.completed_batches.drain(..).flatten().collect();
        parent_ref.push(PendingPageOp::StreamingTable {
            headers,
            widths,
            aligns,
            repeat_header: self.repeat_header,
            rows,
            mode: self.mode.clone(),
            sample_rows: self.sample_rows,
            min_col_width_pt: self.min_col_width_pt,
            max_col_width_pt: self.max_col_width_pt,
            max_rowspan: self.max_rowspan,
        })?;
        drop(parent_ref);
        Ok(parent_handle)
    }
}

impl PyStreamingTable {
    /// Move the current batch into `completed_batches` (a no-op if the current
    /// batch is empty). Frees the batch buffer.
    fn _flush_batch(&mut self, _py: Python<'_>) -> PyResult<()> {
        if !self.current_batch.is_empty() {
            let batch = std::mem::take(&mut self.current_batch);
            self.completed_batches.push(batch);
        }
        Ok(())
    }
}

// =============================================================================
// HTML+CSS pipeline — thin wrappers on PyPdf
// =============================================================================
//
// `Pdf.from_html_css[_with_fonts]` exposes the HTML+CSS → PDF pipeline
// to Python. The Rust side is `crate::api::Pdf::from_html_css`
// `from_html_css_with_fonts`.

#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "PageTemplate",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct PyPageTemplate {
    pub inner: crate::writer::PageTemplate,
}
#[pymethods]
impl PyPageTemplate {
    #[new]
    fn new() -> Self {
        Self {
            inner: crate::writer::PageTemplate::new(),
        }
    }
    fn header<'a>(
        mut slf: PyRefMut<'a, Self>,
        h: &Bound<'_, PyAny>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        let a = if let Ok(ph) = h.extract::<PyHeader>() {
            ph.inner.inner.clone()
        } else {
            h.extract::<PyArtifact>()?.inner.clone()
        };
        slf.inner = slf.inner.clone().header(a);
        Ok(slf)
    }
    fn footer<'a>(
        mut slf: PyRefMut<'a, Self>,
        f: &Bound<'_, PyAny>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        let a = if let Ok(pf) = f.extract::<PyFooter>() {
            pf.inner.inner.clone()
        } else {
            f.extract::<PyArtifact>()?.inner.clone()
        };
        slf.inner = slf.inner.clone().footer(a);
        Ok(slf)
    }
}

// pyo3_log caches Python logger levels per target for performance. When
// users change Python logger configuration (or call `set_log_level`) after
// the cache has been populated, the cache must be reset for the change to
// take effect on already-seen targets. We hold the ResetHandle returned by
// `pyo3_log::try_init()` so both `setup_logging` and `set_log_level` can
// clear the cache.
static PYO3_LOG_RESET_HANDLE: std::sync::OnceLock<pyo3_log::ResetHandle> =
    std::sync::OnceLock::new();

fn init_pyo3_log_handle() {
    PYO3_LOG_RESET_HANDLE.get_or_init(|| {
        pyo3_log::try_init().unwrap_or_else(|_| {
            // Another logger was already installed (e.g. by an embedding
            // host). In that case, do not replace the global logger;
            // instead, create a standalone default `Logger` value and take
            // its `ResetHandle`. `reset_handle()` is available on any
            // `Logger` instance and does not itself perform installation.
            pyo3_log::Logger::default().reset_handle()
        })
    });
}

fn reset_pyo3_log_cache() {
    if let Some(handle) = PYO3_LOG_RESET_HANDLE.get() {
        handle.reset();
    }
}

/// Bridge Rust `log` macros into Python's `logging` module.
///
/// After this is called, all log messages emitted by pdf_oxide flow through
/// Python's standard `logging` module, which is silent by default. Users
/// control verbosity with the normal Python API, e.g.:
///
/// ```python
/// import logging
/// logging.basicConfig(level=logging.WARNING)
/// Python wrapper: set the global content-stream
/// operator cap. `None` restores the default (1,000,000). Returns the
/// previous override value or None if default was active.
///
/// Use case: large technical PDFs (textbooks, ISO standards) with
/// legitimate content streams exceeding 1,000,000 operators. Set to
/// `None` (the default) for adversarial-input protection; set to a
/// large value when the inputs are trusted.
#[pyfunction]
fn set_max_ops_per_stream(limit: Option<usize>) -> Option<usize> {
    crate::content::parser::set_max_ops_per_stream(limit)
}

/// Python wrapper: toggle the global U+FFFD
/// preservation flag. When `True`, `extract_text` / `extract_words` /
/// `extract_spans` emit U+FFFD chars for unmapped glyphs (matching
/// `extract_chars` behaviour). When `False` (the default),
/// the high-level accessors filter them. Returns the previous value.
#[pyfunction]
fn set_preserve_unmapped_glyphs(preserve: bool) -> bool {
    crate::extractors::text::set_preserve_unmapped_glyphs(preserve)
}

/// Generate a 1D barcode as an SVG string.
///
/// `barcode_type`: 0=Code128, 1=Code39, 2=EAN13, 3=EAN8, 4=UPCA, 5=ITF, 6=Code93, 7=Codabar.
#[cfg(feature = "barcodes")]
#[pyfunction]
fn generate_barcode_svg(barcode_type: i32, data: String) -> PyResult<String> {
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
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "unknown barcode_type {barcode_type}; valid values are 0–7"
            )))
        },
    };
    BarcodeGenerator::generate_1d_svg(bt, &data, &BarcodeOptions::default())
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Generate a QR code as an SVG string.
///
/// `error_correction`: 0=Low, 1=Medium (default), 2=Quartile, 3=High.
/// `size`: target pixel size (advisory for module sizing).
#[cfg(feature = "barcodes")]
#[pyfunction]
fn generate_qr_svg(data: String, error_correction: i32, size: u32) -> PyResult<String> {
    use crate::writer::{BarcodeGenerator, QrCodeOptions, QrErrorCorrection};
    let ec = match error_correction {
        0 => QrErrorCorrection::Low,
        2 => QrErrorCorrection::Quartile,
        3 => QrErrorCorrection::High,
        _ => QrErrorCorrection::Medium,
    };
    let opts = QrCodeOptions::new().size(size).error_correction(ec);
    BarcodeGenerator::generate_qr_svg(&data, &opts)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Called automatically when the `pdf_oxide` module is imported — users do
/// not need to call it directly. Kept as a public function for backward
/// compatibility.
#[pyfunction]
fn setup_logging() {
    init_pyo3_log_handle();
    // Reset the level cache in case Python-side logger config changed since
    // the last time any target was checked.
    reset_pyo3_log_cache();
}

/// Set the maximum log level for pdf_oxide messages that cross into Python.
///
/// Accepts one of: `"off"`, `"error"`, `"warn"` / `"warning"`, `"info"`,
/// `"debug"`, `"trace"`. Case-insensitive.
///
/// This sets Rust's `log::max_level` filter gate *and* clears pyo3_log's
/// per-target level cache so the change takes effect on targets that have
/// already been logged to. Without the cache reset, loggers like
/// `pdf_oxide.xref` — which pyo3_log probes and caches on first use — would
/// keep their stale level and ignore subsequent calls to this function.
#[pyfunction]
fn set_log_level(level: &str) -> PyResult<()> {
    use log::LevelFilter;
    let filter = match level.to_ascii_lowercase().as_str() {
        "off" | "none" | "disabled" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" | "warning" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        other => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "invalid log level '{}': expected off, error, warn, info, debug, or trace",
                other
            )));
        },
    };
    log::set_max_level(filter);
    reset_pyo3_log_cache();
    Ok(())
}

/// Disable all pdf_oxide log output — convenience wrapper for
/// `set_log_level("off")`.
#[pyfunction]
fn disable_logging() {
    log::set_max_level(log::LevelFilter::Off);
    reset_pyo3_log_cache();
}

/// Return the current Rust-side log level filter as a lowercase string.
///
/// One of: `"off"`, `"error"`, `"warn"`, `"info"`, `"debug"`, `"trace"`.
///
/// This mirrors the gate controlled by `set_log_level` and is useful for
/// tests or context managers that want to save/restore the log level
/// around a block without hard-coding a "default".
#[pyfunction]
fn get_log_level() -> &'static str {
    match log::max_level() {
        log::LevelFilter::Off => "off",
        log::LevelFilter::Error => "error",
        log::LevelFilter::Warn => "warn",
        log::LevelFilter::Info => "info",
        log::LevelFilter::Debug => "debug",
        log::LevelFilter::Trace => "trace",
    }
}

/// Computed adaptive layout parameters for a PDF page.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "LayoutParams", frozen)]
pub struct PyLayoutParams {
    pub word_gap_threshold: f32,
    pub line_gap_threshold: f32,
    pub median_char_width: f32,
    pub median_font_size: f32,
    pub median_line_spacing: f32,
    pub column_count: usize,
}

#[pymethods]
impl PyLayoutParams {
    #[getter]
    fn word_gap_threshold(&self) -> f32 {
        self.word_gap_threshold
    }
    #[getter]
    fn line_gap_threshold(&self) -> f32 {
        self.line_gap_threshold
    }
    #[getter]
    fn median_char_width(&self) -> f32 {
        self.median_char_width
    }
    #[getter]
    fn median_font_size(&self) -> f32 {
        self.median_font_size
    }
    #[getter]
    fn median_line_spacing(&self) -> f32 {
        self.median_line_spacing
    }
    #[getter]
    fn column_count(&self) -> usize {
        self.column_count
    }

    fn __repr__(&self) -> String {
        format!(
            "LayoutParams(word_gap={:.2}, line_gap={:.2}, char_width={:.2}, font_size={:.2}, line_spacing={:.2}, columns={})",
            self.word_gap_threshold,
            self.line_gap_threshold,
            self.median_char_width,
            self.median_font_size,
            self.median_line_spacing,
            self.column_count,
        )
    }
}

/// Pre-tuned extraction profile for different document types.
#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "ExtractionProfile",
    frozen,
    from_py_object
)]
#[derive(Clone)]
pub struct PyExtractionProfile {
    inner: crate::config::ExtractionProfile,
}

#[pymethods]
impl PyExtractionProfile {
    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name
    }
    #[getter]
    fn tj_offset_threshold(&self) -> f32 {
        self.inner.tj_offset_threshold
    }
    #[getter]
    fn word_margin_ratio(&self) -> f32 {
        self.inner.word_margin_ratio
    }
    #[getter]
    fn space_threshold_em_ratio(&self) -> f32 {
        self.inner.space_threshold_em_ratio
    }
    #[getter]
    fn space_char_multiplier(&self) -> f32 {
        self.inner.space_char_multiplier
    }
    #[getter]
    fn use_adaptive_threshold(&self) -> bool {
        self.inner.use_adaptive_threshold
    }

    #[staticmethod]
    fn conservative() -> Self {
        Self {
            inner: crate::config::ExtractionProfile::CONSERVATIVE,
        }
    }
    #[staticmethod]
    fn aggressive() -> Self {
        Self {
            inner: crate::config::ExtractionProfile::AGGRESSIVE,
        }
    }
    #[staticmethod]
    fn balanced() -> Self {
        Self {
            inner: crate::config::ExtractionProfile::BALANCED,
        }
    }
    #[staticmethod]
    fn academic() -> Self {
        Self {
            inner: crate::config::ExtractionProfile::ACADEMIC,
        }
    }
    #[staticmethod]
    fn policy() -> Self {
        Self {
            inner: crate::config::ExtractionProfile::POLICY,
        }
    }
    #[staticmethod]
    fn form() -> Self {
        Self {
            inner: crate::config::ExtractionProfile::FORM,
        }
    }
    #[staticmethod]
    fn government() -> Self {
        Self {
            inner: crate::config::ExtractionProfile::GOVERNMENT,
        }
    }
    #[staticmethod]
    fn scanned_ocr() -> Self {
        Self {
            inner: crate::config::ExtractionProfile::SCANNED_OCR,
        }
    }
    #[staticmethod]
    fn adaptive() -> Self {
        Self {
            inner: crate::config::ExtractionProfile::ADAPTIVE,
        }
    }

    #[staticmethod]
    fn available() -> Vec<&'static str> {
        crate::config::ExtractionProfile::all_profiles().to_vec()
    }

    fn __repr__(&self) -> String {
        format!(
            "ExtractionProfile('{}', word_margin_ratio={}, tj_offset_threshold={})",
            self.inner.name, self.inner.word_margin_ratio, self.inner.tj_offset_threshold,
        )
    }
}

/// RFC 3161 Time Stamp Authority client.
/// Only available when pdf_oxide was built with the `tsa-client`
/// Rust-core feature — otherwise every call raises
/// `NotImplementedError`.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "TsaClient")]
pub struct PyTsaClient {
    #[cfg(feature = "tsa-client")]
    inner: crate::signatures::TsaClient,
}

#[pymethods]
impl PyTsaClient {
    /// Build a new client.
    #[new]
    #[pyo3(signature = (
        url,
        username=None,
        password=None,
        timeout_seconds=30,
        hash_algorithm=2,
        use_nonce=true,
        cert_req=true,
    ))]
    fn new(
        url: String,
        username: Option<String>,
        password: Option<String>,
        timeout_seconds: i32,
        hash_algorithm: i32,
        use_nonce: bool,
        cert_req: bool,
    ) -> PyResult<Self> {
        #[cfg(feature = "tsa-client")]
        {
            let algo = match hash_algorithm {
                1 => crate::signatures::HashAlgorithm::Sha1,
                2 => crate::signatures::HashAlgorithm::Sha256,
                3 => crate::signatures::HashAlgorithm::Sha384,
                4 => crate::signatures::HashAlgorithm::Sha512,
                _ => crate::signatures::HashAlgorithm::Sha256,
            };
            let cfg = crate::signatures::TsaClientConfig {
                url,
                username,
                password,
                timeout: if timeout_seconds > 0 {
                    std::time::Duration::from_secs(timeout_seconds as u64)
                } else {
                    std::time::Duration::from_secs(30)
                },
                hash_algorithm: algo,
                use_nonce,
                cert_req,
            };
            Ok(Self {
                inner: crate::signatures::TsaClient::new(cfg),
            })
        }
        #[cfg(not(feature = "tsa-client"))]
        {
            let _ = (url, username, password, timeout_seconds, hash_algorithm, use_nonce, cert_req);
            Err(PyNotImplementedError::new_err(
                "pdf_oxide was built without the `tsa-client` feature",
            ))
        }
    }

    /// Hash `data` and request a timestamp. Network call.
    fn request_timestamp(&self, data: &Bound<'_, PyBytes>) -> PyResult<PyTimestamp> {
        #[cfg(feature = "tsa-client")]
        {
            let ts = self
                .inner
                .request_timestamp(data.as_bytes())
                .map_err(|e| PyRuntimeError::new_err(format!("TSA error: {e}")))?;
            Ok(PyTimestamp { inner: ts })
        }
        #[cfg(not(feature = "tsa-client"))]
        {
            let _ = data;
            Err(PyNotImplementedError::new_err(
                "pdf_oxide was built without the `tsa-client` feature",
            ))
        }
    }

    /// Request a timestamp for a pre-computed digest.
    fn request_timestamp_hash(
        &self,
        hash: &Bound<'_, PyBytes>,
        hash_algorithm: i32,
    ) -> PyResult<PyTimestamp> {
        #[cfg(feature = "tsa-client")]
        {
            let algo = match hash_algorithm {
                1 => crate::signatures::HashAlgorithm::Sha1,
                2 => crate::signatures::HashAlgorithm::Sha256,
                3 => crate::signatures::HashAlgorithm::Sha384,
                4 => crate::signatures::HashAlgorithm::Sha512,
                _ => crate::signatures::HashAlgorithm::Sha256,
            };
            let ts = self
                .inner
                .request_timestamp_hash(hash.as_bytes(), algo)
                .map_err(|e| PyRuntimeError::new_err(format!("TSA error: {e}")))?;
            Ok(PyTimestamp { inner: ts })
        }
        #[cfg(not(feature = "tsa-client"))]
        {
            let _ = (hash, hash_algorithm);
            Err(PyNotImplementedError::new_err(
                "pdf_oxide was built without the `tsa-client` feature",
            ))
        }
    }
}

/// X.509 certificate parsed from a raw DER blob. Mirrors the C# /
/// Node `Certificate` class — `subject` / `issuer` / `serial` /
/// `validity` / `is_valid` getters only; signing uses the PKCS#12
/// loader once the Rust core's chain-of-trust work lands.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Certificate")]
pub struct PyCertificate {
    creds: crate::signatures::SigningCredentials,
}

#[pymethods]
impl PyCertificate {
    /// Load a certificate from a DER-encoded X.509 blob. Raises
    /// ValueError if the DER doesn't parse.
    #[staticmethod]
    fn load(data: &Bound<'_, PyBytes>) -> PyResult<Self> {
        #[cfg(feature = "signatures")]
        {
            let bytes = data.as_bytes();
            if bytes.is_empty() {
                return Err(PyValueError::new_err("Certificate data must not be empty"));
            }
            let creds = crate::signatures::SigningCredentials::from_der(bytes.to_vec())
                .map_err(|e| PyValueError::new_err(format!("Invalid certificate: {e}")))?;
            Ok(Self { creds })
        }
        #[cfg(not(feature = "signatures"))]
        {
            let _ = data;
            Err(PyNotImplementedError::new_err(
                "Certificate.load(): pdf_oxide was built without --features signatures",
            ))
        }
    }

    /// Load a signer certificate + private key from separate PEM strings.
    /// `cert_pem` must begin with `-----BEGIN CERTIFICATE-----`.
    /// `key_pem` must begin with `-----BEGIN PRIVATE KEY-----` (PKCS#8) or
    /// `-----BEGIN RSA PRIVATE KEY-----` (PKCS#1).
    #[staticmethod]
    fn load_pem(cert_pem: &str, key_pem: &str) -> PyResult<Self> {
        #[cfg(feature = "signatures")]
        {
            let creds = crate::signatures::SigningCredentials::from_pem(cert_pem, key_pem)
                .map_err(|e| {
                    PyValueError::new_err(format!("Failed to load PEM credentials: {e}"))
                })?;
            Ok(Self { creds })
        }
        #[cfg(not(feature = "signatures"))]
        {
            let _ = (cert_pem, key_pem);
            Err(PyNotImplementedError::new_err(
                "Certificate.load_pem(): pdf_oxide was built without --features signatures",
            ))
        }
    }

    /// Load a signer certificate + private key from a PKCS#12 (.p12/.pfx) blob.
    /// `password` is the passphrase protecting the key bag.
    #[staticmethod]
    fn load_pkcs12(data: &Bound<'_, PyBytes>, password: &str) -> PyResult<Self> {
        #[cfg(feature = "signatures")]
        {
            let bytes = data.as_bytes();
            if bytes.is_empty() {
                return Err(PyValueError::new_err("PKCS#12 data must not be empty"));
            }
            let creds = crate::signatures::SigningCredentials::from_pkcs12(bytes, password)
                .map_err(|e| PyValueError::new_err(format!("Failed to load PKCS#12: {e}")))?;
            Ok(Self { creds })
        }
        #[cfg(not(feature = "signatures"))]
        {
            let _ = (data, password);
            Err(PyNotImplementedError::new_err(
                "Certificate.load_pkcs12(): pdf_oxide was built without --features signatures",
            ))
        }
    }

    /// Subject distinguished name (e.g. `CN=pdfoxide-test, O=pdf_oxide, C=US`).
    #[getter]
    fn subject(&self) -> PyResult<String> {
        self.creds
            .subject()
            .map_err(|e| PyValueError::new_err(format!("{e}")))
    }

    /// Issuer distinguished name — the DN of the CA that signed this
    /// certificate (self-signed certs have `issuer == subject`).
    #[getter]
    fn issuer(&self) -> PyResult<String> {
        self.creds
            .issuer()
            .map_err(|e| PyValueError::new_err(format!("{e}")))
    }

    /// Serial number as a hex string (no `0x` prefix).
    #[getter]
    fn serial(&self) -> PyResult<String> {
        self.creds
            .serial()
            .map_err(|e| PyValueError::new_err(format!("{e}")))
    }

    /// Validity window as a `(not_before, not_after)` tuple of Unix
    /// epoch seconds. Use `datetime.fromtimestamp(t, tz=timezone.utc)`
    /// to get a Python datetime.
    #[getter]
    fn validity(&self) -> PyResult<(i64, i64)> {
        self.creds
            .validity()
            .map_err(|e| PyValueError::new_err(format!("{e}")))
    }

    /// Whether the certificate is currently within its validity
    /// window. Does NOT verify the signature chain, trust-root, or
    /// revocation — this is a time-window check only.
    #[getter]
    fn is_valid(&self) -> PyResult<bool> {
        self.creds
            .is_valid()
            .map_err(|e| PyValueError::new_err(format!("{e}")))
    }

    fn __repr__(&self) -> String {
        let subject = self
            .creds
            .subject()
            .unwrap_or_else(|_| "<unreadable>".into());
        let serial = self
            .creds
            .serial()
            .unwrap_or_else(|_| "<unreadable>".into());
        format!("Certificate(subject={subject:?}, serial={serial:?})")
    }
}

/// RFC 3161 timestamp parsed from a DER TimeStampToken or bare
/// TSTInfo. Mirrors the C# `Timestamp` class.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Timestamp")]
pub struct PyTimestamp {
    inner: crate::signatures::Timestamp,
}

#[pymethods]
impl PyTimestamp {
    /// Parse a DER blob that may be either a full TimeStampToken
    /// (CMS-wrapped) or the bare TSTInfo SEQUENCE.
    #[staticmethod]
    fn parse(data: &Bound<'_, PyBytes>) -> PyResult<Self> {
        let bytes = data.as_bytes();
        if bytes.is_empty() {
            return Err(PyValueError::new_err("Timestamp data must not be empty"));
        }
        let inner = crate::signatures::Timestamp::from_der(bytes)
            .map_err(|e| PyValueError::new_err(format!("Invalid timestamp: {e}")))?;
        Ok(Self { inner })
    }

    /// Generation time as Unix epoch seconds.
    #[getter]
    fn time(&self) -> i64 {
        self.inner.time()
    }

    /// Serial number as a hex string (no `0x` prefix).
    #[getter]
    fn serial(&self) -> String {
        self.inner.serial()
    }

    /// TSA policy OID in dotted-decimal form.
    #[getter]
    fn policy_oid(&self) -> String {
        self.inner.policy_oid()
    }

    /// TSA name from the token (may be empty if the TSA didn't
    /// include its name).
    #[getter]
    fn tsa_name(&self) -> String {
        self.inner.tsa_name()
    }

    /// Hash algorithm enum value (1=SHA1, 2=SHA256, 3=SHA384,
    /// 4=SHA512, 0=unknown) — same contract as the FFI.
    #[getter]
    fn hash_algorithm(&self) -> i32 {
        self.inner.hash_algorithm() as i32
    }

    /// Raw message-imprint hash bytes.
    #[getter]
    fn message_imprint(&self, py: Python) -> Py<PyBytes> {
        PyBytes::new(py, self.inner.message_imprint_ref()).into()
    }

    /// Cryptographically verify this TimeStampToken.
    ///
    /// Parses the outer CMS SignedData and verifies the TSA's signature
    /// `messageDigest` attribute (RSA-PKCS#1 v1.5, RSA-PSS, ECDSA P-256/P-384).
    ///
    /// Returns `True` when the token is cryptographically valid, `False` when
    /// the check fails. Raises `RuntimeError` if the token is not CMS-wrapped
    /// or uses an unsupported algorithm.
    fn verify(&self) -> PyResult<bool> {
        self.inner
            .verify()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "Timestamp(time={}, serial={:?}, policy_oid={:?})",
            self.inner.time(),
            self.inner.serial(),
            self.inner.policy_oid(),
        )
    }
}

/// A single existing PDF signature surfaced by
/// `PdfDocument.signatures()`. `verify()` runs the signer-attributes
/// check; `verify_detached()` adds the `messageDigest` content-hash
/// check. Supported algorithms: RSA-PKCS#1 v1.5, RSA-PSS, ECDSA P-256/P-384.
/// Unsupported-algorithm signers return `Unknown`.
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Signature")]
pub struct PySignature {
    info: crate::signatures::SignatureInfo,
}

#[pymethods]
impl PySignature {
    /// `/Name` from the signature dictionary, or `None`.
    #[getter]
    fn signer_name(&self) -> Option<String> {
        self.info.signer_name.clone()
    }

    /// `/Reason` from the signature dictionary, or `None`.
    #[getter]
    fn reason(&self) -> Option<String> {
        self.info.reason.clone()
    }

    /// `/Location` from the signature dictionary, or `None`.
    #[getter]
    fn location(&self) -> Option<String> {
        self.info.location.clone()
    }

    /// `/ContactInfo` from the signature dictionary, or `None`.
    #[getter]
    fn contact_info(&self) -> Option<String> {
        self.info.contact_info.clone()
    }

    /// Signing time as Unix epoch seconds (parsed from the PDF date
    /// string in `/M`), or `None` if the entry is missing or
    /// unparseable.
    #[getter]
    fn signing_time(&self) -> Option<i64> {
        self.info
            .signing_time
            .as_deref()
            .and_then(crate::signatures::parse_pdf_date_to_epoch)
    }

    /// True iff `/ByteRange` covers the whole document (4-element array).
    #[getter]
    fn covers_whole_document(&self) -> bool {
        self.info.covers_whole_document
    }

    /// PAdES baseline level from this signature's CMS attributes alone
    /// (`B_B` vs `B_T`). `B_LT` additionally needs the document `/DSS`
    /// — combine with :meth:`PdfDocument.dss` and re-classify there.
    #[getter]
    fn pades_level(&self) -> PyPadesLevel {
        #[cfg(feature = "signatures")]
        {
            PyPadesLevel::from_core(crate::signatures::classify_pades_level(&self.info, None))
        }
        #[cfg(not(feature = "signatures"))]
        {
            PyPadesLevel::BB
        }
    }

    /// Run the RFC 5652 §5.4 signer-attributes crypto check against the
    /// certificate embedded in this signature's CMS blob. Today this
    /// covers RSA-PKCS#1 v1.5 over SHA-1/256/384/512 — the padding
    /// used by essentially every PDF signature in the wild.
    ///
    /// A True return proves the signer held the private key matching
    /// the embedded certificate and that the signed-attribute bundle
    /// is authentic. It does **not** verify the messageDigest
    /// attribute against the document's byte-range content hash —
    /// call `verify_detached()` for that end-to-end check.
    ///
    /// Raises NotImplementedError for RSA-PSS, ECDSA, unknown digest
    /// OIDs, or signatures missing signed_attrs.
    fn verify(&self) -> PyResult<bool> {
        let Some(contents) = self.info.contents() else {
            return Err(PyNotImplementedError::new_err(
                "Signature has no /Contents blob — nothing to verify",
            ));
        };
        match crate::signatures::verify_signer(contents) {
            Ok(crate::signatures::SignerVerify::Valid) => Ok(true),
            Ok(crate::signatures::SignerVerify::Invalid) => Ok(false),
            Ok(crate::signatures::SignerVerify::Unknown) => Err(PyNotImplementedError::new_err(
                "Signature.verify(): signer uses RSA-PSS, ECDSA, an unknown \
                     digest OID, or the CMS blob lacks signed_attrs",
            )),
            Err(e) => Err(PyValueError::new_err(format!(
                "Signature.verify(): failed to parse /Contents as CMS: {e}"
            ))),
        }
    }

    /// End-to-end detached-signature verification. Runs the
    /// signer-attributes RSA-PKCS#1 v1.5 crypto check AND the RFC 5652
    /// §11.2 messageDigest attribute check against the portion of
    /// `pdf_data` this signature covers (extracted via the
    /// signature's /ByteRange).
    ///
    /// `pdf_data` must be the full PDF file. A True result proves
    /// both that the signer is authentic and that the document bytes
    /// under the signature's ByteRange have not been altered since
    /// signing. A False result means either the signer check failed
    /// or the content was modified after signing.
    ///
    /// Raises NotImplementedError when the digest OID is unrecognised
    /// or signed attributes / messageDigest are absent.
    fn verify_detached(&self, pdf_data: &[u8]) -> PyResult<bool> {
        let Some(contents) = self.info.contents() else {
            return Err(PyNotImplementedError::new_err(
                "Signature has no /Contents blob — nothing to verify",
            ));
        };
        let br = self.info.byte_range();
        if br.len() != 4 {
            return Err(PyValueError::new_err(
                "Signature has no /ByteRange — cannot extract signed bytes",
            ));
        }
        let byte_range: [i64; 4] = [br[0], br[1], br[2], br[3]];
        let signed_bytes =
            crate::signatures::ByteRangeCalculator::extract_signed_bytes(pdf_data, &byte_range)
                .map_err(|e| {
                    PyValueError::new_err(format!("Failed to extract signed bytes: {e}"))
                })?;
        match crate::signatures::verify_signer_detached(contents, &signed_bytes) {
            Ok(crate::signatures::SignerVerify::Valid) => Ok(true),
            Ok(crate::signatures::SignerVerify::Invalid) => Ok(false),
            Ok(crate::signatures::SignerVerify::Unknown) => Err(PyNotImplementedError::new_err(
                "Signature.verify_detached(): signer uses RSA-PSS, ECDSA, an \
                 unknown digest, or the CMS blob lacks signed_attrs / messageDigest",
            )),
            Err(e) => Err(PyValueError::new_err(format!("Signature.verify_detached(): {e}"))),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Signature(signer_name={:?}, reason={:?}, location={:?})",
            self.info.signer_name, self.info.reason, self.info.location,
        )
    }
}

#[pymodule(gil_used = false)]
fn pdf_oxide(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Bridge Rust `log` to Python `logging` (silent by default, user
    // configures with `logging.basicConfig(level=...)`). Fixes issue #280.
    // We hold the ResetHandle so `set_log_level` can flush pyo3_log's
    // per-target cache — fixes issue #283 regression where per-logger
    // cached levels survived set_log_level calls.
    init_pyo3_log_handle();
    m.add_function(wrap_pyfunction!(setup_logging, m)?)?;
    m.add_function(wrap_pyfunction!(set_log_level, m)?)?;
    m.add_function(wrap_pyfunction!(set_max_ops_per_stream, m)?)?;
    m.add_function(wrap_pyfunction!(set_preserve_unmapped_glyphs, m)?)?;
    m.add_function(wrap_pyfunction!(get_log_level, m)?)?;
    m.add_function(wrap_pyfunction!(disable_logging, m)?)?;
    m.add_class::<PyPdfDocument>()?;
    m.add_class::<PyPageCount>()?;
    m.add_class::<PyPdf>()?;
    m.add_class::<PyPdfPage>()?;
    m.add_class::<PyPdfText>()?;
    m.add_class::<PyPdfTextId>()?;
    m.add_class::<PyPdfImage>()?;
    m.add_class::<PyPdfElement>()?;
    m.add_class::<PyAnnotationWrapper>()?;
    m.add_class::<PyTextChar>()?;
    m.add_class::<PyTextSpan>()?;
    m.add_class::<PyWord>()?;
    m.add_class::<PyTextLine>()?;
    m.add_class::<PyPdfPageRegion>()?;
    m.add_class::<PyDocPage>()?;
    m.add_class::<PyDocPageIter>()?;
    m.add_class::<PyLayoutParams>()?;
    m.add_class::<PyExtractionProfile>()?;
    m.add_class::<PyFormField>()?;
    m.add_class::<PyOcrEngine>()?;
    m.add_class::<PyOcrConfig>()?;
    m.add_class::<PyColor>()?;
    m.add_class::<PyBlendMode>()?;
    m.add_class::<PyExtGState>()?;
    // Write-side API (DocumentBuilder + embedded fonts)
    m.add_class::<PyDocumentBuilder>()?;
    m.add_class::<PyFluentPageBuilder>()?;
    m.add_class::<PyEmbeddedFont>()?;
    // v0.3.39 table + primitive surface (#393 step 6a)
    m.add_class::<PyAlign>()?;
    m.add_class::<PyColumn>()?;
    m.add_class::<PyTable>()?;
    m.add_class::<PyStreamingTable>()?;
    m.add_class::<PyPageTemplate>()?;
    m.add_class::<PyArtifact>()?;
    m.add_class::<PyHeader>()?;
    m.add_class::<PyFooter>()?;
    m.add_class::<PyArtifactStyle>()?;
    m.add_class::<PyLinearGradient>()?;
    m.add_class::<PyRadialGradient>()?;
    m.add_class::<PyLineCap>()?;
    m.add_class::<PyLineJoin>()?;
    m.add_class::<PyPatternPresets>()?;
    m.add_class::<PyOfficeConverter>()?;
    m.add_class::<PySignature>()?;
    m.add_class::<PyCertificate>()?;
    m.add_class::<PyTimestamp>()?;
    m.add_class::<PyTsaClient>()?;
    m.add_class::<PyPadesLevel>()?;
    m.add_class::<PyRevocationMaterial>()?;
    m.add_class::<PyDss>()?;
    m.add_function(pyo3::wrap_pyfunction!(py_sign_pdf_bytes, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_sign_pdf_bytes_pades, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_has_document_timestamp, m)?)?;
    #[cfg(feature = "barcodes")]
    m.add_function(pyo3::wrap_pyfunction!(generate_barcode_svg, m)?)?;
    #[cfg(feature = "barcodes")]
    m.add_function(pyo3::wrap_pyfunction!(generate_qr_svg, m)?)?;
    m.add("VERSION", env!("CARGO_PKG_VERSION"))?;
    // Cryptographic-provider surface — exposes the
    // FIPS-validated AwsLcProvider as a runtime opt-in. Functions
    // (not classes) so callers don't need to instantiate Rust types.
    m.add_function(pyo3::wrap_pyfunction!(crypto_active_provider, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(crypto_available_providers, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(crypto_use_fips, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(crypto_set_policy, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(crypto_policy, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(crypto_inventory, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(crypto_cbom, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(plan_split_by_bookmarks, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(split_by_bookmarks, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(prefetch_models, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(model_manifest, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(prefetch_available, m)?)?;
    Ok(())
}

fn split_opts(
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

fn seg_to_pydict<'py>(
    py: pyo3::Python<'py>,
    seg: &crate::split_bookmarks::BookmarkSegment,
) -> pyo3::PyResult<pyo3::Bound<'py, pyo3::types::PyDict>> {
    let d = pyo3::types::PyDict::new(py);
    d.set_item("index", seg.index)?;
    d.set_item("start_page", seg.start_page)?;
    d.set_item("end_page", seg.end_page)?;
    d.set_item("title", seg.title.clone())?;
    d.set_item("file_stem", &seg.file_stem)?;
    d.set_item("page_label", seg.page_label.clone())?;
    Ok(d)
}

/// Plan a bookmark split for `src_bytes` without producing PDFs.
///
/// Returns ``list[dict]`` (keys: index, start_page, end_page, title,
/// file_stem, page_label). ``level``: 0 = all depths, 1 = top-level
/// (default), n = up to depth n. Raises ``RuntimeError`` if the
/// document has no outline or nothing resolves.
#[pyfunction]
#[pyo3(signature = (src_bytes, title_prefix=None, ignore_case=false, level=1, include_front_matter=true))]
fn plan_split_by_bookmarks(
    py: pyo3::Python<'_>,
    src_bytes: &[u8],
    title_prefix: Option<String>,
    ignore_case: bool,
    level: u32,
    include_front_matter: bool,
) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyList>> {
    let doc = crate::document::PdfDocument::from_bytes(src_bytes.to_vec())
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let opts = split_opts(title_prefix, ignore_case, level, include_front_matter);
    let segs = crate::split_bookmarks::plan_split_by_bookmarks(&doc, &opts)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let list = pyo3::types::PyList::empty(py);
    for s in &segs {
        list.append(seg_to_pydict(py, s)?)?;
    }
    Ok(list.unbind())
}

/// Split `src_bytes` at bookmark boundaries.
///
/// Returns ``list[tuple[dict, bytes]]`` — each segment's metadata
/// (see :func:`plan_split_by_bookmarks`) paired with its PDF bytes.
/// The source is not modified. Raises ``RuntimeError`` on failure.
#[pyfunction]
#[pyo3(signature = (src_bytes, title_prefix=None, ignore_case=false, level=1, include_front_matter=true))]
fn split_by_bookmarks(
    py: pyo3::Python<'_>,
    src_bytes: &[u8],
    title_prefix: Option<String>,
    ignore_case: bool,
    level: u32,
    include_front_matter: bool,
) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyList>> {
    let opts = split_opts(title_prefix, ignore_case, level, include_front_matter);
    let parts = crate::split_bookmarks::split_by_bookmarks_to_bytes(src_bytes, &opts)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let list = pyo3::types::PyList::empty(py);
    for (seg, blob) in &parts {
        let tup = pyo3::types::PyTuple::new(
            py,
            [
                seg_to_pydict(py, seg)?.into_any(),
                pyo3::types::PyBytes::new(py, blob).into_any(),
            ],
        )?;
        list.append(tup)?;
    }
    Ok(list.unbind())
}

/// #519: Build-time OCR model provisioning. Downloads the shared
/// detector + the recognition model/dict for each requested language
/// into the model cache dir (``$PDF_OXIDE_MODEL_DIR`` / the platform
/// cache) and returns that dir as a string. ``languages`` is a list of
/// language codes (e.g. ``["english", "chinese", "arabic"]``); empty →
/// English. Unknown codes are skipped. Idempotent. Actual download
/// requires the wheel built with the ``ocr`` feature; without it the
/// cache dir is still created (no fetch) — query
/// :func:`prefetch_available`.
#[pyfunction]
fn prefetch_models(languages: Vec<String>) -> PyResult<String> {
    use crate::extractors::auto::{AutoExtractor, OcrLanguage};
    let langs: Vec<OcrLanguage> = languages
        .iter()
        .filter_map(|s| OcrLanguage::from_code(s.trim()))
        .collect();
    let want = if langs.is_empty() {
        vec![OcrLanguage::English]
    } else {
        langs
    };
    let dir = AutoExtractor::prefetch_models(&want)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(dir.to_string_lossy().into_owned())
}

/// #519: Air-gapped OCR model manifest — JSON (detector + every
/// supported language's cache filenames and source URLs). Never errors.
#[pyfunction]
fn model_manifest() -> String {
    crate::extractors::auto::AutoExtractor::model_manifest()
}

/// #519: Whether this build can actually download models (compiled
/// with the ``ocr`` feature). ``True`` = downloads work; ``False`` =
/// :func:`prefetch_models` only creates the cache dir (no fetch).
#[pyfunction]
fn prefetch_available() -> bool {
    crate::extractors::auto::AutoExtractor::prefetch_available()
}

/// Return the name of the currently active cryptographic provider
/// (``"rust-crypto"`` for the default permissive provider, or
/// ``"aws-lc-rs"`` for the FIPS-validated provider once installed via
/// :func:`crypto_use_fips`).
///
/// **Non-initializing** — if no provider has been installed yet, this
/// returns ``"rust-crypto (default, lazy)"`` *without* committing the
/// process-wide registry. That means calling this for display/audit
/// before :func:`crypto_use_fips` is safe and won't lock the FIPS
/// opt-in out. Issue #236.
#[pyfunction]
fn crypto_active_provider() -> String {
    use crate::crypto::CryptoProvider;
    if crate::crypto::is_set() {
        crate::crypto::active().name().to_string()
    } else {
        format!("{} (default, lazy)", crate::crypto::RustCryptoProvider.name())
    }
}

/// List the cryptographic providers compiled into this build.
///
/// Always contains ``"rust-crypto"`` (the default permissive
/// provider). Also contains ``"aws-lc-rs"`` when the wheel was built
/// with ``--features fips`` — the FIPS 140-3 validated path.
#[pyfunction]
fn crypto_available_providers() -> Vec<String> {
    let v = vec!["rust-crypto".to_string()];
    #[cfg(feature = "fips")]
    let v = {
        let mut v = v;
        v.push("aws-lc-rs".to_string());
        v
    };
    v
}

/// Install the FIPS-validated ``aws-lc-rs`` provider as the
/// process-wide active cryptographic backend.
///
/// Must be called before any PDF operation that uses crypto (open
/// encrypted document, verify signature). Set-once: a second call,
/// or any call after a default provider was lazily-installed,
/// raises ``RuntimeError`` (the message is forwarded from
/// ``SetProviderError::to_string()`` and is not part of the public
/// contract — match on the exception type, not its text).
///
/// Raises ``RuntimeError("FIPS feature not compiled in")`` when the
/// wheel was built without ``--features fips``.
#[pyfunction]
fn crypto_use_fips() -> pyo3::PyResult<()> {
    #[cfg(feature = "fips")]
    {
        use std::sync::Arc;
        crate::crypto::set_provider(Arc::new(crate::crypto::AwsLcProvider::new()))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
    #[cfg(not(feature = "fips"))]
    {
        Err(pyo3::exceptions::PyRuntimeError::new_err(
            "FIPS provider not compiled in; build wheel with --features fips",
        ))
    }
}

/// Install the process-wide runtime crypto-governance policy (#230).
///
/// ``spec`` grammar: ``mode[;clause]*`` where
/// ``mode ∈ {compat, strict, fips-strict}``
/// ``clause = (allow|deny):<alg>@<read|write>`` — e.g.
/// ``"compat;deny:rc4@write;deny:md5@write"`` or ``"fips-strict"``.
///
/// Set-once (a runtime policy downgrade is an attack vector). Raises
/// ``RuntimeError`` on an unparseable spec (**fail-closed** — treat
/// this as fatal; the policy is *not* installed) or if a policy is
/// already set. Default (never set) behaviour is ``compat`` —
/// byte-for-byte identical to pre-#230.
#[pyfunction]
fn crypto_set_policy(spec: &str) -> pyo3::PyResult<()> {
    let policy: crate::crypto::SecurityPolicy =
        spec.parse().map_err(|e: crate::crypto::PolicyParseError| {
            pyo3::exceptions::PyRuntimeError::new_err(e.to_string())
        })?;
    crate::crypto::set_policy(policy)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Return the active crypto policy as its canonical grammar string
/// (e.g. ``"strict"`` or ``"compat;deny:rc4@write"``). Non-mutating
/// beyond the lazy ``compat`` default that every crypto path shares.
#[pyfunction]
fn crypto_policy() -> String {
    crate::crypto::active_policy().to_string()
}

/// The cryptographic algorithms exercised so far this process, as
/// stable lowercase tokens (e.g. ``["md5", "aes256"]``) — the minimal
/// crypto-inventory / CBOM-adjacent governance report.
#[pyfunction]
fn crypto_inventory() -> Vec<String> {
    crate::crypto::inventory()
        .into_iter()
        .map(|a| a.token().to_string())
        .collect()
}

/// A CycloneDX 1.6 Cryptographic Bill of Materials (JSON string) of the
/// algorithms exercised so far this process (#230 Phase F).
#[pyfunction]
fn crypto_cbom() -> String {
    crate::crypto::cbom_json()
}

/// Sign raw PDF bytes and return the signed PDF as `bytes`.
///
/// `cert` must be a :class:`Certificate` loaded via
/// :meth:`Certificate.load_pem` or :meth:`Certificate.load_pkcs12`
/// (i.e. it must carry a private key, not just a certificate).
///
/// # Example
///
/// ```python
/// from pdf_oxide import Certificate, sign_pdf_bytes
///
/// cert = Certificate.load_pem(open("cert.pem").read(), open("key.pem").read())
/// with open("input.pdf", "rb") as f:
///     signed = sign_pdf_bytes(f.read(), cert, reason="Approved", location="HQ")
/// with open("signed.pdf", "wb") as f:
///     f.write(signed)
/// ```
#[pyfunction]
#[pyo3(name = "sign_pdf_bytes")]
#[pyo3(signature = (pdf_data, cert, reason=None, location=None))]
pub fn py_sign_pdf_bytes<'py>(
    py: pyo3::Python<'py>,
    pdf_data: &Bound<'py, PyBytes>,
    cert: &PyCertificate,
    reason: Option<&str>,
    location: Option<&str>,
) -> PyResult<Bound<'py, PyBytes>> {
    #[cfg(feature = "signatures")]
    {
        use crate::signatures::{sign_pdf_bytes, SignOptions};
        let opts = SignOptions {
            reason: reason.map(str::to_owned),
            location: location.map(str::to_owned),
            ..Default::default()
        };
        let signed = sign_pdf_bytes(pdf_data.as_bytes(), &cert.creds, opts).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("sign_pdf_bytes failed: {e}"))
        })?;
        Ok(PyBytes::new(py, &signed))
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (pdf_data, cert, reason, location);
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "sign_pdf_bytes(): pdf_oxide was built without --features signatures",
        ))
    }
}

// ─── PAdES LTV (#235) ───────────────────────────────────────────────────────

/// PAdES baseline level. Frozen integer mapping (B_B=0, B_T=1, B_LT=2,
/// B_LTA=3) shared with the C ABI and every binding — never renumber.
#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "PadesLevel",
    eq,
    eq_int,
    skip_from_py_object
)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyPadesLevel {
    BB = 0,
    BT = 1,
    BLt = 2,
    BLta = 3,
}

#[cfg(feature = "signatures")]
impl PyPadesLevel {
    fn to_core(self) -> crate::signatures::PadesLevel {
        // `from_code` is total over the four frozen codes.
        crate::signatures::PadesLevel::from_code(self as i32)
            .expect("frozen PadesLevel code round-trips")
    }
    fn from_core(level: crate::signatures::PadesLevel) -> PyPadesLevel {
        match level.code() {
            0 => PyPadesLevel::BB,
            1 => PyPadesLevel::BT,
            2 => PyPadesLevel::BLt,
            _ => PyPadesLevel::BLta,
        }
    }
}

#[pymethods]
impl PyPadesLevel {
    #[classattr]
    const B_B: PyPadesLevel = PyPadesLevel::BB;
    #[classattr]
    const B_T: PyPadesLevel = PyPadesLevel::BT;
    #[classattr]
    const B_LT: PyPadesLevel = PyPadesLevel::BLt;
    #[classattr]
    const B_LTA: PyPadesLevel = PyPadesLevel::BLta;

    fn __int__(&self) -> i32 {
        *self as i32
    }

    fn __repr__(&self) -> &'static str {
        match self {
            PyPadesLevel::BB => "PadesLevel.B_B",
            PyPadesLevel::BT => "PadesLevel.B_T",
            PyPadesLevel::BLt => "PadesLevel.B_LT",
            PyPadesLevel::BLta => "PadesLevel.B_LTA",
        }
    }
}

/// Offline validation material for B-LT (DER certificates / CRLs /
/// OCSP responses). Mirrors Rust `signatures::RevocationMaterial`.
#[pyclass(
    module = "pdf_oxide.pdf_oxide",
    name = "RevocationMaterial",
    skip_from_py_object
)]
#[derive(Clone, Default)]
pub struct PyRevocationMaterial {
    certs: Vec<Vec<u8>>,
    crls: Vec<Vec<u8>>,
    ocsps: Vec<Vec<u8>>,
}

#[pymethods]
impl PyRevocationMaterial {
    #[new]
    #[pyo3(signature = (certs=None, crls=None, ocsps=None))]
    fn new(
        certs: Option<Vec<Vec<u8>>>,
        crls: Option<Vec<Vec<u8>>>,
        ocsps: Option<Vec<Vec<u8>>>,
    ) -> Self {
        PyRevocationMaterial {
            certs: certs.unwrap_or_default(),
            crls: crls.unwrap_or_default(),
            ocsps: ocsps.unwrap_or_default(),
        }
    }
}

/// A parsed Document Security Store (`/DSS`, ISO 32000-2 §12.8.4.3).
/// `certs`/`crls`/`ocsps` are document-level DER blobs; `vri` is the
/// list of per-signature VRI keys (uppercase-hex SHA-1 of `/Contents`).
#[pyclass(module = "pdf_oxide.pdf_oxide", name = "Dss")]
pub struct PyDss {
    #[cfg(feature = "signatures")]
    dss: crate::signatures::DocumentSecurityStore,
}

#[cfg(feature = "signatures")]
#[pymethods]
impl PyDss {
    #[getter]
    fn certs<'py>(&self, py: pyo3::Python<'py>) -> Vec<Bound<'py, PyBytes>> {
        self.dss
            .certificates
            .iter()
            .map(|d| PyBytes::new(py, d))
            .collect()
    }
    #[getter]
    fn crls<'py>(&self, py: pyo3::Python<'py>) -> Vec<Bound<'py, PyBytes>> {
        self.dss.crls.iter().map(|d| PyBytes::new(py, d)).collect()
    }
    #[getter]
    fn ocsps<'py>(&self, py: pyo3::Python<'py>) -> Vec<Bound<'py, PyBytes>> {
        self.dss
            .ocsp_responses
            .iter()
            .map(|d| PyBytes::new(py, d))
            .collect()
    }
    #[getter]
    fn vri(&self) -> Vec<String> {
        self.dss
            .vri
            .iter()
            .map(|v| v.signature_digest.clone())
            .collect()
    }
}

/// Sign raw PDF bytes at a PAdES baseline level and return the signed
/// PDF as `bytes`.
///
/// `level` is a :class:`PadesLevel` (``B_LTA`` is reserved →
/// ``NotImplementedError``). For ``B_T``/``B_LT`` a `tsa_url` is
/// required (RFC 3161 source; needs the ``tsa-client`` build feature).
/// `revocation` supplies the B-LT DSS material.
///
/// ```python
/// from pdf_oxide import Certificate, PadesLevel, RevocationMaterial, sign_pdf_bytes_pades
///
/// cert = Certificate.load_pkcs12(open("id.p12","rb").read(), "pw")
/// signed = sign_pdf_bytes_pades(
///     open("in.pdf","rb").read(), cert, PadesLevel.B_LT,
///     tsa_url="https://freetsa.org/tsr",
///     revocation=RevocationMaterial(certs=[cert_der]),
/// )
/// ```
#[pyfunction]
#[pyo3(name = "sign_pdf_bytes_pades")]
#[pyo3(signature = (pdf_data, cert, level, tsa_url=None, reason=None, location=None, revocation=None))]
#[allow(clippy::too_many_arguments)]
pub fn py_sign_pdf_bytes_pades<'py>(
    py: pyo3::Python<'py>,
    pdf_data: &Bound<'py, PyBytes>,
    cert: &PyCertificate,
    level: &PyPadesLevel,
    tsa_url: Option<&str>,
    reason: Option<&str>,
    location: Option<&str>,
    revocation: Option<&PyRevocationMaterial>,
) -> PyResult<Bound<'py, PyBytes>> {
    #[cfg(feature = "signatures")]
    {
        use crate::signatures::{sign_pdf_bytes_pades, RevocationMaterial, SignOptions};
        let opts = SignOptions {
            reason: reason.map(str::to_owned),
            location: location.map(str::to_owned),
            ..Default::default()
        };
        let material = revocation
            .map(|r| RevocationMaterial {
                certificates: r.certs.clone(),
                crls: r.crls.clone(),
                ocsp_responses: r.ocsps.clone(),
                ..Default::default()
            })
            .unwrap_or_default();

        #[cfg(feature = "tsa-client")]
        let ts_closure: Option<Box<dyn Fn(&[u8]) -> crate::error::Result<Vec<u8>>>> =
            tsa_url.map(|u| {
                let client = crate::signatures::TsaClient::new(
                    crate::signatures::TsaClientConfig::new(u.to_owned()),
                );
                Box::new(move |sig: &[u8]| {
                    client
                        .request_timestamp(sig)
                        .map(|t| t.token_bytes().to_vec())
                }) as Box<dyn Fn(&[u8]) -> crate::error::Result<Vec<u8>>>
            });
        #[cfg(not(feature = "tsa-client"))]
        let ts_closure: Option<Box<dyn Fn(&[u8]) -> crate::error::Result<Vec<u8>>>> = {
            let _ = tsa_url;
            None
        };

        let signed = sign_pdf_bytes_pades(
            pdf_data.as_bytes(),
            &cert.creds,
            opts,
            level.to_core(),
            ts_closure.as_deref(),
            &material,
        )
        .map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("sign_pdf_bytes_pades failed: {e}"))
        })?;
        Ok(PyBytes::new(py, &signed))
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (pdf_data, cert, level, tsa_url, reason, location, revocation);
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "sign_pdf_bytes_pades(): pdf_oxide was built without --features signatures",
        ))
    }
}

/// Whether `pdf_data` carries a document-scoped RFC 3161
/// `/DocTimeStamp` archival timestamp (PAdES-B-LTA). This is the
/// document-level reader signal; `Signature.pades_level` is
/// signature-scoped and tops out at B-LT by design.
///
/// ```python
/// from pdf_oxide import has_document_timestamp
/// with open("ltv.pdf", "rb") as f:
///     is_lta = has_document_timestamp(f.read())
/// ```
#[pyfunction]
#[pyo3(name = "has_document_timestamp")]
pub fn py_has_document_timestamp(pdf_data: &Bound<'_, PyBytes>) -> bool {
    #[cfg(feature = "signatures")]
    {
        crate::signatures::has_document_timestamp(pdf_data.as_bytes())
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = pdf_data;
        false
    }
}
