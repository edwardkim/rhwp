//! 인스턴스별 선택 변환. 지정하지 않은 decoder는 원래 알고리즘을 유지한다.

#[path = "ijg_upsampler.rs"]
mod ijg;
/// 실제 성분 경계를 사용하는 IJG 중심 표본 보간. 명시적으로 설치한 인스턴스에만 적용한다.
pub use ijg::select as ijg_fancy_upsampling;

#[path = "ijg_replay.rs"]
mod replay;
pub use replay::{idct as ijg_replay_idct, ycbcr_to_rgb as ijg_replay_ycbcr_to_rgb};

use crate::components::{Components, SampleRatios, UpSampler};
use crate::decoder::{IDCTPtr, JpegDecoder};
use crate::errors::DecodeErrors;
use crate::upsampler::{
    choose_horizontal_samp_function, choose_hv_samp_function, choose_v_samp_function,
    generic_sampler, upsample_no_op,
};
use zune_core::bytestream::ZByteReaderTrait;

/// MCU 패딩과 구별한 실제 성분 폭을 받는 표본 보간 함수.
pub type BoundedUpSampler = fn(&[i16], &[i16], &[i16], &mut [i16], &mut [i16], usize);
/// 수평·수직 확대 비율에 해당하는 선택 함수. None이면 기존 보간을 사용한다.
pub type SamplingSelector = fn(usize, usize) -> Option<BoundedUpSampler>;

#[derive(Clone, Copy)]
pub(crate) struct BoundedSampler {
    pub sample: BoundedUpSampler,
    pub width: usize,
    pub height: usize,
}

impl<T: ZByteReaderTrait> JpegDecoder<'_, T> {
    /// Set up-sampling routines in case an image is down sampled
    pub(crate) fn set_upsampling(&mut self) -> Result<(), DecodeErrors> {
        // no sampling, return early
        // check if horizontal max ==1
        if self.h_max == self.v_max && self.h_max == 1 {
            return Ok(());
        }

        for comp in &mut self.components {
            let hs = self.h_max / comp.horizontal_sample;
            let vs = self.v_max / comp.vertical_sample;

            let samp_factor = match (hs, vs) {
                (1, 1) => {
                    comp.sample_ratio = SampleRatios::None;
                    upsample_no_op
                }
                (2, 1) => {
                    comp.sample_ratio = SampleRatios::H;
                    choose_horizontal_samp_function(&self.options)
                }
                (1, 2) => {
                    comp.sample_ratio = SampleRatios::V;
                    choose_v_samp_function(&self.options)
                }
                (2, 2) => {
                    comp.sample_ratio = SampleRatios::HV;
                    choose_hv_samp_function(&self.options)
                }
                (hs, vs) => {
                    comp.sample_ratio = SampleRatios::Generic(hs, vs);
                    generic_sampler()
                }
            };
            comp.setup_upsample_scanline(self.allocation)?;
            comp.up_sampler = samp_factor;
            comp.bounded_upsampler = self
                .sampling_transform
                .and_then(|select| select(hs, vs))
                .map(|sample| crate::transforms::BoundedSampler {
                    sample,
                    width: (usize::from(self.info.width) * comp.horizontal_sample)
                        .div_ceil(self.h_max),
                    height: (usize::from(self.info.height) * comp.vertical_sample)
                        .div_ceil(self.v_max),
                });
        }

        return Ok(());
    }
    /// 헤더를 읽기 전에 IDCT와 선택 보간을 설치한다. 기본 decoder에는 영향이 없다.
    pub fn set_transforms(
        &mut self,
        idct: IDCTPtr,
        sampling: SamplingSelector,
    ) -> Result<(), DecodeErrors> {
        if self.seen_sof || self.headers_decoded || self.stream.position()? != 0 {
            return Err(DecodeErrors::FormatStatic(
                "transforms must precede header parsing",
            ));
        }
        self.idct_func = idct;
        self.idct_4x4_func = idct;
        self.idct_1x1_func = idct;
        self.sampling_transform = Some(sampling);
        Ok(())
    }
}

/// 마지막 실제 성분 행을 복제한다. DCT가 만든 패딩 행은 보간의 이웃이 아니다.
pub(crate) fn extend_bottom_edge(
    component: &mut Components,
    bounds: BoundedSampler,
    mcu_row: usize,
) -> Result<(), DecodeErrors> {
    let stride = component.width_stride;
    if stride == 0 || bounds.width == 0 || bounds.width > stride || bounds.height == 0 {
        return Err(DecodeErrors::FormatStatic(
            "invalid bounded component dimensions",
        ));
    }
    let rows = component.raw_coeff.len() / stride;
    let start = mcu_row
        .checked_mul(rows)
        .ok_or(DecodeErrors::AllocationLimit)?;
    let remaining = bounds.height.saturating_sub(start);
    if remaining > 0 && remaining < rows {
        let last = (remaining - 1) * stride;
        for row in remaining..rows {
            component
                .raw_coeff
                .copy_within(last..last + stride, row * stride);
        }
    }
    Ok(())
}

/// 기본 함수와 선택 함수가 같은 MCU 이어받기·출력 버퍼 계약을 소비한다.
pub(crate) fn upsample(
    default: UpSampler,
    bounded: Option<BoundedSampler>,
    input: &[i16],
    near: &[i16],
    far: &[i16],
    scratch: &mut [i16],
    output: &mut [i16],
) {
    if let Some(bounds) = bounded {
        (bounds.sample)(input, near, far, scratch, output, bounds.width);
    } else {
        default(input, near, far, scratch, output);
    }
}
