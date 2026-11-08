//! Parsing of layers and their segments, as specified in Annex B.

use alloc::boxed::Box;
use rhwp_codec_resource::VecResource as _;

use super::build::Segment;
use super::codestream::markers::{EPH, SOP};
use super::codestream::{ComponentInfo, Header};
use super::decode::DecompositionStorage;
use super::progression::ProgressionData;
use super::tile::{Tile, TilePart};
use crate::error::{Result, TileError, bail};
use crate::reader::BitReader;

pub(crate) const MAX_BITPLANE_COUNT: u8 = 32;

pub(crate) fn parse<'a, 'b>(
    tile: &'b Tile<'a>,
    mut progression_iterator: Box<dyn Iterator<Item = ProgressionData> + '_>,
    header: &Header<'_>,
    storage: &mut DecompositionStorage<'a>,
) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    {
        let (__rhwp_iter_1, __rhwp_iter_1_prepaid) =
            rhwp_codec_resource::iterations(&tile.tile_parts)?;
        for tile_part in __rhwp_iter_1 {
            if !__rhwp_iter_1_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            if parse_inner(
                tile_part.try_clone()?,
                &mut progression_iterator,
                &tile.component_infos,
                storage,
            )
            .is_none()
                && header.strict
            {
                bail!(TileError::Invalid);
            }
        }
    }

    Ok(())
}

