//! RFC 3161 Time Stamp Authority (TSA) client.
//!
//! Encodes a `TimeStampReq` (the message imprint over bytes the caller
//! wants timestamped) and POSTs it to a TSA HTTP endpoint with
//! `Content-Type: application/timestamp-query`. The response is
//! parsed as a `TimeStampResp`; on PKI status "granted" / "grantedWithMods"
//! the embedded `TimeStampToken` is handed back as a ready-to-use
//! [`crate::signatures::Timestamp`].
//!
//! Gated behind the `tsa-client` feature so binaries / WASM builds
//! that don't need TSA-over-HTTP don't pay the `ureq` compile cost.

#![cfg(all(feature = "signatures", feature = "tsa-client"))]

use crate::error::{Error, Result};
use crate::signatures::timestamp::HashAlgorithm;
use crate::signatures::tsp::{MessageImprint, TimeStampReq, TimeStampResp, TspVersion};
use crate::signatures::Timestamp;
use cms::cert::x509::spki::AlgorithmIdentifier;
use der::asn1::OctetString;
use der::{Any, Decode, Encode};
use std::time::Duration;

/// Configuration for a [`TsaClient`].
#[derive(Debug, Clone)]
pub struct TsaClientConfig {
    /// TSA endpoint URL, e.g. `https://freetsa.org/tsr`.
    pub url: String,
    /// Optional HTTP Basic auth username.
    pub username: Option<String>,
    /// Optional HTTP Basic auth password.
    pub password: Option<String>,
    /// Request timeout. Defaults to 30s when omitted.
    pub timeout: Duration,
    /// Hash algorithm to use for the message imprint.
    pub hash_algorithm: HashAlgorithm,
    /// Whether to include a random 8-byte nonce — strongly recommended
    /// to prevent response replay but some TSAs require it off.
    pub use_nonce: bool,
    /// Whether to ask the TSA to include its cert in the response.
    pub cert_req: bool,
}

impl TsaClientConfig {
    /// Sensible defaults for a URL — SHA-256, nonce on, cert-req on,
    /// 30-second timeout, no auth.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            username: None,
            password: None,
            timeout: Duration::from_secs(30),
            hash_algorithm: HashAlgorithm::Sha256,
            use_nonce: true,
            cert_req: true,
        }
    }
}

/// Client that can POST RFC 3161 TimeStamp requests to a TSA.
#[derive(Debug, Clone)]
pub struct TsaClient {
    config: TsaClientConfig,
}

impl TsaClient {
    /// Build a new client.
    pub fn new(config: TsaClientConfig) -> Self {
        Self { config }
    }

    /// Hash `data` with the configured algorithm and request a
    /// timestamp for the digest.
    pub fn request_timestamp(&self, data: &[u8]) -> Result<Timestamp> {
        let digest = self.digest(data);
        self.request_timestamp_hash(&digest, self.config.hash_algorithm)
    }

    /// Request a timestamp for a pre-computed digest. `hash_algo`
    /// must describe what algorithm produced `hash`.
    pub fn request_timestamp_hash(
        &self,
        hash: &[u8],
        hash_algo: HashAlgorithm,
    ) -> Result<Timestamp> {
        let req_bytes =
            encode_request(hash, hash_algo, self.config.use_nonce, self.config.cert_req)?;
        let resp_bytes = self.post(&req_bytes)?;
        let resp = TimeStampResp::from_der(&resp_bytes).map_err(|e| {
            Error::InvalidPdf(format!("TSA response is not a valid TimeStampResp: {e}"))
        })?;

        use crate::signatures::tsp::PkiStatus;
        // PkiStatus = Accepted(0) | GrantedWithMods(1) | (…rejections).
        // Anything other than the two "your token is here" states is a
        // rejection — surface the failure rather than try to interpret
        // a missing timeStampToken.
        match resp.status.status {
            PkiStatus::Accepted | PkiStatus::GrantedWithMods => {},
            other => {
                let message = resp
                    .status
                    .fail_info
                    .map(|fi| format!("PKI fail-info: {fi:?}"))
                    .unwrap_or_else(|| format!("PKI status: {other:?}"));
                return Err(Error::InvalidPdf(format!("TSA rejected request ({message})")));
            },
        }
        let token = resp.time_stamp_token.ok_or_else(|| {
            Error::InvalidPdf("TSA granted request but returned no timeStampToken".into())
        })?;
        let token_bytes = token
            .to_der()
            .map_err(|e| Error::InvalidPdf(format!("failed to re-encode TSA token: {e}")))?;
        Timestamp::from_der(&token_bytes)
    }

    fn digest(&self, data: &[u8]) -> Vec<u8> {
        super::crypto::hash_with_algorithm(self.config.hash_algorithm, data)
    }

