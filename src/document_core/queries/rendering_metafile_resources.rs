//! 요청한 쪽 순서 대신 원본 컨트롤 순서로 WMF의 최초 자원을 확정한다.

use crate::model::{
    control::Control,
    document::Document,
    image::Picture,
    paragraph::Paragraph,
    shape::{Caption, ShapeObject, SizeCriterion},
};
use crate::paint::metafile_print::{Catalogue, SourceUse};
use crate::renderer::pagination::{PageItem, PaginationResult};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Copy, PartialEq)]
enum Order {
    Known,
    LocalUnknown,
    Repeated,
}

impl Order {
    fn group(self) -> Self {
        if self == Self::Repeated {
            self
        } else {
            Self::LocalUnknown
        }
    }
}

struct Collector {
    resources: HashMap<u16, Arc<[u8]>>,
    seen: HashSet<u16>,
    uses: Vec<(usize, u8, SourceUse)>,
    anchor: usize,
    first_paragraphs: HashMap<usize, usize>,
}

impl Collector {
    fn picture(&mut self, picture: &Picture, order: Order, depth: usize) -> Option<()> {
        if let Some(data) = self.resources.get(&picture.image_attr.bin_data_id) {
            if self.uses.len() >= 4096 {
                return None;
            }
            let common = &picture.common;
            let shape = &picture.shape_attr;
            let width = (order == Order::Known
                && common.width_criterion == SizeCriterion::Absolute
                && common.width > 0
                && shape.rotation_angle == 0
                && !shape.horz_flip
                && !shape.vert_flip
                && !picture.reverse
                && shape.render_sx == 1.0
                && shape.render_sy == 1.0
                && shape.render_b == 0.0
                && shape.render_c == 0.0
                && shape.render_tx == 0.0
                && shape.render_ty == 0.0)
                .then_some(f64::from(common.width) / 75.0);
            self.seen.insert(picture.image_attr.bin_data_id);
            let (anchor, priority) = if order == Order::Repeated {
                // 머리말·꼬리말은 정의 문단이 속한 첫 페이지부터 반복된다.
                // 그 페이지의 본문보다 앞에서 불확실성 경계를 세운다.
                (*self.first_paragraphs.get(&self.anchor).unwrap_or(&0), 0)
            } else {
                (self.anchor, 1)
            };
            self.uses.push((
                anchor,
                priority,
                SourceUse {
                    data: data.clone(),
                    order_known: order == Order::Known,
                    width,
                    crop: picture.render_crop_rect(),
                    crop_reference: picture.crop_reference_size(),
                },
            ));
        }
        self.caption(picture.caption.as_ref(), order, depth + 1)
    }

    fn caption(&mut self, caption: Option<&Caption>, order: Order, depth: usize) -> Option<()> {
        if let Some(caption) = caption {
            self.paragraphs(&caption.paragraphs, order, depth + 1)?;
        }
        Some(())
    }

    fn shape(&mut self, shape: &ShapeObject, order: Order, depth: usize) -> Option<()> {
        if depth > 64 {
            return None;
        }
        if let Some(drawing) = shape.drawing() {
            if let Some(text) = &drawing.text_box {
                self.paragraphs(&text.paragraphs, order, depth + 1)?;
            }
            self.caption(drawing.caption.as_ref(), order, depth + 1)?;
        }
        match shape {
            ShapeObject::Group(group) => {
                for child in &group.children {
                    self.shape(child, order.group(), depth + 1)?;
                }
                self.caption(group.caption.as_ref(), order.group(), depth + 1)?;
            }
            ShapeObject::Picture(picture) => self.picture(picture, order, depth + 1)?,
            _ => (),
        }
        Some(())
    }

