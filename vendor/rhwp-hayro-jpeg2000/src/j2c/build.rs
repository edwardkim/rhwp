//! Building and setting up decompositions, sub-bands, precincts and code-blocks.

use super::decode::{DecompositionStorage, TileDecompositions};
use super::rect::IntRect;
use super::tag_tree::TagTree;
use super::tile::{ResolutionTile, Tile};
use crate::error::{DecodingError, Result};
use core::iter;
use core::ops::Range;
use rhwp_codec_resource::VecResource as _;

/// Build and allocate all necessary structures to process the code-blocks
/// for a specific tile. Also parses the segments for each code-block.
pub(crate) fn build(tile: &Tile<'_>, storage: &mut DecompositionStorage<'_>) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    build_decompositions(tile, storage)
}

fn build_decompositions(tile: &Tile<'_>, storage: &mut DecompositionStorage<'_>) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    let mut total_coefficients = 0_usize;

    {
        let (__rhwp_iter_1, __rhwp_iter_1_prepaid) =
            rhwp_codec_resource::iterations(tile.component_tiles())?;
        for component_tile in __rhwp_iter_1 {
            if !__rhwp_iter_1_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            total_coefficients = total_coefficients
                .checked_add(rhwp_codec_resource::product(&[
                    component_tile.rect.width() as usize,
                    component_tile.rect.height() as usize,
                ])?)
                .ok_or_else(rhwp_codec_resource::reject)?;
        }
    }

    if storage.coefficients.is_empty() {
        // Fast path that requests pre-zeroed memory from the OS where available.
        storage.coefficients = rhwp_codec_resource::filled(0.0, total_coefficients)?;
    } else {
        storage.coefficients.rhwp_resize(total_coefficients, 0.0)?;
    }
    let mut coefficient_counter = 0;

    {
        let (__rhwp_iter_3, __rhwp_iter_3_prepaid) =
            rhwp_codec_resource::iterations(tile.component_tiles().enumerate())?;
        for (component_idx, component_tile) in __rhwp_iter_3 {
            if !__rhwp_iter_3_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            let d_start = storage.decompositions.len();
            let mut resolution_tiles = component_tile.resolution_tiles();

            let mut build_sub_band = |sub_band_type: SubBandType,
                                      resolution_tile: &ResolutionTile<'_>,
                                      storage: &mut DecompositionStorage<'_>|
             -> Result<usize> {
                let sub_band_rect = resolution_tile.sub_band_rect(sub_band_type);

                trace!(
                    "r {} making sub-band {} for component {}",
                    resolution_tile.resolution, sub_band_type as u8, component_idx
                );
                trace!(
                    "Sub-band rect: [{},{} {}x{}], ll rect [{},{} {}x{}]",
                    sub_band_rect.x0,
                    sub_band_rect.y0,
                    sub_band_rect.width(),
                    sub_band_rect.height(),
                    resolution_tile.rect.x0,
                    resolution_tile.rect.y0,
                    resolution_tile.rect.width(),
                    resolution_tile.rect.height(),
                );

                let precincts = build_precincts(resolution_tile, sub_band_rect, tile, storage)?;

                let added_coefficients = (sub_band_rect.width() * sub_band_rect.height()) as usize;
                let coefficients = coefficient_counter..(coefficient_counter + added_coefficients);
                coefficient_counter += added_coefficients;

                let idx = storage.sub_bands.len();
                storage.sub_bands.rhwp_push(SubBand {
                    sub_band_type,
                    rect: sub_band_rect,
                    precincts: precincts.clone(),
                    coefficients,
                })?;

                Ok(idx)
            };

            // Resolution 0 always is the LL sub-band.
            let ll_resolution_tile = resolution_tiles.next().unwrap();
            let first_ll_sub_band =
                build_sub_band(SubBandType::LowLow, &ll_resolution_tile, storage)?;

            {
                let (__rhwp_iter_2, __rhwp_iter_2_prepaid) =
                    rhwp_codec_resource::iterations(resolution_tiles)?;
                for resolution_tile in __rhwp_iter_2 {
                    if !__rhwp_iter_2_prepaid {
                        rhwp_codec_resource::work(1)?;
                    }
                    let decomposition = Decomposition {
                        sub_bands: [
                            build_sub_band(SubBandType::HighLow, &resolution_tile, storage)?,
                            build_sub_band(SubBandType::LowHigh, &resolution_tile, storage)?,
                            build_sub_band(SubBandType::HighHigh, &resolution_tile, storage)?,
                        ],
                        rect: resolution_tile.rect,
                    };

                    storage.decompositions.rhwp_push(decomposition)?;
                }
            }

            let d_end = storage.decompositions.len();

            storage.tile_decompositions.rhwp_push(TileDecompositions {
                decompositions: d_start..d_end,
                first_ll_sub_band,
            })?;
        }
    }

    assert_eq!(coefficient_counter, storage.coefficients.len());

    Ok(())
}

