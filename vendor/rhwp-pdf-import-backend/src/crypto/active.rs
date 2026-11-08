//! Active provider registry — the single global [`CryptoProvider`]
//! every PDF operation routes through.
//!
//! There's exactly one active provider at a time. It's set at most
//! once (via [`set_provider`]); subsequent calls return
//! [`SetProviderError::AlreadySet`]. After a provider is in use, swapping
//! mid-process would be a soundness hazard for in-flight crypto state
//! (e.g., the FIPS module's per-process self-test).
//!
//! Default behaviour: if no provider has been registered when
//! [`active`] is first called, [`RustCryptoProvider`] (the
//! permissive Rust-only default) is installed lazily. FIPS
//! deployments call [`set_provider`] at process startup before any
//! PDF operation.
//!
//! [`set_provider`]: self::set_provider
//! [`active`]: self::active
//! [`RustCryptoProvider`]: super::RustCryptoProvider

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use super::policy::{AlgorithmId, SecurityPolicy};
use super::provider::CryptoProvider;
use super::rust_provider::RustCryptoProvider;

/// Errors from [`set_provider`].
#[derive(Debug)]
pub enum SetProviderError {
    /// A provider has already been installed (either by a prior call
    /// or by lazy default initialization on first [`active`] call).
    AlreadySet,
}

impl std::fmt::Display for SetProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetProviderError::AlreadySet => f.write_str(
                "crypto provider already set — set_provider() must be called \
                 once at process startup, before any PDF operation",
            ),
        }
    }
}

impl std::error::Error for SetProviderError {}

static ACTIVE: OnceLock<Arc<dyn CryptoProvider>> = OnceLock::new();

/// Install `provider` as the process-wide active [`CryptoProvider`].
///
/// Must be called before any PDF operation that uses crypto (open
/// encrypted document, verify signature, etc.). Returns
/// [`SetProviderError::AlreadySet`] if a provider is already
/// installed.
///
/// FIPS deployments call this once with `AwsLcProvider` (behind the
/// `fips` feature) at process startup.
///
/// Tests that need a fresh provider registry must run in their own
/// process — `cargo test` reuses one binary per crate target, so
/// `#[test]` functions in the same lib/integration target share this
/// `OnceLock`. Use a separate integration-test target (`tests/<name>.rs`
/// invoked via `cargo test --test <name>`) or a custom test harness so
/// each test gets its own process and a fresh `ACTIVE` cell.
pub fn set_provider(provider: Arc<dyn CryptoProvider>) -> Result<(), SetProviderError> {
    ACTIVE
        .set(provider)
        .map_err(|_| SetProviderError::AlreadySet)
}

/// Returns the active provider, lazily initializing
/// [`RustCryptoProvider`] on first call if none was registered.
pub fn active() -> &'static Arc<dyn CryptoProvider> {
    ACTIVE.get_or_init(|| Arc::new(RustCryptoProvider::new()))
}

/// Reports whether a provider has been installed (either explicitly
/// via [`set_provider`] or lazily by a previous [`active`] call).
pub fn is_set() -> bool {
    ACTIVE.get().is_some()
}

/// Errors from [`set_policy`].
#[derive(Debug)]
pub enum SetPolicyError {
    /// A policy has already been installed (by a prior [`set_policy`]
    /// call or lazily by a previous [`active_policy`] call). Like the
    /// provider, a mid-flight policy downgrade is a soundness/attack
    /// hazard, so the policy is set at most once.
    AlreadySet,
}

impl std::fmt::Display for SetPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetPolicyError::AlreadySet => f.write_str(
                "crypto policy already set — set_policy() must be called once at \
                 process startup, before any PDF crypto operation",
            ),
        }
    }
}

impl std::error::Error for SetPolicyError {}

static ACTIVE_POLICY: OnceLock<SecurityPolicy> = OnceLock::new();

