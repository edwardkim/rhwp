//! Fallible copies for consumers of an immutable, request-local operator cache.
use super::{Operator, TextElement};
use crate::{error::Result, import_limits::ImportBudget, object::Object};
fn slots<T>(len: usize, b: &ImportBudget) -> Result<Vec<T>> {
    let bytes = len
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(|| b.reject("operator copy size"))?;
    if bytes > b.limits.stream_bytes {
        return Err(b.reject("operator copy buffer"));
    }
    // Operator collections are owned parser metadata, not private codec
    // scratch. Keep codec_allocation_bytes specific to image/Flate internals.
    b.charge_decoded(bytes)?;
    let mut v = Vec::new();
    v.try_reserve_exact(len)
        .map_err(|_| b.reject("operator copy allocation"))?;
    Ok(v)
}
fn string(s: &str, b: &ImportBudget) -> Result<String> {
    String::from_utf8(b.copy_bytes(s.as_bytes())?).map_err(|_| b.reject("operator string copy"))
}
fn values<T: Copy>(s: &[T], b: &ImportBudget) -> Result<Vec<T>> {
    let mut v = slots(s.len(), b)?;
    v.extend_from_slice(s);
    Ok(v)
}
// Operator keeps boxed names to preserve its <=40-byte storage contract.
// This copier must retain that representation and its request reservations.
#[allow(clippy::box_collection)]
fn name(s: &Option<Box<String>>, b: &ImportBudget) -> Result<Option<Box<String>>> {
    s.as_ref().map(|s| string(s, b).map(Box::new)).transpose()
}
fn objects(s: &[Object], b: &ImportBudget) -> Result<Vec<Object>> {
    let mut v = slots(s.len(), b)?;
    for o in s {
        v.push(crate::object_copy::clone_object(o, Some(b))?);
    }
    Ok(v)
}
pub(crate) fn copy(ops: &[Operator], b: &ImportBudget) -> Result<Vec<Operator>> {
    let mut out = slots(ops.len(), b)?;
    for op in ops {
        b.check()?;
        out.push(match op {
            Operator::Tj { text } => Operator::Tj {
                text: b.copy_bytes(text)?,
            },
            Operator::Quote { text } => Operator::Quote {
                text: b.copy_bytes(text)?,
            },
            Operator::DoubleQuote {
                word_space,
                char_space,
                text,
            } => Operator::DoubleQuote {
                word_space: *word_space,
                char_space: *char_space,
                text: b.copy_bytes(text)?,
            },
            Operator::TJ { array } => {
                let mut v = slots(array.len(), b)?;
                for item in array {
                    v.push(match item {
                        TextElement::String(s) => TextElement::String(b.copy_bytes(s)?),
                        other => other.clone(),
                    });
                }
                Operator::TJ { array: v }
            },
            Operator::Tf { font, size } => Operator::Tf {
                font: string(font, b)?,
                size: *size,
            },
            Operator::SetFillColorSpace { name } => Operator::SetFillColorSpace {
                name: string(name, b)?,
            },
            Operator::SetStrokeColorSpace { name } => Operator::SetStrokeColorSpace {
                name: string(name, b)?,
            },
            Operator::SetFillColor { components } => Operator::SetFillColor {
                components: values(components, b)?,
            },
            Operator::SetStrokeColor { components } => Operator::SetStrokeColor {
                components: values(components, b)?,
            },
            Operator::SetFillColorN {
                components,
                name: n,
            } => Operator::SetFillColorN {
                components: values(components, b)?,
                name: name(n, b)?,
            },
            Operator::SetStrokeColorN {
                components,
                name: n,
            } => Operator::SetStrokeColorN {
                components: values(components, b)?,
                name: name(n, b)?,
            },
            Operator::Do { name } => Operator::Do {
                name: string(name, b)?,
            },
            Operator::SetDash { array, phase } => Operator::SetDash {
                array: values(array, b)?,
                phase: *phase,
            },
            Operator::SetRenderingIntent { intent } => Operator::SetRenderingIntent {
                intent: string(intent, b)?,
            },
            Operator::SetExtGState { dict_name } => Operator::SetExtGState {
                dict_name: string(dict_name, b)?,
            },
            Operator::PaintShading { name } => Operator::PaintShading {
                name: string(name, b)?,
            },
            Operator::InlineImage { dict, data } => Operator::InlineImage {
                dict: Box::new(crate::object_copy::clone_dictionary(dict, Some(b))?),
                data: b.copy_bytes(data)?,
            },
            Operator::BeginMarkedContent { tag } => Operator::BeginMarkedContent {
                tag: string(tag, b)?,
            },
            Operator::BeginMarkedContentDict { tag, properties } => {
                Operator::BeginMarkedContentDict {
                    tag: string(tag, b)?,
                    properties: Box::new(crate::object_copy::clone_object(properties, Some(b))?),
                }
            },
            Operator::Other { name, operands } => Operator::Other {
                name: string(name, b)?,
                operands: Box::new(objects(operands, b)?),
            },
            other => other.clone(), // Remaining variants contain only numeric/boolean fields.
        });
    }
    Ok(out)
}
