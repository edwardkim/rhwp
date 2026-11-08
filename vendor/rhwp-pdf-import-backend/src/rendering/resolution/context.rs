//! Read-only context the pipeline borrows for the duration of a single
//! resolution call.
//!
//! All the cross-cutting state the resolver stages need lives here:
//! - The document handle, for object resolution (tint-transform streams,
//!   ICC profiles, function dictionaries).
//! - The page's resolved colour-space dictionary, so `Spaced` logical
//!   colours can be evaluated against the spaces the resource map declared.
//! - The document `/OutputIntents` CMYK profile, when present, so the
//!   colour stage can convert `/DeviceCMYK` paint (and `/Separation` /
//!   `/DeviceN` alternates that land in `/DeviceCMYK`) through the
//!   press-target ICC profile instead of the §10.3.5 additive-clamp
//!   fallback. Precedence between embedded ICC, page-level `/DefaultCMYK`,
//!   the document `/OutputIntents` profile, and the additive-clamp
//!   fallback (ISO 32000-1:2008 §14.11.5 / §10) is enforced inside the
//!   resolver — this struct just carries the inputs.
//! - The active graphics-state rendering intent (§10.7.3 `/RI`) so every
//!   ICC conversion is dispatched to the matching qcms intent.
//! - Page-level `/DefaultGray` / `/DefaultRGB` / `/DefaultCMYK` colour-
//!   space overrides (§8.6.5.6) so paint operators using the bare
//!   device families are routed through the page's declared default
//!   before any document-level OutputIntent lookup.
//!
//! The context is a struct of borrows so that the operator walker can build
//! it once per page (or once per Form XObject scope) and hand it to every
//! `resolve` call without per-intent allocation.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::color::{
    CmykRetargetTransform, IccProfile, RenderingIntent, SrgbToCmykTransform, Transform,
};
use crate::document::PdfDocument;
use crate::object::Object;

