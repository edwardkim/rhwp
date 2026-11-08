//! Document Security Store reader (#235, feature plan §3.3 / §5.2,
//! TODO #9).
//!
//! Parses the Catalog `/DSS` dictionary (ISO 32000-2:2020 §12.8.4.3 /
//! ETSI EN 319 142-1): `/Certs`, `/CRLs`, `/OCSPs` are arrays of
//! indirect references to stream objects whose decoded bytes are the
//! raw DER cert / CRL / OCSP response; `/VRI` maps each signature's
//! uppercase-hex SHA-1(`/Contents`) to a per-signature validation dict.
//!
//! Read-side only — touches no signed bytes; the pure dictionary→model
//! step is split out so it is fully unit-testable against synthetic
//! objects without a real PDF (the EU DSS validator is the *writer*'s
//! conformance oracle; the reader is proven here + by a checked-in
//! reference sample per the feature plan §5.5).

use super::{DocumentSecurityStore, VriEntry};
use crate::document::PdfDocument;
use crate::error::Result;
use crate::object::{Object, ObjectRef};

/// Resolves an indirect reference to its target object (returns `None`
/// for a dangling reference). A direct (non-reference) object passes
/// through unchanged at the call sites.
type Resolver<'a> = dyn Fn(ObjectRef) -> Option<Object> + 'a;

/// Follow one level of indirection if `o` is a reference, else clone.
fn deref(o: &Object, resolve: &Resolver) -> Option<Object> {
    match o.as_reference() {
        Some(r) => resolve(r),
        None => Some(o.clone()),
    }
}

/// Decode an array of (usually indirect) stream objects to their raw
/// DER bodies. Unreadable / non-stream entries are skipped (a partial
/// DSS is still useful; never panic on a malformed store).
fn stream_array(arr_owner: Option<&Object>, resolve: &Resolver) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let Some(arr) = arr_owner.and_then(|o| o.as_array()) else {
        return out;
    };
    for item in arr {
        if let Some(obj) = deref(item, resolve) {
            if let Ok(bytes) = obj.decode_stream_data() {
                if !bytes.is_empty() {
                    out.push(bytes);
                }
            }
        }
    }
    out
}

/// Parse a resolved `/DSS` dictionary object into a
/// [`DocumentSecurityStore`]. `resolve` follows indirect references to
/// stream / sub-dictionary objects.
pub fn parse_dss(dss: &Object, resolve: &Resolver) -> DocumentSecurityStore {
    let mut store = DocumentSecurityStore::default();
    let Some(d) = dss.as_dict() else {
        return store;
    };

    store.certificates = stream_array(d.get("Certs"), resolve);
    store.crls = stream_array(d.get("CRLs"), resolve);
    store.ocsp_responses = stream_array(d.get("OCSPs"), resolve);

    if let Some(vri_obj) = d.get("VRI").and_then(|o| deref(o, resolve)) {
        if let Some(vri_dict) = vri_obj.as_dict() {
            for (key, val) in vri_dict {
                // The /Type key (if present) is not a VRI entry.
                if key == "Type" {
                    continue;
                }
                let Some(entry_obj) = deref(val, resolve) else {
                    continue;
                };
                let Some(ed) = entry_obj.as_dict() else {
                    continue;
                };
                store.vri.push(VriEntry {
                    signature_digest: key.clone(),
                    certificates: stream_array(ed.get("Cert"), resolve),
                    crls: stream_array(ed.get("CRL"), resolve),
                    ocsp_responses: stream_array(ed.get("OCSP"), resolve),
                    timestamp: ed
                        .get("TU")
                        .and_then(|o| o.as_string())
                        .map(|b| String::from_utf8_lossy(b).into_owned()),
                });
            }
        }
    }
    store
}

