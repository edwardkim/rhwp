//! Non-separable PDF blend modes (Hue, Saturation, Color, Luminosity)
//! per ISO 32000-1:2008 §11.3.5.3.
//!
//! tiny_skia has no native non-separable blend mode; these are
//! implemented out-of-band by rendering the source paint into a fresh
//! scratch pixmap (which captures the source's contribution as `Source`
//! mode RGBA) and then per-pixel compositing against the destination
//! pixmap using the §11.3.5.3 algorithm.
//!
//! The four non-separable modes share a luminance-projection +
//! re-encoding skeleton:
//!
//! - **Hue**: SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb))
//! - **Saturation**: SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb))
//! - **Color**: SetLum(Cs, Lum(Cb))
//! - **Luminosity**: SetLum(Cb, Lum(Cs))
//!
//! `Lum`, `Sat`, `SetLum`, `SetSat`, and `ClipColor` are defined in
//! §11.3.5.3 and implemented below.

/// PDF non-separable blend modes per §11.3.5.3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NonSeparableBlend {
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl NonSeparableBlend {
    /// Recognise the PDF blend-mode name.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "Hue" => Some(Self::Hue),
            "Saturation" => Some(Self::Saturation),
            "Color" => Some(Self::Color),
            "Luminosity" => Some(Self::Luminosity),
            _ => None,
        }
    }
}

