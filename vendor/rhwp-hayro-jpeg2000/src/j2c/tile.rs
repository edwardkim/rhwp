//! Creating tiles and parsing their constituent tile parts.

use alloc::vec;
use alloc::vec::Vec;
use rhwp_codec_resource::IteratorResource as _;
use rhwp_codec_resource::VecResource as _;

use super::build::{PrecinctData, SubBandType};
use super::codestream::{ComponentInfo, Header, ProgressionOrder, markers, skip_marker_segment};
use super::rect::IntRect;
use crate::error::{MarkerError, Result, TileError, ValidationError, bail, err};
use crate::j2c::codestream;
use crate::reader::BitReader;

/// A single tile in the image.
#[derive(Clone, Debug)]
pub(crate) struct Tile<'a> {
    /// The index of the tile, in row-major order.
    pub(crate) idx: u32,
    /// The concatenated tile parts that contain all the information for all
    /// constituent codeblocks.
    pub(crate) tile_parts: Vec<TilePart<'a>>,
    /// Parameters for each component. In most cases, those are directly
    /// inherited from the main header. But in some cases, individual tiles
    /// might override them.
    pub(crate) component_infos: Vec<ComponentInfo>,
    /// The rectangle making up the area of the tile. `x1` and `y1` are
    /// exclusive.
    pub(crate) rect: IntRect,
    pub(crate) progression_order: ProgressionOrder,
    pub(crate) num_layers: u8,
    pub(crate) mct: bool,
}

/// A tile part where packet headers and packet data are interleaved.
#[derive(Clone, Debug)]
pub(crate) struct MergedTilePart<'a> {
    pub(crate) data: BitReader<'a>,
}

/// A tile part where packet headers and packet data are separated.
#[derive(Clone, Debug)]
pub(crate) struct SeparatedTilePart<'a> {
    pub(crate) headers: Vec<BitReader<'a>>,
    pub(crate) active_header_reader: usize,
    pub(crate) body: BitReader<'a>,
}

#[derive(Clone, Debug)]
pub(crate) enum TilePart<'a> {
    Merged(MergedTilePart<'a>),
    Separated(SeparatedTilePart<'a>),
}

impl<'a> TilePart<'a> {
    pub(crate) fn header(&mut self) -> &mut BitReader<'a> {
        match self {
            TilePart::Merged(m) => &mut m.data,
            TilePart::Separated(s) => {
                if s.headers[s.active_header_reader].at_end()
                    && s.headers.len() - 1 > s.active_header_reader
                {
                    s.active_header_reader += 1;
                }

                &mut s.headers[s.active_header_reader]
            }
        }
    }

    pub(crate) fn body(&mut self) -> &mut BitReader<'a> {
        match self {
            TilePart::Merged(m) => &mut m.data,
            TilePart::Separated(s) => &mut s.body,
        }
    }
}

impl<'a> Tile<'a> {
    fn new(idx: u32, header: &Header<'_>) -> Result<Self> {
        let rect = {
            let size_data = &header.size_data;

            let x_coord = size_data.tile_x_coord(idx);
            let y_coord = size_data.tile_y_coord(idx);

            // See B-7, B-8, B-9 and B-10.
            let x0 = u32::max(
                u32::try_from(
                    u64::from(size_data.tile_x_offset)
                        + u64::from(x_coord) * u64::from(size_data.tile_width),
                )
                .map_err(|_| ValidationError::InvalidDimensions)?,
                size_data.image_area_x_offset,
            );
            let y0 = u32::max(
                u32::try_from(
                    u64::from(size_data.tile_y_offset)
                        + u64::from(y_coord) * u64::from(size_data.tile_height),
                )
                .map_err(|_| ValidationError::InvalidDimensions)?,
                size_data.image_area_y_offset,
            );

            // Note that `x1` and `y1` are exclusive.
            let x1 = u32::min(
                (u64::from(size_data.tile_x_offset)
                    + (u64::from(x_coord) + 1) * u64::from(size_data.tile_width))
                .min(u64::from(size_data.reference_grid_width)) as u32,
                size_data.reference_grid_width,
            );
            let y1 = u32::min(
                (u64::from(size_data.tile_y_offset)
                    + (u64::from(y_coord) + 1) * u64::from(size_data.tile_height))
                .min(u64::from(size_data.reference_grid_height)) as u32,
                size_data.reference_grid_height,
            );

            IntRect::from_ltrb(x0, y0, x1, y1)
        };

        Ok(Tile {
            idx,
            // Will be filled once we start parsing.
            tile_parts: vec![],
            rect,
            // By default, each tile inherits the settings from the main
            // header. When parsing the tile parts, some of these settings
            // might be overridden.
            component_infos: rhwp_codec_resource::try_collect(
                header.component_infos.iter().map(ComponentInfo::try_clone),
            )?,
            progression_order: header.global_coding_style.progression_order,
            mct: header.global_coding_style.mct,
            num_layers: header.global_coding_style.num_layers,
        })
    }

