//! Performing the inverse discrete wavelet transform, as specified in Annex F.

use alloc::vec;
use alloc::vec::Vec;
use rhwp_codec_resource::VecResource as _;

use super::build::{Decomposition, SubBand};
use super::codestream::WaveletTransform;
use super::decode::{DecompositionStorage, TileDecodeContext};
use super::rect::IntRect;
use crate::j2c::Header;
use crate::math::{self, Level, SIMD_WIDTH, Simd, dispatch, f32x8};

/// The output from performing the IDWT operation.
pub(crate) struct IDWTOutput {
    /// The buffer that will hold the final coefficients.
    pub(crate) coefficients: Vec<f32>,
    /// The rect that the coefficients belong to. This will be equivalent
    /// to the rectangle that forms the smallest decomposition level. It does
    /// not have to be equivalent to the original size of the tile, as the
    /// sub-bands that form a tile aren't necessarily aligned to it. Therefore,
    /// the samples need to be trimmed to the tile rectangle afterward.
    pub(crate) rect: IntRect,
}

impl Default for IDWTOutput {
    fn default() -> Self {
        Self {
            coefficients: vec![],
            rect: IntRect::from_ltrb(0, 0, u32::MAX, u32::MAX),
        }
    }
}

struct IDWTTempOutput {
    rect: IntRect,
}

/// Apply the inverse discrete wavelet transform (see Annex F). The output
/// will be transformed samples covering the rectangle of the smallest
/// decomposition level.
pub(crate) fn apply(
    storage: &DecompositionStorage<'_>,
    tile_ctx: &mut TileDecodeContext,
    component_idx: usize,
    header: &Header<'_>,
    transform: WaveletTransform,
) -> crate::error::Result<()> {
    let tile_decompositions = &storage.tile_decompositions[component_idx];

    let mut decompositions = &storage.decompositions[tile_decompositions.decompositions.clone()];
    // If we requested a lower resolution level, we can skip some decompositions.
    decompositions = &decompositions[..decompositions
        .len()
        .saturating_sub(header.skipped_resolution_levels as usize)];
    let ll_sub_band = &storage.sub_bands[tile_decompositions.first_ll_sub_band];

    // To explain a bit why we have this scratch buffer and another coefficient
    // buffer: During IDWT, we need to continuously interleave the 4 sub-bands
    // into a new buffer, which is then either returned or used as the input
    // for the next decomposition, etc. It would be very inefficient if we
    // kept allocating new buffers each time. Therefore, we try to reuse them,
    // not only for all decompositions of a single tile, but all decompositions
    // of _all_ tiles.
    // Due to the fact that the output from the previous iteration might be
    // used as the input of the next, we need two separate buffers, which
    // are continuously swapped.
    let (scratch_buf, output) = (&mut tile_ctx.idwt_scratch_buffer, &mut tile_ctx.idwt_output);

    let estimate_buffer_size = |decomposition: &Decomposition| {
        let total_width = decomposition.rect.width() as usize;
        let total_height = decomposition.rect.height() as usize;

        let min = rhwp_codec_resource::product(&[total_width, total_height])?;
        // Different sub-bands can have shifts by one, so add padding
        // for the maximum case.
        let max = rhwp_codec_resource::product(&[
            total_width
                .checked_add(1)
                .ok_or_else(rhwp_codec_resource::reject)?,
            total_height
                .checked_add(1)
                .ok_or_else(rhwp_codec_resource::reject)?,
        ])?;

        Ok::<_, rhwp_codec_resource::ResourceError>((min, max))
    };

    if decompositions.is_empty() {
        // Single decomposition, just copy the coefficients from the sub-band.
        output.coefficients.clear();
        output
            .coefficients
            .rhwp_extend_from_slice(&storage.coefficients[ll_sub_band.coefficients.clone()])?;

        output.rect = ll_sub_band.rect;

        return Ok(());
    }

    // The coefficient array will always be the one that holds the coefficients
    // from the highest decomposition. Therefore, reserve as much.
    let (s_min, s_max) = estimate_buffer_size(decompositions.last().unwrap())?;
    if output.coefficients.len() < s_min {
        output
            .coefficients
            .rhwp_reserve(s_max - output.coefficients.len())?;
    }

    if decompositions.len() > 1 {
        // Due to the above, the intermediate buffer will never need more than
        // the second-highest decomposition.
        let (s_min, s_max) = estimate_buffer_size(&decompositions[decompositions.len() - 2])?;

        if scratch_buf.len() < s_min {
            scratch_buf.rhwp_reserve(s_max - scratch_buf.len())?;
        }
    }

    // Determine which buffer we should use first, such that the `coefficients`
    // array will always hold the final values.
    let mut use_scratch = decompositions.len().is_multiple_of(2);

    let mut temp_output = filter_2d(
        IDWTInput::from_sub_band(ll_sub_band, storage),
        if use_scratch {
            scratch_buf
        } else {
            &mut output.coefficients
        },
        &decompositions[0],
        transform,
        storage,
    )?;

    for decomposition in decompositions.iter().skip(1) {
        use_scratch = !use_scratch;

        temp_output = if use_scratch {
            filter_2d(
                IDWTInput::from_output(&output.coefficients),
                scratch_buf,
                decomposition,
                transform,
                storage,
            )?
        } else {
            filter_2d(
                IDWTInput::from_output(scratch_buf),
                &mut output.coefficients,
                decomposition,
                transform,
                storage,
            )?
        };
    }

    output.rect = temp_output.rect;
    Ok(())
}

