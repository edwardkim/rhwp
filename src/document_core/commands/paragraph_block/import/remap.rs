use super::super::{invalid, validation::Resource};
use super::resources::Resources;
use crate::{
    error::HwpError,
    model::{
        control::Control,
        identity::walk::{walk, Node},
        paragraph::{CharShapeRef, Paragraph},
        shape::ShapeObject,
    },
};

pub(super) fn paragraphs(paras: &mut [Paragraph], resources: &Resources) -> Result<(), HwpError> {
    // Same owned tree as A's identity assignment, after strict support validation.
    // This pass changes only shared references; A assigns owned identities once afterward.
    walk(paras, |node| {
        match node {
            Node::Paragraph(p) => {
                p.style_id = u8::try_from(resources.id(Resource::Style(p.style_id))?)
                    .map_err(|_| invalid("style exceeds u8"))?;
                p.para_shape_id = resources.id16(Resource::Para(p.para_shape_id))?;
                if p.char_shapes.is_empty() {
                    p.char_shapes.push(CharShapeRef {
                        start_pos: 0,
                        char_shape_id: resources.id(Resource::Char(0))?,
                    });
                } else {
                    for run in &mut p.char_shapes {
                        run.char_shape_id = resources.id(Resource::Char(run.char_shape_id))?;
                    }
                }
            }
            Node::Control(Control::Table(t)) => {
                t.border_fill_id = resources.border(t.border_fill_id)?;
                for zone in &mut t.zones {
                    zone.border_fill_id = resources.border(zone.border_fill_id)?;
                }
                for cell in &mut t.cells {
                    cell.border_fill_id = resources.border(cell.border_fill_id)?;
                }
            }
            Node::Control(Control::Picture(p)) | Node::Shape(ShapeObject::Picture(p)) => {
                p.image_attr.bin_data_id =
                    resources.id16(Resource::Image(p.image_attr.bin_data_id))?;
            }
            Node::Control(Control::Field(f)) => {
                if let Some(residue) = &mut f.guide_residue {
                    residue.char_shape_id = resources.id(Resource::Char(residue.char_shape_id))?;
                }
            }
            Node::Shape(shape) => {
                if let Some(drawing) = shape.drawing_mut() {
                    resources.fill(&mut drawing.fill)?;
                }
            }
            _ => {}
        }
        Ok(())
    })
}
