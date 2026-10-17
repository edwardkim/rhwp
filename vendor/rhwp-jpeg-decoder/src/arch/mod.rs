#![allow(unsafe_code)]

mod neon;
mod ssse3;
#[cfg(all(target_feature = "simd128", target_arch = "wasm32"))]
mod wasm;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::is_x86_feature_detected;

/// Arch-specific implementation of YCbCr conversion. Returns the number of pixels that were
/// converted.
#[allow(clippy::type_complexity)]
pub fn get_color_convert_line_ycbcr() -> Option<unsafe fn(&[u8], &[u8], &[u8], &mut [u8]) -> usize>
{
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[allow(unsafe_code)]
    {
        if is_x86_feature_detected!("ssse3") {
            return Some(ssse3::color_convert_line_ycbcr);
        }
    }
    // Runtime detection is not needed on aarch64.
    #[cfg(all(feature = "nightly_aarch64_neon", target_arch = "aarch64"))]
    {
        return Some(neon::color_convert_line_ycbcr);
    }
    #[cfg(all(target_feature = "simd128", target_arch = "wasm32"))]
    {
        return Some(wasm::color_convert_line_ycbcr);
    }
    #[allow(unreachable_code)]
    None
}

/// Arch-specific implementation of 8x8 IDCT.
#[allow(clippy::type_complexity)]
pub fn get_dequantize_and_idct_block_8x8(
) -> Option<unsafe fn(&[i16; 64], &[u16; 64], usize, &mut [u8])> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[allow(unsafe_code)]
    {
        if is_x86_feature_detected!("ssse3") {
            return Some(ssse3::dequantize_and_idct_block_8x8);
        }
    }
    // Runtime detection is not needed on aarch64.
    #[cfg(all(feature = "nightly_aarch64_neon", target_arch = "aarch64"))]
    {
        return Some(neon::dequantize_and_idct_block_8x8);
    }
    #[cfg(all(target_feature = "simd128", target_arch = "wasm32"))]
    {
        return Some(wasm::dequantize_and_idct_block_8x8);
    }
    #[allow(unreachable_code)]
    None
}
