//! Read and decode a JPEG2000 codestream, as described in Annex A.

use alloc::vec;
use alloc::vec::Vec;
use rhwp_codec_resource::IteratorResource as _;
use rhwp_codec_resource::VecResource as _;

use super::DecodeSettings;
use super::bitplane::BITPLANE_BIT_SIZE;
use super::build::SubBandType;
use crate::error::{MarkerError, Result, ValidationError, bail, err};
use crate::reader::BitReader;

const MAX_LAYER_COUNT: u8 = 32;
const MAX_RESOLUTION_COUNT: u8 = 32;
const MAX_PRECINCT_EXPONENT: u8 = 31;

#[derive(Debug)]
pub(crate) struct Header<'a> {
    pub(crate) size_data: SizeData,
    pub(crate) global_coding_style: CodingStyleDefault,
    pub(crate) component_infos: Vec<ComponentInfo>,
    pub(crate) ppm_packets: Vec<PpmPacket<'a>>,
    pub(crate) skipped_resolution_levels: u8,
    /// Whether strict mode is enabled for decoding.
    pub(crate) strict: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PpmMarkerData<'a> {
    pub(crate) sequence_idx: u8,
    pub(crate) packets: Vec<PpmPacket<'a>>,
}

#[derive(Debug, Clone)]
pub(crate) struct PpmPacket<'a> {
    pub(crate) data: &'a [u8],
}

pub(crate) fn read_header<'a>(
    reader: &mut BitReader<'a>,
    settings: &DecodeSettings,
) -> Result<Header<'a>> {
    rhwp_codec_resource::work(1)?;
    if reader.read_marker()? != markers::SIZ {
        bail!(MarkerError::Expected("SIZ"));
    }

    let mut size_data = size_marker(reader)?;

    let mut cod = None;
    let mut qcd = None;

    let num_components = size_data.component_sizes.len() as u16;
    let mut cod_components = rhwp_codec_resource::filled(None, num_components as usize)?;
    let mut qcd_components = rhwp_codec_resource::filled(None, num_components as usize)?;
    let mut ppm_markers = vec![];

    loop {
        rhwp_codec_resource::work(1)?;
        match reader.peek_marker().ok_or(MarkerError::Invalid)? {
            markers::SOT => break,
            markers::COD => {
                reader.read_marker()?;
                cod = Some(cod_marker(reader).ok_or(MarkerError::ParseFailure("COD"))?);
            }
            markers::COC => {
                reader.read_marker()?;
                let (component_index, coc) =
                    coc_marker(reader, num_components).ok_or(MarkerError::ParseFailure("COC"))?;
                *cod_components
                    .get_mut(component_index as usize)
                    .ok_or(MarkerError::ParseFailure("COC"))? = Some(coc);
            }
            markers::QCD => {
                reader.read_marker()?;
                qcd = Some(qcd_marker(reader).ok_or(MarkerError::ParseFailure("QCD"))?);
            }
            markers::QCC => {
                reader.read_marker()?;
                let (component_index, qcc) =
                    qcc_marker(reader, num_components).ok_or(MarkerError::ParseFailure("QCC"))?;
                *qcd_components
                    .get_mut(component_index as usize)
                    .ok_or(MarkerError::ParseFailure("QCC"))? = Some(qcc);
            }
            markers::RGN => {
                reader.read_marker()?;
                rgn_marker(reader).ok_or(MarkerError::ParseFailure("RGN"))?;
            }
            markers::TLM => {
                reader.read_marker()?;
                tlm_marker(reader).ok_or(MarkerError::ParseFailure("TLM"))?;
            }
            markers::COM => {
                reader.read_marker()?;
                com_marker(reader).ok_or(MarkerError::ParseFailure("COM"))?;
            }
            markers::PPM => {
                reader.read_marker()?;
                ppm_markers
                    .rhwp_push(ppm_marker(reader).ok_or(MarkerError::ParseFailure("PPM"))?)?;
            }
            markers::CRG => {
                reader.read_marker()?;
                skip_marker_segment(reader);
            }
            (0x30..=0x3F) => {
                // "All markers with the marker code between 0xFF30 and 0xFF3F
                // have no marker segment parameters. They shall be skipped by
                // the decoder."
                reader.read_marker()?;
                // skip_marker_segment(reader);
            }
            _ => {
                bail!(MarkerError::Unsupported);
            }
        }
    }

    let cod = cod.ok_or(MarkerError::Missing("COD"))?;
    let qcd = qcd.ok_or(MarkerError::Missing("QCD"))?;

    let mut component_infos = rhwp_codec_resource::capacity(num_components as usize)?;
    for (idx, csi) in size_data.component_sizes.iter().enumerate() {
        let mut coding_style = cod_components[idx]
            .as_ref()
            .unwrap_or(&cod.component_parameters)
            .try_clone()?;
        coding_style.flags.raw |= cod.component_parameters.flags.raw;
        component_infos.rhwp_push(ComponentInfo {
            size_info: *csi,
            coding_style,
            quantization_info: qcd_components[idx].as_ref().unwrap_or(&qcd).try_clone()?,
        })?;
    }

    // Components can have different number of resolution levels. In that case, we
    // can only skip as many resolution levels as the component with the smallest
    // number of resolution levels.
    let min_num_resolution_levels = component_infos
        .iter()
        .map(|c| c.num_resolution_levels())
        .min()
        .unwrap();
    let skipped_resolution_levels =
        if let Some((target_width, target_height)) = settings.target_resolution {
            let width_log = size_data
                .image_width()
                .checked_div(target_width)
                .and_then(|v| v.checked_ilog2())
                .unwrap_or(0);
            let height_log = size_data
                .image_height()
                .checked_div(target_height)
                .and_then(|v| v.checked_ilog2())
                .unwrap_or(0);

            width_log.min(height_log) as u8
        } else {
            0
        }
        .min(min_num_resolution_levels - 1);

    // If the user defined a maximum resolution level that is lower than the
    // maximum available one, the final image needs to be shrinked further.
    size_data.x_resolution_shrink_factor *= 1 << skipped_resolution_levels;
    size_data.y_resolution_shrink_factor *= 1 << skipped_resolution_levels;

    rhwp_codec_resource::sort(&mut ppm_markers, |p0, p1| {
        p0.sequence_idx.cmp(&p1.sequence_idx)
    })?;

    let header = Header {
        size_data,
        global_coding_style: cod,
        component_infos,
        ppm_packets: ppm_markers
            .into_iter()
            .flat_map(|i| i.packets)
            .filter_map(|p| if p.data.is_empty() { None } else { Some(p) })
            .rhwp_collect()?,
        skipped_resolution_levels,
        strict: settings.strict,
    };

    validate(&header)?;

    Ok(header)
}