fn build_precincts(
    resolution_tile: &ResolutionTile<'_>,
    sub_band_rect: IntRect,
    tile: &Tile<'_>,
    storage: &mut DecompositionStorage<'_>,
) -> Result<Range<usize>> {
    rhwp_codec_resource::work(1)?;
    let start = storage.precincts.len();

    {
        let (__rhwp_iter_4, __rhwp_iter_4_prepaid) = rhwp_codec_resource::iterations(
            resolution_tile
                .precincts()
                .ok_or(DecodingError::InvalidPrecinct)?,
        )?;
        for precinct_data in __rhwp_iter_4 {
            if !__rhwp_iter_4_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            let precinct_rect = precinct_data.rect;

            let cb_width = resolution_tile.code_block_width();
            let cb_height = resolution_tile.code_block_height();

            // See Figure B.9. Conceptually, the area of code-blocks is aligned
            // to the width/height of a code block.
            let cb_x0 = (u32::max(precinct_rect.x0, sub_band_rect.x0) / cb_width) * cb_width;
            let cb_y0 = (u32::max(precinct_rect.y0, sub_band_rect.y0) / cb_height) * cb_height;
            let cb_x1 =
                (u32::min(precinct_rect.x1, sub_band_rect.x1).div_ceil(cb_width)) * cb_width;
            let cb_y1 =
                (u32::min(precinct_rect.y1, sub_band_rect.y1).div_ceil(cb_height)) * cb_height;

            let code_block_area = IntRect::from_ltrb(cb_x0, cb_y0, cb_x1, cb_y1);

            // If the sub-band is empty, there are no code-blocks, but due to our
            // flooring/ceiling above, we would get 1 code-block in each direction.
            // Because of this, we need to special-case this.
            let code_blocks_x = if sub_band_rect.width() == 0 {
                0
            } else {
                code_block_area.width() / cb_width
            };

            let code_blocks_y = if sub_band_rect.height() == 0 {
                0
            } else {
                code_block_area.height() / cb_height
            };

            trace!(
                "Precinct rect: [{},{} {}x{}], num_code_blocks_wide: {}, num_code_blocks_high: {}",
                precinct_rect.x0,
                precinct_rect.y0,
                precinct_rect.width(),
                precinct_rect.height(),
                code_blocks_x,
                code_blocks_y
            );

            let blocks = build_code_blocks(
                code_block_area,
                sub_band_rect,
                resolution_tile,
                code_blocks_x,
                code_blocks_y,
                tile,
                storage,
            )?;

            let code_inclusion_tree =
                TagTree::new(code_blocks_x, code_blocks_y, &mut storage.tag_tree_nodes)?;
            let zero_bitplane_tree =
                TagTree::new(code_blocks_x, code_blocks_y, &mut storage.tag_tree_nodes)?;

            storage.precincts.rhwp_push(Precinct {
                code_blocks: blocks,
                code_inclusion_tree,
                zero_bitplane_tree,
            })?;
        }
    }

    let end = storage.precincts.len();

    Ok(start..end)
}

