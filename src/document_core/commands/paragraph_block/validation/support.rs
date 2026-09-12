use super::{
    reject, Located, ParagraphBlockPathStep as Step, ParagraphBlockValidationError as Error,
    SourceNode,
};
use crate::document_core::commands::paragraph_block::RepeatParagraphBlockRequest;
use crate::model::{
    control::{Control, FieldType},
    image::Picture,
    paragraph::{ColumnBreakType, Paragraph},
    shape::{Caption, CommonObjAttr, ShapeObject},
};

pub(super) fn inspect<'a>(
    paras: &'a [Paragraph],
    request: &RepeatParagraphBlockRequest,
) -> Result<Vec<Located<'a>>, Error> {
    let mut scan = Scan {
        path: vec![],
        nodes: vec![],
        visited: 0,
        path_bytes: 0,
        request,
    };
    scan.paras(paras)?;
    Ok(scan.nodes)
}

struct Scan<'a, 'r> {
    path: Vec<Step>,
    nodes: Vec<Located<'a>>,
    visited: usize,
    path_bytes: usize,
    request: &'r RepeatParagraphBlockRequest,
}
impl<'a> Scan<'a, '_> {
    fn at(
        &mut self,
        step: Step,
        operation: impl FnOnce(&mut Self) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.visited += 1;
        let limits = self.request.limits;
        if self.visited > limits.max_nodes / self.request.count
            || self.path.len() >= limits.max_depth
        {
            return Err(reject(
                &self.path,
                "budgetOrAddress",
                "owned node/depth budget exceeded",
            ));
        }
        self.path_bytes = self
            .path_bytes
            .checked_add((self.path.len() + 1) * 32 + 64)
            .ok_or_else(|| reject(&self.path, "budgetOrAddress", "path budget overflow"))?;
        if self.path_bytes > limits.max_mapping_bytes / self.request.count {
            return Err(reject(
                &self.path,
                "budgetOrAddress",
                "mapping byte budget exceeded",
            ));
        }
        self.path.push(step);
        let result = operation(self);
        self.path.pop();
        result
    }
    fn keep(&mut self, node: SourceNode<'a>) {
        self.nodes.push(Located {
            node,
            path: self.path.clone(),
        });
    }
    fn unsupported(&self, detail: impl Into<String>) -> Error {
        reject(&self.path, "unsupported", detail)
    }
    fn raw_empty(&self, bytes: &[u8], name: &str) -> Result<(), Error> {
        if bytes.is_empty() {
            Ok(())
        } else {
            Err(self.unsupported(format!("uninterpreted {name}")))
        }
    }
    fn common(&self, c: &CommonObjAttr, raw: &[u8]) -> Result<(), Error> {
        self.raw_empty(&c.raw_extra, "common tail")?;
        let valid = match raw.len() {
            0 | 36 | 40 => true,
            // Legacy writer omits optional prevent_page_break and writes an
            // empty UTF-16 description length after the 36-byte common header.
            38 => raw[36..] == [0, 0],
            n if n >= 42 => n == 42 + usize::from(u16::from_le_bytes([raw[40], raw[41]])) * 2,
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(self.unsupported("truncated or extended common raw payload"))
        }
    }
    fn paras(&mut self, paras: &'a [Paragraph]) -> Result<(), Error> {
        for (i, p) in paras.iter().enumerate() {
            self.at(Step::Paragraph(i), |s| s.para(p))?;
        }
        Ok(())
    }
    fn para(&mut self, p: &'a Paragraph) -> Result<(), Error> {
        // RangeTag's kind/data namespace can carry extension semantics. No
        // identity remap contract exists for it in the initial strict profile.
        if !p.range_tags.is_empty() {
            return Err(self.unsupported("unvalidated paragraph range tags"));
        }
        if p.ctrl_data_records.len()
            > self.request.limits.max_structure_bytes
                / self.request.count
                / std::mem::size_of::<Option<Vec<u8>>>()
        {
            return Err(reject(
                &self.path,
                "budgetOrAddress",
                "CTRL_DATA array budget exceeded",
            ));
        }
        if matches!(
            p.column_type,
            ColumnBreakType::Section | ColumnBreakType::MultiColumn
        ) || p.raw_break_type & 3 != 0
        {
            return Err(self.unsupported("section/multicolumn paragraph boundary"));
        }
        if !(p.raw_header_extra.is_empty()
            || p.raw_header_extra.len() == 10
            || (p.raw_header_extra.len() == 12 && p.raw_header_extra[10..] == [0, 0]))
        {
            return Err(self.unsupported("paragraph raw extension/change tracking"));
        }
        for (index, data) in p.ctrl_data_records.iter().enumerate() {
            let Some(data) = data.as_ref().filter(|data| !data.is_empty()) else {
                continue;
            };
            // Same exact name-only ParameterSet as serializer::control. It has
            // no object identity/reference; do not accept additional items/tails.
            let name = match p.controls.get(index) {
                Some(Control::Field(f)) if f.field_type == FieldType::ClickHere => {
                    f.ctrl_data_name.as_deref()
                }
                _ => None,
            };
            let supported = name.is_some_and(|name| {
                data.len() >= 12
                    && data[..10] == [0x1b, 2, 1, 0, 0, 0, 0, 0x40, 1, 0]
                    && data.len() == 12 + usize::from(u16::from_le_bytes([data[10], data[11]])) * 2
                    && name.encode_utf16().eq(data[12..]
                        .chunks_exact(2)
                        .map(|b| u16::from_le_bytes([b[0], b[1]])))
            });
            if !supported {
                return Err(self.unsupported("uninterpreted CTRL_DATA payload"));
            }
        }
        self.keep(SourceNode::Paragraph(p));
        for (i, c) in p.controls.iter().enumerate() {
            self.at(Step::Control(i), |s| s.control(c))?;
        }
        Ok(())
    }
    fn caption(&mut self, caption: Option<&'a Caption>) -> Result<(), Error> {
        if let Some(c) = caption {
            self.at(Step::Caption, |s| s.paras(&c.paragraphs))?;
        }
        Ok(())
    }
    fn picture(&mut self, p: &'a Picture) -> Result<(), Error> {
        self.common(&p.common, &[])?;
        if !matches!(p.raw_picture_extra.len(), 0 | 5 | 17 | 18) {
            return Err(self.unsupported("picture raw extension"));
        }
        self.caption(p.caption.as_ref())
    }
    fn control(&mut self, c: &'a Control) -> Result<(), Error> {
        self.keep(SourceNode::Control(c));
        match c {
            Control::Table(t) => {
                self.common(&t.common, &t.raw_ctrl_data)?;
                self.raw_empty(&t.raw_table_record_extra, "table record tail")?;
                for (i, cell) in t.cells.iter().enumerate() {
                    self.at(Step::Cell(i), |s| {
                        let raw = &cell.raw_list_extra;
                        // serializer::control::build_cell_list_extra: ordinary
                        // cell = width(4) + nine zero bytes; named cell adds the
                        // fixed property marker, UTF-16 name and zero trailer.
                        let ordinary = raw.len() == 13 && raw[4..].iter().all(|b| *b == 0);
                        let named = raw.len() >= 25
                            && raw[4..15] == [0xff, 0x1b, 2, 1, 0, 0, 0, 0, 0x40, 1, 0]
                            && raw.len()
                                == 25 + usize::from(u16::from_le_bytes([raw[15], raw[16]])) * 2
                            && raw[raw.len() - 8..].iter().all(|b| *b == 0);
                        if !raw.is_empty() && !ordinary && !named {
                            return Err(s.unsupported("cell LIST_HEADER extension"));
                        }
                        s.paras(&cell.paragraphs)
                    })?;
                }
                self.caption(t.caption.as_ref())
            }
            Control::Shape(s) => self.at(Step::Shape, |scan| scan.shape(s)),
            Control::Picture(p) => self.picture(p),
            Control::Equation(e) => self.common(&e.common, &e.raw_ctrl_data),
            Control::Field(f) => {
                if f.field_type != FieldType::ClickHere
                    || f.raw_type.is_some()
                    || !f.memo_paragraphs.is_empty()
                    || f.memo_index != 0
                    || f.memo_text_direction.is_some()
                {
                    return Err(self.unsupported("only plain ClickHere fields are supported"));
                }
                if f.raw_parameters_xml.is_some() || !f.parameters.is_empty() {
                    return Err(self.unsupported("unvalidated ClickHere parameters"));
                }
                Ok(())
            }
            Control::SectionDef(_) => Err(self.unsupported("SectionDef")),
            Control::ColumnDef(_) => Err(self.unsupported("ColumnDef")),
            Control::Header(_) => Err(self.unsupported("Header")),
            Control::Footer(_) => Err(self.unsupported("Footer")),
            Control::Footnote(_) => Err(self.unsupported("Footnote")),
            Control::Endnote(_) => Err(self.unsupported("Endnote")),
            Control::AutoNumber(_) => Err(self.unsupported("AutoNumber")),
            Control::NewNumber(_) => Err(self.unsupported("NewNumber")),
            Control::PageNumberPos(_) => Err(self.unsupported("PageNumberPos")),
            Control::Bookmark(_) => Err(self.unsupported("Bookmark")),
            Control::IndexMark(_) => Err(self.unsupported("IndexMark")),
            Control::PageNumCtrl(_) => Err(self.unsupported("PageNumCtrl")),
            Control::Hyperlink(_) => Err(self.unsupported("Hyperlink")),
            Control::Ruby(_) => Err(self.unsupported("Ruby")),
            Control::CharOverlap(_) => Err(self.unsupported("CharOverlap")),
            Control::PageHide(_) => Err(self.unsupported("PageHide")),
            Control::HiddenComment(_) => Err(self.unsupported("HiddenComment")),
            Control::Form(_) => Err(self.unsupported("Form")),
            Control::Unknown(_) => Err(self.unsupported("Unknown")),
        }
    }
    fn shape(&mut self, shape: &'a ShapeObject) -> Result<(), Error> {
        self.keep(SourceNode::Shape(shape));
        self.common(shape.common(), &[])?;
        match shape {
            ShapeObject::Chart(_) => return Err(self.unsupported("Chart")),
            ShapeObject::Ole(_) => return Err(self.unsupported("Ole")),
            ShapeObject::Group(g) => {
                for (i, child) in g.children.iter().enumerate() {
                    self.at(Step::GroupChild(i), |s| s.shape(child))?;
                }
                return self.caption(g.caption.as_ref());
            }
            ShapeObject::Picture(p) => return self.picture(p),
            ShapeObject::Line(l) => {
                if let Some(c) = &l.connector {
                    self.raw_empty(&c.raw_trailing, "connector tail")?;
                }
            }
            ShapeObject::Polygon(p) => self.raw_empty(&p.raw_trailing, "polygon tail")?,
            ShapeObject::Rectangle(_)
            | ShapeObject::Ellipse(_)
            | ShapeObject::Arc(_)
            | ShapeObject::Curve(_) => {}
        }
        if let Some(d) = shape.drawing() {
            if let Some(t) = &d.text_box {
                self.at(Step::TextBox, |s| {
                    s.raw_empty(&t.raw_list_header_extra, "textbox LIST_HEADER tail")?;
                    s.paras(&t.paragraphs)
                })?;
            }
            self.caption(d.caption.as_ref())?;
        }
        Ok(())
    }
}