    pub(crate) fn component_tiles(&self) -> impl Iterator<Item = ComponentTile<'_>> {
        self.component_infos
            .iter()
            .map(|i| ComponentTile::new(self, i))
    }
}

/// Create the tiles and parse their constituent tile parts.
pub(crate) fn parse<'a>(
    reader: &mut BitReader<'a>,
    main_header: &'a Header<'a>,
) -> Result<Vec<Tile<'a>>> {
    rhwp_codec_resource::work(1)?;
    let mut tiles = rhwp_codec_resource::try_collect(
        (0..rhwp_codec_resource::product(&[
            main_header.size_data.num_x_tiles() as usize,
            main_header.size_data.num_y_tiles() as usize,
        ])?)
            .map(|idx| Tile::new(idx as u32, main_header)),
    )?;

    let mut tile_part_idx = 0;

    parse_tile_part(reader, main_header, &mut tiles, tile_part_idx)?;
    tile_part_idx += 1;

    while reader.peek_marker() == Some(markers::SOT) {
        rhwp_codec_resource::work(1)?;
        parse_tile_part(reader, main_header, &mut tiles, tile_part_idx)?;
        tile_part_idx += 1;
    }

    if main_header.strict && reader.read_marker()? != markers::EOC {
        bail!(MarkerError::Expected("EOC"));
    }

    Ok(tiles)
}

