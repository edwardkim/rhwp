// Malicious JPEG files can cause operations in the idct to overflow.
// One example is tests/crashtest/images/imagetestsuite/b0b8914cc5f7a6eff409f16d8cc236c5.jpg
// That's why wrapping operators are needed.

// Note: we have many values that are straight from a reference.
// Do not warn on them or try to automatically change them.
#![allow(clippy::excessive_precision)]
// Note: consistency for unrolled, scaled offset loops
#![allow(clippy::erasing_op)]
#![allow(clippy::identity_op)]
use crate::parser::Dimensions;
use core::num::Wrapping;

pub(crate) fn choose_idct_size(full_size: Dimensions, requested_size: Dimensions) -> usize {
    fn scaled(len: u16, scale: usize) -> u16 {
        ((len as u32 * scale as u32 - 1) / 8 + 1) as u16
    }

    for &scale in &[1, 2, 4] {
        if scaled(full_size.width, scale) >= requested_size.width
            || scaled(full_size.height, scale) >= requested_size.height
        {
            return scale;
        }
    }

    8
}

#[test]
fn test_choose_idct_size() {
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 200,
                height: 200
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 500,
                height: 500
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 684,
                height: 456
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 999,
                height: 456
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 684,
                height: 999
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 500,
                height: 333
            },
            Dimensions {
                width: 63,
                height: 42
            }
        ),
        1
    );

    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 685,
                height: 999
            }
        ),
        2
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 1000,
                height: 1000
            }
        ),
        2
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 1400,
                height: 1400
            }
        ),
        4
    );

    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 5472,
                height: 3648
            }
        ),
        8
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 16384,
                height: 16384
            }
        ),
        8
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 1,
                height: 1
            },
            Dimensions {
                width: 65535,
                height: 65535
            }
        ),
        8
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 16384,
                height: 16384
            }
        ),
        8
    );
}

pub(crate) fn dequantize_and_idct_block(
    scale: usize,
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    match scale {
        8 => dequantize_and_idct_block_8x8(
            coefficients,
            quantization_table,
            output_linestride,
            output,
        ),
        4 => dequantize_and_idct_block_4x4(
            coefficients,
            quantization_table,
            output_linestride,
            output,
        ),
        2 => dequantize_and_idct_block_2x2(
            coefficients,
            quantization_table,
            output_linestride,
            output,
        ),
        1 => dequantize_and_idct_block_1x1(
            coefficients,
            quantization_table,
            output_linestride,
            output,
        ),
        _ => panic!("Unsupported IDCT scale {scale}/8"),
    }
}

pub fn dequantize_and_idct_block_8x8(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    #[cfg(not(feature = "platform_independent"))]
    if let Some(idct) = crate::arch::get_dequantize_and_idct_block_8x8() {
        #[allow(unsafe_code)]
        unsafe {
            return idct(coefficients, quantization_table, output_linestride, output);
        }
    }

    let output = output.chunks_mut(output_linestride);
    dequantize_and_idct_block_8x8_inner(coefficients, quantization_table, output)
}

