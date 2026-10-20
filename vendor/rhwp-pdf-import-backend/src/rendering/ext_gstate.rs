//! Shared parser for PDF `ExtGState` dictionary entries.
//!
//! Both the page renderer and the separation-plate renderer need to apply
//! transparency / blend-mode overrides from `gs` operators. Keeping the
//! parser in a single module avoids drift between the two renderers and
//! removes the `pub(crate)` leak that previously crossed module boundaries.

use crate::content::graphics_state::{GraphicsState, SoftMaskForm, SoftMaskSubtype};
use crate::document::PdfDocument;
use crate::error::Result;
use crate::object::Object;

/// A parsed `/SMask` value from an ExtGState dict (§11.4.7 / Table 144).
/// `None` corresponds to the spec `/None` value (clear the current mask);
/// `Form` carries the Form XObject reference plus optional backdrop and
/// transfer function (see [`SoftMaskForm`]).
#[derive(Clone, Debug)]
pub(crate) enum SoftMaskValue {
    /// `/SMask /None` — clear the current mask.
    None,
    /// `/SMask <<` … Form-XObject soft mask.
    Form(SoftMaskForm),
}

/// Parsed effects of a PDF `ExtGState` dictionary. Only the fields actually
/// applied during rendering are captured (fill/stroke alpha, blend mode,
/// the overprint parameters from ISO 32000-1 §11.7.4, and §11.4.7
/// Form-XObject soft masks).
#[derive(Clone, Debug, Default)]
pub(crate) struct ParsedExtGState {
    pub(crate) fill_alpha: Option<f32>,
    pub(crate) stroke_alpha: Option<f32>,
    pub(crate) blend_mode: Option<String>,
    /// Overprint for stroking operations (ExtGState `/OP`, §11.7.4).
    pub(crate) stroke_overprint: Option<bool>,
    /// Overprint for non-stroking operations (ExtGState `/op`, §11.7.4).
    pub(crate) fill_overprint: Option<bool>,
    /// Overprint mode (ExtGState `/OPM`, §11.7.4). 0 = standard, 1 = nonzero.
    pub(crate) overprint_mode: Option<u8>,
    /// Soft mask dispatch (§11.4.7). `None` means the entry was absent —
    /// gs.smask is left untouched. `Some(SoftMaskValue::None)` is the
    /// spec `/None` value (clear). `Some(SoftMaskValue::Form(..))` is a
    /// Form-XObject mask.
    pub(crate) smask: Option<SoftMaskValue>,
}

impl ParsedExtGState {
    /// Apply this dictionary's fields to `gs`. Fields that were not present
    /// in the source dictionary are left untouched on `gs`.
    pub(crate) fn apply(&self, gs: &mut GraphicsState) {
        if let Some(a) = self.fill_alpha {
            gs.fill_alpha = a;
        }
        if let Some(a) = self.stroke_alpha {
            gs.stroke_alpha = a;
        }
        if let Some(ref m) = self.blend_mode {
            gs.blend_mode = m.clone();
        }
        if let Some(v) = self.fill_overprint {
            gs.fill_overprint = v;
        }
        if let Some(v) = self.stroke_overprint {
            gs.stroke_overprint = v;
        }
        if let Some(v) = self.overprint_mode {
            gs.overprint_mode = v;
        }
        if let Some(ref sm) = self.smask {
            match sm {
                SoftMaskValue::None => {
                    gs.smask = None;
                },
                SoftMaskValue::Form(f) => {
                    gs.smask = Some(f.clone());
                },
            }
        }
    }
}

