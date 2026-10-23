use alloc::vec::Vec;
use rhwp_codec_resource::VecResource as _;

pub(crate) const SIMD_WIDTH: usize = 8;

#[cfg(feature = "simd")]
mod inner {
    use super::SIMD_WIDTH;
    use core::ops::{Add, AddAssign, DivAssign, Mul, MulAssign, Sub, SubAssign};
    use fearless_simd::{SimdBase, SimdFloat};

    pub(crate) use fearless_simd::{Level, Simd, dispatch};

    #[derive(Copy, Clone)]
    #[allow(non_camel_case_types)]
    #[repr(C, align(32))]
    pub(crate) struct f32x8<S: Simd> {
        inner: fearless_simd::f32x8<S>,
    }

    impl<S: Simd> f32x8<S> {
        #[inline(always)]
        pub(crate) fn from_slice(simd: S, slice: &[f32]) -> Self {
            Self {
                inner: fearless_simd::f32x8::from_slice(simd, slice),
            }
        }

        #[inline(always)]
        pub(crate) fn splat(simd: S, value: f32) -> Self {
            Self {
                inner: fearless_simd::f32x8::splat(simd, value),
            }
        }

        #[inline(always)]
        pub(crate) fn mul_add(self, mul: Self, addend: Self) -> Self {
            Self {
                inner: self.inner.mul_add(mul.inner, addend.inner),
            }
        }

        #[inline(always)]
        pub(crate) fn floor(self) -> Self {
            Self {
                inner: self.inner.floor(),
            }
        }

        #[inline(always)]
        pub(crate) fn store(self, slice: &mut [f32]) {
            self.inner.store_slice(&mut slice[..SIMD_WIDTH]);
        }

        #[inline(always)]
        pub(crate) fn zip_low(self, other: Self) -> Self {
            Self {
                inner: self.inner.zip_low(other.inner),
            }
        }

        #[inline(always)]
        pub(crate) fn zip_high(self, other: Self) -> Self {
            Self {
                inner: self.inner.zip_high(other.inner),
            }
        }

        #[inline(always)]
        pub(crate) fn min(self, other: Self) -> Self {
            Self {
                inner: self.inner.min(other.inner),
            }
        }

        #[inline(always)]
        pub(crate) fn max(self, other: Self) -> Self {
            Self {
                inner: self.inner.max(other.inner),
            }
        }
    }

    impl<S: Simd> Add for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn add(self, rhs: Self) -> Self {
            Self {
                inner: self.inner + rhs.inner,
            }
        }
    }

    impl<S: Simd> Sub for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn sub(self, rhs: Self) -> Self {
            Self {
                inner: self.inner - rhs.inner,
            }
        }
    }

    impl<S: Simd> Mul for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn mul(self, rhs: Self) -> Self {
            Self {
                inner: self.inner * rhs.inner,
            }
        }
    }

    impl<S: Simd> Add<f32> for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn add(self, rhs: f32) -> Self {
            Self {
                inner: self.inner + rhs,
            }
        }
    }

    impl<S: Simd> Mul<f32> for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn mul(self, rhs: f32) -> Self {
            Self {
                inner: self.inner * rhs,
            }
        }
    }

    impl<S: Simd> AddAssign for f32x8<S> {
        #[inline(always)]
        fn add_assign(&mut self, rhs: Self) {
            self.inner = self.inner + rhs.inner;
        }
    }

    impl<S: Simd> SubAssign for f32x8<S> {
        #[inline(always)]
        fn sub_assign(&mut self, rhs: Self) {
            self.inner = self.inner - rhs.inner;
        }
    }

    impl<S: Simd> MulAssign<f32> for f32x8<S> {
        #[inline(always)]
        fn mul_assign(&mut self, rhs: f32) {
            self.inner = self.inner * rhs;
        }
    }

    impl<S: Simd> DivAssign<f32> for f32x8<S> {
        #[inline(always)]
        fn div_assign(&mut self, rhs: f32) {
            self.inner = self.inner / rhs;
        }
    }
}

#[cfg(not(feature = "simd"))]
mod inner {
    use super::SIMD_WIDTH;
    use core::marker::PhantomData;
    use core::ops::{Add, AddAssign, DivAssign, Mul, MulAssign, Sub, SubAssign};

    pub(crate) trait Simd: Copy + Clone {}

    #[derive(Copy, Clone)]
    pub(crate) struct ScalarSimd;
    impl Simd for ScalarSimd {}

    pub(crate) struct Level;
    impl Level {
        #[inline(always)]
        pub(crate) fn new() -> Self {
            Self
        }
    }

    #[derive(Copy, Clone)]
    #[allow(non_camel_case_types)]
    #[repr(C, align(32))]
    pub(crate) struct f32x8<S: Simd> {
        val: [f32; SIMD_WIDTH],
        _marker: PhantomData<S>,
    }