/// Read the Document Security Store from a parsed PDF, if present.
///
/// `Ok(None)` when the Catalog has no `/DSS` (the common, non-LTV case).
///
/// # Errors
/// [`crate::error::Error`] if the Catalog cannot be loaded.
pub fn read_dss(doc: &PdfDocument) -> Result<Option<DocumentSecurityStore>> {
    let catalog = doc.catalog()?;
    let Some(cat) = catalog.as_dict() else {
        return Ok(None);
    };
    let Some(dss_ref) = cat.get("DSS") else {
        return Ok(None);
    };
    let resolve = |r: ObjectRef| doc.load_object(r).ok();
    let Some(dss) = deref(dss_ref, &resolve) else {
        return Ok(None);
    };
    let store = parse_dss(&dss, &resolve);
    if store.is_empty() {
        Ok(None)
    } else {
        Ok(Some(store))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stream(body: &[u8]) -> Object {
        Object::Stream {
            dict: HashMap::new(),
            data: body.to_vec().into(),
        }
    }
    fn refr(id: u32) -> Object {
        Object::Reference(ObjectRef { id, gen: 0 })
    }
    fn dict(pairs: &[(&str, Object)]) -> Object {
        Object::Dictionary(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        )
    }

    #[test]
    fn parses_doc_level_material_via_indirect_streams() {
        // objects: 10=cert DER, 11=crl DER, 12=ocsp DER
        let mut objs: HashMap<u32, Object> = HashMap::new();
        objs.insert(10, stream(b"\x30\x82CERT"));
        objs.insert(11, stream(b"\x30\x82CRL"));
        objs.insert(12, stream(b"\x30\x82OCSP"));
        let resolve = |r: ObjectRef| objs.get(&r.id).cloned();

        let dss = dict(&[
            ("Type", Object::Name("DSS".into())),
            ("Certs", Object::Array(vec![refr(10)])),
            ("CRLs", Object::Array(vec![refr(11)])),
            ("OCSPs", Object::Array(vec![refr(12)])),
        ]);
        let s = parse_dss(&dss, &resolve);
        assert_eq!(s.certificates, vec![b"\x30\x82CERT".to_vec()]);
        assert_eq!(s.crls, vec![b"\x30\x82CRL".to_vec()]);
        assert_eq!(s.ocsp_responses, vec![b"\x30\x82OCSP".to_vec()]);
        assert!(s.vri.is_empty());
        assert!(!s.is_empty());
    }

    #[test]
    fn parses_vri_keyed_by_signature_digest() {
        let mut objs: HashMap<u32, Object> = HashMap::new();
        objs.insert(20, stream(b"VRICERT"));
        let resolve = |r: ObjectRef| objs.get(&r.id).cloned();

        let vri = dict(&[
            ("Type", Object::Name("VRI".into())), // must be skipped as a key
            (
                "ABCDEF0123456789",
                dict(&[
                    ("Type", Object::Name("VRI".into())),
                    ("Cert", Object::Array(vec![refr(20)])),
                    ("TU", Object::String(b"D:20260516120000Z".to_vec())),
                ]),
            ),
        ]);
        let dss = dict(&[("Type", Object::Name("DSS".into())), ("VRI", vri)]);
        let s = parse_dss(&dss, &resolve);
        assert_eq!(s.vri.len(), 1, "the /Type key must not become a VRI entry");
        let e = s.vri_for("ABCDEF0123456789").expect("VRI entry by key");
        assert_eq!(e.certificates, vec![b"VRICERT".to_vec()]);
        assert_eq!(e.timestamp.as_deref(), Some("D:20260516120000Z"));
    }

    #[test]
    fn dangling_refs_and_non_streams_are_skipped_not_panic() {
        let resolve = |_r: ObjectRef| None; // every ref dangles
        let dss = dict(&[
            ("Certs", Object::Array(vec![refr(99), Object::Integer(5)])),
            ("OCSPs", Object::Array(vec![refr(98)])),
        ]);
        let s = parse_dss(&dss, &resolve);
        assert!(s.is_empty());
    }

    #[test]
    fn non_dict_dss_is_empty() {
        let resolve = |_r: ObjectRef| None;
        assert!(parse_dss(&Object::Null, &resolve).is_empty());
        assert!(parse_dss(&Object::Integer(7), &resolve).is_empty());
    }

    #[test]
    fn direct_inline_streams_also_work() {
        // /Certs element given directly (not via indirect ref).
        let resolve = |_r: ObjectRef| None;
        let dss = dict(&[("Certs", Object::Array(vec![stream(b"INLINE")]))]);
        let s = parse_dss(&dss, &resolve);
        assert_eq!(s.certificates, vec![b"INLINE".to_vec()]);
    }
}