struct IDWTInput<'a> {
    coefficients: &'a [f32],
}

impl<'a> IDWTInput<'a> {
    fn from_sub_band(sub_band: &'a SubBand, storage: &'a DecompositionStorage<'_>) -> Self {
        IDWTInput {
            coefficients: &storage.coefficients[sub_band.coefficients.clone()],
        }
    }

    fn from_output(coefficients: &'a [f32]) -> Self {
        IDWTInput { coefficients }
    }
}

/// The `2D_SR` procedure illustrated in Figure F.6.
fn filter_2d(
    // The LL sub band of the given decomposition level.
    input: IDWTInput<'_>,
    coefficients: &mut Vec<f32>,
    decomposition: &Decomposition,
    transform: WaveletTransform,
    storage: &DecompositionStorage<'_>,
) -> crate::error::Result<IDWTTempOutput> {
    // First interleave all sub-bands into a single buffer.
    interleave_samples(input, decomposition, coefficients, storage)?;
    rhwp_codec_resource::work(rhwp_codec_resource::product(&[
        decomposition.rect.width() as usize,
        decomposition.rect.height() as usize,
        64,
    ])?)?;

    if decomposition.rect.width() > 0 && decomposition.rect.height() > 0 {
        filter_horizontal(coefficients, decomposition.rect, transform);
        filter_vertical(coefficients, decomposition.rect, transform);
    }

    Ok(IDWTTempOutput {
        rect: decomposition.rect,
    })
}

/// The `2D_INTERLEAVE` procedure described in F.3.3.
fn interleave_samples(
    input: IDWTInput<'_>,
    decomposition: &Decomposition,
    coefficients: &mut Vec<f32>,
    storage: &DecompositionStorage<'_>,
) -> crate::error::Result<()> {
    let level = Level::new();
    dispatch!(level, simd => {
        interleave_samples_inner::<_>(simd, input, decomposition, coefficients, storage)
    })
}

#[inline(always)]
fn interleave_samples_inner<S: Simd>(
    simd: S,
    input: IDWTInput<'_>,
    decomposition: &Decomposition,
    coefficients: &mut Vec<f32>,
    storage: &DecompositionStorage<'_>,
) -> crate::error::Result<()> {
    let width = decomposition.rect.width() as usize;
    let height = decomposition.rect.height() as usize;

    // Just a sanity check. We should have allocated enough upfront before
    // starting the IDWT.
    let samples = rhwp_codec_resource::product(&[width, height])?;
    rhwp_codec_resource::work(samples)?;

    // The cleaner way would be to first clear and then resize, so that we
    // have a clean buffer with just zeroes. However, this is actually not
    // necessary, because when interleaving and generating the border values
    // we will replace all the data anyway, so we can save the cost of
    // the clear operation.
    coefficients.rhwp_resize(samples, 0.0)?;

    let IntRect {
        x0: u0,
        x1: u1,
        y0: v0,
        y1: v1,
    } = decomposition.rect;

    let ll = input.coefficients;
    let hl = &storage.coefficients[storage.sub_bands[decomposition.sub_bands[0]]
        .coefficients
        .clone()];
    let lh = &storage.coefficients[storage.sub_bands[decomposition.sub_bands[1]]
        .coefficients
        .clone()];
    let hh = &storage.coefficients[storage.sub_bands[decomposition.sub_bands[2]]
        .coefficients
        .clone()];

    // See Figure F.8.
    let num_u_low = (u1.div_ceil(2) - u0.div_ceil(2)) as usize;
    let num_u_high = (u1 / 2 - u0 / 2) as usize;
    let num_v_low = (v1.div_ceil(2) - v0.div_ceil(2)) as usize;
    let num_v_high = (v1 / 2 - v0 / 2) as usize;

    // Depending on whether the start row is even or odd, either LL/HL comes first
    // or HL/HH.

    let (first_w, second_w) = if u0 % 2 == 0 {
        (num_u_low, num_u_high)
    } else {
        (num_u_high, num_u_low)
    };

    let even_row_start = if v0 % 2 == 0 { 0 } else { 1 };
    let odd_row_start = if v0 % 2 == 0 { 1 } else { 0 };

    // Determine whether LL or HL is the band in the first column.
    let (first_even, second_even) = if u0 % 2 == 0 { (ll, hl) } else { (hl, ll) };
    interleave_rows(
        simd,
        first_even,
        second_even,
        first_w,
        second_w,
        coefficients,
        width,
        height,
        even_row_start,
        num_v_low,
    );

    // Determine whether LH or HH is the band in the first column.
    let (first_odd, second_odd) = if u0 % 2 == 0 { (lh, hh) } else { (hh, lh) };
    interleave_rows(
        simd,
        first_odd,
        second_odd,
        first_w,
        second_w,
        coefficients,
        width,
        height,
        odd_row_start,
        num_v_high,
    );
    Ok(())
}

