//! RFC 3161 Time-Stamp Protocol ASN.1 structures, and the RFC 4210
//! `PKIStatusInfo` an RFC 3161 response carries.
//!
//! These were previously taken from the `x509-tsp` crate (plus `cmpv2`
//! for the status type). That crate has exactly one published version,
//! pinned to `cms 0.2` / `der 0.7`, so keeping it forces a second,
//! parallel copy of the whole RustCrypto formats stack into the build —
//! and the two copies' types do not interoperate, which is what stops
//! `cms 0.3` from being usable at all. The definitions themselves are a
//! hundred lines of `der` derive, so they live here instead.
//!
//! Field names, tag modes, defaults and optionality follow RFC 3161
//! §2.4.1/§2.4.2 and RFC 4210 §5.2.3, and were cross-checked against
//! `x509-tsp` 0.1.0 so the wire format is unchanged.

use cms::cert::x509::ext::pkix::name::GeneralName;
use cms::cert::x509::ext::Extensions;
use cms::cert::x509::spki::AlgorithmIdentifier;
#[cfg(feature = "tsa-client")]
use cms::content_info::ContentInfo;
#[cfg(feature = "tsa-client")]
use der::asn1::{BitString, Utf8StringRef};
use der::asn1::{GeneralizedTime, Int, OctetString};
use der::oid::ObjectIdentifier;
use der::{Any, Enumerated, Sequence};

/// ```text
/// version INTEGER { v1(1) }
/// ```
#[derive(Clone, Copy, Debug, Enumerated, Eq, PartialEq, PartialOrd, Ord)]
#[asn1(type = "INTEGER")]
#[repr(u8)]
pub enum TspVersion {
    /// Syntax version 1 — the only one RFC 3161 defines.
    V1 = 1,
}

/// ```text
/// TSAPolicyId ::= OBJECT IDENTIFIER
/// ```
pub type TsaPolicyId = ObjectIdentifier;

/// ```text
/// MessageImprint ::= SEQUENCE  {
///    hashAlgorithm                AlgorithmIdentifier,
///    hashedMessage                OCTET STRING  }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
pub struct MessageImprint {
    /// Algorithm the `hashed_message` digest was produced with.
    pub hash_algorithm: AlgorithmIdentifier<Any>,
    /// The digest itself.
    pub hashed_message: OctetString,
}

/// ```text
/// TimeStampReq ::= SEQUENCE  {
///    version               INTEGER  { v1(1) },
///    messageImprint        MessageImprint,
///    reqPolicy             TSAPolicyId              OPTIONAL,
///    nonce                 INTEGER                  OPTIONAL,
///    certReq               BOOLEAN                  DEFAULT FALSE,
///    extensions            [0] IMPLICIT Extensions  OPTIONAL  }
/// ```
#[cfg(feature = "tsa-client")]
#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
pub struct TimeStampReq {
    /// Always [`TspVersion::V1`].
    pub version: TspVersion,
    /// Digest the caller wants timestamped.
    pub message_imprint: MessageImprint,
    /// Requested TSA policy, if the caller cares which one is used.
    #[asn1(optional = "true")]
    pub req_policy: Option<TsaPolicyId>,
    /// Replay-detection nonce echoed back in the TSTInfo.
    #[asn1(optional = "true")]
    pub nonce: Option<Int>,
    /// Ask the TSA to include its signing certificate in the token.
    #[asn1(default = "Default::default")]
    pub cert_req: bool,
    /// Request extensions.
    #[asn1(context_specific = "0", tag_mode = "IMPLICIT", optional = "true")]
    pub extensions: Option<Extensions>,
}

/// ```text
/// PKIStatus ::= INTEGER {
///     accepted(0), grantedWithMods(1), rejection(2), waiting(3),
///     revocationWarning(4), revocationNotification(5),
///     keyUpdateWarning(6) }
/// ```
///
/// RFC 4210 §5.2.3.
#[cfg(feature = "tsa-client")]
#[derive(Clone, Copy, Debug, Enumerated, Eq, PartialEq)]
#[asn1(type = "INTEGER")]
#[repr(u8)]
#[allow(missing_docs)]
pub enum PkiStatus {
    Accepted = 0,
    GrantedWithMods = 1,
    Rejection = 2,
    Waiting = 3,
    RevocationWarning = 4,
    RevocationNotification = 5,
    KeyUpdateWarning = 6,
}