    impl<S: Simd> f32x8<S> {
        #[inline(always)]
        pub(crate) fn from_slice(_simd: S, slice: &[f32]) -> Self {
            let mut val = [0.0_f32; SIMD_WIDTH];
            val.copy_from_slice(&slice[..SIMD_WIDTH]);
            Self {
                val,
                _marker: PhantomData,
            }
        }

        #[inline(always)]
        pub(crate) fn splat(_simd: S, value: f32) -> Self {
            Self {
                val: [value; SIMD_WIDTH],
                _marker: PhantomData,
            }
        }

        #[inline(always)]
        pub(crate) fn mul_add(self, mul: Self, addend: Self) -> Self {
            let mut result = [0.0_f32; SIMD_WIDTH];
            for (i, value) in result.iter_mut().enumerate() {
                *value = super::mul_add(self.val[i], mul.val[i], addend.val[i]);
            }
            Self {
                val: result,
                _marker: PhantomData,
            }
        }

        #[inline(always)]
        pub(crate) fn floor(self) -> Self {
            let mut result = [0.0_f32; SIMD_WIDTH];
            for (i, value) in result.iter_mut().enumerate() {
                *value = super::floor_f32(self.val[i]);
            }
            Self {
                val: result,
                _marker: PhantomData,
            }
        }

        #[inline(always)]
        pub(crate) fn store(self, slice: &mut [f32]) {
            slice[..SIMD_WIDTH].copy_from_slice(&self.val);
        }

        #[inline(always)]
        pub(crate) fn zip_low(self, other: Self) -> Self {
            Self {
                val: [
                    self.val[0],
                    other.val[0],
                    self.val[1],
                    other.val[1],
                    self.val[2],
                    other.val[2],
                    self.val[3],
                    other.val[3],
                ],
                _marker: PhantomData,
            }
        }

        #[inline(always)]
        pub(crate) fn zip_high(self, other: Self) -> Self {
            Self {
                val: [
                    self.val[4],
                    other.val[4],
                    self.val[5],
                    other.val[5],
                    self.val[6],
                    other.val[6],
                    self.val[7],
                    other.val[7],
                ],
                _marker: PhantomData,
            }
        }

        #[inline(always)]
        pub(crate) fn min(self, other: Self) -> Self {
            let mut result = [0.0_f32; SIMD_WIDTH];
            for (i, value) in result.iter_mut().enumerate() {
                *value = super::min_f32(self.val[i], other.val[i]);
            }
            Self {
                val: result,
                _marker: PhantomData,
            }
        }

        #[inline(always)]
        pub(crate) fn max(self, other: Self) -> Self {
            let mut result = [0.0_f32; SIMD_WIDTH];
            for (i, value) in result.iter_mut().enumerate() {
                *value = super::max_f32(self.val[i], other.val[i]);
            }
            Self {
                val: result,
                _marker: PhantomData,
            }
        }
    }

    impl<S: Simd> Add for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn add(self, rhs: Self) -> Self {
            let mut result = [0.0_f32; SIMD_WIDTH];
            for (i, value) in result.iter_mut().enumerate() {
                *value = self.val[i] + rhs.val[i];
            }
            Self {
                val: result,
                _marker: PhantomData,
            }
        }
    }

    impl<S: Simd> Sub for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn sub(self, rhs: Self) -> Self {
            let mut result = [0.0_f32; SIMD_WIDTH];
            for (i, value) in result.iter_mut().enumerate() {
                *value = self.val[i] - rhs.val[i];
            }
            Self {
                val: result,
                _marker: PhantomData,
            }
        }
    }

    impl<S: Simd> Mul for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn mul(self, rhs: Self) -> Self {
            let mut result = [0.0_f32; SIMD_WIDTH];
            for (i, value) in result.iter_mut().enumerate() {
                *value = self.val[i] * rhs.val[i];
            }
            Self {
                val: result,
                _marker: PhantomData,
            }
        }
    }

    impl<S: Simd> Add<f32> for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn add(self, rhs: f32) -> Self {
            let mut result = [0.0_f32; SIMD_WIDTH];
            for (i, value) in result.iter_mut().enumerate() {
                *value = self.val[i] + rhs;
            }
            Self {
                val: result,
                _marker: PhantomData,
            }
        }
    }

    impl<S: Simd> Mul<f32> for f32x8<S> {
        type Output = Self;
        #[inline(always)]
        fn mul(self, rhs: f32) -> Self {
            let mut result = [0.0_f32; SIMD_WIDTH];
            for (i, value) in result.iter_mut().enumerate() {
                *value = self.val[i] * rhs;
            }
            Self {
                val: result,
                _marker: PhantomData,
            }
        }
    }

    impl<S: Simd> AddAssign for f32x8<S> {
        #[inline(always)]
        fn add_assign(&mut self, rhs: Self) {
            for i in 0..SIMD_WIDTH {
                self.val[i] += rhs.val[i];
            }
        }
    }

    impl<S: Simd> SubAssign for f32x8<S> {
        #[inline(always)]
        fn sub_assign(&mut self, rhs: Self) {
            for i in 0..SIMD_WIDTH {
                self.val[i] -= rhs.val[i];
            }
        }
    }

    impl<S: Simd> MulAssign<f32> for f32x8<S> {
        #[inline(always)]
        fn mul_assign(&mut self, rhs: f32) {
            for i in 0..SIMD_WIDTH {
                self.val[i] *= rhs;
            }
        }
    }

    impl<S: Simd> DivAssign<f32> for f32x8<S> {
        #[inline(always)]
        fn div_assign(&mut self, rhs: f32) {
            for i in 0..SIMD_WIDTH {
                self.val[i] /= rhs;
            }
        }
    }

    /// Scalar fallback for SIMD dispatch.
    #[macro_export]
    macro_rules! simd_dispatch {
        ($level:expr, $simd:ident => $body:expr) => {{
            let _ = $level;
            let $simd = $crate::math::ScalarSimd;
            $body
        }};
    }

    pub(crate) use simd_dispatch as dispatch;
}

