#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "ssse3")]
unsafe fn idct8(data: &mut [__m128i; 8]) {
    // The fixed-point constants here are obtained by taking the fractional part of the constants
    // from the non-SIMD implementation and scaling them up by 1<<15. This is because
    // _mm_mulhrs_epi16(a, b) is effectively equivalent to (a*b)>>15 (except for possibly some
    // slight differences in rounding).

    // The code here is effectively equivalent to the calls to "kernel" in idct.rs, except that it
    // doesn't apply any further scaling and fixed point constants have a different precision.

    let p2 = data[2];
    let p3 = data[6];
    let p1 = _mm_mulhrs_epi16(_mm_adds_epi16(p2, p3), _mm_set1_epi16(17734)); // 0.5411961
    let t2 = _mm_subs_epi16(
        _mm_subs_epi16(p1, p3),
        _mm_mulhrs_epi16(p3, _mm_set1_epi16(27779)), // 0.847759065
    );
    let t3 = _mm_adds_epi16(p1, _mm_mulhrs_epi16(p2, _mm_set1_epi16(25079))); // 0.765366865

    let p2 = data[0];
    let p3 = data[4];
    let t0 = _mm_adds_epi16(p2, p3);
    let t1 = _mm_subs_epi16(p2, p3);

    let x0 = _mm_adds_epi16(t0, t3);
    let x3 = _mm_subs_epi16(t0, t3);
    let x1 = _mm_adds_epi16(t1, t2);
    let x2 = _mm_subs_epi16(t1, t2);

    let t0 = data[7];
    let t1 = data[5];
    let t2 = data[3];
    let t3 = data[1];

    let p3 = _mm_adds_epi16(t0, t2);
    let p4 = _mm_adds_epi16(t1, t3);
    let p1 = _mm_adds_epi16(t0, t3);
    let p2 = _mm_adds_epi16(t1, t2);
    let p5 = _mm_adds_epi16(p3, p4);
    let p5 = _mm_adds_epi16(p5, _mm_mulhrs_epi16(p5, _mm_set1_epi16(5763))); // 0.175875602

    let t0 = _mm_mulhrs_epi16(t0, _mm_set1_epi16(9786)); // 0.298631336
    let t1 = _mm_adds_epi16(
        _mm_adds_epi16(t1, t1),
        _mm_mulhrs_epi16(t1, _mm_set1_epi16(1741)), // 0.053119869
    );
    let t2 = _mm_adds_epi16(
        _mm_adds_epi16(t2, _mm_adds_epi16(t2, t2)),
        _mm_mulhrs_epi16(t2, _mm_set1_epi16(2383)), // 0.072711026
    );
    let t3 = _mm_adds_epi16(t3, _mm_mulhrs_epi16(t3, _mm_set1_epi16(16427))); // 0.501321110

    let p1 = _mm_subs_epi16(p5, _mm_mulhrs_epi16(p1, _mm_set1_epi16(29490))); // 0.899976223
    let p2 = _mm_subs_epi16(
        _mm_subs_epi16(_mm_subs_epi16(p5, p2), p2),
        _mm_mulhrs_epi16(p2, _mm_set1_epi16(18446)), // 0.562915447
    );

    let p3 = _mm_subs_epi16(
        _mm_mulhrs_epi16(p3, _mm_set1_epi16(-31509)), // -0.961570560
        p3,
    );
    let p4 = _mm_mulhrs_epi16(p4, _mm_set1_epi16(-12785)); // -0.390180644

    let t3 = _mm_adds_epi16(_mm_adds_epi16(p1, p4), t3);
    let t2 = _mm_adds_epi16(_mm_adds_epi16(p2, p3), t2);
    let t1 = _mm_adds_epi16(_mm_adds_epi16(p2, p4), t1);
    let t0 = _mm_adds_epi16(_mm_adds_epi16(p1, p3), t0);

    data[0] = _mm_adds_epi16(x0, t3);
    data[7] = _mm_subs_epi16(x0, t3);
    data[1] = _mm_adds_epi16(x1, t2);
    data[6] = _mm_subs_epi16(x1, t2);
    data[2] = _mm_adds_epi16(x2, t1);
    data[5] = _mm_subs_epi16(x2, t1);
    data[3] = _mm_adds_epi16(x3, t0);
    data[4] = _mm_subs_epi16(x3, t0);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "ssse3")]