/// ```text
/// PKIFreeText ::= SEQUENCE SIZE (1..MAX) OF UTF8String
/// ```
#[cfg(feature = "tsa-client")]
pub type PkiFreeText<'a> = alloc_vec::Vec<Utf8StringRef<'a>>;

// `Vec` under its own path so the type alias above reads as the ASN.1
// does, without shadowing anything in this module.
#[cfg(feature = "tsa-client")]
use std::vec as alloc_vec;

/// ```text
/// PKIStatusInfo ::= SEQUENCE {
///     status        PKIStatus,
///     statusString  PKIFreeText     OPTIONAL,
///     failInfo      PKIFailureInfo  OPTIONAL }
/// ```
///
/// `PKIFailureInfo` is a BIT STRING of named bits (RFC 4210 §5.2.3).
/// Nothing here interprets individual bits — a rejected timestamp is
/// reported, not recovered from — so it stays a [`BitString`] rather
/// than a flag set, which also keeps the decode faithful to any bit
/// positions a future RFC adds.
#[cfg(feature = "tsa-client")]
#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
#[allow(missing_docs)]
pub struct PkiStatusInfo<'a> {
    pub status: PkiStatus,
    #[asn1(optional = "true")]
    pub status_string: Option<PkiFreeText<'a>>,
    #[asn1(optional = "true")]
    pub fail_info: Option<BitString>,
}

/// ```text
/// TimeStampToken ::= ContentInfo
/// ```
#[cfg(feature = "tsa-client")]
pub type TimeStampToken = ContentInfo;

/// ```text
/// TimeStampResp ::= SEQUENCE  {
///     status                  PKIStatusInfo,
///     timeStampToken          TimeStampToken     OPTIONAL  }
/// ```
#[cfg(feature = "tsa-client")]
#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
pub struct TimeStampResp<'a> {
    /// Whether the TSA granted the request, and why not if it did not.
    pub status: PkiStatusInfo<'a>,
    /// Present when `status` is accepted or granted-with-mods.
    #[asn1(optional = "true")]
    pub time_stamp_token: Option<TimeStampToken>,
}

/// ```text
/// Accuracy ::= SEQUENCE {
///     seconds        INTEGER              OPTIONAL,
///     millis     [0] INTEGER  (1..999)    OPTIONAL,
///     micros     [1] INTEGER  (1..999)    OPTIONAL  }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
#[allow(missing_docs)]
pub struct Accuracy {
    #[asn1(optional = "true")]
    pub seconds: Option<u64>,
    #[asn1(context_specific = "0", tag_mode = "IMPLICIT", optional = "true")]
    pub millis: Option<i16>,
    #[asn1(context_specific = "1", tag_mode = "IMPLICIT", optional = "true")]
    pub micros: Option<i16>,
}

/// ```text
/// TSTInfo ::= SEQUENCE  {
///     version                      INTEGER  { v1(1) },
///     policy                       TSAPolicyId,
///     messageImprint               MessageImprint,
///     serialNumber                 INTEGER,
///     genTime                      GeneralizedTime,
///     accuracy                     Accuracy                 OPTIONAL,
///     ordering                     BOOLEAN             DEFAULT FALSE,
///     nonce                        INTEGER                  OPTIONAL,
///     tsa                          [0] GeneralName          OPTIONAL,
///     extensions                   [1] IMPLICIT Extensions  OPTIONAL  }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Sequence)]
#[allow(missing_docs)]
pub struct TstInfo {
    pub version: TspVersion,
    pub policy: TsaPolicyId,
    pub message_imprint: MessageImprint,
    pub serial_number: Int,
    pub gen_time: GeneralizedTime,
    #[asn1(optional = "true")]
    pub accuracy: Option<Accuracy>,
    #[asn1(default = "Default::default")]
    pub ordering: bool,
    #[asn1(optional = "true")]
    pub nonce: Option<Int>,
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
    pub tsa: Option<GeneralName>,
    #[asn1(context_specific = "1", tag_mode = "IMPLICIT", optional = "true")]
    pub extensions: Option<Extensions>,
}
