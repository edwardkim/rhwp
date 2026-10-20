//! PAdES-B-LT Document Security Store writer (#235, feature plan §4.2,
//! TODO #8) — the **2nd incremental update**.
//!
//! B-LT = B-T + a DSS in the Catalog, added by a *separate* incremental
//! update so the B-T signature's `/ByteRange` is **byte-identical**
//! afterwards (the signature still verifies; a validator reports a
//! permitted post-signing increment — exactly how EU DSS / iText /
//! pyHanko add B-LT). We therefore **never** touch the signature
//! byte-range math; we only *append*.
//!
//! Append layout (ISO 32000-1:2008 §7.5.6 incremental update;
//! ISO 32000-2:2020 §12.8.4.3 DSS/VRI):
//!
//! ```text
//! <original PDF bytes, verbatim>
//! N   0 obj  << /Length … >> stream <DER cert>  endstream endobj   (one per cert/CRL/OCSP)
//! …
//! D   0 obj  << /Type /DSS /Certs […] /CRLs […] /OCSPs […] /VRI <<…>> >> endobj
//! R   g obj  << …old Catalog… /DSS D 0 R >> endobj    (Catalog overridden)
//! xref  (subsections: the Catalog + the new objects, exact offsets)
//! trailer << /Size … /Prev <old startxref> /Root R g R >>
//! startxref <offset of this xref> %%EOF
//! ```
//!
//! Driven from a parsed [`PdfDocument`] (real Catalog ref/gen + the old
//! Catalog dict) rather than a tail regex — the unforgiving part is the
//! xref offset math, validated by the integrity invariants in the tests
//! (I2: the pre-DSS file is a strict byte prefix of the post-DSS file;
//! I3: re-parsing resolves the *new* Catalog and its `/DSS`). End-to-end
//! CMS/B-LT conformance is gated by the EU-DSS validator (feature plan
//! §5.5); the *append integrity* proven here is independent of it.

use super::RevocationMaterial;
use crate::document::PdfDocument;
use crate::error::{Error, Result};
use crate::object::{Object, ObjectRef};
use crate::redaction::serialize::serialize_object;
use std::collections::HashMap;