fn validate(header: &Header<'_>) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    {
        let (__rhwp_iter_1, __rhwp_iter_1_prepaid) =
            rhwp_codec_resource::iterations(&header.component_infos)?;
        for info in __rhwp_iter_1 {
            if !__rhwp_iter_1_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            let max_resolution_idx = info.coding_style.parameters.num_resolution_levels - 1;
            let quantization_style = info.quantization_info.quantization_style;
            let num_precinct_exponents = info.quantization_info.step_sizes.len();

            if num_precinct_exponents == 0 {
                bail!(ValidationError::MissingPrecinctExponents);
            } else if matches!(
                quantization_style,
                QuantizationStyle::NoQuantization | QuantizationStyle::ScalarExpounded
            ) {
                // See the accesses in the `exponent_mantissa` method. The largest
                // access is 1 + (max_resolution_idx - 1) * 3 + 2.

                if max_resolution_idx == 0 {
                    if num_precinct_exponents == 0 {
                        bail!(ValidationError::InsufficientExponents);
                    }
                } else if 1 + (max_resolution_idx as usize - 1) * 3 + 2 >= num_precinct_exponents {
                    bail!(ValidationError::InsufficientExponents);
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct ComponentInfo {
    pub(crate) size_info: ComponentSizeInfo,
    pub(crate) coding_style: CodingStyleComponent,
    pub(crate) quantization_info: QuantizationInfo,
}

impl ComponentInfo {
    pub(crate) fn exponent_mantissa(
        &self,
        sub_band_type: SubBandType,
        resolution: u8,
    ) -> Result<(u16, u16)> {
        rhwp_codec_resource::work(1)?;
        let n_ll = self.coding_style.parameters.num_decomposition_levels;

        let sb_index = match sub_band_type {
            // TODO: Shouldn't be reached.
            SubBandType::LowLow => u16::MAX,
            SubBandType::HighLow => 0,
            SubBandType::LowHigh => 1,
            SubBandType::HighHigh => 2,
        };

        let step_sizes = &self.quantization_info.step_sizes;
        match self.quantization_info.quantization_style {
            QuantizationStyle::NoQuantization | QuantizationStyle::ScalarExpounded => {
                let entry = if resolution == 0 {
                    step_sizes.first()
                } else {
                    step_sizes.get(1 + (resolution as usize - 1) * 3 + sb_index as usize)
                };

                Ok(entry
                    .map(|s| (s.exponent, s.mantissa))
                    .ok_or(ValidationError::MissingStepSize)?)
            }
            QuantizationStyle::ScalarDerived => {
                let (e_0, mantissa) = step_sizes
                    .first()
                    .map(|s| (s.exponent, s.mantissa))
                    .ok_or(ValidationError::MissingStepSize)?;
                let n_b = if resolution == 0 {
                    n_ll as u16
                } else {
                    n_ll as u16 + 1 - resolution as u16
                };

                let exponent = e_0
                    .checked_sub(n_ll as u16)
                    .and_then(|e| e.checked_add(n_b))
                    .ok_or(ValidationError::InvalidExponents)?;

                Ok((exponent, mantissa))
            }
        }
    }

    pub(crate) fn wavelet_transform(&self) -> WaveletTransform {
        self.coding_style.parameters.transformation
    }

    pub(crate) fn num_resolution_levels(&self) -> u8 {
        self.coding_style.parameters.num_resolution_levels
    }

    pub(crate) fn num_decomposition_levels(&self) -> u8 {
        self.coding_style.parameters.num_decomposition_levels
    }

    pub(crate) fn code_block_style(&self) -> CodeBlockStyle {
        self.coding_style.parameters.code_block_style
    }
}

/// Progression order (Table A.16).
#[derive(Debug, Clone, Copy)]
pub(crate) enum ProgressionOrder {
    LayerResolutionComponentPosition,
    ResolutionLayerComponentPosition,
    ResolutionPositionComponentLayer,
    PositionComponentResolutionLayer,
    ComponentPositionResolutionLayer,
}

impl ProgressionOrder {
    fn from_u8(value: u8) -> Result<Self> {
        rhwp_codec_resource::work(1)?;
        match value {
            0 => Ok(Self::LayerResolutionComponentPosition),
            1 => Ok(Self::ResolutionLayerComponentPosition),
            2 => Ok(Self::ResolutionPositionComponentLayer),
            3 => Ok(Self::PositionComponentResolutionLayer),
            4 => Ok(Self::ComponentPositionResolutionLayer),
            _ => err!(ValidationError::InvalidProgressionOrder),
        }
    }
}

/// Wavelet transformation type (Table A.20).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WaveletTransform {
    Irreversible97,
    Reversible53,
}

impl WaveletTransform {
    fn from_u8(value: u8) -> Result<Self> {
        rhwp_codec_resource::work(1)?;
        match value {
            0 => Ok(Self::Irreversible97),
            1 => Ok(Self::Reversible53),
            _ => err!(ValidationError::InvalidTransformation),
        }
    }
}

/// Coding style flags (Table A.13).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CodingStyleFlags {
    pub(crate) raw: u8,
}

impl CodingStyleFlags {
    fn from_u8(value: u8) -> Self {
        Self { raw: value }
    }

    pub(crate) fn has_precincts(&self) -> bool {
        (self.raw & 0x01) != 0
    }

    pub(crate) fn may_use_sop_markers(&self) -> bool {
        (self.raw & 0x02) != 0
    }

    pub(crate) fn uses_eph_marker(&self) -> bool {
        (self.raw & 0x04) != 0
    }
}

/// Code-block style flags (Table A.19).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CodeBlockStyle {
    pub(crate) selective_arithmetic_coding_bypass: bool,
    pub(crate) reset_context_probabilities: bool,
    pub(crate) termination_on_each_pass: bool,
    pub(crate) vertically_causal_context: bool,
    pub(crate) segmentation_symbols: bool,
}

impl CodeBlockStyle {
    fn from_u8(value: u8) -> Self {
        Self {
            selective_arithmetic_coding_bypass: (value & 0x01) != 0,
            reset_context_probabilities: (value & 0x02) != 0,
            termination_on_each_pass: (value & 0x04) != 0,
            vertically_causal_context: (value & 0x08) != 0,
            // The predictable termination flag is only informative and
            // can therefore be ignored.
            segmentation_symbols: (value & 0x20) != 0,
        }
    }
}

/// Quantization style (Table A.28).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuantizationStyle {
    NoQuantization,
    ScalarDerived,
    ScalarExpounded,
}