/// Per-page cache of compiled qcms transforms.
///
/// The cache is n_components-agnostic at the storage level: its key is
/// `(profile.content_hash(), intent)`, so the same instance serves N=1
/// (Gray TRC) profiles routed through `/DefaultGray`, N=3 (RGB) profiles
/// routed through `/DefaultRGB`, and N=4 (CMYK) profiles routed through
/// the document `/OutputIntents` or through embedded `/ICCBased` paint.
/// The cache was first introduced to amortise the 17⁴ CLUT precomputation
/// `qcms::Transform::new_to` runs for CMYK input — that's still its
/// dominant payoff — but every ICC arm of the resolver shares it.
///
/// Constructing a `Transform` runs `qcms::Transform::new_to` which
/// precomputes a 17⁴ = 83 521-sample CLUT for CMYK input (see
/// `qcms-0.3.0/src/transform.rs:1245-1281`). The per-pixel
/// `convert_cmyk_pixel` call is then a cheap tetrahedral interpolation
/// against the CLUT; rebuilding the transform per paint operator is
/// the perf trap. A single page can carry thousands of `k`/`f` pairs
/// emitting the same CMYK quadruple — without the cache every one of
/// those paints pays the precomputation cost. The N=3 RGB path
/// doesn't precompute a CLUT but still runs the qcms profile-build
/// overhead per Transform; caching pays back equally there.
///
/// The cache key is `(profile.content_hash(), intent)`:
///
/// * **Profile identity** — the same `Arc<IccProfile>` instance always
///   compiles to the same transform per intent, so hashing the profile
///   bytes is sufficient. The hash of the bytes incorporates the
///   colour-space signature (`'CMYK'` / `'RGB '` / `'GRAY'`) so an
///   RGB and a CMYK profile cannot share a cache entry even if they
///   happen to collide on `DefaultHasher` — and each `Transform`
///   carries its source profile's `n_components`, so callers asking
///   for the wrong conversion (`convert_rgb_buffer` on a CMYK
///   transform) fall through the n_components guard inside
///   `Transform` instead of silently mis-converting.
///   Multiple profiles can coexist on a single page when a Form
///   XObject carries its own `/ICCBased` colour space distinct from
///   the document `/OutputIntents` profile; the content-hash keying
///   separates them automatically. Two profiles with byte-identical
///   contents would collide on the cache key, but the resulting
///   transform is identical so the collision is harmless.
/// * **Rendering intent** — `qcms::Transform::new_to` takes intent as
///   a parameter; qcms 0.3.0 ignores it internally (the `_intent`
///   underscore at `transform.rs:1288`), but the cache key still
///   includes it so a future qcms upgrade that honours the parameter
///   doesn't silently share transforms across intents.
///
/// Interior mutability via `RefCell` because callers hold `&Context`
/// (the resolver is invoked through immutable references; making it
/// `&mut` would force the operator dispatcher to rewire every
/// resolver call to thread mutable borrows through the colour stage).
/// Single-threaded by construction — `ResolutionContext` is never
/// shared across threads within a render call.
pub(crate) struct IccTransformCache {
    entries: RefCell<HashMap<(u64, RenderingIntent), Arc<Transform>>>,
    /// sRGB → destination-CMYK transforms keyed by the destination
    /// profile content hash + intent. The transparency sidecar's
    /// RGB-paint mirror path consults this cache to convert RGB-source
    /// paints into the OutputIntent CMYK space so subsequent
    /// transparent CMYK paints over an RGB backdrop composite against
    /// the converted backdrop per §11.3.4 + §11.4.5.1 (§11.4.5.1
    /// defines the group's /CS as the single blend colour space;
    /// §11.3.4 is the per-pixel computation that runs inside it).
    /// Built on miss (lcms2 builds only — qcms returns `None`); cached
    /// for the page lifetime.
    srgb_to_cmyk_entries:
        RefCell<HashMap<(u64, RenderingIntent), Option<Arc<SrgbToCmykTransform>>>>,
    /// CMYK → CMYK retarget transforms keyed by `(src_fingerprint,
    /// dst_fingerprint, intent)` where each fingerprint is
    /// `(n_components, byte_len, content_hash)`. The wider key (vs the
    /// scalar `content_hash` used elsewhere) is the same hardening the
    /// font-identity cache uses — `content_hash` alone is SipHash u64,
    /// so a same-byte-length / same-hash collision would route a
    /// wrong-profile transform; including `n_components` and
    /// `byte_len` makes a collision strictly stronger to fabricate.
    /// Built on miss; cached for the page lifetime. The retarget path
    /// is on the DeviceN /Process /ICCBased N=4 hot path (called once
    /// per paint by `try_retarget_cmyk_via_embedded_profile`); without
    /// the cache, each paint re-parses the OutputIntent + embedded
    /// profile and rebuilds the lcms2 CLUT.
    cmyk_retarget_entries: RefCell<
        HashMap<
            ((u8, usize, u64), (u8, usize, u64), RenderingIntent),
            Option<Arc<CmykRetargetTransform>>,
        >,
    >,
    /// Test-support counter: every cache miss (i.e. every call that
    /// actually constructs a fresh `Transform`) increments this
    /// instance-local counter. Distinct from the global
    /// `crate::color::TRANSFORM_BUILD_COUNT` so tests can assert on
    /// per-cache hit rates without racing other parallel tests that
    /// might also build transforms.
    #[cfg(feature = "test-support")]
    pub(crate) build_count: std::cell::Cell<usize>,
    /// Test-support counter: every `get_or_build` CALL — hit or miss —
    /// increments this counter. Hoist-correctness probes use it to
    /// distinguish "called once per paint" (hoisted) from "called once
    /// per pixel" (regressed): `build_count` alone cannot, because the
    /// per-page CMYK cache returns the same `Arc<Transform>` on every
    /// hit and the cost we're guarding against is the `content_hash`
    /// call needed to PROBE the cache, not the build itself.
    #[cfg(feature = "test-support")]
    pub(crate) lookup_count: std::cell::Cell<usize>,
    /// Test-support counter: every CMYK→CMYK retarget cache miss
    /// (i.e. every actual `CmykRetargetTransform::new` call) increments
    /// this counter. Used by the M2 perf-bound probe to pin "the
    /// retarget transform builds exactly once per unique profile
    /// pair, even across many paints".
    #[cfg(feature = "test-support")]
    pub(crate) cmyk_retarget_build_count: std::cell::Cell<usize>,
}

impl IccTransformCache {
    pub(crate) fn new() -> Self {
        Self {
            entries: RefCell::new(HashMap::new()),
            srgb_to_cmyk_entries: RefCell::new(HashMap::new()),
            cmyk_retarget_entries: RefCell::new(HashMap::new()),
            #[cfg(feature = "test-support")]
            build_count: std::cell::Cell::new(0),
            #[cfg(feature = "test-support")]
            lookup_count: std::cell::Cell::new(0),
            #[cfg(feature = "test-support")]
            cmyk_retarget_build_count: std::cell::Cell::new(0),
        }
    }