// This is based on stb_image's 'stbi__idct_block'.
fn dequantize_and_idct_block_8x8_inner<'a, I>(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output: I,
) where
    I: IntoIterator<Item = &'a mut [u8]>,
    I::IntoIter: ExactSizeIterator<Item = &'a mut [u8]>,
{
    let output = output.into_iter();
    debug_assert!(
        output.len() >= 8,
        "Output iterator has the wrong length: {}",
        output.len()
    );

    let mut temp = [Wrapping(0); 64];

    // columns
    for i in 0..8 {
        if coefficients[i + 8] == 0
            && coefficients[i + 16] == 0
            && coefficients[i + 24] == 0
            && coefficients[i + 32] == 0
            && coefficients[i + 40] == 0
            && coefficients[i + 48] == 0
            && coefficients[i + 56] == 0
        {
            let dcterm = dequantize(coefficients[i], quantization_table[i]) << 2;
            temp[i] = dcterm;
            temp[i + 8] = dcterm;
            temp[i + 16] = dcterm;
            temp[i + 24] = dcterm;
            temp[i + 32] = dcterm;
            temp[i + 40] = dcterm;
            temp[i + 48] = dcterm;
            temp[i + 56] = dcterm;
        } else {
            let s0 = dequantize(coefficients[i], quantization_table[i]);
            let s1 = dequantize(coefficients[i + 8], quantization_table[i + 8]);
            let s2 = dequantize(coefficients[i + 16], quantization_table[i + 16]);
            let s3 = dequantize(coefficients[i + 24], quantization_table[i + 24]);
            let s4 = dequantize(coefficients[i + 32], quantization_table[i + 32]);
            let s5 = dequantize(coefficients[i + 40], quantization_table[i + 40]);
            let s6 = dequantize(coefficients[i + 48], quantization_table[i + 48]);
            let s7 = dequantize(coefficients[i + 56], quantization_table[i + 56]);

            let Kernel {
                xs: [x0, x1, x2, x3],
                ts: [t0, t1, t2, t3],
            } = kernel(
                [s0, s1, s2, s3, s4, s5, s6, s7],
                // constants scaled things up by 1<<12; let's bring them back
                // down, but keep 2 extra bits of precision
                512,
            );

            temp[i] = (x0 + t3) >> 10;
            temp[i + 56] = (x0 - t3) >> 10;
            temp[i + 8] = (x1 + t2) >> 10;
            temp[i + 48] = (x1 - t2) >> 10;
            temp[i + 16] = (x2 + t1) >> 10;
            temp[i + 40] = (x2 - t1) >> 10;
            temp[i + 24] = (x3 + t0) >> 10;
            temp[i + 32] = (x3 - t0) >> 10;
        }
    }

    for (chunk, output_chunk) in temp.chunks_exact(8).zip(output) {
        let chunk = <&[_; 8]>::try_from(chunk).unwrap();

        // constants scaled things up by 1<<12, plus we had 1<<2 from first
        // loop, plus horizontal and vertical each scale by sqrt(8) so together
        // we've got an extra 1<<3, so 1<<17 total we need to remove.
        // so we want to round that, which means adding 0.5 * 1<<17,
        // aka 65536. Also, we'll end up with -128 to 127 that we want
        // to encode as 0..255 by adding 128, so we'll add that before the shift
        const X_SCALE: i32 = 65536 + (128 << 17);

        // eliminate downstream bounds checks
        let output_chunk = &mut output_chunk[..8];

        // TODO When the minimum rust version supports it
        // let [s0, rest @ ..] = chunk;
        let (s0, rest) = chunk.split_first().unwrap();
        if *rest == [Wrapping(0); 7] {
            let dcterm = stbi_clamp((stbi_fsh(*s0) + Wrapping(X_SCALE)) >> 17);
            output_chunk[0] = dcterm;
            output_chunk[1] = dcterm;
            output_chunk[2] = dcterm;
            output_chunk[3] = dcterm;
            output_chunk[4] = dcterm;
            output_chunk[5] = dcterm;
            output_chunk[6] = dcterm;
            output_chunk[7] = dcterm;
        } else {
            let Kernel {
                xs: [x0, x1, x2, x3],
                ts: [t0, t1, t2, t3],
            } = kernel(*chunk, X_SCALE);

            output_chunk[0] = stbi_clamp((x0 + t3) >> 17);
            output_chunk[7] = stbi_clamp((x0 - t3) >> 17);
            output_chunk[1] = stbi_clamp((x1 + t2) >> 17);
            output_chunk[6] = stbi_clamp((x1 - t2) >> 17);
            output_chunk[2] = stbi_clamp((x2 + t1) >> 17);
            output_chunk[5] = stbi_clamp((x2 - t1) >> 17);
            output_chunk[3] = stbi_clamp((x3 + t0) >> 17);
            output_chunk[4] = stbi_clamp((x3 - t0) >> 17);
        }
    }
}

struct Kernel {
    xs: [Wrapping<i32>; 4],
    ts: [Wrapping<i32>; 4],
}