fn parse_tile_part<'a>(
    reader: &mut BitReader<'a>,
    main_header: &'a Header<'a>,
    tiles: &mut [Tile<'a>],
    tile_part_idx: usize,
) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    if reader.read_marker()? != markers::SOT {
        bail!(MarkerError::Expected("SOT"));
    }

    let tile_part_header = sot_marker(reader).ok_or(MarkerError::ParseFailure("SOT"))?;

    if tile_part_header.tile_index as u32 >= main_header.size_data.num_tiles() {
        bail!(TileError::InvalidIndex);
    }

    let data_len = if tile_part_header.tile_part_length == 0 {
        reader.tail().map(|d| d.len()).unwrap_or(0)
    } else {
        // Subtract 12 to account for the marker length.

        (tile_part_header.tile_part_length as usize)
            .checked_sub(12)
            .ok_or(TileError::Invalid)?
    };

    let start = reader.offset();

    let tile = &mut tiles[tile_part_header.tile_index as usize];
    let num_components = tile.component_infos.len();

    let mut ppt_headers = vec![];

    loop {
        rhwp_codec_resource::work(1)?;
        let Some(marker) = reader.peek_marker() else {
            return if main_header.strict {
                err!(MarkerError::Invalid)
            } else {
                Ok(())
            };
        };

        match marker {
            markers::SOD => {
                reader.read_marker()?;
                break;
            }
            // COD, COC, QCD and QCC should only be used in the _first_
            // tile-part header, if they appear at all.
            markers::COD => {
                reader.read_marker()?;
                let cod = codestream::cod_marker(reader).ok_or(MarkerError::ParseFailure("COD"))?;

                tile.mct = cod.mct;
                tile.num_layers = cod.num_layers;
                tile.progression_order = cod.progression_order;

                {
                    let (__rhwp_iter_1, __rhwp_iter_1_prepaid) =
                        rhwp_codec_resource::iterations(&mut tile.component_infos)?;
                    for component in __rhwp_iter_1 {
                        if !__rhwp_iter_1_prepaid {
                            rhwp_codec_resource::work(1)?;
                        }
                        component.coding_style.flags.raw |= cod.component_parameters.flags.raw;
                        component.coding_style.parameters =
                            cod.component_parameters.parameters.try_clone()?;
                    }
                }
            }
            markers::COC => {
                reader.read_marker()?;

                let (component_index, coc) = codestream::coc_marker(reader, num_components as u16)
                    .ok_or(MarkerError::ParseFailure("COC"))?;

                let old = tile
                    .component_infos
                    .get_mut(component_index as usize)
                    .ok_or(ValidationError::InvalidComponentMetadata)?;

                old.coding_style.parameters = coc.parameters;
                old.coding_style.flags.raw |= coc.flags.raw;
            }
            markers::QCD => {
                reader.read_marker()?;
                let qcd = codestream::qcd_marker(reader).ok_or(MarkerError::ParseFailure("QCD"))?;

                {
                    let (__rhwp_iter_2, __rhwp_iter_2_prepaid) =
                        rhwp_codec_resource::iterations(&mut tile.component_infos)?;
                    for component_info in __rhwp_iter_2 {
                        if !__rhwp_iter_2_prepaid {
                            rhwp_codec_resource::work(1)?;
                        }
                        component_info.quantization_info = qcd.try_clone()?;
                    }
                }
            }
            markers::QCC => {
                reader.read_marker()?;
                let (component_index, qcc) = codestream::qcc_marker(reader, num_components as u16)
                    .ok_or(MarkerError::ParseFailure("QCC"))?;

                tile.component_infos
                    .get_mut(component_index as usize)
                    .ok_or(ValidationError::InvalidComponentMetadata)?
                    .quantization_info = qcc;
            }
            markers::EOC => break,
            markers::PPT => {
                if !main_header.ppm_packets.is_empty() {
                    bail!(TileError::PpmPptConflict);
                }

                reader.read_marker()?;
                ppt_headers
                    .rhwp_push(ppt_marker(reader).ok_or(MarkerError::ParseFailure("PPT"))?)?;
            }
            markers::PLT => {
                // Can be inferred ourselves.
                reader.read_marker()?;
                skip_marker_segment(reader).ok_or(MarkerError::ParseFailure("PLT"))?;
            }
            markers::COM => {
                reader.read_marker()?;
                skip_marker_segment(reader).ok_or(MarkerError::ParseFailure("COM"))?;
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

    let remaining_bytes = if let Some(len) = data_len.checked_sub(reader.offset() - start) {
        len
    } else {
        return if main_header.strict {
            err!(TileError::Invalid)
        } else {
            Ok(())
        };
    };

    rhwp_codec_resource::sort(&mut ppt_headers, |p1, p2| {
        p1.sequence_idx.cmp(&p2.sequence_idx)
    })?;
    let mut headers: Vec<_> = ppt_headers
        .iter()
        .map(|i| BitReader::new(i.data))
        .rhwp_collect()?;

    if let Some(ppm_marker) = main_header.ppm_packets.get(tile_part_idx) {
        headers.rhwp_push(BitReader::new(ppm_marker.data))?;
    }

    let data = reader
        .read_bytes(remaining_bytes)
        .ok_or(TileError::Invalid)?;

    let tile_part = if !headers.is_empty() {
        TilePart::Separated(SeparatedTilePart {
            headers,
            active_header_reader: 0,
            body: BitReader::new(data),
        })
    } else {
        TilePart::Merged(MergedTilePart {
            data: BitReader::new(data),
        })
    };

    tile.tile_parts.rhwp_push(tile_part)?;

    Ok(())
}

/// A tile, instantiated to a specific component.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ComponentTile<'a> {
    pub(crate) tile: &'a Tile<'a>,
    /// The information of the component of the tile.
    pub(crate) component_info: &'a ComponentInfo,
    /// The rectangle of the component tile.
    pub(crate) rect: IntRect,
}

impl<'a> ComponentTile<'a> {
    pub(crate) fn new(tile: &'a Tile<'a>, component_info: &'a ComponentInfo) -> Self {
        let tile_rect = tile.rect;

        let rect = if component_info.size_info.horizontal_resolution == 1
            && component_info.size_info.vertical_resolution == 1
        {
            tile_rect
        } else {
            // As described in B-12.
            let t_x0 = tile_rect
                .x0
                .div_ceil(component_info.size_info.horizontal_resolution as u32);
            let t_y0 = tile_rect
                .y0
                .div_ceil(component_info.size_info.vertical_resolution as u32);
            let t_x1 = tile_rect
                .x1
                .div_ceil(component_info.size_info.horizontal_resolution as u32);
            let t_y1 = tile_rect
                .y1
                .div_ceil(component_info.size_info.vertical_resolution as u32);

            IntRect::from_ltrb(t_x0, t_y0, t_x1, t_y1)
        };

        ComponentTile {
            tile,
            component_info,
            rect,
        }
    }

    pub(crate) fn resolution_tiles(&self) -> impl Iterator<Item = ResolutionTile<'_>> {
        (0..self
            .component_info
            .coding_style
            .parameters
            .num_resolution_levels)
            .map(|r| ResolutionTile::new(*self, r))
    }
}

