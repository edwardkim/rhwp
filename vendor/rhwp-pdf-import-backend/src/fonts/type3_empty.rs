//! Proof of a non-painting Type3 font for editable import, ISO 32000-1 §9.6.5.
//! A font name, missing program, or empty FontBBox is never such a proof.
use crate::{
    content::Operator,
    document::PdfDocument,
    error::Result,
    object::{Object, ObjectRef},
};

pub(super) fn proven_empty(dict: &Object, doc: &PdfDocument) -> Result<bool> {
    let Some(budget) = doc.import_budget() else {
        return Ok(false);
    };
    let Some(value) = dict.as_dict().and_then(|d| d.get("CharProcs")) else {
        return Ok(false);
    };
    let value = resolve(doc, value, budget)?;
    let Some(procedures) = value.as_dict().filter(|p| !p.is_empty()) else {
        return Ok(false);
    };
    budget.charge_font_entries(procedures.len())?;
    for value in procedures.values() {
        budget.check()?;
        let object = resolve(doc, value, budget)?;
        if !matches!(&*object, Object::Stream { .. }) {
            return Ok(false);
        }
        let bytes = doc.decode_stream_with_encryption(&object, ObjectRef::new(0, 0))?;
        let mut count = 0usize;
        let mut empty = true;
        crate::content::parser::execute_content_with_budget(&bytes, budget, |op| {
            count += 1;
            empty &= match op {
                Operator::Other { name, operands } => {
                    ((name == "d0" && operands.len() == 2) || (name == "d1" && operands.len() == 6))
                        && operands.iter().all(|v| {
                            v.as_integer().is_some() || v.as_real().is_some_and(|v| v.is_finite())
                        })
                },
                _ => false,
            };
            Ok(())
        })?;
        // Every glyph must consist of its metrics alone. Unknown/mixed paint,
        // nested resources and malformed programs keep their original behavior.
        if !empty || count != 1 {
            return Ok(false);
        }
    }
    Ok(true)
}

fn resolve<'a>(
    doc: &PdfDocument,
    value: &'a Object,
    budget: &crate::import_limits::ImportBudget,
) -> Result<std::borrow::Cow<'a, Object>> {
    let mut value = std::borrow::Cow::Borrowed(value);
    let mut depth = 0;
    while let Some(reference) = value.as_reference() {
        budget.check()?;
        if depth >= budget.limits.reference_depth {
            return Err(budget.reject("Type3 CharProcs reference depth"));
        }
        value = std::borrow::Cow::Owned(doc.load_object(reference)?);
        depth += 1;
    }
    Ok(value)
}