    fn post(&self, body: &[u8]) -> Result<Vec<u8>> {
        use std::io::Read;
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(self.config.timeout))
            .build()
            .new_agent();
        let mut req = agent
            .post(&self.config.url)
            .header("Content-Type", "application/timestamp-query")
            .header("Accept", "application/timestamp-reply");
        if let (Some(u), Some(p)) = (&self.config.username, &self.config.password) {
            use base64::Engine as _;
            let creds =
                base64::engine::general_purpose::STANDARD.encode(format!("{u}:{p}").as_bytes());
            req = req.header("Authorization", &format!("Basic {creds}"));
        }
        let mut resp = req
            .send(body)
            .map_err(|e| Error::Io(std::io::Error::other(format!("TSA HTTP error: {e}"))))?;
        let mut out = Vec::new();
        resp.body_mut()
            .as_reader()
            .read_to_end(&mut out)
            .map_err(|e| Error::Io(std::io::Error::other(format!("TSA read error: {e}"))))?;
        Ok(out)
    }
}

/// Encode a RFC 3161 TimeStampReq as DER bytes.
fn encode_request(
    hash: &[u8],
    hash_algo: HashAlgorithm,
    use_nonce: bool,
    cert_req: bool,
) -> Result<Vec<u8>> {
    let oid = match super::crypto::oid_for_algorithm(hash_algo) {
        Some(o) => o,
        None => {
            return Err(Error::InvalidPdf(
                "cannot encode TimeStampReq with Unknown hash algorithm".into(),
            ));
        },
    };
    let hash_algorithm = AlgorithmIdentifier::<Any> {
        oid,
        parameters: None,
    };
    let hashed_message = OctetString::new(hash)
        .map_err(|e| Error::InvalidPdf(format!("invalid digest bytes: {e}")))?;
    let nonce = if use_nonce {
        Some(random_nonce())
    } else {
        None
    };
    let req = TimeStampReq {
        version: TspVersion::V1,
        message_imprint: MessageImprint {
            hash_algorithm,
            hashed_message,
        },
        req_policy: None,
        nonce,
        cert_req,
        extensions: None,
    };
    req.to_der()
        .map_err(|e| Error::InvalidPdf(format!("failed to encode TimeStampReq: {e}")))
}

fn random_nonce() -> der::asn1::Int {
    // 8-byte cryptographically-random nonce. getrandom is transitively
    // already in the tree via rand-based deps, so no extra dep.
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).unwrap_or_else(|_| {
        // On platforms where getrandom is unavailable (shouldn't
        // happen on our targets), fall back to a time-derived nonce.
        // Timestamps alone don't give crypto security, but they do
        // prevent the trivial replay case.
        // WASM note: if signatures are ever enabled for wasm32, SystemTime::now()
        // here will also need cfg-gating (currently masked because the `signatures`
        // feature is not enabled in the wasm build).
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        bytes.copy_from_slice(&now.to_be_bytes());
    });
    // Make sure the high bit is 0 so we're always a positive INTEGER.
    bytes[0] &= 0x7F;
    // Strip leading zero bytes: DER canonical encoding forbids unnecessary
    // leading zeros on a positive INTEGER. The FIPS-mode der crate enforces
    // this strictly on decode; non-FIPS parsers are more lenient.
    let start = bytes
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(bytes.len() - 1);
    der::asn1::Int::new(&bytes[start..]).expect("positive nonce bytes always fit in Int")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_request_roundtrips() {
        let hash = [0xAB; 32]; // dummy SHA-256 digest
        let bytes = encode_request(&hash, HashAlgorithm::Sha256, true, true).unwrap();
        let req = TimeStampReq::from_der(&bytes).unwrap();
        assert_eq!(req.version, TspVersion::V1);
        assert_eq!(req.message_imprint.hash_algorithm.oid, der::oid::db::rfc5912::ID_SHA_256);
        assert_eq!(req.message_imprint.hashed_message.as_bytes(), hash);
        assert!(req.nonce.is_some());
        assert!(req.cert_req);
    }

    #[test]
    fn encode_rejects_unknown_hash() {
        let err = encode_request(&[0; 32], HashAlgorithm::Unknown, false, false).unwrap_err();
        assert!(matches!(err, Error::InvalidPdf(_)), "expected InvalidPdf, got {err:?}");
    }

    #[test]
    fn config_defaults_look_sane() {
        let cfg = TsaClientConfig::new("https://freetsa.org/tsr");
        assert_eq!(cfg.timeout, Duration::from_secs(30));
        assert_eq!(cfg.hash_algorithm, HashAlgorithm::Sha256);
        assert!(cfg.use_nonce);
        assert!(cfg.cert_req);
        assert!(cfg.username.is_none());
        assert!(cfg.password.is_none());
    }
}