/// A tile instantiated to a specific resolution of a component tile.
pub(crate) struct ResolutionTile<'a> {
    /// The resolution of the tile.
    pub(crate) resolution: u8,
    /// The decomposition level of the tile.
    pub(crate) decomposition_level: u8,
    /// The underlying component tile.
    pub(crate) component_tile: ComponentTile<'a>,
    /// The rectangle of the resolution tile.
    pub(crate) rect: IntRect,
}

impl<'a> ResolutionTile<'a> {
    pub(crate) fn new(component_tile: ComponentTile<'a>, resolution: u8) -> Self {
        assert!(
            component_tile
                .component_info
                .coding_style
                .parameters
                .num_resolution_levels
                > resolution
        );

        let rect = {
            // See formula B-14.
            let n_l = component_tile
                .component_info
                .coding_style
                .parameters
                .num_decomposition_levels;

            let tx0 = (component_tile.rect.x0 as u64)
                .div_ceil(2_u64.pow(n_l as u32 - resolution as u32)) as u32;
            let ty0 = (component_tile.rect.y0 as u64)
                .div_ceil(2_u64.pow(n_l as u32 - resolution as u32)) as u32;
            let tx1 = (component_tile.rect.x1 as u64)
                .div_ceil(2_u64.pow(n_l as u32 - resolution as u32)) as u32;
            let ty1 = (component_tile.rect.y1 as u64)
                .div_ceil(2_u64.pow(n_l as u32 - resolution as u32)) as u32;

            IntRect::from_ltrb(tx0, ty0, tx1, ty1)
        };

        // Decomposition level and resolution level are inversely related
        // to each other. In addition to that, there is always one more
        // resolution than decomposition levels (resolution level 0 only
        // include the LL subband of the N_L decomposition, resolution level
        // 1 includes the HL, LH and HH subbands of the N_L decomposition.
        let decomposition_level = {
            if resolution == 0 {
                component_tile
                    .component_info
                    .coding_style
                    .parameters
                    .num_decomposition_levels
            } else {
                component_tile
                    .component_info
                    .coding_style
                    .parameters
                    .num_decomposition_levels
                    - (resolution - 1)
            }
        };

        ResolutionTile {
            resolution,
            decomposition_level,
            component_tile,
            rect,
        }
    }

    pub(crate) fn sub_band_rect(&self, sub_band_type: SubBandType) -> IntRect {
        // This is the only permissible sub-band type for the given resolution.
        if self.resolution == 0 {
            assert_eq!(sub_band_type, SubBandType::LowLow);
        }

        // Formula B-15.

        let xo_b = if matches!(sub_band_type, SubBandType::HighLow | SubBandType::HighHigh) {
            1
        } else {
            0
        };
        let yo_b = if matches!(sub_band_type, SubBandType::LowHigh | SubBandType::HighHigh) {
            1
        } else {
            0
        };

        let mut numerator_x = 0;
        let mut numerator_y = 0;

        // If decomposition level is 0, xo_b and yo_b are 0 as well.
        if self.decomposition_level > 0 {
            numerator_x = 2_u64.pow(self.decomposition_level as u32 - 1) * xo_b as u64;
            numerator_y = 2_u64.pow(self.decomposition_level as u32 - 1) * yo_b as u64;
        }

        let denominator = 2_u64.pow(self.decomposition_level as u32);

        let tbx_0 = (self.component_tile.rect.x0 as u64)
            .saturating_sub(numerator_x)
            .div_ceil(denominator) as u32;
        let tbx_1 = (self.component_tile.rect.x1 as u64)
            .saturating_sub(numerator_x)
            .div_ceil(denominator) as u32;
        let tby_0 = (self.component_tile.rect.y0 as u64)
            .saturating_sub(numerator_y)
            .div_ceil(denominator) as u32;
        let tby_1 = (self.component_tile.rect.y1 as u64)
            .saturating_sub(numerator_y)
            .div_ceil(denominator) as u32;

        IntRect::from_ltrb(tbx_0, tby_0, tbx_1, tby_1)
    }

