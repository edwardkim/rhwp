//! CMS / PKCS#7 SignedData helpers used by the signature surface.
//!
//! A PDF signature dictionary's `/Contents` entry is a DER-encoded
//! PKCS#7 (RFC 2315) / CMS (RFC 5652) `SignedData` blob. To go from
//! the raw bytes to a usable signer certificate we:
//!
//! 1. Decode the outer `ContentInfo`.
//! 2. Decode its inner `SignedData`.
//! 3. Pull the first `CertificateChoices::Certificate` — conventionally
//!    the signer's cert (the PDF spec explicitly calls this out).
//! 4. Re-encode that X.509 certificate as DER.
//!
//! The caller can then feed the result into
//! [`crate::signatures::SigningCredentials::from_der`] for the
//! inspection accessors that Certificate already offers
//! (subject / issuer / serial / validity / is_valid).

use crate::error::{Error, Result};
use cms::cert::CertificateChoices;
use cms::content_info::ContentInfo;
use cms::signed_data::SignedData;
use der::{Decode, Encode};

/// Extract the signer's certificate (as DER-encoded X.509 bytes) from
/// a PDF `/Contents` blob. Returns an error if the bytes aren't valid
/// CMS SignedData or if the SignedData doesn't carry any certificates.
pub fn extract_signer_certificate_der(contents: &[u8]) -> Result<Vec<u8>> {
    let ci = ContentInfo::from_der(contents).map_err(|e| {
        Error::InvalidPdf(format!("signature /Contents is not valid CMS ContentInfo: {e}"))
    })?;

    let sd_bytes = ci
        .content
        .to_der()
        .map_err(|e| Error::InvalidPdf(format!("failed to re-encode ContentInfo content: {e}")))?;
    let sd = SignedData::from_der(&sd_bytes)
        .map_err(|e| Error::InvalidPdf(format!("CMS content is not valid SignedData: {e}")))?;

    let certs = sd
        .certificates
        .ok_or_else(|| Error::InvalidPdf("SignedData has no certificates".into()))?;

    for choice in certs.0.iter() {
        if let CertificateChoices::Certificate(cert) = choice {
            return cert.to_der().map_err(|e| {
                Error::InvalidPdf(format!("failed to re-encode signer certificate: {e}"))
            });
        }
    }

    Err(Error::InvalidPdf(
        "SignedData certificates present but no X.509 Certificate choice".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_cms_bytes() {
        let err = extract_signer_certificate_der(b"not a CMS blob").unwrap_err();
        assert!(matches!(err, Error::InvalidPdf(_)), "got {err:?}");
    }

    #[test]
    fn rejects_empty_bytes() {
        let err = extract_signer_certificate_der(&[]).unwrap_err();
        assert!(matches!(err, Error::InvalidPdf(_)), "got {err:?}");
    }
}
