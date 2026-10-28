//! Fallible, request-accounted copies of owned PDF object payloads.
//!
//! Stream bytes and their request context are shared, as in `Object::clone`.
//! Accounting covers requested collection slots and owned strings, not hash
//! table bucket rounding, allocator headers, or process-wide live memory.

use crate::error::Result;
use crate::import_limits::ImportBudget;
use crate::object::Object;
use std::collections::HashMap;

type Dictionary = HashMap<String, Object>;

pub(crate) fn clone_object(value: &Object, budget: Option<&ImportBudget>) -> Result<Object> {
    let Some(budget) = budget else {
        return Ok(value.clone());
    };
    budget.check()?;
    let bytes = object_bytes(value, budget, 0)?;
    // Object metadata is not a decoded stream buffer. Keep stream_bytes for
    // stream/text output contracts; the complete owned tree is charged against
    // the remaining request budget before any copy allocation.
    budget.charge_decoded(bytes)?;
    copy_object(value, budget)
}

pub(crate) fn clone_dictionary(
    value: &Dictionary,
    budget: Option<&ImportBudget>,
) -> Result<Dictionary> {
    let Some(budget) = budget else {
        return Ok(value.clone());
    };
    budget.check()?;
    let bytes = dictionary_bytes(value, budget, 0)?;
    // Object metadata is not a decoded stream buffer. Keep stream_bytes for
    // stream/text output contracts; the complete owned tree is charged against
    // the remaining request budget before any copy allocation.
    budget.charge_decoded(bytes)?;
    copy_dictionary(value, budget)
}

fn add(a: usize, b: usize, budget: &ImportBudget) -> Result<usize> {
    a.checked_add(b)
        .ok_or_else(|| budget.reject("object copy size overflow"))
}

fn slots<T>(len: usize, budget: &ImportBudget) -> Result<usize> {
    len.checked_mul(std::mem::size_of::<T>())
        .ok_or_else(|| budget.reject("object copy size overflow"))
}

fn object_bytes(value: &Object, budget: &ImportBudget, depth: usize) -> Result<usize> {
    if depth > budget.limits.nesting {
        return Err(budget.reject("object copy nesting"));
    }
    match value {
        Object::String(s) => Ok(s.len()),
        Object::Name(s) => Ok(s.len()),
        Object::Array(values) => {
            let mut bytes = slots::<Object>(values.len(), budget)?;
            for value in values {
                bytes = add(bytes, object_bytes(value, budget, depth + 1)?, budget)?;
            }
            Ok(bytes)
        },
        Object::Dictionary(dict) | Object::Stream { dict, .. } => {
            dictionary_bytes(dict, budget, depth)
        },
        _ => Ok(0),
    }
}

fn dictionary_bytes(dict: &Dictionary, budget: &ImportBudget, depth: usize) -> Result<usize> {
    let mut bytes = slots::<(String, Object)>(dict.len(), budget)?;
    for (key, value) in dict {
        bytes = add(bytes, key.len(), budget)?;
        bytes = add(bytes, object_bytes(value, budget, depth + 1)?, budget)?;
    }
    Ok(bytes)
}

fn copy_string(value: &str, budget: &ImportBudget) -> Result<String> {
    let mut copy = String::new();
    copy.try_reserve_exact(value.len())
        .map_err(|_| budget.reject("object copy allocation"))?;
    copy.push_str(value);
    Ok(copy)
}

fn copy_dictionary(value: &Dictionary, budget: &ImportBudget) -> Result<Dictionary> {
    let mut copy = Dictionary::new();
    copy.try_reserve(value.len())
        .map_err(|_| budget.reject("object copy allocation"))?;
    for (key, value) in value {
        copy.insert(copy_string(key, budget)?, copy_object(value, budget)?);
    }
    Ok(copy)
}

fn copy_object(value: &Object, budget: &ImportBudget) -> Result<Object> {
    Ok(match value {
        Object::String(bytes) => {
            let mut copy = Vec::new();
            copy.try_reserve_exact(bytes.len())
                .map_err(|_| budget.reject("object copy allocation"))?;
            copy.extend_from_slice(bytes);
            Object::String(copy)
        },
        Object::Name(name) => Object::Name(copy_string(name, budget)?),
        Object::Array(values) => {
            let mut copy = Vec::new();
            copy.try_reserve_exact(values.len())
                .map_err(|_| budget.reject("object copy allocation"))?;
            for value in values {
                copy.push(copy_object(value, budget)?);
            }
            Object::Array(copy)
        },
        Object::Dictionary(dict) => Object::Dictionary(copy_dictionary(dict, budget)?),
        Object::Stream { dict, data } => Object::Stream {
            dict: copy_dictionary(dict, budget)?,
            data: data.clone(),
        },
        _ => value.clone(),
    })
}
