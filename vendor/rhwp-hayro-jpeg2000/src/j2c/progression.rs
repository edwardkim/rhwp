//! Progression iterators, defined in Section B.12.
//!
//! A progression iterator essentially yields tuples of
//! (`layer_num`, resolution, component, precinct) in a specific order that
//! determines in which order the data appears in the codestream.

use alloc::vec;
use alloc::vec::Vec;
use rhwp_codec_resource::IteratorResource as _;
use rhwp_codec_resource::VecResource as _;

use super::tile::{ComponentTile, ResolutionTile, Tile};
use core::cmp::Ordering;
use core::iter;

#[derive(Default, Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub(crate) struct ProgressionData {
    pub(crate) layer_num: u8,
    pub(crate) resolution: u8,
    pub(crate) component: u8,
    pub(crate) precinct: u64,
}

pub(crate) struct IteratorInput<'a> {
    layers: (u8, u8),
    tile: &'a Tile<'a>,
    resolutions: (u8, u8),
    components: (u8, u8),
}

impl<'a> IteratorInput<'a> {
    pub(crate) fn new(tile: &'a Tile<'a>) -> Self {
        Self::new_with_custom_bounds(
            tile,
            // Will be clamped automatically.
            (0, u8::MAX),
            (0, u8::MAX),
            (0, u8::MAX),
        )
    }

    pub(crate) fn new_with_custom_bounds(
        tile: &'a Tile<'a>,
        mut resolutions: (u8, u8),
        mut layers: (u8, u8),
        mut components: (u8, u8),
    ) -> Self {
        let max_resolution = tile
            .component_infos
            .iter()
            .map(|c| c.coding_style.parameters.num_resolution_levels)
            .max()
            .unwrap_or(0);
        let max_layer = tile.num_layers;
        let max_component = tile.component_infos.len() as u8;

        // Make sure we don't exceed what's actually possible
        resolutions.1 = resolutions.1.min(max_resolution);
        layers.1 = layers.1.min(max_layer);
        components.1 = components.1.min(max_component);

        assert!(resolutions.1 > resolutions.0);
        assert!(layers.1 > layers.0);
        assert!(components.1 > components.0);

        Self {
            layers,
            tile,
            resolutions,
            components,
        }
    }

    fn min_layer(&self) -> u8 {
        self.layers.0
    }

    fn max_layer(&self) -> u8 {
        self.layers.1
    }

    fn min_resolution(&self) -> u8 {
        self.resolutions.0
    }

    fn total_max_resolution(&self) -> u8 {
        self.resolutions.1
    }

    fn max_resolution(&self, component_idx: u8) -> u8 {
        self.total_max_resolution()
            // It's possible that the different component tiles have different resolution levels
            // (self.resolutions.1 stores the maximum across all component tiles), so
            // take the minimum of both.
            .min(self.tile.component_infos[component_idx as usize].num_resolution_levels())
    }

    fn min_comp(&self) -> u8 {
        self.components.0
    }

    fn max_comp(&self) -> u8 {
        self.components.1
    }

    fn component_tiles(&self) -> crate::error::Result<Vec<ComponentTile<'a>>> {
        Ok(self
            .tile
            .component_infos
            .iter()
            .map(|c| ComponentTile::new(self.tile, c))
            .rhwp_collect()?)
    }
}

/// B.12.1.1 Layer-resolution level-component-position progression.
pub(crate) fn layer_resolution_component_position_progression<'a>(
    input: IteratorInput<'a>,
) -> crate::error::Result<impl Iterator<Item = ProgressionData> + 'a> {
    let component_tiles = input.component_tiles()?;

    let mut layer = input.min_layer();
    let mut resolution = input.min_resolution();
    let mut component_idx = input.min_comp();

    let mut resolution_tile = ResolutionTile::new(component_tiles[0], resolution);
    let mut precinct = 0;

    Ok(iter::from_fn(move || {
        rhwp_codec_resource::work(1).ok()?;
        if layer == input.max_layer() || resolution == input.total_max_resolution() {
            return None;
        }

        if precinct == resolution_tile.num_precincts() {
            loop {
                rhwp_codec_resource::work(1).ok()?;
                precinct = 0;
                component_idx += 1;

                if component_idx == input.max_comp() {
                    component_idx = input.min_comp();

                    resolution += 1;

                    if resolution == input.max_resolution(component_idx) {
                        resolution = input.min_resolution();
                        layer += 1;

                        if layer == input.max_layer() {
                            return None;
                        }
                    }
                }

                resolution_tile =
                    ResolutionTile::new(component_tiles[component_idx as usize], resolution);

                // Only yield if the resolution tile has precincts, otherwise
                // we need to keep advancing.
                if resolution_tile.num_precincts() != 0 {
                    break;
                }
            }
        }

        let data = ProgressionData {
            layer_num: layer,
            resolution,
            component: component_idx,
            precinct,
        };

        precinct += 1;

        Some(data)
    }))
}

