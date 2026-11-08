#[cfg(all(feature = "nightly_aarch64_neon", target_arch = "aarch64"))]
use core::arch::aarch64::*;

#[cfg(all(feature = "nightly_aarch64_neon", target_arch = "aarch64"))]
#[target_feature(enable = "neon")]
unsafe fn idct8(data: &mut [int16x8_t; 8]) {
    // The fixed-point constants here are obtained by taking the fractional part of the constants
    // from the non-SIMD implementation and scaling them up by 1<<15. This is because
    // vqrdmulhq_n_s16(a, b) is effectively equivalent to (a*b)>>15 (except for possibly some
    // slight differences in rounding).

    // The code here is effectively equivalent to the calls to "kernel" in idct.rs, except that it
    // doesn't apply any further scaling and fixed point constants have a different precision.

    let p2 = data[2];
    let p3 = data[6];
    let p1 = vqrdmulhq_n_s16(vqaddq_s16(p2, p3), 17734); // 0.5411961
    let t2 = vqsubq_s16(
        vqsubq_s16(p1, p3),
        vqrdmulhq_n_s16(p3, 27779), // 0.847759065
    );
    let t3 = vqaddq_s16(p1, vqrdmulhq_n_s16(p2, 25079)); // 0.765366865

    let p2 = data[0];
    let p3 = data[4];
    let t0 = vqaddq_s16(p2, p3);
    let t1 = vqsubq_s16(p2, p3);

    let x0 = vqaddq_s16(t0, t3);
    let x3 = vqsubq_s16(t0, t3);
    let x1 = vqaddq_s16(t1, t2);
    let x2 = vqsubq_s16(t1, t2);

    let t0 = data[7];
    let t1 = data[5];
    let t2 = data[3];
    let t3 = data[1];

    let p3 = vqaddq_s16(t0, t2);
    let p4 = vqaddq_s16(t1, t3);
    let p1 = vqaddq_s16(t0, t3);
    let p2 = vqaddq_s16(t1, t2);
    let p5 = vqaddq_s16(p3, p4);
    let p5 = vqaddq_s16(p5, vqrdmulhq_n_s16(p5, 5763)); // 0.175875602

    let t0 = vqrdmulhq_n_s16(t0, 9786); // 0.298631336
    let t1 = vqaddq_s16(
        vqaddq_s16(t1, t1),
        vqrdmulhq_n_s16(t1, 1741), // 0.053119869
    );
    let t2 = vqaddq_s16(
        vqaddq_s16(t2, vqaddq_s16(t2, t2)),
        vqrdmulhq_n_s16(t2, 2383), // 0.072711026
    );
    let t3 = vqaddq_s16(t3, vqrdmulhq_n_s16(t3, 16427)); // 0.501321110

    let p1 = vqsubq_s16(p5, vqrdmulhq_n_s16(p1, 29490)); // 0.899976223
    let p2 = vqsubq_s16(
        vqsubq_s16(vqsubq_s16(p5, p2), p2),
        vqrdmulhq_n_s16(p2, 18446), // 0.562915447
    );

    let p3 = vqsubq_s16(
        vqrdmulhq_n_s16(p3, -31509), // -0.961570560
        p3,
    );
    let p4 = vqrdmulhq_n_s16(p4, -12785); // -0.390180644

    let t3 = vqaddq_s16(vqaddq_s16(p1, p4), t3);
    let t2 = vqaddq_s16(vqaddq_s16(p2, p3), t2);
    let t1 = vqaddq_s16(vqaddq_s16(p2, p4), t1);
    let t0 = vqaddq_s16(vqaddq_s16(p1, p3), t0);

    data[0] = vqaddq_s16(x0, t3);
    data[7] = vqsubq_s16(x0, t3);
    data[1] = vqaddq_s16(x1, t2);
    data[6] = vqsubq_s16(x1, t2);
    data[2] = vqaddq_s16(x2, t1);
    data[5] = vqsubq_s16(x2, t1);
    data[3] = vqaddq_s16(x3, t0);
    data[4] = vqsubq_s16(x3, t0);
}

#[cfg(all(feature = "nightly_aarch64_neon", target_arch = "aarch64"))]
#[target_feature(enable = "neon")]
unsafe fn transpose8(data: &mut [int16x8_t; 8]) {
    // Use NEON's 2x2 matrix transposes (vtrn) to do the transposition in each 4x4 block, then
    // combine the 4x4 blocks.
    let a01 = vtrnq_s16(data[0], data[1]);
    let a23 = vtrnq_s16(data[2], data[3]);

    let four0 = vtrnq_s32(vreinterpretq_s32_s16(a01.0), vreinterpretq_s32_s16(a23.0));
    let four1 = vtrnq_s32(vreinterpretq_s32_s16(a01.1), vreinterpretq_s32_s16(a23.1));

    let a45 = vtrnq_s16(data[4], data[5]);
    let a67 = vtrnq_s16(data[6], data[7]);

    let four2 = vtrnq_s32(vreinterpretq_s32_s16(a45.0), vreinterpretq_s32_s16(a67.0));
    let four3 = vtrnq_s32(vreinterpretq_s32_s16(a45.1), vreinterpretq_s32_s16(a67.1));

    data[0] = vreinterpretq_s16_s32(vcombine_s32(vget_low_s32(four0.0), vget_low_s32(four2.0)));
    data[1] = vreinterpretq_s16_s32(vcombine_s32(vget_low_s32(four1.0), vget_low_s32(four3.0)));
    data[2] = vreinterpretq_s16_s32(vcombine_s32(vget_low_s32(four0.1), vget_low_s32(four2.1)));
    data[3] = vreinterpretq_s16_s32(vcombine_s32(vget_low_s32(four1.1), vget_low_s32(four3.1)));
    data[4] = vreinterpretq_s16_s32(vcombine_s32(vget_high_s32(four0.0), vget_high_s32(four2.0)));
    data[5] = vreinterpretq_s16_s32(vcombine_s32(vget_high_s32(four1.0), vget_high_s32(four3.0)));
    data[6] = vreinterpretq_s16_s32(vcombine_s32(vget_high_s32(four0.1), vget_high_s32(four2.1)));
    data[7] = vreinterpretq_s16_s32(vcombine_s32(vget_high_s32(four1.1), vget_high_s32(four3.1)));
}