/// Parse the fields we need from an `ExtGState` *entry* (the inner dict, not
/// the resource dict that holds it). Resolves `state_obj` once if it is a
/// reference.
pub(crate) fn parse_ext_g_state_inner(
    state_obj: &Object,
    doc: &PdfDocument,
) -> Result<ParsedExtGState> {
    let mut out = ParsedExtGState::default();
    let state_resolved = doc.resolve_object(state_obj)?;
    let state_dict = match state_resolved.as_dict() {
        Some(d) => d,
        None => return Ok(out),
    };

    // ISO 32000-1 §7.3.10: ANY direct object can be replaced by an indirect
    // reference. Reading `/ca 3 0 R` as-is yields a `Reference` whose typed
    // accessors all return None; the field would silently drop to its
    // default. Resolve every value once before classifying. `/SMask` has
    // its own resolve below because its dict entries need the same
    // treatment.
    let read = |key: &str| -> Option<Object> {
        let raw = state_dict.get(key)?;
        doc.resolve_object(raw).ok()
    };

    if let Some(ca) = read("ca") {
        out.fill_alpha = ca
            .as_real()
            .map(|v| v as f32)
            .or_else(|| ca.as_integer().map(|v| v as f32));
    }
    if let Some(ca_upper) = read("CA") {
        out.stroke_alpha = ca_upper
            .as_real()
            .map(|v| v as f32)
            .or_else(|| ca_upper.as_integer().map(|v| v as f32));
    }
    if let Some(bm) = read("BM") {
        // ISO 32000-1 §11.3.5 + §11.6.3: `/BM` may be a name OR an array of
        // names. For an array, "the first name that names a blend mode
        // supported by the conforming reader shall be used". Unrecognised
        // names fall back to `/Normal` per §11.6.3. The classifier in
        // `crate::rendering::sidecar::is_recognised_mode` enumerates every
        // standard mode from §11.3.5.2 + §11.3.5.3; we share that list so
        // detection and dispatch stay in lockstep.
        //
        // Array elements may themselves be indirect refs (§7.3.10), so
        // each is resolved before pattern-matching its name.
        let mode = match &bm {
            Object::Name(n) => n.clone(),
            Object::Array(arr) => arr
                .iter()
                .filter_map(|elem| doc.resolve_object(elem).ok())
                .filter_map(|elem| elem.as_name().map(str::to_string))
                .find(|name| crate::rendering::sidecar::is_recognised_mode(name))
                .unwrap_or_else(|| "Normal".to_string()),
            _ => "Normal".to_string(),
        };
        out.blend_mode = Some(mode);
    }

    // ISO 32000-1 §11.7.4 / Table 128. `/OP` is the stroking overprint;
    // `/op` (lowercase) is the non-stroking overprint. When `/OP` is
    // present without `/op`, the spec says it sets both.
    let op_stroke = read("OP").and_then(|v| v.as_bool());
    let op_fill = read("op").and_then(|v| v.as_bool());
    out.stroke_overprint = op_stroke;
    out.fill_overprint = op_fill.or(op_stroke);

    if let Some(opm) = read("OPM").and_then(|v| v.as_integer()) {
        // Spec defines only 0 (standard) and 1 (nonzero). Any other
        // value is undefined; clamp to 0 so a malformed PDF doesn't
        // accidentally enable nonzero-overprint mode.
        out.overprint_mode = Some(if opm == 1 { 1 } else { 0 });
    }

    // ISO 32000-1:2008 §11.4.7 / Table 144. `/SMask` is either the
    // name `/None` (clear the current soft mask) or a soft-mask
    // dictionary referencing a Form XObject. Image-attached soft
    // masks (via an image XObject's own /SMask entry) are handled
    // at the image-blit site; this parser covers the ExtGState
    // path.
    if let Some(smask_obj) = state_dict.get("SMask") {
        // Resolve through references before classifying.
        let resolved = doc.resolve_object(smask_obj).unwrap_or(smask_obj.clone());
        match &resolved {
            Object::Name(n) if n == "None" => {
                out.smask = Some(SoftMaskValue::None);
            },
            Object::Dictionary(mask_dict) => {
                // §7.3.10: sub-entries of a SMask dict may themselves be
                // indirect refs. The /G entry is always a Reference by
                // design (it's the Form XObject id, used as a key into
                // the xref) so it stays an explicit Reference-match. The
                // /S, /BC, and /TR entries are values that the spec
                // allows to be direct OR indirect — resolve before
                // reading.
                let resolve_in_smask = |key: &str| -> Option<Object> {
                    let raw = mask_dict.get(key)?;
                    doc.resolve_object(raw).ok()
                };

                // Subtype: /S /Alpha or /S /Luminosity (default Alpha
                // per spec). Anything else falls through to None — a
                // malformed mask must not silently mis-render.
                let subtype = match resolve_in_smask("S").as_ref().and_then(Object::as_name) {
                    Some("Alpha") => SoftMaskSubtype::Alpha,
                    Some("Luminosity") => SoftMaskSubtype::Luminosity,
                    _ => SoftMaskSubtype::Alpha,
                };

                // /G — required Form XObject reference. Stays as a raw
                // Reference; the renderer loads the form via xref.
                let form_ref = mask_dict.get("G").and_then(|o| match o {
                    Object::Reference(r) => Some(*r),
                    _ => None,
                });

                if let Some(form_ref) = form_ref {
                    // /BC backdrop colour — array of N reals. Only
                    // honoured for /S /Luminosity per §11.4.7; for
                    // /S /Alpha the spec ignores /BC. Each array
                    // element may itself be an indirect ref (§7.3.10).
                    let backdrop = if subtype == SoftMaskSubtype::Luminosity {
                        resolve_in_smask("BC").and_then(|o| {
                            o.as_array().map(|arr| {
                                arr.iter()
                                    .filter_map(|v| doc.resolve_object(v).ok())
                                    .filter_map(|v| {
                                        v.as_real()
                                            .map(|r| r as f32)
                                            .or_else(|| v.as_integer().map(|i| i as f32))
                                    })
                                    .collect::<Vec<f32>>()
                            })
                        })
                    } else {
                        None
                    };

                    // /TR transfer function — stored as the resolved
                    // value; the renderer evaluates per-pixel via the
                    // Function evaluator already used for tint
                    // transforms. Indirect-ref TR (very common — `/TR
                    // 12 0 R` pointing at a Function dict) is now
                    // resolved at parse time rather than at every
                    // per-pixel call.
                    let transfer = resolve_in_smask("TR");

                    out.smask = Some(SoftMaskValue::Form(SoftMaskForm {
                        form_ref,
                        subtype,
                        backdrop,
                        transfer,
                    }));
                }
            },
            _ => {},
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Minimal PDF document used purely as a `&PdfDocument` argument for
    /// `parse_ext_g_state_inner`. The parser only calls `resolve_object`
    /// on the input; when the input is already an inline dict (not a
    /// `Reference`), that call short-circuits to a clone and never touches
    /// the document's xref. So any successfully-parsed PDF is sufficient.
    fn fixture_doc() -> PdfDocument {
        // Construct the smallest valid PDF that `from_bytes` will accept.
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"%PDF-1.4\n");
        let cat_off = buf.len();
        buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        let pages_off = buf.len();
        buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\n");
        let xref_off = buf.len();
        buf.extend_from_slice(b"xref\n0 3\n0000000000 65535 f \n");
        buf.extend_from_slice(format!("{:010} 00000 n \n", cat_off).as_bytes());
        buf.extend_from_slice(format!("{:010} 00000 n \n", pages_off).as_bytes());
        buf.extend_from_slice(
            format!("trailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off)
                .as_bytes(),
        );
        PdfDocument::from_bytes(buf).expect("fixture PDF parses")
    }

    fn dict(entries: &[(&str, Object)]) -> Object {
        let mut m = HashMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        Object::Dictionary(m)
    }

    #[test]
    fn parses_op_op_opm_from_extgstate_dict() {
        let obj = dict(&[
            ("OP", Object::Boolean(true)),
            ("op", Object::Boolean(false)),
            ("OPM", Object::Integer(1)),
        ]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_overprint, Some(true));
        assert_eq!(parsed.fill_overprint, Some(false));
        assert_eq!(parsed.overprint_mode, Some(1));
    }

    #[test]
    fn op_without_op_sets_both_overprints() {
        // §11.7.4 / Table 128: "Specifying an OP entry sets both
        // parameters unless there is also an op entry in the same
        // graphics state parameter dictionary".
        let obj = dict(&[("OP", Object::Boolean(true))]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_overprint, Some(true));
        assert_eq!(parsed.fill_overprint, Some(true));
    }

    #[test]
    fn op_without_op_uppercase_only_does_not_affect_stroke() {
        // /op is the non-stroking parameter only; /OP is absent so the
        // stroking overprint stays unset (caller falls back to gs default).
        let obj = dict(&[("op", Object::Boolean(true))]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_overprint, None);
        assert_eq!(parsed.fill_overprint, Some(true));
    }

    #[test]
    fn opm_clamps_unknown_values_to_zero() {
        // §11.7.4: OPM is 0 or 1; any other value is undefined. We clamp
        // to 0 (standard mode) to preserve the spec-default behavior on
        // malformed PDFs.
        let obj = dict(&[("OPM", Object::Integer(42))]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.overprint_mode, Some(0));
    }

    #[test]
    fn missing_overprint_keys_leave_options_none() {
        // Empty dict → no fields touched. Apply() is a no-op on the gs.
        let obj = dict(&[]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_overprint, None);
        assert_eq!(parsed.fill_overprint, None);
        assert_eq!(parsed.overprint_mode, None);
    }

    /// PDF whose xref carries primitive indirect objects we can reference
    /// from a synthetic ExtGState dict.
    ///
    ///   3 0 obj  0.5            (real)
    ///   4 0 obj  true           (bool)
    ///   5 0 obj  1              (integer)
    ///   6 0 obj  /Multiply      (name)
    ///   7 0 obj  [/Multiply]    (array of names)
    fn fixture_doc_with_indirect_values() -> PdfDocument {
        use crate::object::ObjectRef;
        let _ = ObjectRef::new(0, 0); // ensure the type is in scope for callers
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"%PDF-1.4\n");
        let mut offsets: Vec<usize> = Vec::new();
        let mut emit = |buf: &mut Vec<u8>, body: &str| {
            let off = buf.len();
            buf.extend_from_slice(body.as_bytes());
            offsets.push(off);
        };
        emit(&mut buf, "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        emit(&mut buf, "2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\n");
        emit(&mut buf, "3 0 obj\n0.5\nendobj\n");
        emit(&mut buf, "4 0 obj\ntrue\nendobj\n");
        emit(&mut buf, "5 0 obj\n1\nendobj\n");
        emit(&mut buf, "6 0 obj\n/Multiply\nendobj\n");
        emit(&mut buf, "7 0 obj\n[/Multiply]\nendobj\n");
        let xref_off = buf.len();
        buf.extend_from_slice(b"xref\n0 8\n0000000000 65535 f \n");
        for off in &offsets {
            buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }
        buf.extend_from_slice(
            format!("trailer\n<< /Size 8 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off)
                .as_bytes(),
        );
        PdfDocument::from_bytes(buf).expect("fixture PDF parses")
    }

    fn obj_ref(num: u32) -> Object {
        use crate::object::ObjectRef;
        Object::Reference(ObjectRef::new(num, 0))
    }

    // ISO 32000-1 §7.3.10 — any direct object value may be replaced by an
    // indirect reference. The ExtGState parser MUST resolve indirect
    // references for every value before reading the typed accessor, or
    // PDFs that emit e.g. `/ca 3 0 R` silently fall back to defaults.
    //
    // These probes pin that resolution for every value the parser reads.
    // Sensitivity-verify by reverting the resolved_value() call inside the
    // parser to a bare `state_dict.get(...)`: every probe below fails
    // because the typed accessor (.as_real / .as_bool / .as_integer /
    // .as_name / .as_array) returns None on an unresolved Reference.

    #[test]
    fn resolves_indirect_fill_alpha() {
        let obj = dict(&[("ca", obj_ref(3))]); // 3 0 R → 0.5
        let doc = fixture_doc_with_indirect_values();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.fill_alpha, Some(0.5_f32));
    }

    #[test]
    fn resolves_indirect_stroke_alpha() {
        let obj = dict(&[("CA", obj_ref(3))]); // 3 0 R → 0.5
        let doc = fixture_doc_with_indirect_values();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_alpha, Some(0.5_f32));
    }

    #[test]
    fn resolves_indirect_blend_mode_name() {
        let obj = dict(&[("BM", obj_ref(6))]); // 6 0 R → /Multiply
        let doc = fixture_doc_with_indirect_values();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.blend_mode.as_deref(), Some("Multiply"));
    }

    #[test]
    fn resolves_indirect_blend_mode_array() {
        let obj = dict(&[("BM", obj_ref(7))]); // 7 0 R → [/Multiply]
        let doc = fixture_doc_with_indirect_values();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.blend_mode.as_deref(), Some("Multiply"));
    }

    #[test]
    fn resolves_indirect_op_op_opm() {
        let obj = dict(&[
            ("OP", obj_ref(4)),  // 4 0 R → true
            ("op", obj_ref(4)),  // 4 0 R → true
            ("OPM", obj_ref(5)), // 5 0 R → 1
        ]);
        let doc = fixture_doc_with_indirect_values();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_overprint, Some(true));
        assert_eq!(parsed.fill_overprint, Some(true));
        assert_eq!(parsed.overprint_mode, Some(1));
    }

    #[test]
    fn resolves_indirect_blend_mode_array_with_indirect_elements() {
        // PDFs in the wild emit `/BM [5 0 R 6 0 R]` where each element is
        // itself an indirect reference to a name object. §7.3.10 lets any
        // direct value be an indirect ref, including inside an array.
        // The parser must resolve each element before classifying.
        use crate::object::ObjectRef;
        let array_with_indirect_name = Object::Array(vec![
            Object::Reference(ObjectRef::new(6, 0)), // → /Multiply
        ]);
        let obj = dict(&[("BM", array_with_indirect_name)]);
        let doc = fixture_doc_with_indirect_values();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.blend_mode.as_deref(), Some("Multiply"));
    }
}
