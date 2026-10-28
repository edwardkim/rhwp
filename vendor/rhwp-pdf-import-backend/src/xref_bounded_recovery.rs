//! Request-bounded recovery. Object bodies are parsed as units, never regex
//! matches inside strings, comments or binary streams. Ambiguous binary extents
//! are rejected rather than guessing an object boundary.
use crate::{
    error::{Error, Result},
    import_limits::ImportBudget,
    object::{Object, ObjectRef},
    parser::parse_object_with_budget,
    xref::{CrossRefTable, XRefEntry},
};
use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    sync::Arc,
};

type Recovered = (CrossRefTable, Object, Vec<(ObjectRef, Object)>);

fn whitespace(data: &[u8], mut at: usize) -> usize {
    while at < data.len() {
        if data[at].is_ascii_whitespace() || data[at] == 0 {
            at += 1;
        } else if data[at] == b'%' {
            while at < data.len() && !matches!(data[at], b'\r' | b'\n') {
                at += 1;
            }
        } else {
            break;
        }
    }
    at
}
fn delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace() || b"\0()<>[]{}/%".contains(&byte)
}
fn keyword(data: &[u8], at: usize, word: &[u8]) -> bool {
    data.get(at..at + word.len()) == Some(word)
        && data.get(at + word.len()).is_none_or(|b| delimiter(*b))
}
fn number(data: &[u8], at: usize) -> Option<(usize, u32)> {
    let mut end = at;
    let mut value = 0u32;
    while data.get(end).is_some_and(u8::is_ascii_digit) {
        value = value
            .checked_mul(10)?
            .checked_add(u32::from(data[end] - b'0'))?;
        end += 1;
    }
    (end > at && data.get(end).is_none_or(|b| delimiter(*b))).then_some((end, value))
}