#[cfg(all(feature = "nightly_aarch64_neon", target_arch = "aarch64"))]
#[target_feature(enable = "neon")]
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

    const SHIFT: i32 = 3;

    // Read the DCT coefficients, scale them up and dequantize them.
    let mut data = [vdupq_n_s16(0); 8];
    for i in 0..8 {
        data[i] = vshlq_n_s16(
            vmulq_s16(
                vld1q_s16(coefficients.as_ptr().wrapping_add(i * 8)),
                vreinterpretq_s16_u16(vld1q_u16(quantization_table.as_ptr().wrapping_add(i * 8))),
            ),
            SHIFT,
        );
    }

    // Usual column IDCT - transpose - column IDCT - transpose approach.
    idct8(&mut data);
    transpose8(&mut data);
    idct8(&mut data);
    transpose8(&mut data);

    for i in 0..8 {
        // The two passes of the IDCT algorithm give us a factor of 8, so the shift here is
        // increased by 3.
        // As values will be stored in a u8, they need to be 128-centered and not 0-centered.
        // We add 128 with the appropriate shift for that purpose.
        const OFFSET: i16 = 128 << (SHIFT + 3);
        // We want rounding right shift, so we should add (1/2) << (SHIFT+3) before shifting.
        const ROUNDING_BIAS: i16 = (1 << (SHIFT + 3)) >> 1;

        let data_with_offset = vqaddq_s16(data[i], vdupq_n_s16(OFFSET + ROUNDING_BIAS));

        vst1_u8(
            output.as_mut_ptr().wrapping_add(output_linestride * i),
            vqshrun_n_s16(data_with_offset, SHIFT + 3),
        );
    }
}

#[cfg(all(feature = "nightly_aarch64_neon", target_arch = "aarch64"))]
#[target_feature(enable = "neon")]
pub unsafe fn color_convert_line_ycbcr(y: &[u8], cb: &[u8], cr: &[u8], output: &mut [u8]) -> usize {
    assert!(output.len() % 3 == 0);
    let num = output.len() / 3;
    assert!(num <= y.len());
    assert!(num <= cb.len());
    assert!(num <= cr.len());
    let num_vecs = num / 8;

    for i in 0..num_vecs {
        const SHIFT: i32 = 6;
        // Load.
        let y = vld1_u8(y.as_ptr().wrapping_add(i * 8));
        let cb = vld1_u8(cb.as_ptr().wrapping_add(i * 8));
        let cr = vld1_u8(cr.as_ptr().wrapping_add(i * 8));

        // Convert to 16 bit and shift.
        let y = vreinterpretq_s16_u16(vshll_n_u8(y, SHIFT));
        let cb = vreinterpretq_s16_u16(vshll_n_u8(cb, SHIFT));
        let cr = vreinterpretq_s16_u16(vshll_n_u8(cr, SHIFT));

        // Add offsets
        let y = vqaddq_s16(y, vdupq_n_s16((1 << SHIFT) >> 1));
        let c128 = vdupq_n_s16(128 << SHIFT);
        let cb = vqsubq_s16(cb, c128);
        let cr = vqsubq_s16(cr, c128);

        // Compute cr * 1.402, cb * 0.34414, cr * 0.71414, cb * 1.772
        let cr_140200 = vqaddq_s16(vqrdmulhq_n_s16(cr, 13173), cr);
        let cb_034414 = vqrdmulhq_n_s16(cb, 11276);
        let cr_071414 = vqrdmulhq_n_s16(cr, 23401);
        let cb_177200 = vqaddq_s16(vqrdmulhq_n_s16(cb, 25297), cb);

        // Last conversion step.
        let r = vqaddq_s16(y, cr_140200);
        let g = vqsubq_s16(y, vqaddq_s16(cb_034414, cr_071414));
        let b = vqaddq_s16(y, cb_177200);

        // Shift back and convert to u8.
        let r = vqshrun_n_s16(r, SHIFT);
        let g = vqshrun_n_s16(g, SHIFT);
        let b = vqshrun_n_s16(b, SHIFT);

        // Shuffle + store.
        vst3_u8(
            output.as_mut_ptr().wrapping_add(24 * i),
            uint8x8x3_t(r, g, b),
        );
    }

    num_vecs * 8
}