    fn paragraphs(&mut self, paragraphs: &[Paragraph], order: Order, depth: usize) -> Option<()> {
        if depth > 64 {
            return None;
        }
        for paragraph in paragraphs {
            for control in &paragraph.controls {
                match control {
                    Control::Picture(picture) => self.picture(picture, order, depth + 1)?,
                    Control::Shape(shape) => self.shape(shape, order, depth + 1)?,
                    Control::Table(table) => {
                        for cell in &table.cells {
                            self.paragraphs(&cell.paragraphs, order, depth + 1)?;
                        }
                        self.caption(table.caption.as_ref(), order, depth + 1)?;
                    }
                    Control::Header(header) => {
                        self.paragraphs(&header.paragraphs, Order::Repeated, depth + 1)?
                    }
                    Control::Footer(footer) => {
                        self.paragraphs(&footer.paragraphs, Order::Repeated, depth + 1)?
                    }
                    Control::Footnote(note) => {
                        self.paragraphs(&note.paragraphs, order, depth + 1)?
                    }
                    Control::Endnote(note) => {
                        self.paragraphs(&note.paragraphs, Order::Repeated, depth + 1)?
                    }
                    Control::HiddenComment(comment) => {
                        self.paragraphs(&comment.paragraphs, Order::Repeated, depth + 1)?
                    }
                    Control::Field(field) => {
                        self.paragraphs(&field.memo_paragraphs, Order::Repeated, depth + 1)?
                    }
                    Control::SectionDef(def) => {
                        for master in &def.master_pages {
                            self.paragraphs(&master.paragraphs, Order::Repeated, depth + 1)?;
                        }
                    }
                    _ => (),
                }
            }
        }
        Some(())
    }
}

pub(super) fn catalogue(
    document: &Document,
    pagination: &[PaginationResult],
) -> Option<Arc<Catalogue>> {
    let mut collector = Collector {
        resources: HashMap::new(),
        seen: HashSet::new(),
        uses: Vec::new(),
        anchor: 0,
        first_paragraphs: HashMap::new(),
    };
    let mut bytes = 0usize;
    for item in &document.bin_data_content {
        // 파일명·쪽 번호로 자원을 고르지 않는다. 메타파일 원본의 실제 magic을 읽는다.
        if let Some(data) = item
            .data
            .load_limited_shared(crate::model::bin_data::MAX_BIN_DATA_BYTES)
        {
            if crate::renderer::image_resolver::detect_image_mime_type(&data) == "image/x-wmf" {
                bytes = bytes.checked_add(data.len())?;
                if bytes > 32 * 1024 * 1024 {
                    return None;
                }
                collector.resources.insert(item.id, data);
            }
        }
    }
    if collector.resources.is_empty() {
        return None;
    }
    let mut uses = Vec::new();
    for (section_index, section) in document.sections.iter().enumerate() {
        collector.first_paragraphs.clear();
        if let Some(pagination) = pagination.get(section_index) {
            for page in &pagination.pages {
                let indices = page
                    .column_contents
                    .iter()
                    .flat_map(|column| &column.items)
                    .filter_map(|item| match item {
                        PageItem::FullParagraph { para_index }
                        | PageItem::PartialParagraph { para_index, .. }
                        | PageItem::Table { para_index, .. }
                        | PageItem::PartialTable { para_index, .. }
                        | PageItem::Shape { para_index, .. } => Some(*para_index),
                        PageItem::EndnoteSeparator { .. } => None,
                    })
                    .collect::<Vec<_>>();
                let first = indices.iter().copied().min().unwrap_or(0);
                for index in indices {
                    collector.first_paragraphs.entry(index).or_insert(first);
                }
                for reference in [&page.active_header, &page.active_footer]
                    .into_iter()
                    .flatten()
                {
                    if reference.source_section_index == section_index {
                        collector
                            .first_paragraphs
                            .entry(reference.para_index)
                            .or_insert(first);
                    }
                }
            }
        }
        collector.anchor = 0;
        for master in &section.section_def.master_pages {
            collector.paragraphs(&master.paragraphs, Order::Repeated, 0)?;
        }
        for (index, paragraph) in section.paragraphs.iter().enumerate() {
            collector.anchor = index;
            collector.paragraphs(std::slice::from_ref(paragraph), Order::Known, 0)?;
        }
        collector
            .uses
            .sort_by_key(|(anchor, priority, _)| (*anchor, *priority));
        uses.extend(collector.uses.drain(..).map(|(_, _, usage)| usage));
    }
    // 아직 해석하지 않은 fill/OLE/목록의 사용을 빈 자원처럼 건너뛰지 않는다.
    if collector.seen.len() != collector.resources.len() {
        return None;
    }
    Catalogue::from_uses(&uses)
}
