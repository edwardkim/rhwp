//! Request-bounded stream parameters (ISO 32000-1 7.3.10, 7.4.1 Table 5).
//! Resolve indirect dictionaries before decoding, and keep each DecodeParms
//! array entry aligned with its own filter instead of choosing the first dict.
use crate::{
    decoders::{decode_stream_with_budget, DecodeParams},
    document::PdfDocument,
    error::{Error, Result},
    import_limits::{ImportBudget, StreamData},
    object::Object,
};
use std::{borrow::Cow, collections::HashMap};

fn invalid() -> Error {
    Error::Decode("invalid or unresolved stream filter parameters".into())
}

fn resolve(
    doc: &PdfDocument,
    value: &Object,
    budget: &ImportBudget,
    depth: usize,
) -> Result<Object> {
    budget.check()?;
    if depth >= budget.limits.reference_depth {
        return Err(budget.reject("stream parameter reference depth"));
    }
    match value {
        Object::Reference(reference) => {
            resolve(doc, &doc.load_object(*reference)?, budget, depth + 1)
        },
        Object::Array(values) => {
            if values.len() > budget.limits.filters {
                return Err(budget.reject("filter parameter array depth"));
            }
            budget.charge_entries(values.len())?;
            budget.charge_decoded(values.len() * std::mem::size_of::<Object>())?;
            let mut resolved = Vec::new();
            resolved
                .try_reserve_exact(values.len())
                .map_err(|_| budget.reject("stream parameter array"))?;
            for value in values {
                resolved.push(resolve(doc, value, budget, depth + 1)?);
            }
            Ok(Object::Array(resolved))
        },
        Object::Dictionary(dict) => {
            let mut result = crate::object_copy::clone_dictionary(dict, Some(budget))?;
            // Only these entries are consumed by the bounded generic predictor.
            // Do not chase unrelated streams such as JBIG2Globals here.
            for key in ["Predictor", "Columns", "Colors", "BitsPerComponent"] {
                if let Some(value) = dict.get(key) {
                    result.insert(key.into(), resolve(doc, value, budget, depth + 1)?);
                }
            }
            Ok(Object::Dictionary(result))
        },
        _ => crate::object_copy::clone_object(value, Some(budget)),
    }
}

pub(crate) fn decode(doc: &PdfDocument, stream: &Object, budget: &ImportBudget) -> Result<Vec<u8>> {
    let Object::Stream { dict, data } = stream else {
        return stream.decode_stream_data_with_budget(budget);
    };
    // Payload bytes stay shared. Account only the two entries actually consumed
    // by this decoder, not a second compressed image-sized allocation.
    budget.charge_entries(2)?;
    budget.charge_decoded(2 * (std::mem::size_of::<Object>() + std::mem::size_of::<String>()))?;
    let mut parameters = HashMap::new();
    parameters
        .try_reserve(2)
        .map_err(|_| budget.reject("stream parameter dictionary"))?;
    for key in ["Filter", "DecodeParms"] {
        if key == "DecodeParms" && matches!(parameters.get("Filter"), None | Some(Object::Null)) {
            continue; // No filter consumes DecodeParms (including an empty dictionary).
        }
        if let Some(value) = dict.get(key) {
            parameters.insert(key.into(), resolve(doc, value, budget, 0)?);
        }
    }
    decode_resolved(&parameters, data, budget)
}

fn predictor(value: Option<&Object>) -> Result<Option<DecodeParams>> {
    let dict = match value {
        None | Some(Object::Null) => return Ok(None),
        Some(Object::Dictionary(dict)) => dict,
        _ => return Err(invalid()),
    };
    let number = |key: &str, default: usize| -> Result<usize> {
        match dict.get(key) {
            None | Some(Object::Null) => Ok(default),
            Some(Object::Integer(n)) => usize::try_from(*n)
                .ok()
                .filter(|n| *n != 0)
                .ok_or_else(invalid),
            _ => Err(invalid()),
        }
    };
    Ok(Some(DecodeParams {
        predictor: i64::try_from(number("Predictor", 1)?).map_err(|_| invalid())?,
        columns: number("Columns", 1)?,
        colors: number("Colors", 1)?,
        bits_per_component: number("BitsPerComponent", 8)?,
    }))
}

pub(crate) fn decode_resolved(
    dict: &HashMap<String, Object>,
    data: &StreamData,
    budget: &ImportBudget,
) -> Result<Vec<u8>> {
    budget.check()?;
    let filters: &[Object] = match dict.get("Filter") {
        None | Some(Object::Null) => &[],
        Some(value @ Object::Name(_)) => std::slice::from_ref(value),
        Some(Object::Array(values)) => values,
        _ => return Err(invalid()),
    };
    if filters.len() > budget.limits.filters {
        return Err(budget.reject("filter chain depth"));
    }
    if filters.is_empty() {
        return decode_stream_with_budget(data, &[], None, budget);
    }
    let params = dict.get("DecodeParms");
    if let Some(Object::Array(values)) = params {
        if values.len() != filters.len() {
            return Err(invalid());
        }
    } else if !matches!(params, None | Some(Object::Null))
        && (filters.len() != 1 || !matches!(params, Some(Object::Dictionary(_))))
    {
        return Err(invalid());
    }
    let mut current = Cow::Borrowed(data.as_ref());
    for (index, filter) in filters.iter().enumerate() {
        let Object::Name(name) = filter else {
            return Err(invalid());
        };
        let value = match params {
            Some(Object::Array(values)) => Some(&values[index]),
            other => other,
        };
        let parameters = if matches!(name.as_str(), "FlateDecode" | "Fl" | "LZWDecode" | "LZW") {
            predictor(value)?
        } else {
            if !matches!(value, None | Some(Object::Null | Object::Dictionary(_))) {
                return Err(invalid());
            }
            None
        };
        current = Cow::Owned(decode_stream_with_budget(
            &current,
            std::slice::from_ref(name),
            parameters.as_ref(),
            budget,
        )?);
    }
    Ok(current.into_owned())
}
