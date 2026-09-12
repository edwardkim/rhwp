//! Read-only, reachable DocInfo graph. No style import, repair or image loading.
use super::{
    reject, Located, ParagraphBlockPathStep as Step, ParagraphBlockValidationError as Error,
    SourceNode,
};
use crate::document_core::commands::paragraph_block::RepeatParagraphBlockRequest;
use crate::model::{
    control::Control,
    document::Document,
    shape::ShapeObject,
    style::{Fill, HeadType},
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Resource {
    Style(u8),
    Para(u16),
    Char(u32),
    Border(u16),
    Tab(u16),
    Number(u16),
    Bullet(u16),
    Font(usize, u16),
    Image(u16),
    Storage(u16),
}

pub(super) fn validate(
    doc: &Document,
    nodes: &[Located<'_>],
    request: &RepeatParagraphBlockRequest,
) -> Result<(), Error> {
    let mut scan = Scan {
        doc,
        outline: doc.sections[request.section_index]
            .section_def
            .outline_numbering_id,
        remaining: request.limits.max_document_nodes,
        seen: BTreeSet::new(),
        pending: Vec::new(),
    };
    for located in nodes {
        let path = &located.path;
        match located.node {
            SourceNode::Paragraph(p) => {
                scan.add(Resource::Style(p.style_id), path)?;
                scan.add(Resource::Para(p.para_shape_id), path)?;
                // Empty runs use the existing character-shape zero fallback.
                if p.char_shapes.is_empty() {
                    scan.add(Resource::Char(0), path)?;
                }
                for run in &p.char_shapes {
                    scan.add(Resource::Char(run.char_shape_id), path)?;
                }
            }
            SourceNode::Control(Control::Table(t)) => {
                scan.border(t.border_fill_id, path)?;
                for zone in &t.zones {
                    scan.border(zone.border_fill_id, path)?;
                }
                scan.drain(path)?;
                for (i, cell) in t.cells.iter().enumerate() {
                    let mut cell_path = path.clone();
                    cell_path.push(Step::Cell(i));
                    scan.border(cell.border_fill_id, &cell_path)?;
                    scan.drain(&cell_path)?;
                }
            }
            SourceNode::Control(Control::Picture(p))
            | SourceNode::Shape(ShapeObject::Picture(p)) => {
                scan.add(Resource::Image(p.image_attr.bin_data_id), path)?;
            }
            SourceNode::Shape(s) => {
                if let Some(d) = s.drawing() {
                    scan.fill(&d.fill, path)?;
                }
            }
            SourceNode::Control(_) => {} // Support profile was checked before this pass.
        }
        scan.drain(path)?;
    }
    Ok(())
}

struct Scan<'a> {
    doc: &'a Document,
    outline: u16,
    remaining: usize,
    seen: BTreeSet<Resource>,
    pending: Vec<Resource>,
}
impl Scan<'_> {
    fn tick(&mut self, path: &[Step]) -> Result<(), Error> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            reject(
                path,
                "budgetOrAddress",
                "shared resource edge/lookup budget exceeded",
            )
        })?;
        Ok(())
    }
    fn add(&mut self, resource: Resource, path: &[Step]) -> Result<(), Error> {
        self.tick(path)?;
        if self.seen.insert(resource) {
            self.pending.push(resource);
        }
        Ok(())
    }
    fn border(&mut self, id: u16, path: &[Step]) -> Result<(), Error> {
        if id != 0 {
            self.add(Resource::Border(id), path)?;
        } else {
            self.tick(path)?;
        }
        Ok(())
    }
    fn fill(&mut self, fill: &Fill, path: &[Step]) -> Result<(), Error> {
        if let Some(image) = &fill.image {
            self.add(Resource::Image(image.bin_data_id), path)?;
        }
        Ok(())
    }
    fn embedded(&mut self, embedded: bool, id: Option<u16>, path: &[Step]) -> Result<(), Error> {
        if embedded {
            let id = id.ok_or_else(|| {
                reject(
                    path,
                    "missingResource",
                    "embedded font has no resolved storage ID",
                )
            })?;
            self.add(Resource::Storage(id), path)?;
        }
        Ok(())
    }
    fn drain(&mut self, path: &[Step]) -> Result<(), Error> {
        while let Some(resource) = self.pending.pop() {
            let missing = || reject(path, "missingResource", format!("unresolved {resource:?}"));
            let info = &self.doc.doc_info;
            match resource {
                Resource::Style(id) => {
                    let s = info.styles.get(usize::from(id)).ok_or_else(missing)?;
                    self.add(Resource::Style(s.next_style_id), path)?;
                    self.add(Resource::Para(s.para_shape_id), path)?;
                    self.add(Resource::Char(u32::from(s.char_shape_id)), path)?;
                }
                Resource::Para(id) => {
                    let p = info.para_shapes.get(usize::from(id)).ok_or_else(missing)?;
                    self.add(Resource::Tab(p.tab_def_id), path)?;
                    self.border(p.border_fill_id, path)?;
                    let id = if p.head_type == HeadType::Outline && p.numbering_id == 0 {
                        self.outline
                    } else {
                        p.numbering_id
                    };
                    // Zero is the built-in/no explicit definition, not index zero.
                    if id != 0 {
                        match p.head_type {
                            HeadType::Number | HeadType::Outline => {
                                self.add(Resource::Number(id), path)?
                            }
                            HeadType::Bullet => self.add(Resource::Bullet(id), path)?,
                            HeadType::None => {}
                        }
                    }
                }
                Resource::Char(id) => {
                    let c = info.char_shapes.get(id as usize).ok_or_else(missing)?;
                    self.border(c.border_fill_id, path)?;
                    for (language, id) in c.font_ids.iter().enumerate() {
                        self.add(Resource::Font(language, *id), path)?;
                    }
                }
                Resource::Font(language, id) => {
                    let f = info
                        .font_faces
                        .get(language)
                        .and_then(|fonts| fonts.get(usize::from(id)))
                        .ok_or_else(missing)?;
                    self.embedded(f.is_embedded, f.resolved_bin_data_id, path)?;
                    if let Some(s) = &f.subst_font {
                        self.embedded(s.is_embedded, s.resolved_bin_data_id, path)?;
                    }
                }
                Resource::Border(id) => {
                    let b = info
                        .border_fills
                        .get(usize::from(id) - 1)
                        .ok_or_else(missing)?;
                    self.fill(&b.fill, path)?;
                }
                Resource::Tab(id) => {
                    info.tab_defs.get(usize::from(id)).ok_or_else(missing)?;
                }
                Resource::Number(id) => {
                    let n = info
                        .numberings
                        .get(usize::from(id) - 1)
                        .ok_or_else(missing)?;
                    for head in &n.heads {
                        // HWP's signed -1 means use the paragraph character shape.
                        if head.char_shape_id != u32::MAX {
                            self.add(Resource::Char(head.char_shape_id), path)?;
                        }
                    }
                }
                Resource::Bullet(id) => {
                    let b = info.bullets.get(usize::from(id) - 1).ok_or_else(missing)?;
                    if b.char_shape_id != u32::MAX {
                        self.add(Resource::Char(b.char_shape_id), path)?;
                    }
                    if b.image_bullet != 0 {
                        return Err(reject(
                            path,
                            "unsupportedResource",
                            "image bullet reference encoding requires a separate contract",
                        ));
                    }
                }
                Resource::Image(id) => {
                    if id == 0 {
                        return Err(missing());
                    }
                    // Same ordinal-first lookup as renderer::layout::utils::find_bin_data_index.
                    // Content.id is a storage ID, so an in-range slot need not have id == ref.
                    if self.doc.bin_data_content.get(usize::from(id) - 1).is_none() {
                        self.storage(id, path)?;
                    }
                }
                Resource::Storage(id) => self.storage(id, path)?,
            }
        }
        Ok(())
    }
    fn storage(&mut self, id: u16, path: &[Step]) -> Result<(), Error> {
        // Charge each examined metadata entry, never call data.len/is_empty/load.
        for content in &self.doc.bin_data_content {
            self.tick(path)?;
            if id != 0 && content.id == id {
                return Ok(());
            }
        }
        Err(reject(
            path,
            "missingResource",
            format!("unresolved BinData storage ID {id}"),
        ))
    }
}