// Note that these polyfills can be very imprecise, but hopefully good enough
// for the vast majority of cases.

#[inline(always)]
pub(crate) fn mul_add(a: f32, b: f32, c: f32) -> f32 {
    #[cfg(all(
        feature = "std",
        any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "fma"
            ),
            all(target_arch = "aarch64", target_feature = "neon")
        )
    ))]
    {
        f32::mul_add(a, b, c)
    }
    #[cfg(not(all(
        feature = "std",
        any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "fma"
            ),
            all(target_arch = "aarch64", target_feature = "neon")
        )
    )))]
    {
        a * b + c
    }
}

#[inline(always)]
pub(crate) fn floor_f32(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.floor()
    }
    #[cfg(not(feature = "std"))]
    {
        let xi = x as i32;
        let xf = xi as f32;
        if x < xf { xf - 1.0 } else { xf }
    }
}

#[inline(always)]
pub(crate) fn round_f32(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.round()
    }
    #[cfg(not(feature = "std"))]
    {
        if x >= 0.0 {
            floor_f32(x + 0.5)
        } else {
            -floor_f32(-x + 0.5)
        }
    }
}

#[inline(always)]
pub(crate) fn pow2i(exp: i32) -> f32 {
    if exp >= 0 {
        (1_u32 << exp) as f32
    } else {
        1.0 / (1_u32 << -exp) as f32
    }
}

#[inline(always)]
#[cfg_attr(feature = "simd", allow(dead_code))]
pub(crate) fn min_f32(a: f32, b: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        a.min(b)
    }
    #[cfg(not(feature = "std"))]
    {
        if a < b { a } else { b }
    }
}

#[inline(always)]
#[cfg_attr(feature = "simd", allow(dead_code))]
pub(crate) fn max_f32(a: f32, b: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        a.max(b)
    }
    #[cfg(not(feature = "std"))]
    {
        if a > b { a } else { b }
    }
}

pub(crate) use inner::*;

/// A wrapper around `Vec<f32>` that pads the vector to a multiple of `N` elements.
/// This allows SIMD operations to safely process the data without bounds checking
/// at the end of the buffer.
#[derive(Debug, Clone)]
pub(crate) struct SimdBuffer<const N: usize> {
    data: Vec<f32>,
    original_len: usize,
}

impl<const N: usize> SimdBuffer<N> {
    pub(crate) fn try_clone(&self) -> crate::error::Result<Self> {
        Ok(Self {
            data: rhwp_codec_resource::copy(&self.data)?,
            original_len: self.original_len,
        })
    }

    /// Create a new `SimdBuffer` from a `Vec<f32>`, padding it to a multiple of `N`.
    pub(crate) fn new(mut data: Vec<f32>) -> crate::error::Result<Self> {
        let original_len = data.len();
        let padded_len = Self::padded_len(original_len)?;
        if padded_len > original_len {
            data.rhwp_resize(padded_len, 0.0)?;
        }
        Ok(Self { data, original_len })
    }

    /// Create a new `SimdBuffer` filled with zeros.
    pub(crate) fn zeros(original_len: usize) -> crate::error::Result<Self> {
        let padded_len = Self::padded_len(original_len)?;
        let data = rhwp_codec_resource::filled(0.0, padded_len)?;
        Ok(Self { data, original_len })
    }

    /// Returns only the original (non-padded) data as an immutable slice.
    pub(crate) fn truncated(&self) -> &[f32] {
        &self.data[..self.original_len]
    }

    /// Returns the length padded to a multiple of `N`
    fn padded_len(original_len: usize) -> crate::error::Result<usize> {
        let remainder = original_len % N;
        let padding = N - remainder;
        original_len
            .checked_add(padding)
            .ok_or_else(rhwp_codec_resource::reject)
            .map_err(Into::into)
    }
}

impl<const N: usize> core::ops::Deref for SimdBuffer<N> {
    type Target = [f32];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<const N: usize> core::ops::DerefMut for SimdBuffer<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