#[inline]
fn kernel_x([s0, s2, s4, s6]: [Wrapping<i32>; 4], x_scale: i32) -> [Wrapping<i32>; 4] {
    // Even `chunk` indicies
    let (t2, t3);
    {
        let p2 = s2;
        let p3 = s6;

        let p1 = (p2 + p3) * stbi_f2f(0.5411961);
        t2 = p1 + p3 * stbi_f2f(-1.847759065);
        t3 = p1 + p2 * stbi_f2f(0.765366865);
    }

    let (t0, t1);
    {
        let p2 = s0;
        let p3 = s4;

        t0 = stbi_fsh(p2 + p3);
        t1 = stbi_fsh(p2 - p3);
    }

    let x0 = t0 + t3;
    let x3 = t0 - t3;
    let x1 = t1 + t2;
    let x2 = t1 - t2;

    let x_scale = Wrapping(x_scale);

    [x0 + x_scale, x1 + x_scale, x2 + x_scale, x3 + x_scale]
}

#[inline]
fn kernel_t([s1, s3, s5, s7]: [Wrapping<i32>; 4]) -> [Wrapping<i32>; 4] {
    // Odd `chunk` indicies
    let mut t0 = s7;
    let mut t1 = s5;
    let mut t2 = s3;
    let mut t3 = s1;

    let p3 = t0 + t2;
    let p4 = t1 + t3;
    let p1 = t0 + t3;
    let p2 = t1 + t2;
    let p5 = (p3 + p4) * stbi_f2f(1.175875602);

    t0 *= stbi_f2f(0.298631336);
    t1 *= stbi_f2f(2.053119869);
    t2 *= stbi_f2f(3.072711026);
    t3 *= stbi_f2f(1.501321110);

    let p1 = p5 + p1 * stbi_f2f(-0.899976223);
    let p2 = p5 + p2 * stbi_f2f(-2.562915447);
    let p3 = p3 * stbi_f2f(-1.961570560);
    let p4 = p4 * stbi_f2f(-0.390180644);

    t3 += p1 + p4;
    t2 += p2 + p3;
    t1 += p2 + p4;
    t0 += p1 + p3;

    [t0, t1, t2, t3]
}

#[inline]
fn kernel([s0, s1, s2, s3, s4, s5, s6, s7]: [Wrapping<i32>; 8], x_scale: i32) -> Kernel {
    Kernel {
        xs: kernel_x([s0, s2, s4, s6], x_scale),
        ts: kernel_t([s1, s3, s5, s7]),
    }
}

#[inline(always)]
fn dequantize(c: i16, q: u16) -> Wrapping<i32> {
    Wrapping(i32::from(c) * i32::from(q))
}