    /// The exponent for determining the horizontal size of a precinct.
    ///
    /// `PPx` in the specification.
    fn precinct_exponent_x(&self) -> u8 {
        self.component_tile
            .component_info
            .coding_style
            .parameters
            .precinct_exponents[self.resolution as usize]
            .0
    }

    /// The exponent for determining the vertical size of a precinct.
    ///
    /// `PPx` in the specification.
    fn precinct_exponent_y(&self) -> u8 {
        self.component_tile
            .component_info
            .coding_style
            .parameters
            .precinct_exponents[self.resolution as usize]
            .1
    }

    fn num_precincts_x(&self) -> u32 {
        // See B-16.
        let IntRect { x0, x1, .. } = self.rect;

        if x0 == x1 {
            0
        } else {
            x1.div_ceil(2_u32.pow(self.precinct_exponent_x() as u32))
                - x0 / 2_u32.pow(self.precinct_exponent_x() as u32)
        }
    }

    fn num_precincts_y(&self) -> u32 {
        // See B-16.
        let IntRect { y0, y1, .. } = self.rect;

        if y0 == y1 {
            0
        } else {
            y1.div_ceil(2_u32.pow(self.precinct_exponent_y() as u32))
                - y0 / 2_u32.pow(self.precinct_exponent_y() as u32)
        }
    }

    pub(crate) fn num_precincts(&self) -> u64 {
        self.num_precincts_x() as u64 * self.num_precincts_y() as u64
    }

    /// Return an iterator over the data of the precincts in this resolution
    /// tile.
    pub(crate) fn precincts(&self) -> Option<impl Iterator<Item = PrecinctData>> {
        rhwp_codec_resource::work(1).ok()?;
        let num_precincts_y = self.num_precincts_y();
        let num_precincts_x = self.num_precincts_x();

        let mut ppx = self.precinct_exponent_x();
        let mut ppy = self.precinct_exponent_y();

        let mut y_start = (self.rect.y0 / (1 << ppy)) * (1 << ppy);
        let mut x_start = (self.rect.x0 / (1 << ppx)) * (1 << ppx);

        // It is unclear why this is necessary, but it is. The spec only
        // mentions that ppx/ppy must be decreased when calculating codeblock
        // dimensions, but not that it's necessary for precincts as well.
        if self.resolution > 0 {
            ppx = ppx.checked_sub(1)?;
            ppy = ppy.checked_sub(1)?;

            x_start /= 2;
            y_start /= 2;
        }

        let ppx_pow2 = 1_u32 << ppx;
        let ppy_pow2 = 1_u32 << ppy;

        let nl_minus_r = self
            .component_tile
            .component_info
            .num_decomposition_levels()
            - self.resolution;

        let x_stride =
            1_u32.checked_shl(self.precinct_exponent_x().checked_add(nl_minus_r)? as u32)?;
        let y_stride =
            1_u32.checked_shl(self.precinct_exponent_y().checked_add(nl_minus_r)? as u32)?;

        let precinct_x_step = (self
            .component_tile
            .component_info
            .size_info
            .horizontal_resolution as u32)
            .checked_mul(x_stride)?;

        let precinct_y_step = (self
            .component_tile
            .component_info
            .size_info
            .vertical_resolution as u32)
            .checked_mul(y_stride)?;

        // These variables are used to map the start coordinates of each
        // precinct _on the reference grid_. Remember that the first
        // precinct in each row/column is at the start position of the tile
        // which might not be a multiple of precinct exponent, but all subsequent
        // precincts are at a multiple of the exponent.
        let mut r_x = self.component_tile.tile.rect.x0;
        let mut r_y = self.component_tile.tile.rect.y0;

        // The second part of the condition in the formula in B.12.1.3. If it
        // is divisible, then we can't take the x/y position of the tile
        // as the start of the precinct, but instead have to advance to the
        // next multiple.
        if !r_x.is_multiple_of(precinct_x_step)
            && (self.rect.x0 * (1 << nl_minus_r)).is_multiple_of(precinct_x_step)
        {
            r_x = r_x.checked_next_multiple_of(precinct_x_step)?;
        }

        // Same as above.
        if !r_y.is_multiple_of(precinct_y_step)
            && (self.rect.y0 * (1 << nl_minus_r)).is_multiple_of(precinct_y_step)
        {
            r_y = r_y.checked_next_multiple_of(precinct_y_step)?;
        }

        let iter = (0..num_precincts_y).flat_map(move |y| {
            let y0 = y * ppy_pow2 + y_start;
            let mut r_x = r_x;

            let res = (0..num_precincts_x).map(move |x| {
                let x0 = x * ppx_pow2 + x_start;

                let data = PrecinctData {
                    r_x,
                    r_y,
                    rect: IntRect::from_xywh(x0, y0, ppx_pow2, ppy_pow2),
                    idx: num_precincts_x as u64 * y as u64 + x as u64,
                };

                // If r_x is already aligned, we simply step by `precinct_x_step`.
                // Otherwise (can only be the case for precincts in the first
                // row or column), align to the next multiple.
                r_x = (r_x + 1).next_multiple_of(precinct_x_step);

                data
            });

            // Same as for r_x.
            r_y = (r_y + 1).next_multiple_of(precinct_y_step);

            res
        });

        Some(iter)
    }