/// Install `policy` as the process-wide active [`SecurityPolicy`].
///
/// Set-once, exactly like [`set_provider`] (a runtime policy downgrade
/// would be an attack vector). Returns [`SetPolicyError::AlreadySet`]
/// if a policy is already installed — treat that as fatal. The policy
/// is orthogonal to the provider: it never widens behaviour, only
/// narrows it.
///
/// The same test-isolation rule as [`set_provider`] applies — registry
/// tests must run in their own integration-test binary.
pub fn set_policy(policy: SecurityPolicy) -> Result<(), SetPolicyError> {
    ACTIVE_POLICY
        .set(policy)
        .map_err(|_| SetPolicyError::AlreadySet)
}

/// Returns the active [`SecurityPolicy`], lazily initialising
/// [`SecurityPolicy::compat`] (behaviour-preserving) on first call if
/// none was registered.
pub fn active_policy() -> &'static SecurityPolicy {
    ACTIVE_POLICY.get_or_init(SecurityPolicy::compat)
}

/// Reports whether a policy has been installed (explicitly via
/// [`set_policy`] or lazily by a previous [`active_policy`] call).
pub fn is_policy_set() -> bool {
    ACTIVE_POLICY.get().is_some()
}

/// Process-wide crypto inventory: bit `AlgorithmId::index()` is set
/// the first time that algorithm is exercised at an enforcement
/// boundary. This is the minimal "what crypto did this run use?"
/// report regulated buyers ask for (CBOM-adjacent — see #230 plan
/// §3.7).
///
/// It is a **content-keyed** atomic bitset, *not* a pointer-keyed
/// global cache — explicitly the allowed shape per the shared
/// foundation §6.2 (no #505-class data race). 17 ids fit in 64 bits
/// with head-room.
static INVENTORY: AtomicU64 = AtomicU64::new(0);

/// Record that `alg` was exercised this process (idempotent, lock-free).
pub fn record_algorithm_use(alg: AlgorithmId) {
    INVENTORY.fetch_or(1u64 << alg.index(), Ordering::Relaxed);
}

/// The set of [`AlgorithmId`]s exercised so far this process, in
/// declaration order. Cheap; no allocation beyond the result `Vec`.
pub fn inventory() -> Vec<AlgorithmId> {
    let bits = INVENTORY.load(Ordering::Relaxed);
    AlgorithmId::ALL
        .into_iter()
        .filter(|a| bits & (1u64 << a.index()) != 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::HashAlgorithm;
    use super::*;

    /// `active()` must succeed for a fresh process and return the
    /// permissive default (which permits MD5).
    #[test]
    fn lazy_default_is_rust_crypto() {
        let p = active();
        assert_eq!(p.name(), "rust-crypto");
        #[cfg(feature = "legacy-crypto")]
        assert!(p.is_legacy_allowed());
        #[cfg(not(feature = "legacy-crypto"))]
        assert!(!p.is_legacy_allowed());
        // Sanity: MD5 hasher works under default provider (when legacy-crypto is on).
        #[cfg(feature = "legacy-crypto")]
        {
            let mut h = p.hasher(HashAlgorithm::Md5).unwrap();
            h.update(b"abc");
            let out = h.finalize();
            assert_eq!(out.len(), 16);
        }
        // When legacy-crypto is off, MD5 should return an error.
        #[cfg(not(feature = "legacy-crypto"))]
        {
            assert!(p.hasher(HashAlgorithm::Md5).is_err());
        }
    }

    /// Once active() has lazily installed the default,
    /// `set_provider` rejects any further attempt — proves the
    /// "set-at-most-once" invariant for downstream callers that rely
    /// on it.
    #[test]
    fn set_provider_after_lazy_init_fails() {
        let _ = active();
        let attempt = set_provider(Arc::new(RustCryptoProvider::new()));
        assert!(matches!(attempt, Err(SetProviderError::AlreadySet)));
    }
}
