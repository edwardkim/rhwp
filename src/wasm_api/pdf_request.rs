//! One disposable Worker request retains its import budget through first save.
use super::{pdf_import, HwpDocument, PdfImportInput};
use crate::parser::pdf::{PdfImportOptions, PdfImportPolicy, PdfImportProfile};
use crate::pdf_runtime::PdfTransferLimits;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
pub struct PdfImportRequest {
    session: crate::PdfImportSession,
    document: Option<HwpDocument>,
    options: Option<PdfImportOptions>,
    policy: PdfImportPolicy,
    reserved_input: Option<usize>,
    limits: PdfTransferLimits,
}

#[wasm_bindgen]
impl PdfImportRequest {
    #[wasm_bindgen(js_name = importPdf)]
    pub fn import_pdf(
        data: &PdfImportInput,
        options: JsValue,
    ) -> Result<PdfImportRequest, JsValue> {
        let mut request = Self::prepare(options)?;
        request.open(data)?;
        Ok(request)
    }

    /// Construct the request before its owner makes the private transfer copy.
    #[wasm_bindgen(js_name = prepare)]
    pub fn prepare(options: JsValue) -> Result<PdfImportRequest, JsValue> {
        let (options, policy, profile) = pdf_import::options(options)?;
        Ok(Self {
            session: crate::PdfImportSession::new(policy)?,
            document: None,
            options: Some(options),
            policy,
            reserved_input: None,
            limits: match profile {
                PdfImportProfile::Standard => PdfTransferLimits::standard(),
                PdfImportProfile::Constrained => PdfTransferLimits::constrained(),
            },
        })
    }

    #[wasm_bindgen(js_name = reserveInput)]
    pub fn reserve_input(&mut self, length: u32) -> Result<(), JsValue> {
        if self.options.is_none()
            || self.reserved_input.is_some()
            || length as usize > self.policy.max_input_bytes()
        {
            return Err(JsValue::from_str(
                "PDF_RESOURCE_POLICY: invalid input reservation",
            ));
        }
        self.session.reserve_transfer(length as usize)?;
        self.reserved_input = Some(length as usize);
        Ok(())
    }

    #[wasm_bindgen(js_name = open)]
    pub fn open(&mut self, data: &PdfImportInput) -> Result<(), JsValue> {
        self.open_with_progress(data, None)
    }

    /// Notify the Worker synchronously at actual page boundaries, without polling.
    #[wasm_bindgen(js_name = openWithProgress)]
    pub fn open_with_progress(
        &mut self,
        data: &PdfImportInput,
        progress: Option<js_sys::Function>,
    ) -> Result<(), JsValue> {
        let options = self
            .options
            .take()
            .ok_or_else(|| JsValue::from_str("PDF_IMPORT_SESSION: request already started"))?;
        if let Some(length) = self.reserved_input {
            if data
                .dyn_ref::<js_sys::Uint8Array>()
                .map(|v| v.length() as usize)
                != Some(length)
            {
                return Err(JsValue::from_str(
                    "PDF_INVALID_INPUT: reserved view length changed",
                ));
            }
        }
        let data = pdf_import::copy_input(data, &self.policy, &mut self.session)?;
        let core = self
            .session
            .open_with_progress(&data, &options, &mut |update| {
                if let Some(callback) = &progress {
                    callback
                        .call4(
                            &JsValue::NULL,
                            &JsValue::from_f64(update.source_page as f64),
                            &JsValue::from_f64(update.source_page_count as f64),
                            &JsValue::from_f64(update.completed_pages as f64),
                            &JsValue::from_f64(update.total_pages as f64),
                        )
                        .map_err(|_| crate::parser::pdf::PdfImportError {
                            code: "PDF_PROGRESS_FAILED",
                            message: "page progress callback failed",
                            diagnostics: None,
                        })?;
                }
                Ok(())
            })?;
        self.document = Some(HwpDocument { core });
        Ok(())
    }

    #[wasm_bindgen(js_name = getTransferLimits)]
    pub fn get_transfer_limits(&self) -> String {
        serde_json::to_string(&self.limits).expect("fixed transfer limits")
    }

    #[wasm_bindgen(js_name = remainingDecodedBytes)]
    pub fn remaining_decoded_bytes(&self) -> f64 {
        self.policy
            .backend
            .decoded_bytes
            .saturating_sub(self.session.resource_usage().decoded_bytes)
            .min(9_007_199_254_740_991) as f64
    }

