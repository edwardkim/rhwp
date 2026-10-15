use crate::paint::{CacheHint, RenderProfile};

use super::image_conv::ImageSampling;

/// Nested hints accumulate; restoring the parent context restores its policy.
#[derive(Debug, Clone, Copy)]
pub(super) struct ReplayPolicy {
    profile: RenderProfile,
    prefer_raster: bool,
    prefer_vector: bool,
}

impl ReplayPolicy {
    pub fn new(profile: RenderProfile) -> Self {
        Self {
            profile,
            prefer_raster: false,
            prefer_vector: false,
        }
    }

    pub fn with_hint(mut self, hint: CacheHint) -> Self {
        self.prefer_raster |= hint == CacheHint::PreferRaster;
        self.prefer_vector |= hint == CacheHint::PreferVectorRecording;
        self
    }

    pub fn image_sampling(self) -> ImageSampling {
        if self.profile == RenderProfile::FastPreview || self.prefer_raster {
            ImageSampling::nearest()
        } else if matches!(
            self.profile,
            RenderProfile::Print | RenderProfile::HighQuality
        ) || self.prefer_vector
        {
            ImageSampling::linear_mipmap()
        } else {
            ImageSampling::linear()
        }
    }

    pub fn antialias(self) -> bool {
        self.profile != RenderProfile::FastPreview || !self.prefer_raster
    }
}