fn parse_inner<'a>(
    mut tile_part: TilePart<'a>,
    progression_iterator: &mut dyn Iterator<Item = ProgressionData>,
    component_infos: &[ComponentInfo],
    storage: &mut DecompositionStorage<'a>,
) -> Option<()> {
    rhwp_codec_resource::work(1).ok()?;
    while !tile_part.header().at_end() {
        rhwp_codec_resource::work(1).ok()?;
        let progression_data = progression_iterator.next()?;
        let resolution = progression_data.resolution;
        let component_info = &component_infos[progression_data.component as usize];
        let tile_decompositions =
            &mut storage.tile_decompositions[progression_data.component as usize];
        let sub_band_iter = tile_decompositions.sub_band_iter(resolution, &storage.decompositions);

        let body_reader = tile_part.body();

        if component_info.coding_style.flags.may_use_sop_markers()
            && body_reader.peek_marker() == Some(SOP)
        {
            body_reader.read_marker().ok()?;
            body_reader.skip_bytes(4)?;
        }

        let header_reader = tile_part.header();

        let zero_length = header_reader.read_bits_with_stuffing(1)? == 0;

        // B.10.3 Zero length packet
        // "The first bit in the packet header denotes whether the packet has a length of zero
        // (empty packet). The value 0 indicates a zero length; no code-blocks are included in this
        // case. The value 1 indicates a non-zero length."
        if !zero_length {
            {
                let (__rhwp_iter_2, __rhwp_iter_2_prepaid) =
                    rhwp_codec_resource::iterations(sub_band_iter.clone()).ok()?;
                for sub_band in __rhwp_iter_2 {
                    if !__rhwp_iter_2_prepaid {
                        rhwp_codec_resource::work(1).ok()?;
                    }
                    resolve_segments(
                        sub_band,
                        &progression_data,
                        header_reader,
                        storage,
                        component_info,
                    )?;
                }
            }
        }

        header_reader.align();

        if component_info.coding_style.flags.uses_eph_marker()
            && header_reader.read_marker().ok()? != EPH
        {
            return None;
        }

        // Now read the packet body.
        let body_reader = tile_part.body();

        if !zero_length {
            {
                let (__rhwp_iter_5, __rhwp_iter_5_prepaid) =
                    rhwp_codec_resource::iterations(sub_band_iter).ok()?;
                for sub_band in __rhwp_iter_5 {
                    if !__rhwp_iter_5_prepaid {
                        rhwp_codec_resource::work(1).ok()?;
                    }
                    let sub_band = &mut storage.sub_bands[sub_band];
                    let precinct = &mut storage.precincts[sub_band.precincts.clone()]
                        [progression_data.precinct as usize];
                    let code_blocks = &mut storage.code_blocks[precinct.code_blocks.clone()];

                    {
                        let (__rhwp_iter_4, __rhwp_iter_4_prepaid) =
                            rhwp_codec_resource::iterations(code_blocks).ok()?;
                        for code_block in __rhwp_iter_4 {
                            if !__rhwp_iter_4_prepaid {
                                rhwp_codec_resource::work(1).ok()?;
                            }
                            let layer = &mut storage.layers[code_block.layers.clone()]
                                [progression_data.layer_num as usize];

                            if let Some(segments) = layer.segments.clone() {
                                let segments = &mut storage.segments[segments.clone()];

                                {
                                    let (__rhwp_iter_3, __rhwp_iter_3_prepaid) =
                                        rhwp_codec_resource::iterations(segments).ok()?;
                                    for segment in __rhwp_iter_3 {
                                        if !__rhwp_iter_3_prepaid {
                                            rhwp_codec_resource::work(1).ok()?;
                                        }
                                        segment.data =
                                            body_reader.read_bytes(segment.data_length as usize)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Some(())
}

fn resolve_segments(
    sub_band_dx: usize,
    progression_data: &ProgressionData,
    reader: &mut BitReader<'_>,
    storage: &mut DecompositionStorage<'_>,
    component_info: &ComponentInfo,
) -> Option<()> {
    rhwp_codec_resource::work(1).ok()?;
    // We don't support more than 32-bit precision.
    const MAX_CODING_PASSES: u8 = 1 + 3 * (MAX_BITPLANE_COUNT - 1);

    let sub_band = &storage.sub_bands[sub_band_dx];
    let precincts = &mut storage.precincts[sub_band.precincts.clone()];
    let Some(precinct) = precincts.get_mut(progression_data.precinct as usize) else {
        // An invalid file could trigger this code path.
        warn!("progression data yielded invalid precinct index");

        return None;
    };
    let code_blocks = &mut storage.code_blocks[precinct.code_blocks.clone()];

    {
        let (__rhwp_iter_7, __rhwp_iter_7_prepaid) =
            rhwp_codec_resource::iterations(code_blocks).ok()?;
        for code_block in __rhwp_iter_7 {
            if !__rhwp_iter_7_prepaid {
                rhwp_codec_resource::work(1).ok()?;
            }
            // B.10.4 Code-block inclusion
            let is_included = if code_block.has_been_included {
                // "For code-blocks that have been included in a previous packet,
                // a single bit is used to represent the information, where a 1
                // means that the code-block is included in this layer and a 0 means
                // that it is not."
                reader.read_bits_with_stuffing(1)? == 1
            } else {
                // "For code-blocks that have not been previously included in any packet,
                // this information is signalled with a separate tag tree code for each precinct
                // as confined to a sub-band. The values in this tag tree are the number of the
                // layer in which the current code-block is first included. Although the exact
                // sequence of bits that represent the inclusion tag tree appears in the bit
                // stream, only the bits needed for determining whether the code-block is
                // included are placed in the packet header. If some of the tag tree is already
                // known from previous code-blocks or previous layers, it is not repeated.
                // Likewise, only as much of the tag tree as is needed to determine inclusion in
                // the current layer is included. If a code-block is not included until a later
                // layer, then only a partial tag tree is included at that point in the bit
                // stream."
                precinct.code_inclusion_tree.read(
                    code_block.x_idx,
                    code_block.y_idx,
                    reader,
                    progression_data.layer_num as u32 + 1,
                    &mut storage.tag_tree_nodes,
                )? <= progression_data.layer_num as u32
            };

            trace!("code-block inclusion: {}", is_included);

            if !is_included {
                continue;
            }

            let layer =
                &mut storage.layers[code_block.layers.clone()][progression_data.layer_num as usize];

            let included_first_time = is_included && !code_block.has_been_included;

            // B.10.5 Zero bit-plane information
            // "If a code-block is included for the first time, the packet header contains
            // information identifying the actual number of bit-planes used to represent
            // coefficients from the code-block. The maximum number of bit-planes available
            // for the representation of coefficients in any sub-band, b, is given by Mb as
            // defined in Equation (E-2). In general, however, the
            // number of actual bit-planes for which coding passes are generated is Mb – P,
            // where the number of missing most significant bit-planes, P, may vary from
            // code-block to code-block; these missing bit-planes are all taken to be zero. The
            // value of P is coded in the packet header with a separate tag tree for every
            // precinct, in the same manner as the code block inclusion information."
            if included_first_time {
                code_block.missing_bit_planes = precinct.zero_bitplane_tree.read(
                    code_block.x_idx,
                    code_block.y_idx,
                    reader,
                    u32::MAX,
                    &mut storage.tag_tree_nodes,
                )? as u8;
                trace!(
                    "zero bit-plane information: {}",
                    code_block.missing_bit_planes
                );
            }

            code_block.has_been_included |= is_included;

            // B.10.6 Number of coding passes
            // "The number of coding passes included in this packet from each code-block is
            // identified in the packet header using the codewords shown in Table B.4. This
            // table provides for the possibility of signalling up to 164 coding passes."
            let added_coding_passes = if reader.peak_bits_with_stuffing(9) == Some(0x1ff) {
                reader.read_bits_with_stuffing(9)?;
                reader.read_bits_with_stuffing(7)? + 37
            } else if reader.peak_bits_with_stuffing(4) == Some(0x0f) {
                reader.read_bits_with_stuffing(4)?;
                reader.read_bits_with_stuffing(5)? + 6
            } else if reader.peak_bits_with_stuffing(4) == Some(0b1110) {
                reader.read_bits_with_stuffing(4)?;
                5
            } else if reader.peak_bits_with_stuffing(4) == Some(0b1101) {
                reader.read_bits_with_stuffing(4)?;
                4
            } else if reader.peak_bits_with_stuffing(4) == Some(0b1100) {
                reader.read_bits_with_stuffing(4)?;
                3
            } else if reader.peak_bits_with_stuffing(2) == Some(0b10) {
                reader.read_bits_with_stuffing(2)?;
                2
            } else if reader.peak_bits_with_stuffing(1) == Some(0) {
                reader.read_bits_with_stuffing(1)?;
                1
            } else {
                return None;
            } as u8;

            trace!("number of coding passes: {}", added_coding_passes);

            let mut k = 0;

            while reader.read_bits_with_stuffing(1)? == 1 {
                rhwp_codec_resource::work(1).ok()?;
                k += 1;
            }

            code_block.l_block += k;

            let previous_layers_passes = code_block.number_of_coding_passes;
            let cumulative_passes = previous_layers_passes.checked_add(added_coding_passes)?;

            if cumulative_passes > MAX_CODING_PASSES {
                return None;
            }

            let get_segment_idx = |pass_idx: u8| {
                if component_info.code_block_style().termination_on_each_pass {
                    // If we terminate on each pass, the segment is just the index
                    // of the pass.
                    pass_idx
                } else if component_info
                    .code_block_style()
                    .selective_arithmetic_coding_bypass
                {
                    // Use the formula derived from the table in the spec.
                    segment_idx_for_bypass(pass_idx)
                } else {
                    // If none of the above flags is activated, the number of
                    // segments just corresponds to the number of layers.
                    code_block.non_empty_layer_count
                }
            };

            let start = storage.segments.len();

            let mut push_segment = |segment: u8, coding_passes_for_segment: u8| {
                let length = {
                    assert!(coding_passes_for_segment > 0);

                    // "A codeword segment is the number of bytes contributed to a packet by a
                    // code-block. The length of a codeword segment is represented by a binary number of length:
                    // bits = Lblock + floor(log_2(coding passes added))
                    // where Lblock is a code-block state variable. A separate Lblock is used for each
                    // code-block in the precinct. The value of Lblock is initially set to three. The
                    // number of bytes contributed by each code-block is preceded by signalling bits
                    // that increase the value of Lblock, as needed. A signalling bit of zero indicates
                    // the current value of Lblock is sufficient. If there are k ones followed by a
                    // zero, the value of Lblock is incremented by k. While Lblock can only increase,
                    // the number of bits used to signal the length of the code-block contribution can
                    // increase or decrease depending on the number of coding passes included."
                    let length_bits = code_block.l_block + coding_passes_for_segment.ilog2();
                    reader.read_bits_with_stuffing(length_bits as u8)
                }?;

                storage
                    .segments
                    .rhwp_push(Segment {
                        idx: segment,
                        data_length: length,
                        coding_pases: coding_passes_for_segment,
                        // Will be set later.
                        data: &[],
                    })
                    .ok()?;

                trace!("length({segment}) {}", length);

                Some(())
            };

            let mut last_segment = get_segment_idx(previous_layers_passes);
            let mut coding_passes_for_segment = 0;

            {
                let (__rhwp_iter_6, __rhwp_iter_6_prepaid) =
                    rhwp_codec_resource::iterations(previous_layers_passes..cumulative_passes)
                        .ok()?;
                for coding_pass in __rhwp_iter_6 {
                    if !__rhwp_iter_6_prepaid {
                        rhwp_codec_resource::work(1).ok()?;
                    }
                    let segment = get_segment_idx(coding_pass);

                    if segment != last_segment {
                        push_segment(last_segment, coding_passes_for_segment)?;
                        last_segment = segment;
                        coding_passes_for_segment = 1;
                    } else {
                        coding_passes_for_segment += 1;
                    }
                }
            }

            // Flush the final segment if applicable.
            if coding_passes_for_segment > 0 {
                push_segment(last_segment, coding_passes_for_segment)?;
            }

            let end = storage.segments.len();
            layer.segments = Some(start..end);
            code_block.number_of_coding_passes += added_coding_passes;
            code_block.non_empty_layer_count += 1;
        }
    }

    Some(())
}

/// Calculate the segment index for the given pass in arithmetic decoder
/// bypass (see section D.6, Table D.9).
fn segment_idx_for_bypass(pass_idx: u8) -> u8 {
    if pass_idx < 10 {
        0
    } else {
        1 + (2 * ((pass_idx - 10) / 3)) + (if ((pass_idx - 10) % 3) == 2 { 1 } else { 0 })
    }
}