/// Find the byte value of the last `startxref` in the file (for `/Prev`).
/// Scans the tail (xref pointer is always near EOF).
fn scan_last_startxref(pdf: &[u8]) -> Option<u64> {
    let tail = &pdf[pdf.len().saturating_sub(2048)..];
    let s = String::from_utf8_lossy(tail);
    let idx = s.rfind("startxref")?;
    s[idx + "startxref".len()..]
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// One classic xref entry: `nnnnnnnnnn ggggg n \r\n` (exactly 20 bytes).
fn xref_in_use(offset: usize, gen: u16) -> String {
    format!("{offset:010} {gen:05} n \r\n")
}

/// Append a Document Security Store as a spec-correct second incremental
/// update, returning the original bytes followed by the DSS update.
///
/// `material` is the document-level validation set; `vri` maps each
/// signature's uppercase-hex `SHA-1(/Contents)` (see
/// [`super::vri_key`]) to its `/VRI` entry. To keep the object graph
/// small, `/VRI` entries reference the shared document-level streams
/// (indirect references may be shared — a valid, common DSS layout).
///
/// # Errors
/// [`Error::InvalidPdf`] if the document has no resolvable Catalog
/// `/Root` reference or trailer `/Size` (a fail-closed precondition —
/// never emit a half-written update).
pub fn append_dss(
    pdf: &[u8],
    doc: &PdfDocument,
    material: &RevocationMaterial,
    vri: &[String],
) -> Result<Vec<u8>> {
    let trailer = doc.trailer();
    let tdict = trailer
        .as_dict()
        .ok_or_else(|| Error::InvalidPdf("DSS: trailer is not a dictionary".into()))?;
    let root_ref = tdict
        .get("Root")
        .and_then(|o| o.as_reference())
        .ok_or_else(|| Error::InvalidPdf("DSS: trailer has no /Root reference".into()))?;
    let mut next_id = tdict
        .get("Size")
        .and_then(|o| o.as_integer())
        .ok_or_else(|| Error::InvalidPdf("DSS: trailer has no /Size".into()))?
        as u32;
    let prev_startxref = scan_last_startxref(pdf)
        .ok_or_else(|| Error::InvalidPdf("DSS: cannot find existing startxref".into()))?;

    let old_catalog = doc.load_object(root_ref)?;
    let mut catalog = old_catalog
        .as_dict()
        .ok_or_else(|| Error::InvalidPdf("DSS: Catalog is not a dictionary".into()))?
        .clone();

    // Allocate ids: one stream per material item, then the DSS dict,
    // then the (re-emitted) Catalog reuses root_ref.id.
    let mut alloc = || {
        let id = next_id;
        next_id += 1;
        id
    };

    // (object_id, raw DER body) for every cert/CRL/OCSP stream.
    let mut streams: Vec<(u32, Vec<u8>)> = Vec::new();
    let id_array =
        |items: &[Vec<u8>], streams: &mut Vec<(u32, Vec<u8>)>, a: &mut dyn FnMut() -> u32| {
            let mut refs = Vec::with_capacity(items.len());
            for it in items {
                let id = a();
                streams.push((id, it.clone()));
                refs.push(Object::Reference(ObjectRef { id, gen: 0 }));
            }
            Object::Array(refs)
        };
    let certs = id_array(&material.certificates, &mut streams, &mut alloc);
    let crls = id_array(&material.crls, &mut streams, &mut alloc);
    let ocsps = id_array(&material.ocsp_responses, &mut streams, &mut alloc);

    // /VRI: each signature key → an entry referencing the shared
    // document-level stream arrays (valid: refs may be shared).
    let mut vri_dict: HashMap<String, Object> = HashMap::new();
    vri_dict.insert("Type".into(), Object::Name("VRI".into()));
    for key in vri {
        let mut entry: HashMap<String, Object> = HashMap::new();
        entry.insert("Type".into(), Object::Name("VRI".into()));
        if let Object::Array(a) = &certs {
            if !a.is_empty() {
                entry.insert("Cert".into(), certs.clone());
            }
        }
        if let Object::Array(a) = &crls {
            if !a.is_empty() {
                entry.insert("CRL".into(), crls.clone());
            }
        }
        if let Object::Array(a) = &ocsps {
            if !a.is_empty() {
                entry.insert("OCSP".into(), ocsps.clone());
            }
        }
        vri_dict.insert(key.clone(), Object::Dictionary(entry));
    }

    let dss_id = alloc();
    let mut dss: HashMap<String, Object> = HashMap::new();
    dss.insert("Type".into(), Object::Name("DSS".into()));
    if let Object::Array(a) = &certs {
        if !a.is_empty() {
            dss.insert("Certs".into(), certs);
        }
    }
    if let Object::Array(a) = &crls {
        if !a.is_empty() {
            dss.insert("CRLs".into(), crls);
        }
    }
    if let Object::Array(a) = &ocsps {
        if !a.is_empty() {
            dss.insert("OCSPs".into(), ocsps);
        }
    }
    if vri_dict.len() > 1 {
        dss.insert("VRI".into(), Object::Dictionary(vri_dict));
    }

    catalog.insert("DSS".into(), Object::Reference(ObjectRef { id: dss_id, gen: 0 }));

    // ── Emit the incremental update ──────────────────────────────────
    let mut out = Vec::with_capacity(pdf.len() + 2048);
    out.extend_from_slice(pdf);
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }

    // (id, gen, byte offset) for the xref.
    let mut written: Vec<(u32, u16, usize)> = Vec::new();

    for (id, body) in &streams {
        let off = out.len();
        out.extend_from_slice(
            format!("{id} 0 obj\n<< /Length {} >>\nstream\n", body.len()).as_bytes(),
        );
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendstream\nendobj\n");
        written.push((*id, 0, off));
    }

    let off = out.len();
    out.extend_from_slice(format!("{dss_id} 0 obj\n").as_bytes());
    serialize_object(&mut out, &Object::Dictionary(dss));
    out.extend_from_slice(b"\nendobj\n");
    written.push((dss_id, 0, off));

    let off = out.len();
    out.extend_from_slice(format!("{} {} obj\n", root_ref.id, root_ref.gen).as_bytes());
    serialize_object(&mut out, &Object::Dictionary(catalog));
    out.extend_from_slice(b"\nendobj\n");
    written.push((root_ref.id, root_ref.gen, off));

    // ── xref (subsections: contiguous id runs) ───────────────────────
    written.sort_by_key(|(id, _, _)| *id);
    let xref_offset = out.len();
    out.extend_from_slice(b"xref\n");
    let mut i = 0;
    while i < written.len() {
        let start = written[i].0;
        let mut j = i;
        while j + 1 < written.len() && written[j + 1].0 == written[j].0 + 1 {
            j += 1;
        }
        out.extend_from_slice(format!("{} {}\n", start, j - i + 1).as_bytes());
        for &(_, gen, offset) in &written[i..=j] {
            out.extend_from_slice(xref_in_use(offset, gen).as_bytes());
        }
        i = j + 1;
    }

    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Prev {} /Root {} {} R >>\n",
            next_id, prev_startxref, root_ref.id, root_ref.gen
        )
        .as_bytes(),
    );
    out.extend_from_slice(format!("startxref\n{xref_offset}\n%%EOF\n").as_bytes());
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal valid single-page PDF (catalog→pages→page) the
    /// tolerant parser accepts.
    fn minimal_pdf() -> Vec<u8> {
        let mut p = Vec::new();
        p.extend_from_slice(b"%PDF-1.7\n");
        let mut offs = [0usize; 4];
        offs[1] = p.len();
        p.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        offs[2] = p.len();
        p.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        offs[3] = p.len();
        p.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n",
        );
        let xref = p.len();
        p.extend_from_slice(b"xref\n0 4\n0000000000 65535 f \n");
        for o in offs.iter().skip(1) {
            p.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        p.extend_from_slice(
            format!("trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
        );
        p
    }

    #[test]
    fn i2_pre_is_strict_byte_prefix_of_post() {
        let pre = minimal_pdf();
        let doc = PdfDocument::from_bytes(pre.clone()).expect("parse minimal pdf");
        let material = RevocationMaterial {
            certificates: vec![b"\x30\x03\x02\x01\x2A".to_vec()],
            ..RevocationMaterial::default()
        };
        let post = append_dss(&pre, &doc, &material, &["ABCD".to_string()]).expect("append dss");
        // I2: the original bytes are untouched (append-only) — the
        // B-T/B-B signature byte range would still verify.
        assert!(post.len() > pre.len());
        assert_eq!(&post[..pre.len()], &pre[..], "pre must be a byte prefix");
    }

    #[test]
    fn i3_reparse_resolves_new_catalog_with_dss() {
        let pre = minimal_pdf();
        let doc = PdfDocument::from_bytes(pre.clone()).unwrap();
        let material = RevocationMaterial {
            certificates: vec![b"CERTA".to_vec(), b"CERTB".to_vec()],
            ocsp_responses: vec![b"OCSP1".to_vec()],
            ..RevocationMaterial::default()
        };
        let post = append_dss(&pre, &doc, &material, &["DEADBEEF".to_string()]).unwrap();
        // I3: re-parsing the updated file (most-recent xref wins)
        // resolves the *new* Catalog and read_dss finds the material.
        let doc2 = PdfDocument::from_bytes(post).expect("re-parse post-DSS");
        let dss = super::super::read_dss(&doc2)
            .expect("read_dss ok")
            .expect("DSS present after append");
        assert_eq!(dss.certificates, vec![b"CERTA".to_vec(), b"CERTB".to_vec()]);
        assert_eq!(dss.ocsp_responses, vec![b"OCSP1".to_vec()]);
        assert!(dss.crls.is_empty());
        assert_eq!(
            dss.vri_for("DEADBEEF").map(|e| e.certificates.len()),
            Some(2),
            "VRI entry present and references the shared cert streams"
        );
    }

    #[test]
    fn empty_material_still_appends_a_valid_dss() {
        let pre = minimal_pdf();
        let doc = PdfDocument::from_bytes(pre.clone()).unwrap();
        let post = append_dss(&pre, &doc, &RevocationMaterial::default(), &[]).unwrap();
        assert_eq!(&post[..pre.len()], &pre[..]);
        let doc2 = PdfDocument::from_bytes(post).unwrap();
        // No material → read_dss reports None (empty store), but the
        // Catalog still resolves and the file re-parses (no corruption).
        assert!(super::super::read_dss(&doc2).is_ok());
    }
}