#[inline(always)]
fn interleave_rows<S: Simd>(
    simd: S,
    first_band: &[f32],
    second_band: &[f32],
    first_w: usize,
    second_w: usize,
    output: &mut [f32],
    width: usize,
    height: usize,
    start_row: usize,
    num_rows: usize,
) {
    for v in 0..num_rows {
        let out_row = start_row + v * 2;
        if out_row >= height {
            break;
        }

        let first_row = &first_band[v * first_w..][..first_w];
        let second_row = &second_band[v * second_w..][..second_w];
        let out_slice = &mut output[out_row * width..][..width];

        interleave_row(simd, first_row, second_row, out_slice);
    }
}

#[inline(always)]
fn interleave_row<S: Simd>(simd: S, first: &[f32], second: &[f32], output: &mut [f32]) {
    let num_pairs = first.len().min(second.len());
    let simd_chunks = num_pairs / SIMD_WIDTH;

    // Process as much as possible using SIMD.
    for i in 0..simd_chunks {
        let base = i * SIMD_WIDTH;
        let f = f32x8::from_slice(simd, &first[base..base + SIMD_WIDTH]);
        let s = f32x8::from_slice(simd, &second[base..base + SIMD_WIDTH]);

        f.zip_low(s).store(&mut output[base * 2..]);
        f.zip_high(s).store(&mut output[base * 2 + SIMD_WIDTH..]);
    }

    // Scalar remainder.
    for i in (simd_chunks * SIMD_WIDTH)..num_pairs {
        output[i * 2] = first[i];
        output[i * 2 + 1] = second[i];
    }

    // Handle extra element if first is longer.
    if first.len() > num_pairs {
        output[num_pairs * 2] = first[num_pairs];
    }
}

/// The `HOR_SR` procedure from F.3.4.
fn filter_horizontal(coefficients: &mut [f32], rect: IntRect, transform: WaveletTransform) {
    let width = rect.width() as usize;

    for scanline in coefficients
        .chunks_exact_mut(width)
        .take(rect.height() as usize)
    {
        filter_row(scanline, width, rect.x0 as usize, transform);
    }
}

/// The `1D_SR` procedure from F.3.6.
fn filter_row(scanline: &mut [f32], width: usize, x0: usize, transform: WaveletTransform) {
    if width == 1 {
        if !x0.is_multiple_of(2) {
            scanline[0] *= 0.5;
        }

        return;
    }

    match transform {
        WaveletTransform::Reversible53 => reversible_filter_53r(scanline, width, x0),
        WaveletTransform::Irreversible97 => irreversible_filter_97i(scanline, width, x0),
    }
}

/// The 1D FILTER 5-3R procedure from F.3.8.1.
fn reversible_filter_53r(scanline: &mut [f32], width: usize, x0: usize) {
    let first_even = x0 % 2;
    let first_odd = 1 - first_even;

    // Equation (F-5).
    // Originally: for i in (start / 2)..(end / 2 + 1).
    filter_step_horizontal(
        scanline,
        width,
        first_even,
        #[inline(always)]
        |s, left, right| s - math::floor_f32(math::mul_add(left + right, 0.25, 0.5)),
    );

    // Equation (F-6).
    // Originally: for i in (start / 2)..(end / 2).
    filter_step_horizontal(
        scanline,
        width,
        first_odd,
        #[inline(always)]
        |s, left, right| s + math::floor_f32((left + right) * 0.5),
    );
}