impl QuantizationStyle {
    fn from_u8(value: u8) -> Result<Self> {
        rhwp_codec_resource::work(1)?;
        match value & 0x1F {
            0 => Ok(Self::NoQuantization),
            1 => Ok(Self::ScalarDerived),
            2 => Ok(Self::ScalarExpounded),
            _ => err!(ValidationError::InvalidQuantizationStyle),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StepSize {
    pub(crate) mantissa: u16,
    pub(crate) exponent: u16,
}

/// Quantization properties, from the QCD and QCC markers (A.6.4 and A.6.5).
#[derive(Clone, Debug)]
pub(crate) struct QuantizationInfo {
    pub(crate) quantization_style: QuantizationStyle,
    pub(crate) guard_bits: u8,
    pub(crate) step_sizes: Vec<StepSize>,
}

/// Default values for coding style, from the COD marker (A.6.1).
#[derive(Debug, Clone)]
pub(crate) struct CodingStyleDefault {
    pub(crate) progression_order: ProgressionOrder,
    pub(crate) num_layers: u8,
    pub(crate) mct: bool,
    // This is the default used for all components, if not overridden by COC.
    pub(crate) component_parameters: CodingStyleComponent,
}

/// Values of coding style for each component, from the COC marker (A.6.2).
#[derive(Clone, Debug)]
pub(crate) struct CodingStyleComponent {
    pub(crate) flags: CodingStyleFlags,
    pub(crate) parameters: CodingStyleParameters,
}

/// Shared parameters between the COC and COD marker (A.6.1 and A.6.2).
#[derive(Clone, Debug)]
pub(crate) struct CodingStyleParameters {
    pub(crate) num_decomposition_levels: u8,
    pub(crate) num_resolution_levels: u8,
    pub(crate) code_block_width: u8,
    pub(crate) code_block_height: u8,
    pub(crate) code_block_style: CodeBlockStyle,
    pub(crate) transformation: WaveletTransform,
    pub(crate) precinct_exponents: Vec<(u8, u8)>,
}

#[derive(Debug)]
pub(crate) struct SizeData {
    /// Width of the reference grid (Xsiz).
    pub(crate) reference_grid_width: u32,
    /// Height of the reference grid (Ysiz).
    pub(crate) reference_grid_height: u32,
    /// Horizontal offset from the origin of the reference grid to the
    /// left side of the image area (`XOsiz`).
    pub(crate) image_area_x_offset: u32,
    /// Vertical offset from the origin of the reference grid to the top side of the image area (`YOsiz`).
    pub(crate) image_area_y_offset: u32,
    /// Width of one reference tile with respect to the reference grid (`XTSiz`).
    pub(crate) tile_width: u32,
    /// Height of one reference tile with respect to the reference grid (`YTSiz`).
    pub(crate) tile_height: u32,
    /// Horizontal offset from the origin of the reference grid to the left side of the first tile (`XTOSiz`).
    pub(crate) tile_x_offset: u32,
    /// Vertical offset from the origin of the reference grid to the top side of the first tile (`YTOSiz`).
    pub(crate) tile_y_offset: u32,
    /// Component information (SSiz/XRSiz/YRSiz).
    pub(crate) component_sizes: Vec<ComponentSizeInfo>,
    /// Shrink factor in the x direction. See the comment in the parsing method.
    pub(crate) x_shrink_factor: u32,
    /// Shrink factor in the y direction. See the comment in the parsing method.
    pub(crate) y_shrink_factor: u32,
    /// Shrink factor in the x direction due to requesting a lower resolution level.
    pub(crate) x_resolution_shrink_factor: u32,
    /// Shrink factor in the y direction due to requesting a lower resolution level.
    pub(crate) y_resolution_shrink_factor: u32,
}

impl SizeData {
    pub(crate) fn tile_x_coord(&self, idx: u32) -> u32 {
        // See B-6.
        idx % self.num_x_tiles()
    }

    pub(crate) fn tile_y_coord(&self, idx: u32) -> u32 {
        // See B-6.
        idx / self.num_x_tiles()
    }
}

/// Component information (A.5.1 and Table A.11).
#[derive(Debug, Clone, Copy)]
pub(crate) struct ComponentSizeInfo {
    pub(crate) precision: u8,
    pub(crate) horizontal_resolution: u8,
    pub(crate) vertical_resolution: u8,
}

impl SizeData {
    /// The number of tiles in the x direction.
    pub(crate) fn num_x_tiles(&self) -> u32 {
        // See formula B-5.
        (self.reference_grid_width - self.tile_x_offset).div_ceil(self.tile_width)
    }

    /// The number of tiles in the y direction.
    pub(crate) fn num_y_tiles(&self) -> u32 {
        // See formula B-5.
        (self.reference_grid_height - self.tile_y_offset).div_ceil(self.tile_height)
    }

    /// The total number of tiles.
    pub(crate) fn num_tiles(&self) -> u32 {
        self.num_x_tiles() * self.num_y_tiles()
    }

    /// Return the overall width of the image.
    pub(crate) fn image_width(&self) -> u32 {
        (self.reference_grid_width - self.image_area_x_offset)
            .div_ceil(self.x_shrink_factor * self.x_resolution_shrink_factor)
    }

    /// Return the overall height of the image.
    pub(crate) fn image_height(&self) -> u32 {
        (self.reference_grid_height - self.image_area_y_offset)
            .div_ceil(self.y_shrink_factor * self.y_resolution_shrink_factor)
    }
}

/// SIZ marker (A.5.1).
fn size_marker(reader: &mut BitReader<'_>) -> Result<SizeData> {
    rhwp_codec_resource::work(1)?;
    let size_data = size_marker_inner(reader).ok_or(MarkerError::ParseFailure("SIZ"))?;

    if size_data.tile_width == 0
        || size_data.tile_height == 0
        || size_data.reference_grid_width == 0
        || size_data.reference_grid_height == 0
    {
        bail!(ValidationError::InvalidDimensions);
    }

    if size_data.tile_x_offset >= size_data.reference_grid_width
        || size_data.tile_y_offset >= size_data.reference_grid_height
    {
        bail!(ValidationError::InvalidDimensions);
    }

    if size_data
        .num_x_tiles()
        .checked_mul(size_data.num_y_tiles())
        .is_none()
    {
        bail!(ValidationError::InvalidDimensions);
    }

    // The tile grid offsets (XTOsiz, YTOsiz) are constrained to be no greater than the
    // image area offsets (B-3).
    if size_data.tile_x_offset > size_data.image_area_x_offset
        || size_data.tile_y_offset > size_data.image_area_y_offset
    {
        bail!(crate::TileError::InvalidOffsets);
    }

    // Also, the tile size plus the tile offset shall be greater than the image area offset.
    // This ensures that the first tile (tile 0) will contain at least one reference grid point
    // from the image area (B-4).
    if size_data
        .tile_x_offset
        .checked_add(size_data.tile_width)
        .ok_or(crate::TileError::InvalidOffsets)?
        <= size_data.image_area_x_offset
        || size_data
            .tile_y_offset
            .checked_add(size_data.tile_height)
            .ok_or(crate::TileError::InvalidOffsets)?
            <= size_data.image_area_y_offset
    {
        bail!(crate::TileError::InvalidOffsets);
    }

    {
        let (__rhwp_iter_2, __rhwp_iter_2_prepaid) =
            rhwp_codec_resource::iterations(&size_data.component_sizes)?;
        for comp in __rhwp_iter_2 {
            if !__rhwp_iter_2_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            if comp.precision == 0
                || comp.vertical_resolution == 0
                || comp.horizontal_resolution == 0
            {
                bail!(ValidationError::InvalidComponentMetadata);
            }
        }
    }

    const MAX_DIMENSIONS: usize = 60000;

    if size_data.image_width() as usize > MAX_DIMENSIONS
        || size_data.image_height() as usize > MAX_DIMENSIONS
    {
        bail!(ValidationError::ImageTooLarge);
    }

    Ok(size_data)
}

fn size_marker_inner(reader: &mut BitReader<'_>) -> Option<SizeData> {
    rhwp_codec_resource::work(1).ok()?;
    // Length.
    let _ = reader.read_u16()?;
    // Decoder capabilities.
    let _ = reader.read_u16()?;

    let xsiz = reader.read_u32()?;
    let ysiz = reader.read_u32()?;
    let x_osiz = reader.read_u32()?;
    let y_osiz = reader.read_u32()?;
    let xt_siz = reader.read_u32()?;
    let yt_siz = reader.read_u32()?;
    let xto_siz = reader.read_u32()?;
    let yto_siz = reader.read_u32()?;
    let csiz = reader.read_u16()?;

    if x_osiz >= xsiz || y_osiz >= ysiz {
        return None;
    }

    if csiz == 0 {
        return None;
    }

    let mut components = rhwp_codec_resource::capacity(csiz as usize).ok()?;
    {
        let (__rhwp_iter_3, __rhwp_iter_3_prepaid) =
            rhwp_codec_resource::iterations(0..csiz).ok()?;
        for _ in __rhwp_iter_3 {
            if !__rhwp_iter_3_prepaid {
                rhwp_codec_resource::work(1).ok()?;
            }
            let ssiz = reader.read_byte()?;
            let x_rsiz = reader.read_byte()?;
            let y_rsiz = reader.read_byte()?;

            let precision = (ssiz & 0x7F) + 1;
            // No idea how to process signed images, but as far as I can tell
            // openjpeg and others just accept it as is, so let's do the same.
            let _is_signed = (ssiz & 0x80) != 0;

            // In theory up to 38 is allowed, but we don't support more than that.
            if precision as u32 > BITPLANE_BIT_SIZE {
                return None;
            }

            components
                .rhwp_push(ComponentSizeInfo {
                    precision,
                    horizontal_resolution: x_rsiz,
                    vertical_resolution: y_rsiz,
                })
                .ok()?;
        }
    }

    // In case all components are sub-sampled at the same level, we
    // don't want to render them at the original resolution but instead
    // reduce their dimension so that we can assume a resolution of 1 for
    // all components. This makes the images much smaller.

    let mut x_shrink_factor = 1;
    let mut y_shrink_factor = 1;

    let hr = components[0].horizontal_resolution;
    let vr = components[0].vertical_resolution;
    let mut same_resolution = true;

    {
        let (__rhwp_iter_4, __rhwp_iter_4_prepaid) =
            rhwp_codec_resource::iterations(&components[1..]).ok()?;
        for component in __rhwp_iter_4 {
            if !__rhwp_iter_4_prepaid {
                rhwp_codec_resource::work(1).ok()?;
            }
            same_resolution &= component.horizontal_resolution == hr;
            same_resolution &= component.vertical_resolution == vr;
        }
    }

    if same_resolution {
        x_shrink_factor = hr as u32;
        y_shrink_factor = vr as u32;
    }

    Some(SizeData {
        reference_grid_width: xsiz,
        reference_grid_height: ysiz,
        image_area_x_offset: x_osiz,
        image_area_y_offset: y_osiz,
        tile_width: xt_siz,
        tile_height: yt_siz,
        tile_x_offset: xto_siz,
        tile_y_offset: yto_siz,
        component_sizes: components,
        x_shrink_factor,
        y_shrink_factor,
        x_resolution_shrink_factor: 1,
        y_resolution_shrink_factor: 1,
    })
}

fn coding_style_parameters(
    reader: &mut BitReader<'_>,
    coding_style: &CodingStyleFlags,
) -> Option<CodingStyleParameters> {
    rhwp_codec_resource::work(1).ok()?;
    let num_decomposition_levels = reader.read_byte()?;

    if num_decomposition_levels > MAX_RESOLUTION_COUNT {
        return None;
    }

    let num_resolution_levels = num_decomposition_levels.checked_add(1)?;
    let code_block_width = reader.read_byte()?.checked_add(2)?;
    let code_block_height = reader.read_byte()?.checked_add(2)?;
    let code_block_style = CodeBlockStyle::from_u8(reader.read_byte()?);
    let transformation = WaveletTransform::from_u8(reader.read_byte()?).ok()?;

    let mut precinct_exponents = Vec::new();
    if coding_style.has_precincts() {
        // "Entropy coder with precincts defined below."
        {
            let (__rhwp_iter_5, __rhwp_iter_5_prepaid) =
                rhwp_codec_resource::iterations(0..num_resolution_levels).ok()?;
            for _ in __rhwp_iter_5 {
                if !__rhwp_iter_5_prepaid {
                    rhwp_codec_resource::work(1).ok()?;
                }
                // Table A.21.
                let precinct_size = reader.read_byte()?;
                let width_exp = precinct_size & 0xF;
                let height_exp = precinct_size >> 4;

                if width_exp > MAX_PRECINCT_EXPONENT || height_exp > MAX_PRECINCT_EXPONENT {
                    return None;
                }

                precinct_exponents.rhwp_push((width_exp, height_exp)).ok()?;
            }
        }
    } else {
        // "Entropy coder, precincts with PPx = 15 and PPy = 15"
        {
            let (__rhwp_iter_6, __rhwp_iter_6_prepaid) =
                rhwp_codec_resource::iterations(0..num_resolution_levels).ok()?;
            for _ in __rhwp_iter_6 {
                if !__rhwp_iter_6_prepaid {
                    rhwp_codec_resource::work(1).ok()?;
                }
                precinct_exponents.rhwp_push((15, 15)).ok()?;
            }
        }
    }

    Some(CodingStyleParameters {
        num_decomposition_levels,
        num_resolution_levels,
        code_block_width,
        code_block_height,
        code_block_style,
        transformation,
        precinct_exponents,
    })
}

/// COM Marker (A.9.2).
fn com_marker(reader: &mut BitReader<'_>) -> Option<()> {
    rhwp_codec_resource::work(1).ok()?;
    skip_marker_segment(reader)
}

/// TLM marker (A.7.1).
fn tlm_marker(reader: &mut BitReader<'_>) -> Option<()> {
    rhwp_codec_resource::work(1).ok()?;
    skip_marker_segment(reader)
}

/// PPM marker (A.7.4).
fn ppm_marker<'a>(reader: &mut BitReader<'a>) -> Option<PpmMarkerData<'a>> {
    rhwp_codec_resource::work(1).ok()?;
    let segment_len = reader.read_u16()?.checked_sub(2)? as usize;
    let ppm_data = reader.read_bytes(segment_len)?;
    let mut packets = vec![];

    let mut reader = BitReader::new(ppm_data);
    let sequence_idx = reader.read_byte()?;

    // TODO: Handle case where next packet doesn't have nppm parameter.

    while !reader.at_end() {
        rhwp_codec_resource::work(1).ok()?;
        let packet_len = reader.read_u16()? as usize;
        let data = reader.read_bytes(packet_len)?;

        packets.rhwp_push(PpmPacket { data }).ok()?;
    }

    Some(PpmMarkerData {
        sequence_idx,
        packets,
    })
}

/// RGN marker (A.6.3).
fn rgn_marker(reader: &mut BitReader<'_>) -> Option<()> {
    rhwp_codec_resource::work(1).ok()?;
    skip_marker_segment(reader)
}

pub(crate) fn skip_marker_segment(reader: &mut BitReader<'_>) -> Option<()> {
    rhwp_codec_resource::work(1).ok()?;
    let length = reader.read_u16()?.checked_sub(2)?;
    reader.skip_bytes(length as usize)?;

    Some(())
}

/// COD marker (A.6.1).
pub(crate) fn cod_marker(reader: &mut BitReader<'_>) -> Option<CodingStyleDefault> {
    rhwp_codec_resource::work(1).ok()?;
    // Length.
    let _ = reader.read_u16()?;

    let coding_style_flags = CodingStyleFlags::from_u8(reader.read_byte()?);
    let progression_order = ProgressionOrder::from_u8(reader.read_byte()?).ok()?;

    let num_layers = reader.read_u16()?;

    // We don't support more than 32-bit (and thus 32 layers).
    if num_layers == 0 || num_layers > MAX_LAYER_COUNT as u16 {
        return None;
    }

    let mct = reader.read_byte()? == 1;

    let coding_style_parameters = coding_style_parameters(reader, &coding_style_flags)?;

    Some(CodingStyleDefault {
        progression_order,
        num_layers: num_layers as u8,
        mct,
        component_parameters: CodingStyleComponent {
            flags: coding_style_flags,
            parameters: coding_style_parameters,
        },
    })
}

/// COC marker (A.6.2).
pub(crate) fn coc_marker(
    reader: &mut BitReader<'_>,
    csiz: u16,
) -> Option<(u16, CodingStyleComponent)> {
    rhwp_codec_resource::work(1).ok()?;
    // Length.
    let _ = reader.read_u16()?;

    let component_index = if csiz < 257 {
        reader.read_byte()? as u16
    } else {
        reader.read_u16()?
    };
    let coding_style = CodingStyleFlags::from_u8(reader.read_byte()?);

    let parameters = coding_style_parameters(reader, &coding_style)?;

    let coc = CodingStyleComponent {
        flags: coding_style,
        parameters,
    };

    Some((component_index, coc))
}

/// QCD marker (A.6.4).
pub(crate) fn qcd_marker(reader: &mut BitReader<'_>) -> Option<QuantizationInfo> {
    rhwp_codec_resource::work(1).ok()?;
    // Length.
    let length = reader.read_u16()?;

    let sqcd_val = reader.read_byte()?;
    let quantization_style = QuantizationStyle::from_u8(sqcd_val & 0x1F).ok()?;
    let guard_bits = (sqcd_val >> 5) & 0x07;

    let remaining_bytes = length.checked_sub(3)? as usize;

    let mut parameters = quantization_parameters(reader, quantization_style, remaining_bytes)?;
    parameters.guard_bits = guard_bits;

    Some(parameters)
}

/// QCC marker (A.6.5).
pub(crate) fn qcc_marker(reader: &mut BitReader<'_>, csiz: u16) -> Option<(u16, QuantizationInfo)> {
    rhwp_codec_resource::work(1).ok()?;
    let length = reader.read_u16()?;

    let component_index = if csiz < 257 {
        reader.read_byte()? as u16
    } else {
        reader.read_u16()?
    };

    let sqcc_val = reader.read_byte()?;
    let quantization_style = QuantizationStyle::from_u8(sqcc_val & 0x1F).ok()?;
    let guard_bits = (sqcc_val >> 5) & 0x07;

    let component_index_size = if csiz < 257 { 1 } else { 2 };
    let remaining_bytes = length
        .checked_sub(2)?
        .checked_sub(component_index_size)?
        .checked_sub(1)? as usize;

    let mut parameters = quantization_parameters(reader, quantization_style, remaining_bytes)?;
    parameters.guard_bits = guard_bits;

    Some((component_index, parameters))
}

fn quantization_parameters(
    reader: &mut BitReader<'_>,
    quantization_style: QuantizationStyle,
    remaining_bytes: usize,
) -> Option<QuantizationInfo> {
    rhwp_codec_resource::work(1).ok()?;
    let mut step_sizes = Vec::new();

    let irreversible = |val: u16| {
        let exponent = val >> 11;
        let mantissa = val & ((1 << 11) - 1);

        StepSize { exponent, mantissa }
    };

    match quantization_style {
        QuantizationStyle::NoQuantization => {
            // 8 bits per band (5 bits exponent, 3 bits reserved)
            {
                let (__rhwp_iter_7, __rhwp_iter_7_prepaid) =
                    rhwp_codec_resource::iterations(0..remaining_bytes).ok()?;
                for _ in __rhwp_iter_7 {
                    if !__rhwp_iter_7_prepaid {
                        rhwp_codec_resource::work(1).ok()?;
                    }
                    let value = reader.read_byte()? as u16;
                    step_sizes
                        .rhwp_push(StepSize {
                            // Unused.
                            mantissa: 0,
                            exponent: (value >> 3),
                        })
                        .ok()?;
                }
            }
        }
        QuantizationStyle::ScalarDerived => {
            let value = reader.read_u16()?;
            step_sizes.rhwp_push(irreversible(value)).ok()?;
        }
        QuantizationStyle::ScalarExpounded => {
            let num_bands = remaining_bytes / 2;
            {
                let (__rhwp_iter_8, __rhwp_iter_8_prepaid) =
                    rhwp_codec_resource::iterations(0..num_bands).ok()?;
                for _ in __rhwp_iter_8 {
                    if !__rhwp_iter_8_prepaid {
                        rhwp_codec_resource::work(1).ok()?;
                    }
                    let value = reader.read_u16()?;

                    step_sizes.rhwp_push(irreversible(value)).ok()?;
                }
            }
        }
    }

    Some(QuantizationInfo {
        quantization_style,
        guard_bits: 0,
        step_sizes,
    })
}

#[allow(
    unused,
    reason = "Not all marker codes are used in every decoding path yet"
)]
/// Marker codes (Table A.2).
pub(crate) mod markers {
    /// Start of codestream - 'SOC'.
    pub(crate) const SOC: u8 = 0x4F;
    /// Start of tile-part - 'SOT'.
    pub(crate) const SOT: u8 = 0x90;
    /// Start of data - 'SOD'.
    pub(crate) const SOD: u8 = 0x93;
    /// End of codestream - 'EOC'.
    pub(crate) const EOC: u8 = 0xD9;