    pub(crate) fn code_block_width(&self) -> u32 {
        // See B-17.
        let xcb = self
            .component_tile
            .component_info
            .coding_style
            .parameters
            .code_block_width;

        let xcb = if self.resolution > 0 {
            u8::min(xcb, self.precinct_exponent_x() - 1)
        } else {
            u8::min(xcb, self.precinct_exponent_x())
        };

        2_u32.pow(xcb as u32)
    }

    pub(crate) fn code_block_height(&self) -> u32 {
        // See B-18.
        let ycb = self
            .component_tile
            .component_info
            .coding_style
            .parameters
            .code_block_height;

        let ycb = if self.resolution > 0 {
            u8::min(ycb, self.precinct_exponent_y() - 1)
        } else {
            u8::min(ycb, self.precinct_exponent_y())
        };

        2_u32.pow(ycb as u32)
    }
}

struct TilePartHeader {
    tile_index: u16,
    tile_part_length: u32,
}

struct PptMarkerData<'a> {
    data: &'a [u8],
    sequence_idx: u8,
}

/// PPT marker (A.7.5).
fn ppt_marker<'a>(reader: &mut BitReader<'a>) -> Option<PptMarkerData<'a>> {
    rhwp_codec_resource::work(1).ok()?;
    let length = reader.read_u16()?.checked_sub(2)?;
    let header_len = length.checked_sub(1)?;
    let sequence_idx = reader.read_byte()?;
    Some(PptMarkerData {
        data: reader.read_bytes(header_len as usize)?,
        sequence_idx,
    })
}

