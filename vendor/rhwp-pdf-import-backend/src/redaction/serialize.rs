//! Content-stream re-serialization for destructive redaction (#231, T1).
//!
//! After pruning, the redactor must turn an [`Operator`] sequence back
//! into content-stream bytes. A near-complete serializer already lived in
//! the `crate::editor::document_editor` module; duplicating it in the redactor
//! would be a DRY violation and a second place for a corruption bug to
//! hide. So the canonical serializer lives here as `pub(crate)` free
//! functions (they need no `DocumentEditor` state) and the editor
//! delegates to them — one source of truth.
//!
//! ## Binary-safe string emission (security-relevant — G6)
//!
//! The prior serializer always emitted PDF strings as literal `(…)` with
//! only `(`, `)`, `\` escaped. A redacted byte string containing control
//! bytes or an unbalanced parenthesis could then produce a *malformed*
//! stream from which a raw `grep` might still recover the secret, or that
//! a lenient reader re-interprets. Per ISO 32000-1:2008 §7.3.4.3
//! ("useful for including arbitrary binary data"), any string that is not
//! pure printable ASCII is emitted as an unambiguous hexadecimal string
//! `<…>`; printable strings keep the literal form (§7.3.4.2) with the
//! three mandatory escapes so existing output is byte-for-byte unchanged.

use crate::content::operators::{Operator, TextElement};
use crate::object::Object;

/// Whether `s` is safe to emit as a PDF literal string: every byte is
/// printable ASCII (`0x20..=0x7E`). Control bytes / high bytes force the
/// hexadecimal form so the output cannot be malformed or grep-recovered
/// (feature plan §4.7 / G6).
fn is_literal_safe(s: &[u8]) -> bool {
    s.iter().all(|&b| (0x20..=0x7E).contains(&b))
}

/// Append `s` as a PDF string object. Printable ASCII ⇒ literal `(…)`
/// with `\`, `(`, `)` escaped (ISO 32000-1 §7.3.4.2); anything else ⇒
/// hexadecimal `<…>` (§7.3.4.3) — binary-safe, never malformed.
pub(crate) fn write_pdf_string(output: &mut Vec<u8>, s: &[u8]) {
    if is_literal_safe(s) {
        output.push(b'(');
        for &byte in s {
            if matches!(byte, b'(' | b')' | b'\\') {
                output.push(b'\\');
            }
            output.push(byte);
        }
        output.push(b')');
    } else {
        output.push(b'<');
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        for &byte in s {
            output.push(HEX[(byte >> 4) as usize]);
            output.push(HEX[(byte & 0x0F) as usize]);
        }
        output.push(b'>');
    }
}

