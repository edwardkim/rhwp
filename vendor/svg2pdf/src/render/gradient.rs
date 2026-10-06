use pdf_writer::types::{FunctionShadingType, MaskType};
use pdf_writer::{Chunk, Content, Filter, Finish, Name, Ref};
use usvg::{Paint, Rect, Transform};

use crate::util::context::Context;
use crate::util::helper::{bbox_to_non_zero_rect, NameExt, RectExt, StopExt, TransformExt};
use crate::util::resources::ResourceContainer;

/// An alternative representation of a usvg::Stop that allows us to store
/// both, RGB gradients and grayscale gradients.
#[derive(Copy, Clone)]
pub struct Stop<const COUNT: usize> {
    pub color: [f32; COUNT],
    pub offset: f32,
}

struct GradientProperties {
    coords: Vec<f32>,
    shading_type: FunctionShadingType,
    stops: Vec<usvg::Stop>,
    transform: Transform,
}

impl GradientProperties {
    fn try_from_paint(paint: &Paint) -> Option<Self> {
        match paint {
            Paint::LinearGradient(l) => Some(Self {
                coords: vec![l.x1(), l.y1(), l.x2(), l.y2()],
                shading_type: FunctionShadingType::Axial,
                stops: Vec::from(l.stops()),
                transform: l.transform(),
            }),
            Paint::RadialGradient(r) => Some(Self {
                coords: vec![r.fx(), r.fy(), 0.0, r.cx(), r.cy(), r.r().get()],
                shading_type: FunctionShadingType::Radial,
                stops: Vec::from(r.stops()),
                transform: r.transform(),
            }),
            _ => None,
        }
    }
}

/// Turn a (gradient) paint into a shading pattern object. Stop opacities will be ignored and
/// need to be rendered separately using `create_shading_soft_mask`. The paint
/// needs to be either a linear gradient or a radial gradient.
pub fn create_shading_pattern(
    paint: &Paint,
    chunk: &mut Chunk,
    ctx: &mut Context,
    accumulated_transform: &Transform,
) -> Ref {
    let properties = GradientProperties::try_from_paint(paint).unwrap();
    shading_pattern(&properties, chunk, ctx, accumulated_transform)
}

/// Return a soft mask that will render the stop opacities of a gradient into a gray scale
/// shading. If no soft mask is necessary (because no stops have an opacity),
/// `None` will be returned.
pub fn create_shading_soft_mask(
    paint: &Paint,
    chunk: &mut Chunk,
    ctx: &mut Context,
    bbox: Rect,
) -> Option<Ref> {
    let properties = GradientProperties::try_from_paint(paint).unwrap();
    if properties
        .stops
        .iter()
        .any(|stop| stop.opacity().get() < 1.0)
    {
        Some(shading_soft_mask(&properties, chunk, ctx, bbox))
    } else {
        None
    }
}

fn shading_pattern(
    properties: &GradientProperties,
    chunk: &mut Chunk,
    ctx: &mut Context,
    accumulated_transform: &Transform,
) -> Ref {
    let pattern_ref = ctx.alloc_ref();

    let matrix = accumulated_transform.pre_concat(properties.transform);

    let shading_ref = shading_function(properties, chunk, ctx, false);
    let mut shading_pattern = chunk.shading_pattern(pattern_ref);
    shading_pattern.pair(Name(b"Shading"), shading_ref);
    shading_pattern.matrix(matrix.to_pdf_transform());
    shading_pattern.finish();

    pattern_ref
}

fn shading_soft_mask(
    properties: &GradientProperties,
    chunk: &mut Chunk,
    ctx: &mut Context,
    bbox: Rect,
) -> Ref {
    let mut rc = ResourceContainer::new();
    let x_object_id = ctx.alloc_ref();
    let shading_ref = shading_function(properties, chunk, ctx, true);
    let shading_name = rc.add_shading(shading_ref);
    let bbox = bbox_to_non_zero_rect(Some(bbox)).to_pdf_rect();

    let transform = properties.transform;

    let mut content = Content::new();
    content.transform(transform.to_pdf_transform());
    content.shading(shading_name.to_pdf_name());
    let content_stream = ctx.finish_content(content);

    let mut x_object = chunk.form_xobject(x_object_id, &content_stream);
    rc.finish(&mut x_object.resources());

    x_object
        .group()
        .transparency()
        .isolated(false)
        .knockout(false)
        .color_space()
        .icc_based(ctx.sgray_ref());

    if ctx.options.compress {
        x_object.filter(Filter::FlateDecode);
    }

    x_object.bbox(bbox);
    x_object.finish();

    let gs_ref = ctx.alloc_ref();
    let mut gs = chunk.ext_graphics(gs_ref);
    gs.soft_mask()
        .subtype(MaskType::Luminosity)
        .group(x_object_id)
        .finish();

    gs_ref
}

fn shading_function(
    properties: &GradientProperties,
    chunk: &mut Chunk,
    ctx: &mut Context,
    use_opacities: bool,
) -> Ref {
    let shading_ref = ctx.alloc_ref();
    let function_ref = function(&properties.stops, chunk, ctx, use_opacities);

    let mut shading = chunk.function_shading(shading_ref);
    shading.shading_type(properties.shading_type);
    if use_opacities {
        shading.color_space().icc_based(ctx.sgray_ref());
    } else {
        shading.color_space().icc_based(ctx.srgb_ref());
    }

    shading.function(function_ref);
    shading.coords(properties.coords.iter().copied());
    shading.extend([true, true]);
    shading.finish();
    shading_ref
}