    /// Image and tile size - 'SIZ'.
    pub(crate) const SIZ: u8 = 0x51;

    /// Coding style default - 'COD'.
    pub(crate) const COD: u8 = 0x52;
    /// Coding component - 'COC'.
    pub(crate) const COC: u8 = 0x53;
    /// Region-of-interest - 'RGN'.
    pub(crate) const RGN: u8 = 0x5E;
    /// Quantization default - 'QCD'.
    pub(crate) const QCD: u8 = 0x5C;
    /// Quantization component - 'QCC'.
    pub(crate) const QCC: u8 = 0x5D;
    /// Progression order change - 'POC'.
    pub(crate) const POC: u8 = 0x5F;

    /// Tile-part lengths - 'TLM'.
    pub(crate) const TLM: u8 = 0x55;
    /// Packet length, main header - 'PLM'.
    pub(crate) const PLM: u8 = 0x57;
    /// Packet length, tile-part header - 'PLT'.
    pub(crate) const PLT: u8 = 0x58;
    /// Packed packet headers, main header - 'PPM'.
    pub(crate) const PPM: u8 = 0x60;
    /// Packed packet headers, tile-part header - 'PPT'.
    pub(crate) const PPT: u8 = 0x61;

    /// Start of packet - 'SOP'.
    pub(crate) const SOP: u8 = 0x91;
    /// End of packet header - 'EPH'.
    pub(crate) const EPH: u8 = 0x92;