/// Compose `source` over `dest` in-place using the §11.3.5.3 algorithm.
///
/// Both buffers are RGBA8 row-major, identical dimensions, and — per
/// tiny_skia's storage contract on `Pixmap::data()` / `data_mut()` —
/// hold **premultiplied** RGBA samples. The §11.3.5.3 non-separable
/// blend formulas (and the §11.3.4 compositing equation that consumes
/// their result) operate on **straight** colour values. Each pixel is
/// therefore un-premultiplied on the way in, blended/composited as
/// straight RGB, and re-premultiplied on the way out.
///
/// The composition implements the full §11.3.4 form for a non-isolated
/// non-knockout group:
///   αo = αs + αb · (1 − αs)
///   Co = ((1 − αs) · αb · Cb + αs · ((1 − αb) · Cs + αb · B(Cb, Cs))) / αo
/// (When αo = 0 the output pixel is fully transparent and `Co` is
/// undefined; the buffer is zeroed there.) The opaque-backdrop
/// reduction (αb = 1) drops out of this as a special case.
pub(crate) fn compose_in_place(dest: &mut [u8], source: &[u8], mode: NonSeparableBlend) {
    debug_assert_eq!(dest.len(), source.len());
    debug_assert_eq!(dest.len() % 4, 0);

    for px in 0..(dest.len() / 4) {
        let off = px * 4;
        let src_a = source[off + 3];
        if src_a == 0 {
            // Source fully transparent → dest unchanged (αo = αb,
            // Co = Cb is the §11.3.4 reduction; nothing to write).
            continue;
        }

        // Read source/dest as premultiplied f32 in [0, 1], then
        // un-premultiply to straight colour for the §11.3.5.3 math.
        let sa = src_a as f32 / 255.0;
        let (sr, sg, sb) = unpremultiply(source[off], source[off + 1], source[off + 2], sa);

        let da = dest[off + 3] as f32 / 255.0;
        let (dr, dg, db) = unpremultiply(dest[off], dest[off + 1], dest[off + 2], da);

        // §11.3.5.3 blend B(Cb, Cs) on STRAIGHT colour.
        let (br, bg, bb) = match mode {
            NonSeparableBlend::Hue => {
                // SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb))
                let sat_cb = sat((dr, dg, db));
                let sat_applied = set_sat((sr, sg, sb), sat_cb);
                set_lum(sat_applied, lum((dr, dg, db)))
            },
            NonSeparableBlend::Saturation => {
                // SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb))
                let sat_cs = sat((sr, sg, sb));
                let sat_applied = set_sat((dr, dg, db), sat_cs);
                set_lum(sat_applied, lum((dr, dg, db)))
            },
            NonSeparableBlend::Color => {
                // SetLum(Cs, Lum(Cb))
                set_lum((sr, sg, sb), lum((dr, dg, db)))
            },
            NonSeparableBlend::Luminosity => {
                // SetLum(Cb, Lum(Cs))
                set_lum((dr, dg, db), lum((sr, sg, sb)))
            },
        };

        // §11.3.4 general (non-isolated, non-knockout) composition with
        // arbitrary backdrop alpha, on STRAIGHT colour:
        //   αo = αs + αb · (1 − αs)
        //   Co = ((1 − αs) · αb · Cb + αs · ((1 − αb) · Cs + αb · B)) / αo
        let inv_sa = 1.0 - sa;
        let inv_da = 1.0 - da;
        let out_a = sa + da * inv_sa;

        let (out_r, out_g, out_b) = if out_a <= 0.0 {
            (0.0, 0.0, 0.0)
        } else {
            let blend_r = inv_sa * da * dr + sa * (inv_da * sr + da * br);
            let blend_g = inv_sa * da * dg + sa * (inv_da * sg + da * bg);
            let blend_b = inv_sa * da * db + sa * (inv_da * sb + da * bb);
            (blend_r / out_a, blend_g / out_a, blend_b / out_a)
        };

        // Re-premultiply for tiny_skia storage.
        let out_r_premul = (out_r.clamp(0.0, 1.0) * out_a).clamp(0.0, 1.0);
        let out_g_premul = (out_g.clamp(0.0, 1.0) * out_a).clamp(0.0, 1.0);
        let out_b_premul = (out_b.clamp(0.0, 1.0) * out_a).clamp(0.0, 1.0);

        dest[off] = (out_r_premul * 255.0).round() as u8;
        dest[off + 1] = (out_g_premul * 255.0).round() as u8;
        dest[off + 2] = (out_b_premul * 255.0).round() as u8;
        dest[off + 3] = (out_a.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}

/// Convert one premultiplied RGB byte triple at the given (straight)
/// alpha back to straight RGB in [0, 1]. Returns `(0, 0, 0)` when
/// `alpha == 0` — the spec leaves `Co` undefined in that case and the
/// caller has already short-circuited the source-α==0 branch, but the
/// destination side calls in unconditionally and a 0 backdrop alpha
/// must not produce a divide-by-zero.
fn unpremultiply(r: u8, g: u8, b: u8, alpha: f32) -> (f32, f32, f32) {
    if alpha <= 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let inv = 1.0 / alpha;
    (
        (r as f32 / 255.0 * inv).clamp(0.0, 1.0),
        (g as f32 / 255.0 * inv).clamp(0.0, 1.0),
        (b as f32 / 255.0 * inv).clamp(0.0, 1.0),
    )
}

/// §11.3.5.3 `Lum(C) = 0.30 R + 0.59 G + 0.11 B`.
fn lum(c: (f32, f32, f32)) -> f32 {
    0.30 * c.0 + 0.59 * c.1 + 0.11 * c.2
}

/// §11.3.5.3 `Sat(C) = max(R, G, B) - min(R, G, B)`.
fn sat(c: (f32, f32, f32)) -> f32 {
    c.0.max(c.1).max(c.2) - c.0.min(c.1).min(c.2)
}

/// §11.3.5.3 `SetLum(C, l)`: shift the luminance of `C` to `l`, then
/// clip to the gamut.
fn set_lum(c: (f32, f32, f32), l: f32) -> (f32, f32, f32) {
    let d = l - lum(c);
    let shifted = (c.0 + d, c.1 + d, c.2 + d);
    clip_color(shifted)
}

/// §11.3.5.3 `ClipColor(C)`: project an out-of-gamut color back into
/// the unit RGB cube while preserving its luminance.
fn clip_color(c: (f32, f32, f32)) -> (f32, f32, f32) {
    let l = lum(c);
    let n = c.0.min(c.1).min(c.2);
    let x = c.0.max(c.1).max(c.2);

    let (mut r, mut g, mut b) = c;
    if n < 0.0 {
        // Scale toward the luminance to bring the minimum to 0.
        let denom = l - n;
        if denom.abs() > 1e-9 {
            r = l + (r - l) * l / denom;
            g = l + (g - l) * l / denom;
            b = l + (b - l) * l / denom;
        }
    }
    if x > 1.0 {
        // Scale toward the luminance to bring the maximum to 1.
        let denom = x - l;
        if denom.abs() > 1e-9 {
            r = l + (r - l) * (1.0 - l) / denom;
            g = l + (g - l) * (1.0 - l) / denom;
            b = l + (b - l) * (1.0 - l) / denom;
        }
    }
    (r, g, b)
}

/// §11.3.5.3 `SetSat(C, s)`: rebuild C so it has saturation `s` while
/// preserving the ordering of the channels.
fn set_sat(c: (f32, f32, f32), s: f32) -> (f32, f32, f32) {
    // Identify the channels in (min, mid, max) order. Place s into
    // max - min, mid is scaled proportionally, others zero.
    let (r, g, b) = c;
    // Sort channels by value, tracking original positions.
    let mut chans = [(r, 0u8), (g, 1u8), (b, 2u8)];
    chans.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));

    let (cmin, cmid, cmax) = (chans[0].0, chans[1].0, chans[2].0);
    let (imin, imid, imax) = (chans[0].1, chans[1].1, chans[2].1);

    let (new_min, new_mid, new_max) = if cmax > cmin {
        (0.0_f32, ((cmid - cmin) * s) / (cmax - cmin), s)
    } else {
        (0.0_f32, 0.0_f32, 0.0_f32)
    };

    let mut out = [0.0_f32; 3];
    out[imin as usize] = new_min;
    out[imid as usize] = new_mid;
    out[imax as usize] = new_max;
    (out[0], out[1], out[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn lum_matches_bt601_weights() {
        let l = lum((1.0, 0.0, 0.0));
        assert!(approx(l, 0.30));
        let l = lum((0.0, 1.0, 0.0));
        assert!(approx(l, 0.59));
        let l = lum((0.0, 0.0, 1.0));
        assert!(approx(l, 0.11));
    }

    #[test]
    fn sat_of_grey_is_zero() {
        assert!(approx(sat((0.5, 0.5, 0.5)), 0.0));
    }

    #[test]
    fn sat_of_pure_red_is_one() {
        assert!(approx(sat((1.0, 0.0, 0.0)), 1.0));
    }

    #[test]
    fn luminosity_blend_grey_source_over_red_preserves_red_hue() {
        // Source = mid-grey (Y = 0.5), Dest = red (Y = 0.30).
        // SetLum(Cb, Lum(Cs)) = SetLum((1, 0, 0), 0.5).
        // Shift d = 0.5 - 0.30 = 0.20; shifted = (1.2, 0.20, 0.20).
        // ClipColor: x = 1.2 > 1.0 → scale toward luminance.
        //   denom = 1.2 - 0.5 = 0.7
        //   r = 0.5 + (1.2 - 0.5) * (1.0 - 0.5) / 0.7 = 0.5 + 0.5 = 1.0
        //   g = 0.5 + (0.20 - 0.5) * 0.5 / 0.7 ≈ 0.286
        //   b = 0.286
        // Result is red-dominant (R=1.0 >> G≈0.286, B≈0.286).
        let mut dest = [255u8, 0, 0, 255];
        let source = [128u8, 128, 128, 255];
        compose_in_place(&mut dest, &source, NonSeparableBlend::Luminosity);
        assert!(
            dest[0] > dest[1] + 60 && dest[0] > dest[2] + 60,
            "Luminosity grey-over-red should preserve red hue; got {:?}",
            dest
        );
    }

    #[test]
    fn hue_blend_red_source_over_blue_yields_red() {
        // Source = red (H=0°, S=1, L=0.30), Dest = blue (H=240°, S=1, L=0.11).
        // Hue: SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb)).
        //   Sat(Cb=blue) = 1.0
        //   SetSat(red, 1.0) = red (already at saturation 1)
        //   SetLum(red, 0.11) = shift d = 0.11 - 0.30 = -0.19
        //     shifted = (0.81, -0.19, -0.19)
        //     ClipColor: n = -0.19 < 0 → scale.
        //       denom = 0.11 - (-0.19) = 0.30
        //       r = 0.11 + (0.81 - 0.11) * 0.11 / 0.30 ≈ 0.11 + 0.257 = 0.367
        //       g = 0.11 + (-0.19 - 0.11) * 0.11 / 0.30 ≈ 0.11 - 0.110 = 0.0
        //       b = 0.0
        // Result: red-dominant.
        let mut dest = [0u8, 0, 255, 255];
        let source = [255u8, 0, 0, 255];
        compose_in_place(&mut dest, &source, NonSeparableBlend::Hue);
        assert!(
            dest[0] > 50 && dest[1] < 30 && dest[2] < 30,
            "Hue red-over-blue should yield red-dominant; got {:?}",
            dest
        );
    }

    #[test]
    fn saturation_blend_grey_source_desaturates_dest() {
        // Source = grey (Sat = 0), Dest = red.
        // SetLum(SetSat(Cb, 0), Lum(Cb)) = SetLum((0, 0, 0), 0.30)
        //   = (0.30, 0.30, 0.30) → grey.
        let mut dest = [255u8, 0, 0, 255];
        let source = [128u8, 128, 128, 255];
        compose_in_place(&mut dest, &source, NonSeparableBlend::Saturation);
        // Channels should be near-equal (desaturated).
        let max_diff = (dest[0] as i32 - dest[1] as i32)
            .abs()
            .max((dest[0] as i32 - dest[2] as i32).abs())
            .max((dest[1] as i32 - dest[2] as i32).abs());
        assert!(max_diff < 30, "Saturation grey-over-red should desaturate; got {:?}", dest);
    }

    // ============================================================
    // Partial-alpha byte-exact probes — §11.3.4 + §11.3.5.3
    // ============================================================
    //
    // tiny_skia's `Pixmap::data` is premultiplied RGBA; the §11.3.5.3
    // non-separable formulas + the §11.3.4 compositing equation
    // operate on STRAIGHT colour. The probes below pin the byte-exact
    // result for each of the four non-separable modes at a partial
    // source and partial backdrop alpha. They fail when the function
    // reads premultiplied bytes as if they were straight colour
    // (the bug before the un-premultiply/re-premultiply fix landed)
    // or when the compositing reduces to the αb = 1 special case.
    //
    // Fixture inputs (all stored as premultiplied bytes):
    //   - Backdrop: red at αb_byte = 128 → straight Cb = (1, 0, 0),
    //     stored as (128, 0, 0, 128).
    //   - Source:   blue at αs_byte = 179 → straight Cs = (0, 0, 1),
    //     stored as (0, 0, 179, 179).
    //
    // Output α: αo = αs + αb·(1−αs) ≈ 0.852 → byte 217 (shared by
    // every mode at these inputs).
    //
    // Expected straight-colour blend results B(Cb, Cs):
    //   - Hue        = SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb))
    //   - Saturation = SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb))
    //   - Color      = SetLum(Cs, Lum(Cb))
    //   - Luminosity = SetLum(Cb, Lum(Cs))
    // The hand-derived expected bytes below are reproduced from the
    // §11.3.5.3 + §11.3.4 walk in this test's module docstring; see
    // also the bug write-up for the per-channel arithmetic.

    /// Backdrop pixel — straight red at α≈0.502, premultiplied.
    const PA_BACKDROP: [u8; 4] = [128, 0, 0, 128];
    /// Source pixel — straight blue at α≈0.702, premultiplied.
    const PA_SOURCE: [u8; 4] = [0, 0, 179, 179];

    #[test]
    fn hue_blend_partial_alpha_is_byte_exact() {
        // Hue B = (0.2135, 0.2135, 1.0); composite per §11.3.4 with
        // αs=179/255, αb=128/255 and re-premultiply → (57, 19, 179, 217).
        let mut dest = PA_BACKDROP;
        compose_in_place(&mut dest, &PA_SOURCE, NonSeparableBlend::Hue);
        assert_eq!(
            dest,
            [57, 19, 179, 217],
            "Hue blend partial-alpha: §11.3.4 + §11.3.5.3 produce \
             byte-exact (57, 19, 179, 217); got {:?}",
            dest
        );
    }

    #[test]
    fn saturation_blend_partial_alpha_is_byte_exact() {
        // Saturation B = (1.0, 0, 0); composite per §11.3.4 →
        // (128, 0, 89, 217). The R-channel passes through the backdrop's
        // (1 − αs)·αb·Cb + αs·αb·B_r term unchanged because Cb=Cs_sat-
        // applied=red coincides; B_blue picks up the source-only term.
        let mut dest = PA_BACKDROP;
        compose_in_place(&mut dest, &PA_SOURCE, NonSeparableBlend::Saturation);
        assert_eq!(
            dest,
            [128, 0, 89, 217],
            "Saturation blend partial-alpha: §11.3.4 + §11.3.5.3 \
             produce byte-exact (128, 0, 89, 217); got {:?}",
            dest
        );
    }

    #[test]
    fn color_blend_partial_alpha_is_byte_exact() {
        // Color B = (0.2135, 0.2135, 1.0); composite per §11.3.4 →
        // (57, 19, 179, 217). Identical to Hue at this input because
        // both Cs and Cb sit at saturation 1; B(Cb, Cs) reduces to the
        // SetLum(Cs, Lum(Cb)) form for both modes.
        let mut dest = PA_BACKDROP;
        compose_in_place(&mut dest, &PA_SOURCE, NonSeparableBlend::Color);
        assert_eq!(
            dest,
            [57, 19, 179, 217],
            "Color blend partial-alpha: §11.3.4 + §11.3.5.3 produce \
             byte-exact (57, 19, 179, 217); got {:?}",
            dest
        );
    }

    #[test]
    fn luminosity_blend_partial_alpha_is_byte_exact() {
        // Luminosity B = (0.367, 0, 0); composite per §11.3.4 →
        // (71, 0, 89, 217). The B-channel survives via the source-only
        // (1 − αb)·αs·Cs term — αb < 1 means the source contributes
        // even outside its blended region; without the un-premultiply
        // fix the backdrop would dominate.
        let mut dest = PA_BACKDROP;
        compose_in_place(&mut dest, &PA_SOURCE, NonSeparableBlend::Luminosity);
        assert_eq!(
            dest,
            [71, 0, 89, 217],
            "Luminosity blend partial-alpha: §11.3.4 + §11.3.5.3 \
             produce byte-exact (71, 0, 89, 217); got {:?}",
            dest
        );
    }
}