fn build_code_blocks(
    code_block_area: IntRect,
    sub_band_rect: IntRect,
    tile_instance: &ResolutionTile<'_>,
    code_blocks_x: u32,
    code_blocks_y: u32,
    tile: &Tile<'_>,
    storage: &mut DecompositionStorage<'_>,
) -> Result<Range<usize>> {
    rhwp_codec_resource::work(rhwp_codec_resource::product(&[
        code_blocks_x as usize,
        code_blocks_y as usize,
    ])?)?;
    let mut y = code_block_area.y0;

    let code_block_width = tile_instance.code_block_width();
    let code_block_height = tile_instance.code_block_height();

    let start = storage.code_blocks.len();

    for y_idx in 0..code_blocks_y {
        let mut x = code_block_area.x0;

        for x_idx in 0..code_blocks_x {
            // "Code-blocks in the partition may extend beyond the boundaries of
            // the sub-band coefficients. When this happens, only the
            // coefficients lying within the sub-band are coded using the method
            // described in Annex D."
            let area = IntRect::from_xywh(x, y, code_block_width, code_block_height)
                .intersect(sub_band_rect);

            trace!(
                "Codeblock rect: [{},{} {}x{}]",
                area.x0,
                area.y0,
                area.width(),
                area.height(),
            );

            let start = storage.layers.len();
            storage.layers.rhwp_extend(iter::repeat_n(
                Layer {
                    // This will be updated once we actually read the
                    // layer segments.
                    segments: None,
                },
                tile.num_layers as usize,
            ))?;
            let end = storage.layers.len();

            storage.code_blocks.rhwp_push(CodeBlock {
                x_idx,
                y_idx,
                rect: area,
                has_been_included: false,
                missing_bit_planes: 0,
                l_block: 3,
                number_of_coding_passes: 0,
                layers: start..end,
                non_empty_layer_count: 0,
            })?;

            x += code_block_width;
        }

        y += code_block_height;
    }

    let end = storage.code_blocks.len();

    Ok(start..end)
}

pub(crate) struct Decomposition {
    /// In the order low-high, high-low and high-high.
    pub(crate) sub_bands: [usize; 3],
    /// The rectangle of the decomposition.
    pub(crate) rect: IntRect,
}

#[derive(Clone)]
pub(crate) struct SubBand {
    pub(crate) sub_band_type: SubBandType,
    pub(crate) rect: IntRect,
    pub(crate) precincts: Range<usize>,
    pub(crate) coefficients: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SubBandType {
    LowLow = 0,
    HighLow = 1,
    LowHigh = 2,
    HighHigh = 3,
}

#[derive(Clone)]
pub(crate) struct Precinct {
    pub(crate) code_blocks: Range<usize>,
    pub(crate) code_inclusion_tree: TagTree,
    pub(crate) zero_bitplane_tree: TagTree,
}

pub(crate) struct PrecinctData {
    /// The x coordinate mapped back to the reference grid.
    pub(crate) r_x: u32,
    /// The y coordinate mapped back to the reference grid.
    pub(crate) r_y: u32,
    /// The actual rectangle of the precinct (in the sub-band coordinate
    /// system).
    pub(crate) rect: IntRect,
    /// The index of the precinct in the sub-band.
    pub(crate) idx: u64,
}

#[derive(Clone)]
pub(crate) struct CodeBlock {
    pub(crate) rect: IntRect,
    pub(crate) x_idx: u32,
    pub(crate) y_idx: u32,
    pub(crate) layers: Range<usize>,
    pub(crate) has_been_included: bool,
    pub(crate) missing_bit_planes: u8,
    pub(crate) number_of_coding_passes: u8,
    pub(crate) l_block: u32,
    pub(crate) non_empty_layer_count: u8,
}

pub(crate) struct Segment<'a> {
    pub(crate) idx: u8,
    pub(crate) coding_pases: u8,
    pub(crate) data_length: u32,
    pub(crate) data: &'a [u8],
}

#[derive(Clone)]
pub(crate) struct Layer {
    pub(crate) segments: Option<Range<usize>>,
}