    /// Count and reserve response backing before allocation, using the same
    /// cumulative request as parsing, Core construction and first serialization.
    #[wasm_bindgen(js_name = reserveResponseCopy)]
    pub fn reserve_response_copy(&mut self, bytes: u32) -> Result<(), JsValue> {
        self.session
            .reserve_transfer(bytes as usize)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = inspect)]
    pub fn inspect(&mut self) -> Result<String, JsValue> {
        let document = self
            .document
            .as_ref()
            .ok_or_else(|| JsValue::from_str("PDF_NO_DOCUMENT"))?;
        let bytes = self
            .session
            .diagnostics_json(&document.core, self.limits.diagnostics_bytes)?;
        let text = String::from_utf8(bytes)
            .map_err(|_| JsValue::from_str("PDF_WORKER_PROTOCOL: invalid response encoding"))?;
        self.session
            .reserve_transfer(text.encode_utf16().count().saturating_mul(2))?;
        Ok(text)
    }

    #[wasm_bindgen(js_name = pageCount)]
    pub fn page_count(&self) -> u32 {
        self.document.as_ref().map_or(0, HwpDocument::page_count)
    }

    #[wasm_bindgen(js_name = renderPageSvg)]
    pub fn render_page_svg(&mut self, page: u32) -> Result<String, JsValue> {
        self.render_page_svg_limited(page, self.limits.output_bytes as u32)
    }

    #[wasm_bindgen(js_name = renderPageSvgLimited)]
    pub fn render_page_svg_limited(&mut self, page: u32, maximum: u32) -> Result<String, JsValue> {
        if maximum == 0 || maximum as usize > self.limits.output_bytes {
            return Err(JsValue::from_str(
                "PDF_RESOURCE_POLICY: invalid SVG byte limit",
            ));
        }
        let document = self
            .document
            .as_ref()
            .ok_or_else(|| JsValue::from_str("PDF_NO_DOCUMENT"))?;
        let text = self
            .session
            .render_page_svg(&document.core, page, maximum as usize)?;
        self.session
            .reserve_transfer(text.encode_utf16().count().saturating_mul(2))?;
        Ok(text)
    }

    #[wasm_bindgen(js_name = exportDocument)]
    pub fn export_document(&mut self, format: &str, maximum: u32) -> Result<Vec<u8>, JsValue> {
        if maximum == 0 || maximum as usize > self.limits.output_bytes {
            return Err(JsValue::from_str(
                "PDF_RESOURCE_POLICY: invalid output byte limit",
            ));
        }
        let document = self
            .document
            .as_mut()
            .ok_or_else(|| JsValue::from_str("PDF_NO_DOCUMENT"))?;
        let result = match format {
            "hwp" => self
                .session
                .export_hwp(&mut document.core, maximum as usize),
            "hwpx" => self
                .session
                .export_hwpx(&mut document.core, maximum as usize),
            _ => return Err(JsValue::from_str("PDF_EXPORT_FORMAT: expected hwp or hwpx")),
        };
        let bytes = result.map_err(JsValue::from)?;
        self.session.reserve_transfer(bytes.len())?;
        Ok(bytes)
    }
}

/// A generated editor handoff has its own finite remaining-credit scope. Once
/// taken, the ordinary document and lazy assets own an independent lifetime.
#[wasm_bindgen]
pub struct PdfEditorReopen {
    document: Option<HwpDocument>,
    usage: pdf_oxide::import_limits::ImportBudgetSnapshot,
}

#[wasm_bindgen]
impl PdfEditorReopen {
    #[wasm_bindgen(js_name = open)]
    pub fn open(
        data: &PdfImportInput,
        options: JsValue,
        maximum: u32,
        remaining: f64,
        timeout_ms: f64,
    ) -> Result<PdfEditorReopen, JsValue> {
        let (_, mut policy, profile) = pdf_import::options(options)?;
        let limit = match profile {
            PdfImportProfile::Standard => PdfTransferLimits::standard(),
            PdfImportProfile::Constrained => PdfTransferLimits::constrained(),
        };
        if maximum == 0
            || maximum as usize > limit.output_bytes
            || !remaining.is_finite()
            || remaining.fract() != 0.0
            || remaining <= 0.0
            || remaining > 9_007_199_254_740_991.0
            || remaining as u64 > policy.backend.decoded_bytes
        {
            return Err(JsValue::from_str(
                "PDF_RESOURCE_POLICY: invalid editor reopen allowance",
            ));
        }
        if !timeout_ms.is_finite() || timeout_ms <= 0.0 || timeout_ms > 2_147_483_647.0 {
            return Err(JsValue::from_str(
                "PDF_IMPORT_TIMEOUT: editor reopen deadline expired",
            ));
        }
        policy.backend.decoded_bytes = remaining as u64;
        let performance =
            js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("performance"))?;
        let now: js_sys::Function =
            js_sys::Reflect::get(&performance, &JsValue::from_str("now"))?.dyn_into()?;
        // The closure captures only a timestamp; each check reads the current
        // realm's monotonic clock. Native import requests retain no JS callback.
        let start = now
            .call0(&performance)?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("PDF_WORKER_PROTOCOL: no monotonic clock"))?;
        let deadline = start + timeout_ms;
        let calls = std::sync::atomic::AtomicUsize::new(0);
        let mut session = crate::PdfImportSession::new_with_work_guard(policy, move || {
            if calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed) & 127 != 0 {
                return true;
            }
            let Ok(performance) =
                js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("performance"))
            else {
                return false;
            };
            let Ok(now) = js_sys::Reflect::get(&performance, &JsValue::from_str("now")) else {
                return false;
            };
            let Ok(now) = now.dyn_into::<js_sys::Function>() else {
                return false;
            };
            now.call0(&performance)
                .ok()
                .and_then(|v| v.as_f64())
                .is_some_and(|v| v < deadline)
        })?;
        let result = (|| -> Result<HwpDocument, JsValue> {
            let bytes = pdf_import::copy_input_limited(data, maximum as usize, &mut session)?;
            let core = session.reopen_hwpx_export(&bytes, maximum as usize)?;
            Ok(HwpDocument { core })
        })();
        if now
            .call0(&performance)?
            .as_f64()
            .is_none_or(|time| time >= deadline)
        {
            return Err(JsValue::from_str(
                "PDF_IMPORT_TIMEOUT: editor reopen deadline exceeded",
            ));
        }
        let document = result?;
        let usage = session.resource_usage();
        Ok(Self {
            document: Some(document),
            usage,
        })
    }

    #[wasm_bindgen(js_name = resourceUsage)]
    pub fn resource_usage(&self) -> String {
        serde_json::to_string(&self.usage).expect("fixed resource usage")
    }

    #[wasm_bindgen(js_name = takeDocument)]
    pub fn take_document(&mut self) -> Result<HwpDocument, JsValue> {
        self.document
            .take()
            .ok_or_else(|| JsValue::from_str("PDF_EDITOR_REOPEN: document already taken"))
    }
}
