//! Resolve already-validated owned paths on detached copies, never on the live core.
use super::{invalid, ParagraphBlockPathStep as Step};
use crate::{
    document_core::TableTextReflowKey,
    error::HwpError,
    model::{control::Control, paragraph::Paragraph, shape::ShapeObject},
};

pub(super) fn paragraph<'a>(
    paras: &'a mut [Paragraph],
    path: &[Step],
    tables: &mut Vec<TableTextReflowKey>,
) -> Result<&'a mut Paragraph, HwpError> {
    let Some((Step::Paragraph(index), rest)) = path.split_first() else {
        return Err(invalid("fill path must start with a paragraph"));
    };
    let p = paras
        .get_mut(*index)
        .ok_or_else(|| invalid("fill paragraph missing"))?;
    if rest.is_empty() {
        return Ok(p);
    }
    let Some((Step::Control(index), rest)) = rest.split_first() else {
        return Err(invalid("fill path must descend through a control"));
    };
    match p.controls.get_mut(*index) {
        Some(Control::Table(table)) => match rest.split_first() {
            Some((Step::Cell(index), rest)) => {
                tables.push(TableTextReflowKey::from_table(table));
                let cell = table
                    .cells
                    .get_mut(*index)
                    .ok_or_else(|| invalid("fill cell missing"))?;
                paragraph(&mut cell.paragraphs, rest, tables)
            }
            Some((Step::Caption, rest)) => paragraph(
                &mut table
                    .caption
                    .as_mut()
                    .ok_or_else(|| invalid("fill caption missing"))?
                    .paragraphs,
                rest,
                tables,
            ),
            _ => Err(invalid("invalid table fill path")),
        },
        Some(Control::Shape(s)) => match rest.split_first() {
            Some((Step::Shape, rest)) => shape(s, rest, tables),
            _ => Err(invalid("invalid shape fill path")),
        },
        Some(Control::Picture(p)) => match rest.split_first() {
            Some((Step::Caption, rest)) => paragraph(
                &mut p
                    .caption
                    .as_mut()
                    .ok_or_else(|| invalid("fill caption missing"))?
                    .paragraphs,
                rest,
                tables,
            ),
            _ => Err(invalid("invalid picture fill path")),
        },
        _ => Err(invalid("unsupported fill control path")),
    }
}

fn shape<'a>(
    s: &'a mut ShapeObject,
    path: &[Step],
    tables: &mut Vec<TableTextReflowKey>,
) -> Result<&'a mut Paragraph, HwpError> {
    match (s, path.split_first()) {
        (ShapeObject::Group(g), Some((Step::GroupChild(i), rest))) => shape(
            g.children
                .get_mut(*i)
                .ok_or_else(|| invalid("fill group child missing"))?,
            rest,
            tables,
        ),
        (ShapeObject::Group(g), Some((Step::Caption, rest))) => paragraph(
            &mut g
                .caption
                .as_mut()
                .ok_or_else(|| invalid("fill caption missing"))?
                .paragraphs,
            rest,
            tables,
        ),
        (ShapeObject::Picture(p), Some((Step::Caption, rest))) => paragraph(
            &mut p
                .caption
                .as_mut()
                .ok_or_else(|| invalid("fill caption missing"))?
                .paragraphs,
            rest,
            tables,
        ),
        (s, Some((step, rest))) => {
            let d = s
                .drawing_mut()
                .ok_or_else(|| invalid("fill drawing missing"))?;
            let paras = match step {
                Step::TextBox => {
                    &mut d
                        .text_box
                        .as_mut()
                        .ok_or_else(|| invalid("fill textbox missing"))?
                        .paragraphs
                }
                Step::Caption => {
                    &mut d
                        .caption
                        .as_mut()
                        .ok_or_else(|| invalid("fill caption missing"))?
                        .paragraphs
                }
                _ => return Err(invalid("invalid drawing fill path")),
            };
            paragraph(paras, rest, tables)
        }
        _ => Err(invalid("incomplete shape fill path")),
    }
}