pub(crate) fn recover<R: Read + Seek>(
    reader: &mut R,
    budget: &Arc<ImportBudget>,
) -> Result<Recovered> {
    budget.check()?;
    let size = usize::try_from(reader.seek(SeekFrom::End(0))?)
        .map_err(|_| budget.reject("xref recovery input"))?;
    if size > budget.limits.input_bytes {
        return Err(budget.reject("xref recovery input"));
    }
    budget.charge_decoded(size)?;
    budget.charge_codec_work(size)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| budget.reject("xref recovery allocation"))?;
    reader.seek(SeekFrom::Start(0))?;
    reader.take(size as u64).read_to_end(&mut bytes)?;
    if bytes.len() != size {
        return Err(Error::InvalidXref);
    }
    let mut xref = CrossRefTable::new();
    let mut trailer = HashMap::new();
    let mut catalog = None;
    let mut at = 0;
    while at < bytes.len() {
        budget.check()?;
        at = whitespace(&bytes, at);
        if at == bytes.len() {
            break;
        }
        if keyword(&bytes, at, b"trailer") {
            let start = at + 7;
            let parsed = parse_object_with_budget(&bytes[start..], Some(budget));
            budget.check()?;
            let (rest, object) = parsed.map_err(|_| Error::InvalidXref)?;
            let Object::Dictionary(dict) = object else {
                return Err(Error::InvalidXref);
            };
            // Latest incremental trailer overrides supplied keys. Keep /Info,
            // /ID and /Encrypt: recovery must not erase protection or metadata.
            trailer.extend(dict);
            at = bytes.len() - rest.len();
            continue;
        }
        if keyword(&bytes, at, b"xref") {
            // Traditional free entries must not resurrect deleted objects.
            // Parse only this lexical section, not a corrupt /Prev chain.
            at += 4;
            loop {
                at = whitespace(&bytes, at);
                if !bytes.get(at).is_some_and(u8::is_ascii_digit) {
                    break;
                }
                let (end, first) = number(&bytes, at).ok_or(Error::InvalidXref)?;
                let (end, count) =
                    number(&bytes, whitespace(&bytes, end)).ok_or(Error::InvalidXref)?;
                budget.charge_entries(count as usize)?;
                at = end;
                for index in 0..count {
                    at = whitespace(&bytes, at);
                    // Offsets are u64 in the format; lexical scanning does not
                    // accept them as trusted positions in the recovered table.
                    while bytes.get(at).is_some_and(u8::is_ascii_digit) {
                        at += 1;
                    }
                    let (end, generation) =
                        number(&bytes, whitespace(&bytes, at)).ok_or(Error::InvalidXref)?;
                    at = whitespace(&bytes, end);
                    let id = first.checked_add(index).ok_or(Error::InvalidXref)?;
                    match bytes.get(at) {
                        Some(b'f') => xref.add_entry(
                            id,
                            XRefEntry::free(
                                0,
                                u16::try_from(generation).map_err(|_| Error::InvalidXref)?,
                            ),
                        ),
                        Some(b'n') => {},
                        _ => return Err(Error::InvalidXref),
                    }
                    at += 1;
                }
            }
            continue;
        }
        if let Some((first_end, id)) = number(&bytes, at) {
            if let Some((second_end, generation)) = number(&bytes, whitespace(&bytes, first_end)) {
                let marker = whitespace(&bytes, second_end);
                if keyword(&bytes, marker, b"obj") {
                    let start = whitespace(&bytes, marker + 3);
                    let parsed = parse_object_with_budget(&bytes[start..], Some(budget));
                    budget.check()?;
                    let (rest, object) = parsed.map_err(|_| Error::InvalidXref)?;
                    let end = whitespace(&bytes, bytes.len() - rest.len());
                    if !keyword(&bytes, end, b"endobj") {
                        return Err(Error::InvalidXref);
                    }
                    if let Object::Stream { dict, data } = &object {
                        // No reliable xref means an unresolved forward /Length
                        // cannot distinguish binary fake endstream/endobj tokens.
                        // Accept direct lengths and already located scalar lengths.
                        let length = match dict.get("Length") {
                            Some(Object::Integer(n)) => usize::try_from(*n).ok(),
                            _ => None,
                        };
                        if length != Some(data.len()) {
                            return Err(Error::Decode("ambiguous recovery stream Length".into()));
                        }
                        if dict.get("Type").and_then(Object::as_name) == Some("XRef") {
                            // Keep the stream trailer dictionary. Compressed
                            // object recovery still requires its valid xref data.
                            trailer.extend(dict.clone());
                        }
                    }
                    budget.charge_entries(1)?;
                    xref.entries
                        .try_reserve(1)
                        .map_err(|_| budget.reject("xref recovery entries"))?;
                    let reference = ObjectRef::new(
                        id,
                        u16::try_from(generation).map_err(|_| Error::InvalidXref)?,
                    );
                    xref.add_entry(id, XRefEntry::uncompressed(at as u64, reference.gen));
                    if object
                        .as_dict()
                        .and_then(|d| d.get("Type"))
                        .and_then(Object::as_name)
                        == Some("Catalog")
                    {
                        catalog = Some(reference);
                    }
                    at = end + 6;
                    continue;
                }
            }
        }
        // Parse complete literal/hex strings, arrays, dictionaries and names so
        // their contents can never become top-level object headers.
        if matches!(bytes[at], b'(' | b'<' | b'[' | b'/') {
            let parsed = parse_object_with_budget(&bytes[at..], Some(budget));
            budget.check()?;
            let (rest, _) = parsed.map_err(|_| Error::InvalidXref)?;
            at = bytes.len() - rest.len();
        } else {
            at += 1;
            while at < bytes.len() && !delimiter(bytes[at]) {
                at += 1;
            }
        }
    }
    if xref.is_empty() {
        return Err(Error::InvalidXref);
    }
    if !trailer.contains_key("Root") {
        trailer.insert("Root".into(), Object::Reference(catalog.ok_or(Error::InvalidXref)?));
    }
    trailer.remove("Prev");
    trailer.remove("XRefStm");
    trailer.insert(
        "Size".into(),
        Object::Integer(i64::from(xref.all_object_numbers().max().unwrap_or(0)) + 1),
    );
    budget.check()?;
    Ok((xref, Object::Dictionary(trailer), Vec::new()))
}