unsafe fn transpose8(data: &mut [__m128i; 8]) {
    // Transpose a 8x8 matrix with a sequence of interleaving operations.
    // Naming: dABl contains elements from the *l*ower halves of vectors A and B, interleaved, i.e.
    // A0 B0 A1 B1 ...
    // dABCDll contains elements from the lower quarter (ll) of vectors A, B, C, D, interleaved -
    // A0 B0 C0 D0 A1 B1 C1 D1 ...
    let d01l = _mm_unpacklo_epi16(data[0], data[1]);
    let d23l = _mm_unpacklo_epi16(data[2], data[3]);
    let d45l = _mm_unpacklo_epi16(data[4], data[5]);
    let d67l = _mm_unpacklo_epi16(data[6], data[7]);
    let d01h = _mm_unpackhi_epi16(data[0], data[1]);
    let d23h = _mm_unpackhi_epi16(data[2], data[3]);
    let d45h = _mm_unpackhi_epi16(data[4], data[5]);
    let d67h = _mm_unpackhi_epi16(data[6], data[7]);
    // Operating on 32-bits will interleave *consecutive pairs* of 16-bit integers.
    let d0123ll = _mm_unpacklo_epi32(d01l, d23l);
    let d0123lh = _mm_unpackhi_epi32(d01l, d23l);
    let d4567ll = _mm_unpacklo_epi32(d45l, d67l);
    let d4567lh = _mm_unpackhi_epi32(d45l, d67l);
    let d0123hl = _mm_unpacklo_epi32(d01h, d23h);
    let d0123hh = _mm_unpackhi_epi32(d01h, d23h);
    let d4567hl = _mm_unpacklo_epi32(d45h, d67h);
    let d4567hh = _mm_unpackhi_epi32(d45h, d67h);
    // Operating on 64-bits will interleave *consecutive quadruples* of 16-bit integers.
    data[0] = _mm_unpacklo_epi64(d0123ll, d4567ll);
    data[1] = _mm_unpackhi_epi64(d0123ll, d4567ll);
    data[2] = _mm_unpacklo_epi64(d0123lh, d4567lh);
    data[3] = _mm_unpackhi_epi64(d0123lh, d4567lh);
    data[4] = _mm_unpacklo_epi64(d0123hl, d4567hl);
    data[5] = _mm_unpackhi_epi64(d0123hl, d4567hl);
    data[6] = _mm_unpacklo_epi64(d0123hh, d4567hh);
    data[7] = _mm_unpackhi_epi64(d0123hh, d4567hh);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "ssse3")]
