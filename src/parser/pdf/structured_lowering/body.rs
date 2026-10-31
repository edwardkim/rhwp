//! Commit representable body proposals through the same paragraph result as cells.
use super::*;
use crate::model::shape::{DrawingObjAttr, RectangleShape, TextBox};
use crate::parser::pdf::recognition::BodyRecognition;

fn intersects(a: &CommonObjAttr, b: &CommonObjAttr) -> bool {
    let (ax, ay) = (
        a.horizontal_offset as i32 as f32 / 100.,
        a.vertical_offset as i32 as f32 / 100.,
    );
    let (bx, by) = (
        b.horizontal_offset as i32 as f32 / 100.,
        b.vertical_offset as i32 as f32 / 100.,
    );
    ax < bx + b.width as f32 / 100.
        && bx < ax + a.width as f32 / 100.
        && ay < by + b.height as f32 / 100.
        && by < ay + a.height as f32 / 100.
}
fn paint_order(
    controls: &[Control],
    consumed: &BTreeSet<usize>,
    removed: &BTreeSet<usize>,
) -> Result<i32, String> {
    let mut layers = consumed
        .iter()
        .filter_map(|&i| paint_frame(&controls[i]).map(|f| f.z_order));
    let initial = layers.next().ok_or("body frame has no source paint")?;
    let (first, last) = layers.fold((initial, initial), |(first, last), z| {
        (first.min(z), last.max(z))
    });
    for (i, c) in controls.iter().enumerate() {
        if consumed.contains(&i) || removed.contains(&i) {
            continue;
        }
        let Some(frame) = paint_frame(c) else {
            continue;
        };
        if frame.z_order <= first || frame.z_order >= last {
            continue;
        }
        if consumed
            .iter()
            .any(|&source| paint_frame(&controls[source]).is_some_and(|s| intersects(frame, s)))
        {
            return Err(
                "overlapping source paint interleaves body text; retain source owners".into(),
            );
        }
    }
    Ok(last)
}
struct Prepared<'a> {
    proposal: &'a ParagraphProposal,
    item: FlowItem,
    owned: BTreeSet<usize>,
}
fn prepare<'a>(
    page: &ExtractedPage,
    doc: &mut Document,
    region: &BodyRecognition,
    p: &'a ParagraphProposal,
    span_controls: &[Option<usize>],
    shape_ids: &[Option<u32>],
    styles: &SourceStyles,
    typography: &std::cell::RefCell<Vec<Value>>,
    removed: &BTreeSet<usize>,
) -> Result<Prepared<'a>, String> {
    if p.span_indices
        .iter()
        .any(|&i| super::super::text_clip::has_nonrectangular(page.span_clips[i].as_deref()))
    {
        return Err("source text clip requires positional editable owner".into());
    }
    let allocation = Allocation(&page.import_budget);
    let mut owned = BTreeSet::new();
    for &source in &p.span_indices {
        let control = span_controls[source].ok_or("missing body source owner")?;
        if removed.contains(&control)
            || !allocation
                .insert_set(&mut owned, control)
                .map_err(message)?
        {
            return Err("body source already owned".into());
        }
    }
    let item = match verified_paragraph(page, doc, p, region.bbox, shape_ids, styles, typography) {
        Ok(item) => item,
        Err(reason) => {
            return Err(allocation
                .display(&format_args!("body {reason}"))
                .map_err(message)?)
        }
    };
    Ok(Prepared {
        proposal: p,
        item,
        owned,
    })
}
fn frame(
    doc: &mut Document,
    region: &BodyRecognition,
    mut flow: Vec<FlowItem>,
    consumed: &BTreeSet<usize>,
    controls: &[Control],
    removed: &BTreeSet<usize>,
    id: u32,
    allocation: Allocation<'_>,
) -> Result<Control, String> {
    let z = paint_order(controls, consumed, removed)?;
    let top = flow.first().ok_or("empty body frame")?.top;
    let mut previous_bottom = top;
    let mut previous_style = None;
    for item in &mut flow {
        if item.top < previous_bottom - 0.5 {
            return Err("overlapping body paragraph frames".into());
        }
        let gap = (item.top - previous_bottom).max(0.);
        if let Some(style) = previous_style {
            doc.doc_info.para_shapes[style as usize].spacing_after = physical_margin(gap)?;
        }
        // Publish source vpos cumulatively in the common textbox frame. Fresh
        // reflow uses the same paragraph spacing, not a PDF paint override.
        let offset = hu(item.top - top)? as i32;
        for line in &mut item.paragraph.line_segs {
            line.vertical_pos += offset;
        }
        previous_bottom = item.bottom;
        previous_style = Some(item.paragraph.para_shape_id);
    }
    let mut attrs = super::super::lowering::common(
        region.bbox.x,
        top,
        region.bbox.width,
        previous_bottom - top,
        id,
    )?;
    attrs.z_order = z;
    let (width, height) = (attrs.width, attrs.height);
    Ok(Control::Shape(
        allocation
            .boxed(ShapeObject::Rectangle(RectangleShape {
                common: attrs,
                x_coords: [0, width as i32, width as i32, 0],
                y_coords: [0, 0, height as i32, height as i32],
                drawing: DrawingObjAttr {
                    shape_attr: super::super::lowering::component(width, height),
                    inst_id: id,
                    text_box: Some(TextBox {
                        max_width: width,
                        text_margin_explicit: true,
                        paragraphs: allocation
                            .collect(flow.into_iter().map(|f| f.paragraph))
                            .map_err(message)?,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }))
            .map_err(message)?,
    ))
}
#[derive(Default)]
struct Output {
    frames: Vec<Control>,
    reports: Vec<Value>,
    paragraphs: usize,
}
impl Output {
    fn commit(
        &mut self,
        ri: usize,
        region: &BodyRecognition,
        batch: &mut Vec<Prepared<'_>>,
        cp: StyleCheckpoint,
        doc: &mut Document,
        typography: &std::cell::RefCell<Vec<Value>>,
        controls: &[Control],
        removed: &mut BTreeSet<usize>,
        next_id: &mut u32,
        allocation: Allocation<'_>,
    ) -> Result<bool, String> {
        if batch.is_empty() {
            return Ok(true);
        }
        let count = batch.len();
        let spans: Vec<_> = allocation
            .collect(
                batch
                    .iter()
                    .flat_map(|p| p.proposal.span_indices.iter().copied()),
            )
            .map_err(message)?;
        // Observed text extent of the frame, in page points. Layout modes that
        // place these paragraphs outside the frame use it as their measure.
        let lines = || batch.iter().flat_map(|p| p.proposal.lines.iter());
        let text_box = [
            lines().map(|l| l.left).fold(f32::MAX, f32::min),
            batch.first().map_or(0., |p| p.item.top),
            lines().map(|l| l.right).fold(f32::MIN, f32::max),
            batch.last().map_or(0., |p| p.item.bottom),
        ];
        let mut owned = BTreeSet::new();
        let mut flow = Vec::new();
        for p in batch.drain(..) {
            allocation
                .extend_set(&mut owned, p.owned)
                .map_err(message)?;
            allocation.push(&mut flow, p.item).map_err(message)?;
        }
        let built = next_id
            .checked_add(1)
            .ok_or("body frame ID overflow".into())
            .and_then(|next| {
                frame(
                    doc, region, flow, &owned, controls, removed, *next_id, allocation,
                )
                .map(|c| (next, c))
            });
        allocation.0.check().map_err(|e| e.to_string())?;
        let committed = match built {
            Ok((next, control)) => {
                self.paragraphs += count;
                allocation.push(&mut self.reports, request_json!(allocation, {"region":ri,"status":"paragraph_ir","instanceId":*next_id,"paragraphCount":count,"textBox":text_box,"spans":spans})?).map_err(message)?;
                allocation
                    .push(&mut self.frames, control)
                    .map_err(message)?;
                allocation.extend_set(removed, owned).map_err(message)?;
                *next_id = next;
                true
            }
            Err(reason) => {
                rollback(doc, typography, cp);
                allocation.push(&mut self.reports, request_json!(allocation, {"region":ri,"status":"retained_positioned_spans","reason":reason,"spans":spans})?).map_err(message)?;
                false
            }
        };
        Ok(committed)
    }
}
pub(super) fn lower(
    page: &ExtractedPage,
    doc: &mut Document,
    controls: &[Control],
    span_controls: &[Option<usize>],
    shape_ids: &[Option<u32>],
    styles: &SourceStyles,
    typography: &std::cell::RefCell<Vec<Value>>,
    removed: &mut BTreeSet<usize>,
    next_id: &mut u32,
) -> Result<(Vec<Control>, Value), String> {
    let allocation = Allocation(&page.import_budget);
    let mut output = Output::default();
    for (ri, region) in page.recognition.body.iter().enumerate() {
        let mut batch = Vec::new();
        let mut batch_start = checkpoint(doc, typography);
        for (pi, p) in region.recognition.paragraphs.iter().enumerate() {
            let confirmed = p.status == ParagraphState::Confirmed && p.layout.is_some();
            // One frame repaints its paragraphs at a single layer. Close the
            // frame before a paragraph whose sources would interleave with
            // other paint, so the conflict retains only the paragraphs it
            // actually touches.
            let owned = allocation
                .set_from(p.span_indices.iter().filter_map(|&i| span_controls[i]))
                .map_err(message)?;
            let alone_conflicts = confirmed && paint_order(controls, &owned, removed).is_err();
            if confirmed && !alone_conflicts && !batch.is_empty() {
                let mut joined = allocation
                    .set_from(owned.iter().copied())
                    .map_err(message)?;
                allocation
                    .extend_set(
                        &mut joined,
                        batch
                            .iter()
                            .flat_map(|b: &Prepared<'_>| b.owned.iter().copied()),
                    )
                    .map_err(message)?;
                if paint_order(controls, &joined, removed).is_err() {
                    output.commit(
                        ri,
                        region,
                        &mut batch,
                        batch_start,
                        doc,
                        typography,
                        controls,
                        removed,
                        next_id,
                        allocation,
                    )?;
                }
            }
            let before = checkpoint(doc, typography);
            let prepared = if alone_conflicts {
                Err("overlapping source paint interleaves body text; retain source owners".into())
            } else if confirmed {
                prepare(
                    page,
                    doc,
                    region,
                    p,
                    span_controls,
                    shape_ids,
                    styles,
                    typography,
                    removed,
                )
            } else {
                Err(allocation.string(&p.reason).map_err(message)?)
            };
            allocation.0.check().map_err(|e| e.to_string())?;
            match prepared {
                Ok(prepared) => {
                    // Frames stack paragraphs top to bottom. A paragraph box
                    // that starts above the previous one's end opens a new frame.
                    if batch
                        .last()
                        .is_some_and(|b| prepared.item.top < b.item.bottom - 0.5)
                    {
                        let committed = output.commit(
                            ri,
                            region,
                            &mut batch,
                            batch_start,
                            doc,
                            typography,
                            controls,
                            removed,
                            next_id,
                            allocation,
                        )?;
                        if !committed {
                            // The failed batch rolled back this paragraph's styles too.
                            allocation.push(&mut output.reports, request_json!(allocation, {"region":ri,"paragraph":pi,"status":"retained_positioned_spans","reason":"previous body frame was not representable","spans":p.span_indices})?).map_err(message)?;
                            continue;
                        }
                    }
                    if batch.is_empty() {
                        batch_start = before;
                    }
                    allocation.push(&mut batch, prepared).map_err(message)?;
                    if p.reason == "intervening_object" {
                        output.commit(
                            ri,
                            region,
                            &mut batch,
                            batch_start,
                            doc,
                            typography,
                            controls,
                            removed,
                            next_id,
                            allocation,
                        )?;
                    }
                }
                Err(reason) => {
                    rollback(doc, typography, before);
                    output.commit(
                        ri,
                        region,
                        &mut batch,
                        batch_start,
                        doc,
                        typography,
                        controls,
                        removed,
                        next_id,
                        allocation,
                    )?;
                    allocation.push(&mut output.reports, request_json!(allocation, {"region":ri,"paragraph":pi,"status":"retained_positioned_spans","reason":reason,"spans":p.span_indices})?).map_err(message)?;
                }
            }
        }
        output.commit(
            ri,
            region,
            &mut batch,
            batch_start,
            doc,
            typography,
            controls,
            removed,
            next_id,
            allocation,
        )?;
    }
    Ok((
        output.frames,
        request_json!(allocation, {"frames":output.reports,"semanticParagraphs":output.paragraphs,"policy":"owner-local representable proposals; original scalars/joins; cumulative frame LineSegs; ambiguous metrics and interleaved paint retained"})?,
    ))
}