/// The 1D Filter 9-7I procedure from F.3.8.2.
fn irreversible_filter_97i(scanline: &mut [f32], width: usize, x0: usize) {
    // Table F.4.
    const NEG_ALPHA: f32 = 1.586_134_3;
    const NEG_BETA: f32 = 0.052_980_117;
    const NEG_GAMMA: f32 = -0.882_911_1;
    const NEG_DELTA: f32 = -0.443_506_87;
    const KAPPA: f32 = 1.230_174_1;
    const INV_KAPPA: f32 = 1.0 / KAPPA;

    let first_even = x0 % 2;
    let first_odd = 1 - first_even;

    let (k0, k1) = if first_even == 0 {
        (KAPPA, INV_KAPPA)
    } else {
        (INV_KAPPA, KAPPA)
    };

    // Step 1 and 2.
    // Originally: for i in (start / 2 - 1)..(end / 2 + 2).
    // Originally: for i in (start / 2 - 2)..(end / 2 + 2).
    for i in (0..width.saturating_sub(1)).step_by(2) {
        scanline[i] *= k0;
        scanline[i + 1] *= k1;
    }
    if width % 2 == 1 {
        scanline[width - 1] *= k0;
    }

    // Step 3.
    // Originally: for i in (start / 2 - 1)..(end / 2 + 2).
    filter_step_horizontal(
        scanline,
        width,
        first_even,
        #[inline(always)]
        |s, left, right| math::mul_add(left + right, NEG_DELTA, s),
    );

    // Step 4.
    // Originally: for i in (start / 2 - 1)..((x0 + width) / 2 + 1).
    filter_step_horizontal(
        scanline,
        width,
        first_odd,
        #[inline(always)]
        |s, left, right| math::mul_add(left + right, NEG_GAMMA, s),
    );

    // Step 5.
    // Originally: for i in (start / 2)..(end / 2 + 1).
    filter_step_horizontal(
        scanline,
        width,
        first_even,
        #[inline(always)]
        |s, left, right| math::mul_add(left + right, NEG_BETA, s),
    );

    // Step 6.
    // Originally: for i in (start / 2)..(end / 2).
    filter_step_horizontal(
        scanline,
        width,
        first_odd,
        #[inline(always)]
        |s, left, right| math::mul_add(left + right, NEG_ALPHA, s),
    );
}

#[inline(always)]
fn filter_step_horizontal(
    scanline: &mut [f32],
    width: usize,
    first: usize,
    f: impl Fn(f32, f32, f32) -> f32,
) {
    if first == 0 {
        let left = periodic_symmetric_extension_left(0, 1);
        let right = periodic_symmetric_extension_right(0, 1, width);
        scanline[0] = f(scanline[0], scanline[left], scanline[right]);
    }

    let middle_start = if first == 0 { 2 } else { 1 };
    for i in (middle_start..width - 1).step_by(2) {
        scanline[i] = f(scanline[i], scanline[i - 1], scanline[i + 1]);
    }

    if width > 1 && (width - 1) % 2 == first {
        let i = width - 1;
        let left = periodic_symmetric_extension_left(i, 1);
        let right = periodic_symmetric_extension_right(i, 1, width);
        scanline[i] = f(scanline[i], scanline[left], scanline[right]);
    }
}

#[inline(always)]
fn filter_step_vertical<S: Simd>(
    simd: S,
    scanline: &mut [f32],
    height: usize,
    width: usize,
    simd_width: usize,
    first: usize,
    f_simd: impl Fn(f32x8<S>, f32x8<S>, f32x8<S>) -> f32x8<S>,
    f_scalar: impl Fn(f32, f32, f32) -> f32,
) {
    for row in (first..height).step_by(2) {
        let row_above = periodic_symmetric_extension_left(row, 1);
        let row_below = periodic_symmetric_extension_right(row, 1, height);

        // Process SIMD chunks.
        for base_column in (0..simd_width).step_by(SIMD_WIDTH) {
            let s1 = f32x8::from_slice(simd, &scanline[row * width + base_column..][..SIMD_WIDTH]);
            let s2 = f32x8::from_slice(
                simd,
                &scanline[row_above * width + base_column..][..SIMD_WIDTH],
            );
            let s3 = f32x8::from_slice(
                simd,
                &scanline[row_below * width + base_column..][..SIMD_WIDTH],
            );

            let result = f_simd(s1, s2, s3);
            result.store(&mut scanline[row * width + base_column..][..SIMD_WIDTH]);
        }

        // Process scalar remainder.
        for col in simd_width..width {
            let s1 = scanline[row * width + col];
            let s2 = scanline[row_above * width + col];
            let s3 = scanline[row_below * width + col];
            scanline[row * width + col] = f_scalar(s1, s2, s3);
        }
    }
}