    /// Component registration - 'CRG'.
    pub(crate) const CRG: u8 = 0x63;
    /// Comment - 'COM'.
    pub(crate) const COM: u8 = 0x64;

    pub(crate) fn to_string(marker: u8) -> &'static str {
        match marker {
            // Delimiting markers.
            SOC => "SOC",
            SOT => "SOT",
            SOD => "SOD",
            EOC => "EOC",

            // Fixed information.
            SIZ => "SIZ",

            // Functional markers.
            COD => "COD",
            COC => "COC",
            RGN => "RGN",
            QCD => "QCD",
            QCC => "QCC",
            POC => "POC",

            // Pointer markers.
            TLM => "TLM",
            PLM => "PLM",
            PLT => "PLT",
            PPM => "PPM",
            PPT => "PPT",

            // In-bit-stream markers.
            SOP => "SOP",
            EPH => "EPH",

            // Informational markers.
            CRG => "CRG",
            COM => "COM",

            _ => "UNKNOWN",
        }
    }
}

impl QuantizationInfo {
    pub(crate) fn try_clone(&self) -> Result<Self> {
        Ok(Self {
            quantization_style: self.quantization_style,
            guard_bits: self.guard_bits,
            step_sizes: rhwp_codec_resource::copy(&self.step_sizes)?,
        })
    }
}
impl CodingStyleComponent {
    pub(crate) fn try_clone(&self) -> Result<Self> {
        Ok(Self {
            flags: self.flags,
            parameters: self.parameters.try_clone()?,
        })
    }
}
impl CodingStyleParameters {
    pub(crate) fn try_clone(&self) -> Result<Self> {
        Ok(Self {
            num_decomposition_levels: self.num_decomposition_levels,
            num_resolution_levels: self.num_resolution_levels,
            code_block_width: self.code_block_width,
            code_block_height: self.code_block_height,
            code_block_style: self.code_block_style,
            transformation: self.transformation,
            precinct_exponents: rhwp_codec_resource::copy(&self.precinct_exponents)?,
        })
    }
}
impl ComponentInfo {
    pub(crate) fn try_clone(&self) -> Result<Self> {
        Ok(Self {
            size_info: self.size_info,
            coding_style: self.coding_style.try_clone()?,
            quantization_info: self.quantization_info.try_clone()?,
        })
    }
}