fn function(
    stops: &[usvg::Stop],
    chunk: &mut Chunk,
    ctx: &mut Context,
    use_opacities: bool,
) -> Ref {
    // Gradients with no stops and only one stop should automatically be converted by resvg
    // into no fill / plain fill, so there should be at least two stops
    debug_assert!(stops.len() > 1);

    fn pad_stops<const COUNT: usize>(mut stops: Vec<Stop<COUNT>>) -> Vec<Stop<COUNT>> {
        // We manually pad the stops if necessary so that they are always in the range from 0-1
        if let Some(first) = stops.first() {
            if first.offset != 0.0 {
                let mut new_stop = *first;
                new_stop.offset = 0.0;
                stops.insert(0, new_stop);
            }
        }

        if let Some(last) = stops.last() {
            if last.offset != 1.0 {
                let mut new_stop = *last;
                new_stop.offset = 1.0;
                stops.push(new_stop);
            }
        }

        stops
    }

    if use_opacities {
        let stops = pad_stops(
            stops
                .iter()
                .map(|s| s.opacity_stops())
                .collect::<Vec<Stop<1>>>(),
        );
        select_function(&stops, chunk, ctx)
    } else {
        let stops = pad_stops(
            stops
                .iter()
                .map(|s| s.color_stops())
                .collect::<Vec<Stop<3>>>(),
        );
        select_function(&stops, chunk, ctx)
    }
}

fn select_function<const COUNT: usize>(
    stops: &[Stop<COUNT>],
    chunk: &mut Chunk,
    ctx: &mut Context,
) -> Ref {
    if stops.len() == 2 {
        exponential_function(&stops[0], &stops[1], chunk, ctx)
    } else {
        stitching_function(stops, chunk, ctx)
    }
}

fn stitching_function<const COUNT: usize>(
    stops: &[Stop<COUNT>],
    chunk: &mut Chunk,
    ctx: &mut Context,
) -> Ref {
    assert!(!stops.is_empty());

    // PDF 32000-1 7.10.4 permits 1-input stitching functions as children.
    // Keep every original interval/color, but bound each /Functions array to
    // MuPDF's MAX_STITCHING (256). Child domains stay in the original stop
    // coordinates; matching /Encode endpoints make the parent mapping identity.
    // This applies equally to RGB shading and the grayscale opacity soft mask.
    const MAX_STITCHING: usize = 256;
    struct Interval {
        reference: Ref,
        start: f32,
        end: f32,
        encode: [f32; 2],
    }

    fn write_group<const COUNT: usize>(
        intervals: &[Interval],
        chunk: &mut Chunk,
        ctx: &mut Context,
    ) -> Interval {
        let reference = ctx.alloc_ref();
        let start = intervals.first().unwrap().start;
        let end = intervals.last().unwrap().end;
        let mut function = chunk.stitching_function(reference);
        function.domain([start, end]);
        function.range(get_function_range(COUNT));
        function.functions(intervals.iter().map(|i| i.reference));
        function.bounds(intervals[..intervals.len() - 1].iter().map(|i| i.end));
        function.encode(intervals.iter().flat_map(|i| i.encode));
        function.finish();
        Interval {
            reference,
            start,
            end,
            encode: [start, end],
        }
    }

    // Allocate the flat root first as before, preserving object order and bytes
    // for gradients that already fit the reader limit.
    let reference = ctx.alloc_ref();
    let mut intervals: Vec<_> = stops
        .windows(2)
        .map(|w| Interval {
            reference: exponential_function(&w[0], &w[1], chunk, ctx),
            start: w[0].offset,
            end: w[1].offset,
            encode: [0.0, 1.0],
        })
        .collect();
    while intervals.len() > MAX_STITCHING {
        intervals = intervals
            .chunks(MAX_STITCHING)
            .map(|group| write_group::<COUNT>(group, chunk, ctx))
            .collect();
    }
    let mut function = chunk.stitching_function(reference);
    function.domain([0.0, 1.0]);
    function.range(get_function_range(COUNT));
    function.functions(intervals.iter().map(|i| i.reference));
    function.bounds(intervals[..intervals.len() - 1].iter().map(|i| i.end));
    function.encode(intervals.iter().flat_map(|i| i.encode));
    function.finish();
    reference
}

fn exponential_function<const COUNT: usize>(
    first_stop: &Stop<COUNT>,
    second_stop: &Stop<COUNT>,
    chunk: &mut Chunk,
    ctx: &mut Context,
) -> Ref {
    let reference = ctx.alloc_ref();
    let mut exp = chunk.exponential_function(reference);

    exp.range(get_function_range(COUNT));
    exp.c0(first_stop.color);
    exp.c1(second_stop.color);
    exp.domain([0.0, 1.0]);
    exp.n(1.0);
    exp.finish();
    reference
}

fn get_function_range(count: usize) -> Vec<f32> {
    [0.0, 1.0].repeat(count)
}