/// B.12.1.2 Resolution level-layer-component-position progression.
pub(crate) fn resolution_layer_component_position_progression<'a>(
    input: IteratorInput<'a>,
) -> crate::error::Result<impl Iterator<Item = ProgressionData> + 'a> {
    let component_tiles = input.component_tiles()?;

    let mut layer = 0;
    let mut resolution = 0;
    let mut component_idx = 0;
    let mut resolution_tile =
        ResolutionTile::new(component_tiles[component_idx as usize], resolution);
    let mut precinct = 0;

    Ok(iter::from_fn(move || {
        rhwp_codec_resource::work(1).ok()?;
        if layer == input.max_layer() || resolution == input.total_max_resolution() {
            return None;
        }

        if precinct == resolution_tile.num_precincts() {
            loop {
                rhwp_codec_resource::work(1).ok()?;
                precinct = 0;
                component_idx += 1;

                if component_idx == input.max_comp() {
                    component_idx = 0;
                    layer += 1;

                    if layer == input.max_layer() {
                        layer = 0;
                        resolution += 1;

                        if resolution == input.total_max_resolution() {
                            return None;
                        }
                    }
                }

                // If the given resolution level doesn't exist for the current
                // component, continue.
                if resolution >= input.max_resolution(component_idx) {
                    continue;
                }

                resolution_tile =
                    ResolutionTile::new(component_tiles[component_idx as usize], resolution);

                // Only yield if the resolution tile has precincts, otherwise
                // we need to keep advancing.
                if resolution_tile.num_precincts() != 0 {
                    break;
                }
            }
        }

        let data = ProgressionData {
            layer_num: layer,
            resolution,
            component: component_idx,
            precinct,
        };

        precinct += 1;

        Some(data)
    }))
}

// The formula for the remaining three progressions looks very intimidating.
// But really, all they boil down to is that we need to determine all precinct
// indices for each component/resolution combination and sort them by ascending
// y/x coordinate on the reference grid. Other than that, they can be treated
// exactly the same, except that the sort order precedence of the fields change.

// Note that the order of fields here is important!
struct PrecinctStore {
    resolution: u8,
    precinct_y: u32,
    precinct_x: u32,
    component_idx: u8,
    precinct_idx: u64,
}

fn position_progression_common<'a>(
    input: IteratorInput<'a>,
    sort: impl FnMut(&PrecinctStore, &PrecinctStore) -> Ordering,
) -> Option<impl Iterator<Item = ProgressionData> + 'a> {
    rhwp_codec_resource::work(1).ok()?;
    let mut elements = vec![];

    {
        let (__rhwp_iter_2, __rhwp_iter_2_prepaid) = rhwp_codec_resource::iterations(
            input
                .tile
                .component_tiles()
                .enumerate()
                .skip(input.min_comp() as usize)
                .take(input.max_comp() as usize - input.min_comp() as usize),
        )
        .ok()?;
        for (component_idx, component) in __rhwp_iter_2 {
            if !__rhwp_iter_2_prepaid {
                rhwp_codec_resource::work(1).ok()?;
            }
            {
                let (__rhwp_iter_1, __rhwp_iter_1_prepaid) = rhwp_codec_resource::iterations(
                    component
                        .resolution_tiles()
                        .enumerate()
                        .skip(input.min_resolution() as usize)
                        .take(
                            input.total_max_resolution() as usize - input.min_resolution() as usize,
                        ),
                )
                .ok()?;
                for (resolution, resolution_tile) in __rhwp_iter_1 {
                    if !__rhwp_iter_1_prepaid {
                        rhwp_codec_resource::work(1).ok()?;
                    }
                    elements
                        .rhwp_extend(resolution_tile.precincts()?.map(|d| PrecinctStore {
                            precinct_y: d.r_y,
                            precinct_x: d.r_x,
                            component_idx: component_idx as u8,
                            resolution: resolution as u8,
                            precinct_idx: d.idx,
                        }))
                        .ok()?;
                }
            }
        }
    }

    rhwp_codec_resource::sort(&mut elements, sort).ok()?;

    Some(elements.into_iter().flat_map(move |e| {
        (input.min_layer()..input.max_layer()).map(move |layer| ProgressionData {
            layer_num: layer,
            resolution: e.resolution,
            component: e.component_idx,
            precinct: e.precinct_idx,
        })
    }))
}

/// B.12.1.3 Resolution level-position-component-layer progression.
pub(crate) fn resolution_position_component_layer_progression<'a>(
    input: IteratorInput<'a>,
) -> Option<impl Iterator<Item = ProgressionData> + 'a> {
    rhwp_codec_resource::work(1).ok()?;
    position_progression_common(input, |p, s| {
        p.resolution
            .cmp(&s.resolution)
            .then_with(|| p.precinct_y.cmp(&s.precinct_y))
            .then_with(|| p.precinct_x.cmp(&s.precinct_x))
            .then_with(|| p.component_idx.cmp(&s.component_idx))
            .then_with(|| p.precinct_idx.cmp(&s.precinct_idx))
    })
}

/// B.12.1.4 Position-component-resolution level-layer progression.
pub(crate) fn position_component_resolution_layer_progression<'a>(
    input: IteratorInput<'a>,
) -> Option<impl Iterator<Item = ProgressionData> + 'a> {
    rhwp_codec_resource::work(1).ok()?;
    position_progression_common(input, |p, s| {
        p.precinct_y
            .cmp(&s.precinct_y)
            .then_with(|| p.precinct_x.cmp(&s.precinct_x))
            .then_with(|| p.component_idx.cmp(&s.component_idx))
            .then_with(|| p.resolution.cmp(&s.resolution))
            .then_with(|| p.precinct_idx.cmp(&s.precinct_idx))
    })
}

/// B.12.1.5 Component-position-resolution level-layer progression.
pub(crate) fn component_position_resolution_layer_progression<'a>(
    input: IteratorInput<'a>,
) -> Option<impl Iterator<Item = ProgressionData> + 'a> {
    rhwp_codec_resource::work(1).ok()?;
    position_progression_common(input, |p, s| {
        p.component_idx
            .cmp(&s.component_idx)
            .then_with(|| p.precinct_y.cmp(&s.precinct_y))
            .then_with(|| p.precinct_x.cmp(&s.precinct_x))
            .then_with(|| p.resolution.cmp(&s.resolution))
            .then_with(|| p.precinct_idx.cmp(&s.precinct_idx))
    })
}
