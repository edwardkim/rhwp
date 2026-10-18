//! Commit representable body proposals through the same paragraph result as cells.
use super::*;
use crate::model::shape::{DrawingObjAttr, RectangleShape, TextBox};
use crate::parser::pdf::recognition::BodyRecognition;

type Checkpoint = (usize, usize, usize, usize);
fn checkpoint(doc: &Document, typography: &std::cell::RefCell<Vec<Value>>) -> Checkpoint {
    (
        doc.doc_info.para_shapes.len(),
        doc.doc_info.char_shapes.len(),
        doc.doc_info.tab_defs.len(),
        typography.borrow().len(),
    )
}
fn rollback(doc: &mut Document, typography: &std::cell::RefCell<Vec<Value>>, c: Checkpoint) {
    doc.doc_info.para_shapes.truncate(c.0);
    doc.doc_info.char_shapes.truncate(c.1);
    doc.doc_info.tab_defs.truncate(c.2);
    typography.borrow_mut().truncate(c.3);
}
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
    let layers: Vec<_> = consumed
        .iter()
        .filter_map(|&i| paint_frame(&controls[i]).map(|f| f.z_order))
        .collect();
    let first = *layers
        .iter()
        .min()
        .ok_or("body frame has no source paint")?;
    let last = *layers.iter().max().unwrap();
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
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    typography: &std::cell::RefCell<Vec<Value>>,
    removed: &BTreeSet<usize>,
) -> Result<Prepared<'a>, String> {
    let mut owned = BTreeSet::new();
    for &source in &p.span_indices {
        let control = span_controls[source].ok_or("missing body source owner")?;
        if removed.contains(&control) || !owned.insert(control) {
            return Err("body source already owned".into());
        }
    }
    let start = typography.borrow().len();
    let item = paragraph(
        page,
        doc,
        p,
        &json!(region.bbox),
        shape_ids,
        true,
        styles,
        typography,
    )?;
    if let Some(reason) = typography.borrow()[start..]
        .iter()
        .find(|r| r["status"] == "unverified_legacy_line_frame")
        .and_then(|r| r["reason"].as_str())
    {
        // A tab-only substitute is not a proven body paragraph layout. Retain
        // the original source owners rather than broadening character metrics.
        return Err(format!(
            "body glyph metrics require positioned source: {reason}"
        ));
    }
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
    Ok(Control::Shape(Box::new(ShapeObject::Rectangle(
        RectangleShape {
            common: attrs,
            x_coords: [0, width as i32, width as i32, 0],
            y_coords: [0, 0, height as i32, height as i32],
            drawing: DrawingObjAttr {
                shape_attr: super::super::lowering::component(width, height),
                inst_id: id,
                text_box: Some(TextBox {
                    max_width: width,
                    text_margin_explicit: true,
                    paragraphs: flow.into_iter().map(|f| f.paragraph).collect(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    ))))
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
        cp: Checkpoint,
        doc: &mut Document,
        typography: &std::cell::RefCell<Vec<Value>>,
        controls: &[Control],
        removed: &mut BTreeSet<usize>,
        next_id: &mut u32,
    ) {
        if batch.is_empty() {
            return;
        }
        let count = batch.len();
        let spans: Vec<_> = batch
            .iter()
            .flat_map(|p| p.proposal.span_indices.iter().copied())
            .collect();
        let mut owned = BTreeSet::new();
        let mut flow = Vec::new();
        for p in batch.drain(..) {
            owned.extend(p.owned);
            flow.push(p.item);
        }
        let built = next_id
            .checked_add(1)
            .ok_or("body frame ID overflow".into())
            .and_then(|next| {
                frame(doc, region, flow, &owned, controls, removed, *next_id).map(|c| (next, c))
            });
        match built {
            Ok((next, control)) => {
                self.paragraphs += count;
                self.reports.push(json!({"region":ri,"status":"paragraph_ir","instanceId":*next_id,"paragraphCount":count,"spans":spans}));
                self.frames.push(control);
                removed.extend(owned);
                *next_id = next;
            }
            Err(reason) => {
                rollback(doc, typography, cp);
                self.reports.push(json!({"region":ri,"status":"retained_positioned_spans","reason":reason,"spans":spans}));
            }
        }
    }
}
pub(super) fn lower(
    page: &ExtractedPage,
    doc: &mut Document,
    controls: &[Control],
    span_controls: &[Option<usize>],
    shape_ids: &[Option<u32>],
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    typography: &std::cell::RefCell<Vec<Value>>,
    removed: &mut BTreeSet<usize>,
    next_id: &mut u32,
) -> (Vec<Control>, Value) {
    let mut output = Output::default();
    for (ri, region) in page.recognition.body.iter().enumerate() {
        let mut batch = Vec::new();
        let mut batch_start = checkpoint(doc, typography);
        for (pi, p) in region.recognition.paragraphs.iter().enumerate() {
            let before = checkpoint(doc, typography);
            let prepared = if p.status == ParagraphState::Confirmed && p.layout.is_some() {
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
                Err(p.reason.clone())
            };
            match prepared {
                Ok(prepared) => {
                    if batch.is_empty() {
                        batch_start = before;
                    }
                    batch.push(prepared);
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
                        );
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
                    );
                    output.reports.push(json!({"region":ri,"paragraph":pi,"status":"retained_positioned_spans","reason":reason,"spans":p.span_indices}));
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
        );
    }
    (
        output.frames,
        json!({"frames":output.reports,"semanticParagraphs":output.paragraphs,"policy":"owner-local representable proposals; original scalars/joins; cumulative frame LineSegs; ambiguous metrics and interleaved paint retained"}),
    )
}