/// SOT marker (A.4.2).
fn sot_marker(reader: &mut BitReader<'_>) -> Option<TilePartHeader> {
    rhwp_codec_resource::work(1).ok()?;
    // Length.
    let _ = reader.read_u16()?;

    let tile_index = reader.read_u16()?;
    let tile_part_length = reader.read_u32()?;

    // We infer those ourselves.
    let _tile_part_index = reader.read_byte()? as u16;
    let _num_tile_parts = reader.read_byte()?;

    Some(TilePartHeader {
        tile_index,
        tile_part_length,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::j2c::codestream::{
        CodeBlockStyle, CodingStyleComponent, CodingStyleDefault, CodingStyleFlags,
        CodingStyleParameters, ComponentSizeInfo, QuantizationInfo, QuantizationStyle, SizeData,
        WaveletTransform,
    };

    /// Test case for the example in B.4.
    #[test]
    fn test_jpeg2000_standard_example_b4() {
        let component_size_info_0 = ComponentSizeInfo {
            precision: 8,
            horizontal_resolution: 1,
            vertical_resolution: 1,
        };

        let dummy_component_coding_style = CodingStyleComponent {
            flags: CodingStyleFlags::default(),
            parameters: CodingStyleParameters {
                num_decomposition_levels: 0,
                num_resolution_levels: 0,
                code_block_width: 0,
                code_block_height: 0,
                code_block_style: CodeBlockStyle::default(),
                transformation: WaveletTransform::Irreversible97,
                precinct_exponents: vec![],
            },
        };

        let dummy_quantization_info = QuantizationInfo {
            quantization_style: QuantizationStyle::NoQuantization,
            guard_bits: 0,
            step_sizes: vec![],
        };

        let component_info_0 = ComponentInfo {
            size_info: component_size_info_0,
            coding_style: dummy_component_coding_style.clone(),
            quantization_info: dummy_quantization_info.clone(),
        };

        let component_size_info_1 = ComponentSizeInfo {
            precision: 8,
            horizontal_resolution: 2,
            vertical_resolution: 2,
        };

        let component_info_1 = ComponentInfo {
            size_info: component_size_info_1,
            coding_style: dummy_component_coding_style.clone(),
            quantization_info: dummy_quantization_info.clone(),
        };

        let size_data = SizeData {
            reference_grid_width: 1432,
            reference_grid_height: 954,
            image_area_x_offset: 152,
            image_area_y_offset: 234,
            tile_width: 396,
            tile_height: 297,
            tile_x_offset: 0,
            tile_y_offset: 0,
            component_sizes: vec![component_size_info_0, component_size_info_1],
            x_shrink_factor: 1,
            y_shrink_factor: 1,
            x_resolution_shrink_factor: 1,
            y_resolution_shrink_factor: 1,
        };

        assert_eq!(size_data.image_width(), 1280);
        assert_eq!(size_data.image_height(), 720);

        assert_eq!(size_data.num_x_tiles(), 4);
        assert_eq!(size_data.num_y_tiles(), 4);
        assert_eq!(size_data.num_tiles(), 16);

        let header = Header {
            size_data,
            // Just dummy values.
            global_coding_style: CodingStyleDefault {
                progression_order: ProgressionOrder::LayerResolutionComponentPosition,
                num_layers: 0,
                mct: false,
                component_parameters: CodingStyleComponent {
                    flags: CodingStyleFlags::default(),
                    parameters: CodingStyleParameters {
                        num_decomposition_levels: 0,
                        num_resolution_levels: 0,
                        code_block_width: 0,
                        code_block_height: 0,
                        code_block_style: CodeBlockStyle::default(),
                        transformation: WaveletTransform::Irreversible97,
                        precinct_exponents: vec![],
                    },
                },
            },
            component_infos: vec![],
            ppm_packets: vec![],
            skipped_resolution_levels: 0,
            strict: false,
        };

        let tile_0_0 = Tile::new(0, &header).unwrap();
        let coords_0_0 = ComponentTile::new(&tile_0_0, &component_info_0).rect;
        assert_eq!(coords_0_0.x0, 152);
        assert_eq!(coords_0_0.y0, 234);
        assert_eq!(coords_0_0.x1, 396);
        assert_eq!(coords_0_0.y1, 297);
        assert_eq!(coords_0_0.width(), 244);
        assert_eq!(coords_0_0.height(), 63);

        let tile_1_0 = Tile::new(1, &header).unwrap();
        let coords_1_0 = ComponentTile::new(&tile_1_0, &component_info_0).rect;
        assert_eq!(coords_1_0.x0, 396);
        assert_eq!(coords_1_0.y0, 234);
        assert_eq!(coords_1_0.x1, 792);
        assert_eq!(coords_1_0.y1, 297);
        assert_eq!(coords_1_0.width(), 396);
        assert_eq!(coords_1_0.height(), 63);

        let tile_0_1 = Tile::new(4, &header).unwrap();
        let coords_0_1 = ComponentTile::new(&tile_0_1, &component_info_0).rect;
        assert_eq!(coords_0_1.x0, 152);
        assert_eq!(coords_0_1.y0, 297);
        assert_eq!(coords_0_1.x1, 396);
        assert_eq!(coords_0_1.y1, 594);
        assert_eq!(coords_0_1.width(), 244);
        assert_eq!(coords_0_1.height(), 297);

        let tile_1_1 = Tile::new(5, &header).unwrap();
        let coords_1_1 = ComponentTile::new(&tile_1_1, &component_info_0).rect;
        assert_eq!(coords_1_1.x0, 396);
        assert_eq!(coords_1_1.y0, 297);
        assert_eq!(coords_1_1.x1, 792);
        assert_eq!(coords_1_1.y1, 594);
        assert_eq!(coords_1_1.width(), 396);
        assert_eq!(coords_1_1.height(), 297);

        let tile_3_3 = Tile::new(15, &header).unwrap();
        let coords_3_3 = ComponentTile::new(&tile_3_3, &component_info_0).rect;
        assert_eq!(coords_3_3.x0, 1188);
        assert_eq!(coords_3_3.y0, 891);
        assert_eq!(coords_3_3.x1, 1432);
        assert_eq!(coords_3_3.y1, 954);
        assert_eq!(coords_3_3.width(), 244);
        assert_eq!(coords_3_3.height(), 63);

        let tile_0_0_comp1 = ComponentTile::new(&tile_0_0, &component_info_1).rect;
        assert_eq!(tile_0_0_comp1.x0, 76);
        assert_eq!(tile_0_0_comp1.y0, 117);
        assert_eq!(tile_0_0_comp1.x1, 198);
        assert_eq!(tile_0_0_comp1.y1, 149);
        assert_eq!(tile_0_0_comp1.width(), 122);
        assert_eq!(tile_0_0_comp1.height(), 32);

        let tile_1_0_comp1 = ComponentTile::new(&tile_1_0, &component_info_1).rect;
        assert_eq!(tile_1_0_comp1.x0, 198);
        assert_eq!(tile_1_0_comp1.y0, 117);
        assert_eq!(tile_1_0_comp1.x1, 396);
        assert_eq!(tile_1_0_comp1.y1, 149);
        assert_eq!(tile_1_0_comp1.width(), 198);
        assert_eq!(tile_1_0_comp1.height(), 32);

        let tile_0_1_comp1 = ComponentTile::new(&tile_0_1, &component_info_1).rect;
        assert_eq!(tile_0_1_comp1.x0, 76);
        assert_eq!(tile_0_1_comp1.y0, 149);
        assert_eq!(tile_0_1_comp1.x1, 198);
        assert_eq!(tile_0_1_comp1.y1, 297);
        assert_eq!(tile_0_1_comp1.width(), 122);
        assert_eq!(tile_0_1_comp1.height(), 148);

        let tile_1_1_comp1 = ComponentTile::new(&tile_1_1, &component_info_1).rect;
        assert_eq!(tile_1_1_comp1.x0, 198);
        assert_eq!(tile_1_1_comp1.y0, 149);
        assert_eq!(tile_1_1_comp1.x1, 396);
        assert_eq!(tile_1_1_comp1.y1, 297);
        assert_eq!(tile_1_1_comp1.width(), 198);
        assert_eq!(tile_1_1_comp1.height(), 148);

        let tile_2_1 = Tile::new(6, &header).unwrap();
        let tile_2_1_comp1 = ComponentTile::new(&tile_2_1, &component_info_1).rect;
        assert_eq!(tile_2_1_comp1.x0, 396);
        assert_eq!(tile_2_1_comp1.y0, 149);
        assert_eq!(tile_2_1_comp1.x1, 594);
        assert_eq!(tile_2_1_comp1.y1, 297);
        assert_eq!(tile_2_1_comp1.width(), 198);
        assert_eq!(tile_2_1_comp1.height(), 148);

        assert_eq!(tile_1_1_comp1.width(), tile_2_1_comp1.width());
        assert_eq!(tile_1_1_comp1.height(), tile_2_1_comp1.height());
    }
}

impl<'a> TilePart<'a> {
    pub(crate) fn try_clone(&self) -> Result<Self> {
        Ok(match self {
            Self::Merged(m) => Self::Merged(m.clone()),
            Self::Separated(s) => Self::Separated(SeparatedTilePart {
                headers: rhwp_codec_resource::try_collect(
                    s.headers
                        .iter()
                        .map(|h| Ok::<_, crate::error::DecodeError>(h.clone())),
                )?,
                active_header_reader: s.active_header_reader,
                body: s.body.clone(),
            }),
        })
    }
}