    /// Look up or build the compiled `Transform` for `(profile,
    /// intent)`. On a cache miss the closure builds the transform once
    /// and inserts it; subsequent calls return the cached
    /// `Arc<Transform>`. The borrow on `entries` is released between
    /// the `get` probe and the `insert` so the closure can re-enter
    /// the cache safely (it won't — but defensive locking shape).
    pub(crate) fn get_or_build(
        &self,
        profile: &Arc<IccProfile>,
        intent: RenderingIntent,
    ) -> Arc<Transform> {
        #[cfg(feature = "test-support")]
        self.lookup_count.set(self.lookup_count.get() + 1);
        let key = (profile.content_hash(), intent);
        if let Some(t) = self.entries.borrow().get(&key).cloned() {
            return t;
        }
        let t = Arc::new(Transform::new_srgb_target(Arc::clone(profile), intent));
        self.entries.borrow_mut().insert(key, Arc::clone(&t));
        #[cfg(feature = "test-support")]
        self.build_count.set(self.build_count.get() + 1);
        t
    }

    /// Look up or build the compiled sRGB→destination-CMYK transform
    /// for the document's OutputIntent CMYK profile. Returns `None`
    /// (cached) on backends that can't build the transform (qcms /
    /// no-CMM); call sites then fall back to the §10.3.5 inverse.
    pub(crate) fn get_or_build_srgb_to_cmyk(
        &self,
        dst_profile: &Arc<IccProfile>,
        intent: RenderingIntent,
    ) -> Option<Arc<SrgbToCmykTransform>> {
        let key = (dst_profile.content_hash(), intent);
        if let Some(slot) = self.srgb_to_cmyk_entries.borrow().get(&key) {
            return slot.clone();
        }
        let built = SrgbToCmykTransform::new(Arc::clone(dst_profile), intent).map(Arc::new);
        self.srgb_to_cmyk_entries
            .borrow_mut()
            .insert(key, built.clone());
        built
    }

    /// Look up or build the compiled CMYK→CMYK retarget transform for
    /// `(src_profile, dst_profile, intent)`. `None` (cached) is
    /// returned when the active backend can't compile the transform
    /// (qcms — no CMYK output path). The key uses
    /// `(n_components, byte_len, content_hash)` per profile rather than
    /// the bare `content_hash` to harden against collisions; mirrors
    /// the font-identity-cache shape.
    pub(crate) fn get_or_build_cmyk_retarget(
        &self,
        src_profile: &Arc<IccProfile>,
        dst_profile: &Arc<IccProfile>,
        intent: RenderingIntent,
    ) -> Option<Arc<CmykRetargetTransform>> {
        let src_key = (
            src_profile.n_components(),
            src_profile.bytes().len(),
            src_profile.content_hash(),
        );
        let dst_key = (
            dst_profile.n_components(),
            dst_profile.bytes().len(),
            dst_profile.content_hash(),
        );
        let key = (src_key, dst_key, intent);
        if let Some(slot) = self.cmyk_retarget_entries.borrow().get(&key) {
            return slot.clone();
        }
        let built =
            CmykRetargetTransform::new(Arc::clone(src_profile), Arc::clone(dst_profile), intent)
                .map(Arc::new);
        self.cmyk_retarget_entries
            .borrow_mut()
            .insert(key, built.clone());
        #[cfg(feature = "test-support")]
        self.cmyk_retarget_build_count
            .set(self.cmyk_retarget_build_count.get() + 1);
        built
    }

    /// Drop every entry. Called per page so the cache doesn't leak
    /// transforms across renders when `PageRenderer` is reused.
    pub(crate) fn clear(&self) {
        self.entries.borrow_mut().clear();
        self.srgb_to_cmyk_entries.borrow_mut().clear();
        self.cmyk_retarget_entries.borrow_mut().clear();
        #[cfg(feature = "test-support")]
        self.build_count.set(0);
        #[cfg(feature = "test-support")]
        self.lookup_count.set(0);
        #[cfg(feature = "test-support")]
        self.cmyk_retarget_build_count.set(0);
    }

    /// Number of cache misses observed in the cache's lifetime since
    /// the last `clear()`. Test-only — never exposed on production
    /// builds.
    #[cfg(feature = "test-support")]
    pub(crate) fn build_count(&self) -> usize {
        self.build_count.get()
    }

    /// Total `get_or_build` calls (hits + misses) observed since the
    /// last `clear()`. Test-only. Used by the hot-loop hoist probes:
    /// a single paint may legitimately CALL `get_or_build` once
    /// (cache hit on every pixel), but the `content_hash` cost we are
    /// guarding against runs on every call regardless of hit/miss, so
    /// the only correct steady-state is "one call per paint".
    #[cfg(feature = "test-support")]
    pub(crate) fn lookup_count(&self) -> usize {
        self.lookup_count.get()
    }