/// Serialize one content-stream [`Operator`] to bytes (operands then
/// operator keyword, ISO 32000-1 §7.8.2). Non-string operators are
/// emitted byte-for-byte as before; strings go through
/// [`write_pdf_string`] for binary safety.
pub(crate) fn serialize_operator(output: &mut Vec<u8>, op: &Operator) {
    match op {
        // Graphics state
        Operator::SaveState => output.extend_from_slice(b"q\n"),
        Operator::RestoreState => output.extend_from_slice(b"Q\n"),
        Operator::Cm { a, b, c, d, e, f } => {
            output.extend_from_slice(
                format!("{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} cm\n", a, b, c, d, e, f).as_bytes(),
            );
        },
        Operator::SetLineWidth { width } => {
            output.extend_from_slice(format!("{:.6} w\n", width).as_bytes());
        },
        Operator::SetLineCap { cap_style } => {
            output.extend_from_slice(format!("{} J\n", cap_style).as_bytes());
        },
        Operator::SetLineJoin { join_style } => {
            output.extend_from_slice(format!("{} j\n", join_style).as_bytes());
        },
        Operator::SetMiterLimit { limit } => {
            output.extend_from_slice(format!("{:.6} M\n", limit).as_bytes());
        },
        Operator::SetDash { array, phase } => {
            output.push(b'[');
            for (i, v) in array.iter().enumerate() {
                if i > 0 {
                    output.push(b' ');
                }
                output.extend_from_slice(format!("{:.6}", v).as_bytes());
            }
            output.extend_from_slice(format!("] {:.6} d\n", phase).as_bytes());
        },
        Operator::SetFlatness { tolerance } => {
            output.extend_from_slice(format!("{:.6} i\n", tolerance).as_bytes());
        },
        Operator::SetRenderingIntent { intent } => {
            output.extend_from_slice(format!("/{} ri\n", intent).as_bytes());
        },
        Operator::SetExtGState { dict_name } => {
            output.extend_from_slice(format!("/{} gs\n", dict_name).as_bytes());
        },

        // Path construction
        Operator::MoveTo { x, y } => {
            output.extend_from_slice(format!("{:.6} {:.6} m\n", x, y).as_bytes());
        },
        Operator::LineTo { x, y } => {
            output.extend_from_slice(format!("{:.6} {:.6} l\n", x, y).as_bytes());
        },
        Operator::CurveTo {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => {
            output.extend_from_slice(
                format!("{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} c\n", x1, y1, x2, y2, x3, y3)
                    .as_bytes(),
            );
        },
        Operator::CurveToV { x2, y2, x3, y3 } => {
            output.extend_from_slice(
                format!("{:.6} {:.6} {:.6} {:.6} v\n", x2, y2, x3, y3).as_bytes(),
            );
        },
        Operator::CurveToY { x1, y1, x3, y3 } => {
            output.extend_from_slice(
                format!("{:.6} {:.6} {:.6} {:.6} y\n", x1, y1, x3, y3).as_bytes(),
            );
        },
        Operator::ClosePath => output.extend_from_slice(b"h\n"),
        Operator::Rectangle {
            x,
            y,
            width,
            height,
        } => {
            output.extend_from_slice(
                format!("{:.6} {:.6} {:.6} {:.6} re\n", x, y, width, height).as_bytes(),
            );
        },

        // Path painting
        Operator::Stroke => output.extend_from_slice(b"S\n"),
        Operator::CloseAndStroke => output.extend_from_slice(b"s\n"),
        Operator::Fill => output.extend_from_slice(b"f\n"),
        Operator::FillEvenOdd => output.extend_from_slice(b"f*\n"),
        Operator::CloseFillStroke => output.extend_from_slice(b"b\n"),
        Operator::FillStroke => output.extend_from_slice(b"B\n"),
        Operator::FillStrokeEvenOdd => output.extend_from_slice(b"B*\n"),
        Operator::CloseFillStrokeEvenOdd => output.extend_from_slice(b"b*\n"),
        Operator::EndPath => output.extend_from_slice(b"n\n"),

        // Clipping
        Operator::ClipNonZero => output.extend_from_slice(b"W\n"),
        Operator::ClipEvenOdd => output.extend_from_slice(b"W*\n"),

        // Text object
        Operator::BeginText => output.extend_from_slice(b"BT\n"),
        Operator::EndText => output.extend_from_slice(b"ET\n"),

        // Text state
        Operator::Tc { char_space } => {
            output.extend_from_slice(format!("{:.6} Tc\n", char_space).as_bytes());
        },
        Operator::Tw { word_space } => {
            output.extend_from_slice(format!("{:.6} Tw\n", word_space).as_bytes());
        },
        Operator::Tz { scale } => {
            output.extend_from_slice(format!("{:.6} Tz\n", scale).as_bytes());
        },
        Operator::TL { leading } => {
            output.extend_from_slice(format!("{:.6} TL\n", leading).as_bytes());
        },
        Operator::Tf { font, size } => {
            output.extend_from_slice(format!("/{} {:.6} Tf\n", font, size).as_bytes());
        },
        Operator::Tr { render } => {
            output.extend_from_slice(format!("{} Tr\n", render).as_bytes());
        },
        Operator::Ts { rise } => {
            output.extend_from_slice(format!("{:.6} Ts\n", rise).as_bytes());
        },

        // Text positioning
        Operator::Td { tx, ty } => {
            output.extend_from_slice(format!("{:.6} {:.6} Td\n", tx, ty).as_bytes());
        },
        Operator::TD { tx, ty } => {
            output.extend_from_slice(format!("{:.6} {:.6} TD\n", tx, ty).as_bytes());
        },
        Operator::Tm { a, b, c, d, e, f } => {
            output.extend_from_slice(
                format!("{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} Tm\n", a, b, c, d, e, f).as_bytes(),
            );
        },
        Operator::TStar => output.extend_from_slice(b"T*\n"),

        // Text showing
        Operator::Tj { text } => {
            write_pdf_string(output, text);
            output.extend_from_slice(b" Tj\n");
        },
        Operator::TJ { array } => {
            output.push(b'[');
            for item in array {
                match item {
                    TextElement::String(text) => write_pdf_string(output, text),
                    TextElement::Offset(offset) => {
                        output.extend_from_slice(format!("{:.6}", offset).as_bytes());
                    },
                }
            }
            output.extend_from_slice(b"] TJ\n");
        },
        Operator::Quote { text } => {
            write_pdf_string(output, text);
            output.extend_from_slice(b" '\n");
        },
        Operator::DoubleQuote {
            word_space,
            char_space,
            text,
        } => {
            output.extend_from_slice(format!("{:.6} {:.6} ", word_space, char_space).as_bytes());
            write_pdf_string(output, text);
            output.extend_from_slice(b" \"\n");
        },

        // Color space
        Operator::SetStrokeColorSpace { name } => {
            output.extend_from_slice(format!("/{} CS\n", name).as_bytes());
        },
        Operator::SetFillColorSpace { name } => {
            output.extend_from_slice(format!("/{} cs\n", name).as_bytes());
        },
        Operator::SetStrokeColor { components } => {
            for c in components {
                output.extend_from_slice(format!("{:.6} ", c).as_bytes());
            }
            output.extend_from_slice(b"SC\n");
        },
        Operator::SetFillColor { components } => {
            for c in components {
                output.extend_from_slice(format!("{:.6} ", c).as_bytes());
            }
            output.extend_from_slice(b"sc\n");
        },
        Operator::SetStrokeColorN { components, name } => {
            for c in components {
                output.extend_from_slice(format!("{:.6} ", c).as_bytes());
            }
            if let Some(p) = name {
                output.extend_from_slice(format!("/{} ", p).as_bytes());
            }
            output.extend_from_slice(b"SCN\n");
        },
        Operator::SetFillColorN { components, name } => {
            for c in components {
                output.extend_from_slice(format!("{:.6} ", c).as_bytes());
            }
            if let Some(p) = name {
                output.extend_from_slice(format!("/{} ", p).as_bytes());
            }
            output.extend_from_slice(b"scn\n");
        },
        Operator::SetStrokeGray { gray } => {
            output.extend_from_slice(format!("{:.6} G\n", gray).as_bytes());
        },
        Operator::SetFillGray { gray } => {
            output.extend_from_slice(format!("{:.6} g\n", gray).as_bytes());
        },
        Operator::SetStrokeRgb { r, g, b } => {
            output.extend_from_slice(format!("{:.6} {:.6} {:.6} RG\n", r, g, b).as_bytes());
        },
        Operator::SetFillRgb { r, g, b } => {
            output.extend_from_slice(format!("{:.6} {:.6} {:.6} rg\n", r, g, b).as_bytes());
        },
        Operator::SetStrokeCmyk { c, m, y, k } => {
            output.extend_from_slice(format!("{:.6} {:.6} {:.6} {:.6} K\n", c, m, y, k).as_bytes());
        },
        Operator::SetFillCmyk { c, m, y, k } => {
            output.extend_from_slice(format!("{:.6} {:.6} {:.6} {:.6} k\n", c, m, y, k).as_bytes());
        },

        // XObject
        Operator::Do { name } => {
            output.extend_from_slice(format!("/{} Do\n", name).as_bytes());
        },

        // Marked content
        Operator::BeginMarkedContent { tag } => {
            output.extend_from_slice(format!("/{} BMC\n", tag).as_bytes());
        },
        Operator::BeginMarkedContentDict { tag, properties } => {
            output.extend_from_slice(format!("/{} ", tag).as_bytes());
            serialize_object(output, properties);
            output.extend_from_slice(b" BDC\n");
        },
        Operator::EndMarkedContent => output.extend_from_slice(b"EMC\n"),

        // Shading
        Operator::PaintShading { name } => {
            output.extend_from_slice(format!("/{} sh\n", name).as_bytes());
        },

        // Inline image (BI…ID…EI)
        Operator::InlineImage { dict, data } => {
            output.extend_from_slice(b"BI\n");
            // Sorted for the same reason as `Object::Dictionary` below: the
            // inline-image dictionary is a `HashMap`, and every inline image
            // carries several keys (/W /H /BPC /CS at minimum), so unsorted
            // emission randomizes the content stream's bytes every run.
            let mut keys: Vec<_> = dict.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(value) = dict.get(key) {
                    output.extend_from_slice(format!("/{} ", key).as_bytes());
                    serialize_object(output, value);
                    output.push(b'\n');
                }
            }
            output.extend_from_slice(b"ID ");
            output.extend_from_slice(data);
            output.extend_from_slice(b"\nEI\n");
        },

        // Fallback for unrecognized operators
        Operator::Other { name, operands } => {
            for operand in operands.iter() {
                serialize_object(output, operand);
                output.push(b' ');
            }
            output.extend_from_slice(name.as_bytes());
            output.push(b'\n');
        },
    }
}