/// Part of the `1D_EXTR` procedure, defined in F.3.7.
///
/// Applies the period symmetric extension on the left side.
#[inline(always)]
fn periodic_symmetric_extension_left(idx: usize, offset: usize) -> usize {
    offset.abs_diff(idx)
}

/// Part of the `1D_EXTR` procedure, defined in F.3.7.
///
/// Applies the period symmetric extension on the right side.
#[inline(always)]
fn periodic_symmetric_extension_right(idx: usize, offset: usize, length: usize) -> usize {
    let new_idx = idx + offset;
    if new_idx >= length {
        let overshoot = new_idx - length;
        length - 2 - overshoot
    } else {
        new_idx
    }
}

/// The `VER_SR` procedure from F.3.5.
fn filter_vertical(coefficients: &mut [f32], rect: IntRect, transform: WaveletTransform) {
    dispatch!(Level::new(), simd => filter_vertical_impl(simd, coefficients, rect, transform));
}

#[inline(always)]
fn filter_vertical_impl<S: Simd>(
    simd: S,
    scanline: &mut [f32],
    rect: IntRect,
    transform: WaveletTransform,
) {
    let width = rect.width() as usize;
    let height = rect.height() as usize;
    let y0 = rect.y0 as usize;

    if height == 1 {
        if !y0.is_multiple_of(2) {
            let simd_width = width / SIMD_WIDTH * SIMD_WIDTH;
            for base_column in (0..simd_width).step_by(SIMD_WIDTH) {
                let mut loaded = f32x8::from_slice(simd, &scanline[base_column..][..SIMD_WIDTH]);
                loaded *= 0.5;
                loaded.store(&mut scanline[base_column..][..SIMD_WIDTH]);
            }

            // Scalar remainder.
            #[allow(clippy::needless_range_loop)]
            for col in simd_width..width {
                scanline[col] *= 0.5;
            }
        }
        return;
    }

    match transform {
        WaveletTransform::Reversible53 => {
            reversible_filter_53r_simd(simd, scanline, height, width, y0);
        }
        WaveletTransform::Irreversible97 => {
            irreversible_filter_97i_simd(simd, scanline, height, width, y0);
        }
    }
}

/// The 1D FILTER 5-3R procedure from F.3.8.1.
#[inline(always)]
fn reversible_filter_53r_simd<S: Simd>(
    simd: S,
    scanline: &mut [f32],
    height: usize,
    width: usize,
    y0: usize,
) {
    let first_even = y0 % 2;
    let first_odd = 1 - first_even;
    let simd_width = width / SIMD_WIDTH * SIMD_WIDTH;

    // Equation (F-5).
    // Originally: for i in (start / 2)..(end / 2 + 1).
    filter_step_vertical(
        simd,
        scanline,
        height,
        width,
        simd_width,
        first_even,
        #[inline(always)]
        |s1, s2, s3| s1 - ((s2 + s3 + 2.0) * 0.25).floor(),
        #[inline(always)]
        |s1, s2, s3| s1 - math::floor_f32(math::mul_add(s2 + s3, 0.25, 0.5)),
    );

    // Equation (F-6).
    // Originally: for i in (start / 2)..(end / 2).
    filter_step_vertical(
        simd,
        scanline,
        height,
        width,
        simd_width,
        first_odd,
        #[inline(always)]
        |s1, s2, s3| s1 + ((s2 + s3) * 0.5).floor(),
        #[inline(always)]
        |s1, s2, s3| s1 + math::floor_f32((s2 + s3) * 0.5),
    );
}

