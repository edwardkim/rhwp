/*
 * Copyright (c) 2023.
 *
 * This software is free software;
 *
 * You can redistribute it or modify it under terms of the MIT, Apache License or Zlib license
 */

//! JPEG 헤더 정보. 기존 공개 decoder 경로에서는 동일한 타입을 재수출한다.

use crate::components::SampleRatios;
use crate::misc::SOFMarkers;
use alloc::vec::Vec;

#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct GainMapInfo {
    pub data: Vec<u8>,
}

#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub(crate) struct ExtendedXmpSegment {
    pub(crate) offset: u32,
    pub(crate) total_size: u32,
    pub(crate) guid: Vec<u8>,
    pub(crate) data: Vec<u8>,
}

/// A struct representing Image Information
#[derive(Default, Clone, Eq, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct ImageInfo {
    /// Width of the image
    pub width: u16,
    /// Height of image
    pub height: u16,
    /// PixelDensity
    pub pixel_density: u8,
    /// Start of frame markers
    pub sof: SOFMarkers,
    /// Horizontal sample
    pub x_density: u16,
    /// Vertical sample
    pub y_density: u16,
    /// Number of components
    pub components: u8,
    /// Gain Map information, useful for
    /// UHDR images
    pub gain_map_info: Vec<GainMapInfo>,
    /// Multi picture information, useful for
    /// UHDR images
    pub multi_picture_information: Option<Vec<u8>>,
    /// Exif Data
    pub exif_data: Option<Vec<u8>>,
    /// XMP Data
    pub xmp_data: Option<Vec<u8>>,
    /// IPTC Data
    pub iptc_data: Option<Vec<u8>>,
    /// Extended XMP Data
    pub extended_xmp: Option<Vec<u8>>,
    /// Extended XMP Guid
    pub extended_xmp_guid: Option<Vec<u8>>,
    /// Image sub-sampling ratio
    pub sample_ratio: SampleRatios,
    /// The offset at which Multi picture information was found
    pub multi_picture_information_offset: Option<u64>,
}

impl ImageInfo {
    /// Set width of the image
    ///
    /// Found in the start of frame

    pub(crate) fn set_width(&mut self, width: u16) {
        self.width = width;
    }

    /// Set height of the image
    ///
    /// Found in the start of frame

    pub(crate) fn set_height(&mut self, height: u16) {
        self.height = height;
    }

    /// Set the image density
    ///
    /// Found in the start of frame

    pub(crate) fn set_density(&mut self, density: u8) {
        self.pixel_density = density;
    }

    /// Set image Start of frame marker
    ///
    /// found in the Start of frame header

    pub(crate) fn set_sof_marker(&mut self, marker: SOFMarkers) {
        self.sof = marker;
    }

    /// Set image x-density(dots per pixel)
    ///
    /// Found in the APP(0) marker
    #[allow(dead_code)]
    pub(crate) fn set_x(&mut self, sample: u16) {
        self.x_density = sample;
    }

    /// Set image y-density
    ///
    /// Found in the APP(0) marker
    #[allow(dead_code)]
    pub(crate) fn set_y(&mut self, sample: u16) {
        self.y_density = sample;
    }
}