/// Serialize a PDF [`Object`] to bytes. Strings go through
/// [`write_pdf_string`] (binary-safe); every other variant is emitted as
/// before.
pub(crate) fn serialize_object(output: &mut Vec<u8>, obj: &Object) {
    match obj {
        Object::Null => output.extend_from_slice(b"null"),
        Object::Boolean(b) => {
            output.extend_from_slice(if *b { b"true" } else { b"false" });
        },
        Object::Integer(i) => output.extend_from_slice(format!("{}", i).as_bytes()),
        Object::Real(r) => output.extend_from_slice(format!("{:.6}", r).as_bytes()),
        Object::Name(n) => output.extend_from_slice(format!("/{}", n).as_bytes()),
        Object::String(s) => write_pdf_string(output, s),
        Object::Array(arr) => {
            output.push(b'[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    output.push(b' ');
                }
                serialize_object(output, item);
            }
            output.push(b']');
        },
        Object::Dictionary(dict) => {
            output.extend_from_slice(b"<<");
            // Sort keys for deterministic output. `Object::Dictionary` is a
            // `HashMap`, so iterating it directly emits the entries in the
            // per-process-randomized order and the redacted PDF's bytes differ
            // between runs of the same binary on the same input. Unlike a
            // tie-break this is unconditional: any dictionary with two or more
            // keys reorders. Same fix as `writer::ObjectSerializer`, which
            // already sorts for this reason.
            let mut keys: Vec<_> = dict.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(value) = dict.get(key) {
                    output.extend_from_slice(format!("/{} ", key).as_bytes());
                    serialize_object(output, value);
                }
            }
            output.extend_from_slice(b">>");
        },
        Object::Stream { .. } => {
            // Streams are complex; inline serialization is a placeholder
            // (matches the prior editor behavior — not a real string).
            output.extend_from_slice(b"(stream)");
        },
        Object::Reference(obj_ref) => {
            output.extend_from_slice(format!("{} {} R", obj_ref.id, obj_ref.gen).as_bytes());
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Object;

    fn s(v: &[u8]) -> String {
        String::from_utf8(v.to_vec()).unwrap()
    }

    #[test]
    fn printable_string_is_literal_with_three_escapes() {
        let mut out = Vec::new();
        write_pdf_string(&mut out, b"Hello (World) \\ test");
        // ( ) \ escaped; everything else verbatim, literal form.
        assert_eq!(s(&out), "(Hello \\(World\\) \\\\ test)");
    }

    #[test]
    fn binary_string_is_hexadecimal() {
        let mut out = Vec::new();
        // contains a NUL and a high byte ⇒ not literal-safe.
        write_pdf_string(&mut out, &[0x00, 0xFF, b'A']);
        assert_eq!(s(&out), "<00FF41>");
    }

    #[test]
    fn control_bytes_force_hex_no_malformed_literal() {
        // A newline inside a literal could split the string for a lenient
        // parser and leave the secret grep-recoverable; hex form is safe.
        let mut out = Vec::new();
        write_pdf_string(&mut out, b"sec\nret");
        assert_eq!(s(&out), "<7365630A726574>");
    }

    #[test]
    fn empty_string_is_empty_literal() {
        let mut out = Vec::new();
        write_pdf_string(&mut out, b"");
        assert_eq!(s(&out), "()");
    }

    #[test]
    fn tj_uses_binary_safe_string() {
        let mut out = Vec::new();
        serialize_operator(
            &mut out,
            &Operator::Tj {
                text: vec![0xDE, 0xAD],
            },
        );
        assert_eq!(s(&out), "<DEAD> Tj\n");
    }

    #[test]
    fn tj_printable_unchanged_byte_for_byte() {
        // Regression: prior literal output must be preserved for ASCII.
        let mut out = Vec::new();
        serialize_operator(
            &mut out,
            &Operator::Tj {
                text: b"Hello (World) \\ test".to_vec(),
            },
        );
        assert_eq!(s(&out), "(Hello \\(World\\) \\\\ test) Tj\n");
    }

    #[test]
    fn tj_array_offsets_and_strings() {
        let mut out = Vec::new();
        serialize_operator(
            &mut out,
            &Operator::TJ {
                array: vec![
                    TextElement::String(b"AB".to_vec()),
                    TextElement::Offset(-50.0),
                    TextElement::String(vec![0x01]),
                ],
            },
        );
        assert_eq!(s(&out), "[(AB)-50.000000<01>] TJ\n");
    }

    #[test]
    fn object_string_is_binary_safe() {
        let mut out = Vec::new();
        serialize_object(&mut out, &Object::String(vec![0x80]));
        assert_eq!(s(&out), "<80>");
        out.clear();
        serialize_object(&mut out, &Object::String(b"ok".to_vec()));
        assert_eq!(s(&out), "(ok)");
    }

    #[test]
    fn non_string_operators_byte_identical() {
        let mut out = Vec::new();
        serialize_operator(&mut out, &Operator::SaveState);
        serialize_operator(
            &mut out,
            &Operator::Cm {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 10.0,
                f: 20.0,
            },
        );
        serialize_operator(&mut out, &Operator::RestoreState);
        assert_eq!(s(&out), "q\n1.000000 0.000000 0.000000 1.000000 10.000000 20.000000 cm\nQ\n");
    }

    /// A dictionary with several keys must serialize in sorted key order every
    /// run. `Object::Dictionary` is a `HashMap`, so emitting its entries in
    /// iteration order put the redacted PDF's bytes at the mercy of the
    /// per-process hash seed — this assertion failed on most runs before the
    /// keys were sorted. Unlike a tie-break the defect is unconditional: two
    /// or more keys is enough.
    #[test]
    fn dictionary_serializes_in_sorted_key_order() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("Zebra".to_string(), Object::Integer(3));
        dict.insert("Alpha".to_string(), Object::Integer(1));
        dict.insert("Middle".to_string(), Object::Integer(2));
        let mut out = Vec::new();
        serialize_object(&mut out, &Object::Dictionary(dict));
        assert_eq!(s(&out), "<</Alpha 1/Middle 2/Zebra 3>>");
    }

    /// A single-key dictionary is unaffected by the sort — the ordinary case
    /// must serialize exactly as before.
    #[test]
    fn single_key_dictionary_is_unaffected_by_sorting() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("Only".to_string(), Object::Name("Value".to_string()));
        let mut out = Vec::new();
        serialize_object(&mut out, &Object::Dictionary(dict));
        assert_eq!(s(&out), "<</Only /Value>>");
    }

    /// The inline-image dictionary is emitted through a separate loop and
    /// needs the same guarantee: every inline image carries several keys, so
    /// unsorted emission randomized the content stream on essentially every
    /// image.
    #[test]
    fn inline_image_dictionary_serializes_in_sorted_key_order() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("W".to_string(), Object::Integer(2));
        dict.insert("BPC".to_string(), Object::Integer(8));
        dict.insert("H".to_string(), Object::Integer(1));
        let mut out = Vec::new();
        serialize_operator(
            &mut out,
            &Operator::InlineImage {
                dict: Box::new(dict),
                data: b"ab".to_vec(),
            },
        );
        assert_eq!(s(&out), "BI\n/BPC 8\n/H 1\n/W 2\nID ab\nEI\n");
    }
}