/// The 1D Filter 9-7I procedure from F.3.8.2.
#[inline(always)]
fn irreversible_filter_97i_simd<S: Simd>(
    simd: S,
    scanline: &mut [f32],
    height: usize,
    width: usize,
    y0: usize,
) {
    const NEG_ALPHA: f32 = 1.586_134_3;
    const NEG_BETA: f32 = 0.052_980_117;
    const NEG_GAMMA: f32 = -0.882_911_1;
    const NEG_DELTA: f32 = -0.443_506_87;
    const KAPPA: f32 = 1.230_174_1;

    const INV_KAPPA: f32 = 1.0 / KAPPA;

    let neg_alpha = f32x8::splat(simd, NEG_ALPHA);
    let neg_beta = f32x8::splat(simd, NEG_BETA);
    let neg_gamma = f32x8::splat(simd, NEG_GAMMA);
    let neg_delta = f32x8::splat(simd, NEG_DELTA);
    let kappa = f32x8::splat(simd, KAPPA);
    let inv_kappa = f32x8::splat(simd, INV_KAPPA);

    // Determine which local row indices correspond to even/odd global positions.
    let first_even = y0 % 2;
    let first_odd = 1 - first_even;
    let simd_width = width / SIMD_WIDTH * SIMD_WIDTH;

    let (k0, k1, k0_simd, k1_simd) = if first_even == 0 {
        (KAPPA, INV_KAPPA, kappa, inv_kappa)
    } else {
        (INV_KAPPA, KAPPA, inv_kappa, kappa)
    };

    // Step 1 and 2.
    // Originally: for i in (start / 2 - 1)..(end / 2 + 2).
    // Originally: for i in (start / 2 - 2)..(end / 2 + 2).
    for row in (0..height.saturating_sub(1)).step_by(2) {
        for base_column in (0..simd_width).step_by(SIMD_WIDTH) {
            let mut vals0 =
                f32x8::from_slice(simd, &scanline[row * width + base_column..][..SIMD_WIDTH]);
            let mut vals1 = f32x8::from_slice(
                simd,
                &scanline[(row + 1) * width + base_column..][..SIMD_WIDTH],
            );
            vals0 = vals0 * k0_simd;
            vals1 = vals1 * k1_simd;
            vals0.store(&mut scanline[row * width + base_column..][..SIMD_WIDTH]);
            vals1.store(&mut scanline[(row + 1) * width + base_column..][..SIMD_WIDTH]);
        }
        for col in simd_width..width {
            scanline[row * width + col] *= k0;
            scanline[(row + 1) * width + col] *= k1;
        }
    }

    if height % 2 == 1 {
        let row = height - 1;
        for base_column in (0..simd_width).step_by(SIMD_WIDTH) {
            let mut vals =
                f32x8::from_slice(simd, &scanline[row * width + base_column..][..SIMD_WIDTH]);
            vals = vals * k0_simd;
            vals.store(&mut scanline[row * width + base_column..][..SIMD_WIDTH]);
        }
        for col in simd_width..width {
            scanline[row * width + col] *= k0;
        }
    }

    // Step 3.
    // Originally: for i in (start / 2 - 1)..(end / 2 + 2).
    filter_step_vertical(
        simd,
        scanline,
        height,
        width,
        simd_width,
        first_even,
        #[inline(always)]
        |s1, s2, s3| (s2 + s3).mul_add(neg_delta, s1),
        #[inline(always)]
        |s1, s2, s3| math::mul_add(s2 + s3, NEG_DELTA, s1),
    );

    // Step 4.
    // Originally: for i in (start / 2 - 1)..(end / 2 + 1).
    filter_step_vertical(
        simd,
        scanline,
        height,
        width,
        simd_width,
        first_odd,
        #[inline(always)]
        |s1, s2, s3| (s2 + s3).mul_add(neg_gamma, s1),
        #[inline(always)]
        |s1, s2, s3| math::mul_add(s2 + s3, NEG_GAMMA, s1),
    );

    // Step 5.
    // Originally: for i in (start / 2)..(end / 2 + 1).
    filter_step_vertical(
        simd,
        scanline,
        height,
        width,
        simd_width,
        first_even,
        #[inline(always)]
        |s1, s2, s3| (s2 + s3).mul_add(neg_beta, s1),
        #[inline(always)]
        |s1, s2, s3| math::mul_add(s2 + s3, NEG_BETA, s1),
    );

    // Step 6.
    // Originally: for i in (start / 2)..(end / 2).
    filter_step_vertical(
        simd,
        scanline,
        height,
        width,
        simd_width,
        first_odd,
        #[inline(always)]
        |s1, s2, s3| (s2 + s3).mul_add(neg_alpha, s1),
        #[inline(always)]
        |s1, s2, s3| math::mul_add(s2 + s3, NEG_ALPHA, s1),
    );
}