pub unsafe fn dequantize_and_idct_block_8x8(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    // The loop below will write to positions [output_linestride * i, output_linestride * i + 8)
    // for 0<=i<8. Thus, the last accessed position is at an offset of output_linestrade * 7 + 7,
    // and if that position is in-bounds, so are all other accesses.
    assert!(
        output.len()
            > output_linestride
                .checked_mul(7)
                .unwrap()
                .checked_add(7)
                .unwrap()
    );

    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    const SHIFT: i32 = 3;

    // Read the DCT coefficients, scale them up and dequantize them.
    let mut data = [_mm_setzero_si128(); 8];
    for (i, item) in data.iter_mut().enumerate() {
        *item = _mm_slli_epi16(
            _mm_mullo_epi16(
                _mm_loadu_si128(coefficients.as_ptr().wrapping_add(i * 8) as *const _),
                _mm_loadu_si128(quantization_table.as_ptr().wrapping_add(i * 8) as *const _),
            ),
            SHIFT,
        );
    }

    // Usual column IDCT - transpose - column IDCT - transpose approach.
    idct8(&mut data);
    transpose8(&mut data);
    idct8(&mut data);
    transpose8(&mut data);

    for (i, item) in data.iter_mut().enumerate() {
        let mut buf = [0u8; 16];
        // The two passes of the IDCT algorithm give us a factor of 8, so the shift here is
        // increased by 3.
        // As values will be stored in a u8, they need to be 128-centered and not 0-centered.
        // We add 128 with the appropriate shift for that purpose.
        const OFFSET: i16 = 128 << (SHIFT + 3);
        // We want rounding right shift, so we should add (1/2) << (SHIFT+3) before shifting.
        const ROUNDING_BIAS: i16 = (1 << (SHIFT + 3)) >> 1;

        let data_with_offset = _mm_adds_epi16(*item, _mm_set1_epi16(OFFSET + ROUNDING_BIAS));

        _mm_storeu_si128(
            buf.as_mut_ptr() as *mut _,
            _mm_packus_epi16(
                _mm_srai_epi16(data_with_offset, SHIFT + 3),
                _mm_setzero_si128(),
            ),
        );
        std::ptr::copy_nonoverlapping::<u8>(
            buf.as_ptr(),
            output.as_mut_ptr().wrapping_add(output_linestride * i) as *mut _,
            8,
        );
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "ssse3")]
pub unsafe fn color_convert_line_ycbcr(y: &[u8], cb: &[u8], cr: &[u8], output: &mut [u8]) -> usize {
    assert!(output.len() % 3 == 0);
    let num = output.len() / 3;
    assert!(num <= y.len());
    assert!(num <= cb.len());
    assert!(num <= cr.len());
    // _mm_loadu_si64 generates incorrect code for Rust <1.58. To circumvent this, we use a full
    // 128-bit load, but that requires leaving an extra vector of border to the scalar code.
    // From Rust 1.58 on, the _mm_loadu_si128 can be replaced with _mm_loadu_si64 and this
    // .saturating_sub() can be removed.
    let num_vecs = (num / 8).saturating_sub(1);

    for i in 0..num_vecs {
        const SHIFT: i32 = 6;
        // Load.
        let y = _mm_loadu_si128(y.as_ptr().wrapping_add(i * 8) as *const _);
        let cb = _mm_loadu_si128(cb.as_ptr().wrapping_add(i * 8) as *const _);
        let cr = _mm_loadu_si128(cr.as_ptr().wrapping_add(i * 8) as *const _);

        // Convert to 16 bit.
        let shuf16 = _mm_setr_epi8(
            0, -0x7F, 1, -0x7F, 2, -0x7F, 3, -0x7F, 4, -0x7F, 5, -0x7F, 6, -0x7F, 7, -0x7F,
        );
        let y = _mm_slli_epi16(_mm_shuffle_epi8(y, shuf16), SHIFT);
        let cb = _mm_slli_epi16(_mm_shuffle_epi8(cb, shuf16), SHIFT);
        let cr = _mm_slli_epi16(_mm_shuffle_epi8(cr, shuf16), SHIFT);

        // Add offsets
        let c128 = _mm_set1_epi16(128 << SHIFT);
        let y = _mm_adds_epi16(y, _mm_set1_epi16((1 << SHIFT) >> 1));
        let cb = _mm_subs_epi16(cb, c128);
        let cr = _mm_subs_epi16(cr, c128);

        // Compute cr * 1.402, cb * 0.34414, cr * 0.71414, cb * 1.772
        let cr_140200 = _mm_adds_epi16(_mm_mulhrs_epi16(cr, _mm_set1_epi16(13173)), cr);
        let cb_034414 = _mm_mulhrs_epi16(cb, _mm_set1_epi16(11276));
        let cr_071414 = _mm_mulhrs_epi16(cr, _mm_set1_epi16(23401));
        let cb_177200 = _mm_adds_epi16(_mm_mulhrs_epi16(cb, _mm_set1_epi16(25297)), cb);

        // Last conversion step.
        let r = _mm_adds_epi16(y, cr_140200);
        let g = _mm_subs_epi16(y, _mm_adds_epi16(cb_034414, cr_071414));
        let b = _mm_adds_epi16(y, cb_177200);

        // Shift back and convert to u8.
        let zero = _mm_setzero_si128();
        let r = _mm_packus_epi16(_mm_srai_epi16(r, SHIFT), zero);
        let g = _mm_packus_epi16(_mm_srai_epi16(g, SHIFT), zero);
        let b = _mm_packus_epi16(_mm_srai_epi16(b, SHIFT), zero);

        // Shuffle rrrrrrrrggggggggbbbbbbbb to rgbrgbrgb...

        // Control vectors for _mm_shuffle_epi8. -0x7F is selected so that the resulting position
        // after _mm_shuffle_epi8 will be filled with 0, so that the r, g, and b vectors can then
        // be OR-ed together.
        let shufr = _mm_setr_epi8(
            0, -0x7F, -0x7F, 1, -0x7F, -0x7F, 2, -0x7F, -0x7F, 3, -0x7F, -0x7F, 4, -0x7F, -0x7F, 5,
        );
        let shufg = _mm_setr_epi8(
            -0x7F, 0, -0x7F, -0x7F, 1, -0x7F, -0x7F, 2, -0x7F, -0x7F, 3, -0x7F, -0x7F, 4, -0x7F,
            -0x7F,
        );
        let shufb = _mm_alignr_epi8(shufg, shufg, 15);

        let rgb_low = _mm_or_si128(
            _mm_shuffle_epi8(r, shufr),
            _mm_or_si128(_mm_shuffle_epi8(g, shufg), _mm_shuffle_epi8(b, shufb)),
        );

        // For the next part of the rgb vectors, we need to select R values from 6 up, G and B from
        // 5 up. The highest bit of -0x7F + 6 is still set, so the corresponding location will
        // still be 0.
        let shufr1 = _mm_add_epi8(shufb, _mm_set1_epi8(6));
        let shufg1 = _mm_add_epi8(shufr, _mm_set1_epi8(5));
        let shufb1 = _mm_add_epi8(shufg, _mm_set1_epi8(5));

        let rgb_hi = _mm_or_si128(
            _mm_shuffle_epi8(r, shufr1),
            _mm_or_si128(_mm_shuffle_epi8(g, shufg1), _mm_shuffle_epi8(b, shufb1)),
        );

        let mut data = [0u8; 32];
        _mm_storeu_si128(data.as_mut_ptr() as *mut _, rgb_low);
        _mm_storeu_si128(data.as_mut_ptr().wrapping_add(16) as *mut _, rgb_hi);
        std::ptr::copy_nonoverlapping::<u8>(
            data.as_ptr(),
            output.as_mut_ptr().wrapping_add(24 * i),
            24,
        );
    }

    num_vecs * 8
}