// 4x4 and 2x2 IDCT based on Rakesh Dugad and Narendra Ahuja: "A Fast Scheme for Image Size Change in the Compressed Domain" (2001).
// http://sylvana.net/jpegcrop/jidctred/
fn dequantize_and_idct_block_4x4(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    debug_assert_eq!(coefficients.len(), 64);
    let mut temp = [Wrapping(0i32); 4 * 4];

    const CONST_BITS: usize = 12;
    const PASS1_BITS: usize = 2;
    const FINAL_BITS: usize = CONST_BITS + PASS1_BITS + 3;

    // columns
    for i in 0..4 {
        let s0 = Wrapping(coefficients[i + 8 * 0] as i32 * quantization_table[i + 8 * 0] as i32);
        let s1 = Wrapping(coefficients[i + 8 * 1] as i32 * quantization_table[i + 8 * 1] as i32);
        let s2 = Wrapping(coefficients[i + 8 * 2] as i32 * quantization_table[i + 8 * 2] as i32);
        let s3 = Wrapping(coefficients[i + 8 * 3] as i32 * quantization_table[i + 8 * 3] as i32);

        let x0 = (s0 + s2) << PASS1_BITS;
        let x2 = (s0 - s2) << PASS1_BITS;

        let p1 = (s1 + s3) * stbi_f2f(0.541196100);
        let t0 = (p1 + s3 * stbi_f2f(-1.847759065) + Wrapping(512)) >> (CONST_BITS - PASS1_BITS);
        let t2 = (p1 + s1 * stbi_f2f(0.765366865) + Wrapping(512)) >> (CONST_BITS - PASS1_BITS);

        temp[i + 4 * 0] = x0 + t2;
        temp[i + 4 * 3] = x0 - t2;
        temp[i + 4 * 1] = x2 + t0;
        temp[i + 4 * 2] = x2 - t0;
    }

    for i in 0..4 {
        let s0 = temp[i * 4 + 0];
        let s1 = temp[i * 4 + 1];
        let s2 = temp[i * 4 + 2];
        let s3 = temp[i * 4 + 3];

        let x0 = (s0 + s2) << CONST_BITS;
        let x2 = (s0 - s2) << CONST_BITS;

        let p1 = (s1 + s3) * stbi_f2f(0.541196100);
        let t0 = p1 + s3 * stbi_f2f(-1.847759065);
        let t2 = p1 + s1 * stbi_f2f(0.765366865);

        // constants scaled things up by 1<<12, plus we had 1<<2 from first
        // loop, plus horizontal and vertical each scale by sqrt(8) so together
        // we've got an extra 1<<3, so 1<<17 total we need to remove.
        // so we want to round that, which means adding 0.5 * 1<<17,
        // aka 65536. Also, we'll end up with -128 to 127 that we want
        // to encode as 0..255 by adding 128, so we'll add that before the shift
        let x0 = x0 + Wrapping(1 << (FINAL_BITS - 1)) + Wrapping(128 << FINAL_BITS);
        let x2 = x2 + Wrapping(1 << (FINAL_BITS - 1)) + Wrapping(128 << FINAL_BITS);

        let output = &mut output[i * output_linestride..][..4];
        output[0] = stbi_clamp((x0 + t2) >> FINAL_BITS);
        output[3] = stbi_clamp((x0 - t2) >> FINAL_BITS);
        output[1] = stbi_clamp((x2 + t0) >> FINAL_BITS);
        output[2] = stbi_clamp((x2 - t0) >> FINAL_BITS);
    }
}

fn dequantize_and_idct_block_2x2(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    debug_assert_eq!(coefficients.len(), 64);

    const SCALE_BITS: usize = 3;

    // Column 0
    let s00 = Wrapping(coefficients[8 * 0] as i32 * quantization_table[8 * 0] as i32);
    let s10 = Wrapping(coefficients[8 * 1] as i32 * quantization_table[8 * 1] as i32);

    let x0 = s00 + s10;
    let x2 = s00 - s10;

    // Column 1
    let s01 = Wrapping(coefficients[8 * 0 + 1] as i32 * quantization_table[8 * 0 + 1] as i32);
    let s11 = Wrapping(coefficients[8 * 1 + 1] as i32 * quantization_table[8 * 1 + 1] as i32);

    let x1 = s01 + s11;
    let x3 = s01 - s11;

    let x0 = x0 + Wrapping(1 << (SCALE_BITS - 1)) + Wrapping(128 << SCALE_BITS);
    let x2 = x2 + Wrapping(1 << (SCALE_BITS - 1)) + Wrapping(128 << SCALE_BITS);

    // Row 0
    output[0] = stbi_clamp((x0 + x1) >> SCALE_BITS);
    output[1] = stbi_clamp((x0 - x1) >> SCALE_BITS);

    // Row 1
    output[output_linestride + 0] = stbi_clamp((x2 + x3) >> SCALE_BITS);
    output[output_linestride + 1] = stbi_clamp((x2 - x3) >> SCALE_BITS);
}

fn dequantize_and_idct_block_1x1(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    _output_linestride: usize,
    output: &mut [u8],
) {
    debug_assert_eq!(coefficients.len(), 64);

    let s0 = (Wrapping(coefficients[0] as i32 * quantization_table[0] as i32) + Wrapping(128 * 8))
        / Wrapping(8);
    output[0] = stbi_clamp(s0);
}