    /// Number of CMYK→CMYK retarget cache misses observed since the
    /// last `clear()`. Test-only. M2 probe pins "exactly one build
    /// per unique (src, dst, intent) tuple, even across many paints".
    #[cfg(feature = "test-support")]
    pub(crate) fn cmyk_retarget_build_count(&self) -> usize {
        self.cmyk_retarget_build_count.get()
    }
}

impl Default for IccTransformCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-page (or per-Form XObject) context for the resolution pipeline.
///
/// Lifetime `'a` ties the context to the operator walker's owned state.
pub(crate) struct ResolutionContext<'a> {
    pub(crate) doc: &'a PdfDocument,
    pub(crate) color_spaces: &'a HashMap<String, Object>,
    /// Document `/OutputIntents` CMYK profile, when present. Consumed by
    /// `ColorResolver` for `/DeviceCMYK` paint and for `/Separation` /
    /// `/DeviceN` resolved alternates that land in `/DeviceCMYK`.
    pub(crate) output_intent_cmyk: Option<&'a Arc<IccProfile>>,
    /// Active graphics-state rendering intent (§10.7.3). Defaults to
    /// `/RelativeColorimetric` when the page graphics state hasn't set
    /// `/RI` explicitly.
    pub(crate) rendering_intent: RenderingIntent,
    /// Page-level `/DefaultGray` override (§8.6.5.6), when present.
    pub(crate) default_gray: Option<&'a Object>,
    /// Page-level `/DefaultRGB` override (§8.6.5.6), when present.
    pub(crate) default_rgb: Option<&'a Object>,
    /// Page-level `/DefaultCMYK` override (§8.6.5.6), when present.
    pub(crate) default_cmyk: Option<&'a Object>,
    /// Per-page compiled qcms transform cache. When `Some`, the
    /// colour stage looks up `(profile, intent)` in the cache before
    /// calling `Transform::new_srgb_target` — the latter precomputes
    /// an 17⁴ CLUT and dominates per-paint cost on documents that
    /// repeat the same CMYK colour. The cache is shared across every
    /// `ResolutionContext` instance built within a single page render
    /// so the operator-walker's fresh-context-per-paint pattern still
    /// amortises transform construction. `None` skips caching — the
    /// resolver builds a fresh transform per paint, which is what the
    /// unit-test paths and the `cargo test --lib` resolver tests
    /// exercise.
    pub(crate) icc_transform_cache: Option<&'a IccTransformCache>,
}

impl<'a> ResolutionContext<'a> {
    /// Build a context from the page-resource snapshot the operator walker
    /// already maintains. The walker computes `color_spaces` from
    /// `resources["ColorSpace"]` once per page; we just borrow it.
    ///
    /// Callers chain `with_output_intent` / `with_rendering_intent` /
    /// `with_defaults` to populate the colour-policy fields. The bare
    /// constructor leaves them unset so unit tests that only probe the
    /// `Device*` paths don't need to thread fixture profiles through.
    pub(crate) fn new(doc: &'a PdfDocument, color_spaces: &'a HashMap<String, Object>) -> Self {
        Self {
            doc,
            color_spaces,
            output_intent_cmyk: None,
            rendering_intent: RenderingIntent::default(),
            default_gray: None,
            default_rgb: None,
            default_cmyk: None,
            icc_transform_cache: None,
        }
    }

    /// Attach a per-page CMYK transform cache. The cache lives on
    /// `PageRenderer` (cleared per page) so transform construction is
    /// amortised across the many `ResolutionContext` instances the
    /// operator dispatcher builds inside a single render. `None`
    /// (the default) skips caching — appropriate for unit tests that
    /// only exercise a handful of conversions.
    pub(crate) fn with_icc_transform_cache(mut self, cache: Option<&'a IccTransformCache>) -> Self {
        self.icc_transform_cache = cache;
        self
    }

    /// Attach the document's `/OutputIntents` CMYK profile, when one is
    /// available. `None` is a no-op and leaves the additive-clamp
    /// fallback in place — the colour stage only consults the profile
    /// when it's `Some`.
    pub(crate) fn with_output_intent(mut self, profile: Option<&'a Arc<IccProfile>>) -> Self {
        self.output_intent_cmyk = profile;
        self
    }