// take a -128..127 value and stbi__clamp it and convert to 0..255
fn stbi_clamp(x: Wrapping<i32>) -> u8 {
    x.0.clamp(0, 255) as u8
}

fn stbi_f2f(x: f32) -> Wrapping<i32> {
    Wrapping((x * 4096.0 + 0.5) as i32)
}

fn stbi_fsh(x: Wrapping<i32>) -> Wrapping<i32> {
    x << 12
}

#[test]
fn test_dequantize_and_idct_block_8x8() {
    #[rustfmt::skip]
    let coefficients: [i16; 8 * 8] = [
        -14, -39, 58, -2, 3, 3, 0, 1,
        11, 27, 4, -3, 3, 0, 1, 0,
        -6, -13, -9, -1, -2, -1, 0, 0,
        -4, 0, -1, -2, 0, 0, 0, 0,
        3, 0, 0, 0, 0, 0, 0, 0,
        -3, -2, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0
    ];

    #[rustfmt::skip]
    let quantization_table: [u16; 8 * 8] = [
        8, 6, 5, 8, 12, 20, 26, 31,
        6, 6, 7, 10, 13, 29, 30, 28,
        7, 7, 8, 12, 20, 29, 35, 28,
        7, 9, 11, 15, 26, 44, 40, 31,
        9, 11, 19, 28, 34, 55, 52, 39,
        12, 18, 28, 32, 41, 52, 57, 46,
        25, 32, 39, 44, 52, 61, 60, 51,
        36, 46, 48, 49, 56, 50, 52, 50
    ];
    let output_linestride: usize = 8;
    let mut output = [0u8; 8 * 8];
    dequantize_and_idct_block_8x8(
        &coefficients,
        &quantization_table,
        output_linestride,
        &mut output,
    );
    #[rustfmt::skip]
    let expected_output = [
        118, 92, 110, 83, 77, 93, 144, 198,
        172, 116, 114, 87, 78, 93, 146, 191,
        194, 107, 91, 76, 71, 93, 160, 198,
        196, 100, 80, 74, 67, 92, 174, 209,
        182, 104, 88, 81, 68, 89, 178, 206,
        105, 64, 59, 59, 63, 94, 183, 201,
        35, 27, 28, 37, 72, 121, 203, 204,
        37, 45, 41, 47, 98, 154, 223, 208
    ];
    for i in 0..64 {
        assert!((output[i] as i16 - expected_output[i] as i16).abs() <= 1);
    }
}

#[test]
fn test_dequantize_and_idct_block_8x8_all_zero() {
    let mut output = [0u8; 8 * 8];
    dequantize_and_idct_block_8x8(&[0; 8 * 8], &[666; 8 * 8], 8, &mut output);
    assert_eq!(&output[..], &[128; 8 * 8][..]);
}

#[test]
fn test_dequantize_and_idct_block_8x8_saturated() {
    // Arch-specific IDCT implementations need not handle i16::MAX values.
    #[cfg(not(feature = "platform_independent"))]
    if crate::arch::get_dequantize_and_idct_block_8x8().is_some() {
        return;
    }
    let mut output = [0u8; 8 * 8];
    dequantize_and_idct_block_8x8(&[i16::MAX; 8 * 8], &[u16::MAX; 8 * 8], 8, &mut output);
    #[rustfmt::skip]
    let expected = [
        0, 0, 0, 255, 255, 0, 0, 255,
        0, 0, 215, 0, 0, 255, 255, 0,
        255, 255, 255, 255, 255, 0, 0, 255,
        0, 0, 255, 0, 255, 0, 255, 255,
        0, 0, 255, 255, 0, 255, 0, 0,
        255, 255, 0, 255, 255, 255, 170, 0,
        0, 255, 0, 0, 0, 0, 0, 255,
        255, 255, 0, 255, 0, 255, 0, 0
    ];
    assert_eq!(&output[..], &expected[..]);
}