    /// Set the active rendering intent (§10.7.3) the colour stage
    /// dispatches to qcms with. Defaults to `RelativeColorimetric` per
    /// the spec's "unrecognised → RelativeColorimetric" rule when the
    /// graphics state hasn't otherwise set it.
    pub(crate) fn with_rendering_intent(mut self, intent: RenderingIntent) -> Self {
        self.rendering_intent = intent;
        self
    }

    /// Set the page-level `/DefaultGray` / `/DefaultRGB` / `/DefaultCMYK`
    /// colour-space overrides (§8.6.5.6). Each `None` means the page
    /// didn't declare that override; the colour stage then resolves the
    /// bare device family normally.
    pub(crate) fn with_defaults(
        mut self,
        gray: Option<&'a Object>,
        rgb: Option<&'a Object>,
        cmyk: Option<&'a Object>,
    ) -> Self {
        self.default_gray = gray;
        self.default_rgb = rgb;
        self.default_cmyk = cmyk;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::fixture_doc;
    use super::*;

    #[test]
    fn context_carries_empty_color_spaces() {
        let doc = fixture_doc();
        let color_spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &color_spaces);
        assert!(ctx.color_spaces.is_empty());
        assert!(ctx.output_intent_cmyk.is_none());
        assert_eq!(ctx.rendering_intent, RenderingIntent::RelativeColorimetric);
        assert!(ctx.default_gray.is_none());
        assert!(ctx.default_rgb.is_none());
        assert!(ctx.default_cmyk.is_none());
    }

    #[test]
    fn context_borrows_color_space_map() {
        // The point of taking `&HashMap` is that the walker's page-scope
        // map is reused across intents; building a fresh context per
        // intent must be cheap (no clone).
        let doc = fixture_doc();
        let mut color_spaces = HashMap::new();
        color_spaces.insert("CS1".to_string(), Object::Name("DeviceCMYK".to_string()));

        let ctx = ResolutionContext::new(&doc, &color_spaces);
        assert!(ctx.color_spaces.contains_key("CS1"));
        // Re-build context — must still see the same entries through the
        // same borrow without any heap traffic.
        let ctx2 = ResolutionContext::new(&doc, &color_spaces);
        assert_eq!(ctx2.color_spaces.len(), 1);
    }

    #[test]
    fn context_carries_output_intent_when_set() {
        // Pin that the OutputIntent builder method actually attaches the
        // profile borrow to the context — the colour stage relies on
        // `ctx.output_intent_cmyk.is_some()` to decide whether to consult
        // the ICC path, so a no-op `with_output_intent` would silently
        // fall back to additive-clamp without anyone noticing.
        let doc = fixture_doc();
        let color_spaces = HashMap::new();
        let profile = Arc::new(
            IccProfile::parse(super::tests::header_only_cmyk_profile_bytes(), 4)
                .expect("header-only stub profile parses"),
        );
        let ctx = ResolutionContext::new(&doc, &color_spaces).with_output_intent(Some(&profile));
        assert!(ctx.output_intent_cmyk.is_some());
    }

    #[test]
    fn with_rendering_intent_overrides_default() {
        let doc = fixture_doc();
        let color_spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &color_spaces)
            .with_rendering_intent(RenderingIntent::AbsoluteColorimetric);
        assert_eq!(ctx.rendering_intent, RenderingIntent::AbsoluteColorimetric);
    }

    #[test]
    fn with_defaults_attaches_each_override_independently() {
        let doc = fixture_doc();
        let color_spaces = HashMap::new();
        let gray = Object::Name("DeviceGray".to_string());
        let cmyk = Object::Name("DeviceCMYK".to_string());
        let ctx = ResolutionContext::new(&doc, &color_spaces).with_defaults(
            Some(&gray),
            None,
            Some(&cmyk),
        );
        assert!(ctx.default_gray.is_some());
        assert!(ctx.default_rgb.is_none());
        assert!(ctx.default_cmyk.is_some());
    }

    /// Header-only CMYK stub — same shape as the existing
    /// `tests/test_icc_cmyk_conversion.rs` helper. qcms will reject it
    /// at transform-build time (no tag table), so it's only useful as a
    /// "profile-shaped" Arc for tests probing whether the context
    /// carries the borrow at all.
    pub(crate) fn header_only_cmyk_profile_bytes() -> Vec<u8> {
        let mut v = vec![0u8; 128];
        v[8..12].copy_from_slice(&0x04000000u32.to_be_bytes());
        v[12..16].copy_from_slice(b"prtr");
        v[16..20].copy_from_slice(b"CMYK");
        v[20..24].copy_from_slice(b"Lab ");
        v[36..40].copy_from_slice(b"acsp");
        v
    }
}
