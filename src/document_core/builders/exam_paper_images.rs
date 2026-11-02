//! Media-aware exam generation. Assets are bytes; filesystem access belongs to the host.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::document::{Document, Section};
use crate::model::image::Picture;
use crate::model::paragraph::{CharShapeRef, Paragraph};
use crate::model::shape::{CommonObjAttr, HorzRelTo, ObjectNumberingType, TextWrap, VertRelTo};
use crate::parser::ingest::schema::{IngestDocument, InlineRun, Media, Placement, StemBlock};

/// Resource bounds for external image batches, independent of filesystem access.
#[derive(Debug, Clone, Copy)]
pub struct IngestImageLimits {
    pub max_image_bytes: usize,
    pub max_total_image_bytes: usize,
    pub max_image_pixels: u64,
    pub max_total_image_pixels: u64,
    pub max_image_instances: usize,
}

impl Default for IngestImageLimits {
    fn default() -> Self {
        Self {
            max_image_bytes: 32 * 1024 * 1024,
            max_total_image_bytes: 256 * 1024 * 1024,
            max_image_pixels: 64_000_000,
            max_total_image_pixels: 256_000_000,
            max_image_instances: 16_384,
        }
    }
}

fn error(code: &str, detail: impl std::fmt::Display) -> HwpError {
    HwpError::InvalidFile(format!("INGEST_IMAGE_{code}: {detail}"))
}

/// A validated immutable image. Clones share encoded bytes, never decoded pixel buffers.
#[derive(Debug, Clone)]
pub struct IngestImage {
    bytes: Arc<[u8]>,
    width: u32,
    height: u32,
    extension: &'static str,
    digest: [u8; 32],
}

impl IngestImage {
    pub fn from_bytes(bytes: Vec<u8>, limits: IngestImageLimits) -> Result<Self, HwpError> {
        if bytes.is_empty()
            || bytes.len() > limits.max_image_bytes
            || bytes.len() > crate::model::bin_data::MAX_BIN_DATA_BYTES
        {
            return Err(error("LIMIT", "image byte limit exceeded or empty image"));
        }
        let format = image::guess_format(&bytes).map_err(|e| error("FORMAT", e))?;
        let extension = match format {
            image::ImageFormat::Png => "png",
            image::ImageFormat::Jpeg => "jpg",
            image::ImageFormat::Gif => "gif",
            image::ImageFormat::Bmp => "bmp",
            image::ImageFormat::Tiff => "tif",
            _ => return Err(error("FORMAT", "supported: PNG, JPEG, GIF, BMP, TIFF")),
        };
        let reader = image::ImageReader::with_format(std::io::Cursor::new(&bytes), format);
        let (width, height) = reader.into_dimensions().map_err(|e| error("DECODE", e))?;
        let pixels = u64::from(width) * u64::from(height);
        if width == 0
            || height == 0
            || pixels > limits.max_image_pixels
            || width > i32::MAX as u32 / 75
            || height > i32::MAX as u32 / 75
        {
            return Err(error("LIMIT", "image pixel/coordinate limit exceeded"));
        }
        // Checking only the header accepts truncated payloads and creates a broken picture.
        let mut decode_limits = image::Limits::default();
        decode_limits.max_alloc = Some(limits.max_image_pixels.saturating_mul(8));
        let mut reader = image::ImageReader::with_format(std::io::Cursor::new(&bytes), format);
        reader.limits(decode_limits);
        reader.decode().map_err(|e| error("DECODE", e))?;
        let digest = Sha256::digest(&bytes).into();
        Ok(Self {
            bytes: bytes.into(),
            width,
            height,
            extension,
            digest,
        })
    }

    pub fn pixel_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// Build real image controls without changing the legacy text/placeholder projection API.
/// Block placement overrides media placement; unreferenced media use their own placement.
/// Paragraph runs give precise text/image order within one paragraph.
pub fn build_exam_paper_with_images(
    ingest: &IngestDocument,
    images: &HashMap<String, IngestImage>,
    limits: IngestImageLimits,
) -> Result<Document, HwpError> {
    let mut doc = Document::default();
    let styles = super::init_exam_doc_info(&mut doc, &ingest.default_font);
    let mut section = Section::default();
    section.section_def.page_def = super::page_def_from_ingest(ingest.page_size);
    section.section_def.page_border_fill.border_fill_id = 1;
    section.section_def.page_border_fill.spacing_left = 1417;
    section.section_def.page_border_fill.spacing_right = 1417;
    section.section_def.page_border_fill.spacing_top = 1417;
    section.section_def.page_border_fill.spacing_bottom = 1417;
    let page = &section.section_def.page_def;
    let content_width = page
        .width
        .checked_sub(page.margin_left)
        .and_then(|w| w.checked_sub(page.margin_right))
        .filter(|w| *w > 0)
        .ok_or_else(|| error("SIZE", "page has no content width"))?;
    let content_height = page
        .height
        .checked_sub(page.margin_top)
        .and_then(|h| h.checked_sub(page.margin_bottom))
        .filter(|h| *h > 0)
        .ok_or_else(|| error("SIZE", "page has no content height"))?;
    doc.sections.push(section);
    let mut metadata: HashMap<&str, &Media> = HashMap::new();
    for m in ingest.questions.iter().flat_map(|q| &q.media) {
        if let Some(old) = metadata.get(m.id.as_str()) {
            if old.natural_w != m.natural_w || old.natural_h != m.natural_h {
                return Err(error(
                    "REFERENCE",
                    format!("conflicting dimensions: {}", m.id),
                ));
            }
        } else {
            metadata.insert(&m.id, m);
        }
    }
    let mut state = BuildState {
        doc,
        images,
        metadata,
        resources: HashMap::new(),
        bytes: 0,
        pixels: 0,
        instances: 0,
        limits,
        content_width,
        content_height,
        local_media: &[],
    };
    let mut passages = HashMap::new();
    for passage in &ingest.passages {
        if passages.insert(passage.id.as_str(), passage).is_some() {
            return Err(error(
                "REFERENCE",
                format!("duplicate passage id: {}", passage.id),
            ));
        }
    }
    let mut emitted = HashSet::new();
    for (qi, question) in ingest.questions.iter().enumerate() {
        state.local_media = &question.media;
        let mut ids = HashSet::new();
        for m in &question.media {
            if !ids.insert(m.id.as_str()) {
                return Err(error(
                    "REFERENCE",
                    format!("duplicate media id in question: {}", m.id),
                ));
            }
        }
        let mut output = Vec::new();
        if let Some(id) = question.passage_ref.as_deref() {
            let passage = passages
                .get(id)
                .ok_or_else(|| error("REFERENCE", format!("unknown passage: {id}")))?;
            if emitted.insert(id) {
                let mut no_prefix = None;
                state.container(
                    &passage.blocks,
                    styles.normal_para_shape_id,
                    styles.boxed_para_shape_id,
                    &mut no_prefix,
                    &mut output,
                    0,
                )?;
            }
        }
        let mut referenced = HashSet::new();
        collect_refs(&question.stem_blocks, &mut referenced);
        // Media used by shared passages must not also be inserted as question-only media.
        if let Some(id) = question.passage_ref.as_deref() {
            if let Some(passage) = passages.get(id) {
                collect_refs(&passage.blocks, &mut referenced);
            }
        }
        let mut before = Vec::new();
        let mut between = Vec::new();
        let mut after = Vec::new();
        let mut body = Vec::new();
        let mut inline_target = None;
        let mut prefix = question
            .auto_number
            .then(|| format!("{}. ", question.number));
        if question.stem_blocks.is_empty() {
            prefix = None;
            body.push(super::make_text_para_with_shape(
                &super::apply_number_prefix(question, &question.stem),
                styles.normal_para_shape_id,
            ));
            inline_target = Some(0);
        } else {
            for block in &question.stem_blocks {
                if let StemBlock::Image { ref_, placement } = block {
                    let p = state.image_paragraph(ref_, styles.normal_para_shape_id, question)?;
                    match placement {
                        Placement::Above => before.push(p),
                        Placement::Below => after.push(p),
                        Placement::Between => between.push(p),
                        Placement::Inline => {
                            append_inline_picture(&mut body, &mut inline_target, p)?;
                        }
                    }
                } else {
                    state.container(
                        std::slice::from_ref(block),
                        styles.normal_para_shape_id,
                        styles.boxed_para_shape_id,
                        &mut prefix,
                        &mut body,
                        0,
                    )?;
                    inline_target = if matches!(block, StemBlock::Boxed { .. }) {
                        None
                    } else {
                        body.len().checked_sub(1)
                    };
                }
            }
        }
        if let Some(prefix) = prefix {
            // An image-only stem still owns its question number. Do not attach
            // it to a shared passage or silently discard it.
            body.insert(
                0,
                super::make_text_para_with_shape(&prefix, styles.normal_para_shape_id),
            );
            inline_target = inline_target.map(|index| index + 1);
        }
        for media in &question.media {
            if !referenced.contains(media.id.as_str()) {
                let p = state.image_paragraph(&media.id, styles.normal_para_shape_id, question)?;
                match media.placement {
                    Placement::Above => before.push(p),
                    Placement::Below => after.push(p),
                    Placement::Between => between.push(p),
                    Placement::Inline => {
                        append_inline_picture(&mut body, &mut inline_target, p)?;
                    }
                }
            }
        }
        output.extend(before);
        output.extend(body);
        output.extend(between);
        for choice in &question.choices {
            output.push(super::make_text_para_with_shape(
                &format!("{} {}", choice.label, choice.text),
                styles.normal_para_shape_id,
            ));
        }
        output.extend(after);
        if qi + 1 < ingest.questions.len() {
            output.push(Paragraph::new_empty());
        }
        state.doc.sections[0].paragraphs.extend(output);
    }
    super::attach_exam_header_footer(&mut state.doc, ingest, &styles);
    Ok(state.doc)
}

fn collect_refs<'a>(blocks: &'a [StemBlock], out: &mut HashSet<&'a str>) {
    for block in blocks {
        match block {
            StemBlock::Image { ref_, .. } => {
                out.insert(ref_);
            }
            StemBlock::Paragraph { runs } => {
                for run in runs {
                    if let InlineRun::Image { ref_ } = run {
                        out.insert(ref_);
                    }
                }
            }
            StemBlock::Boxed { blocks, .. } => collect_refs(blocks, out),
            StemBlock::Text { .. } => {}
        }
    }
}

/// Append only to a paragraph owned by this container. A preceding boxed child
/// must never become the implicit host of its parent's following image.
fn append_inline_picture(
    body: &mut Vec<Paragraph>,
    target: &mut Option<usize>,
    image: Paragraph,
) -> Result<(), HwpError> {
    if let Some(index) = *target {
        let paragraph = body
            .get_mut(index)
            .ok_or_else(|| error("REFERENCE", "invalid inline paragraph owner"))?;
        if paragraph.controls.is_empty() {
            let mut stream = 0;
            paragraph.char_offsets = paragraph
                .text
                .chars()
                .map(|c| {
                    let position = stream;
                    stream += c.len_utf16() as u32;
                    position
                })
                .collect();
        }
        paragraph.char_count = paragraph
            .char_count
            .checked_add(8)
            .ok_or_else(|| error("LIMIT", "paragraph stream overflow"))?;
        paragraph.controls.extend(image.controls);
        paragraph.ctrl_data_records.push(None);
        paragraph.control_mask |= 0x00000800;
        paragraph.line_segs.clear();
    } else {
        *target = Some(body.len());
        body.push(image);
    }
    Ok(())
}

/// Stable asset references for hosts to resolve before calling the byte-only builder.
pub fn ingest_image_references(ingest: &IngestDocument) -> Vec<String> {
    let mut refs = HashSet::new();
    for q in &ingest.questions {
        if let Some(id) = q.passage_ref.as_deref() {
            if let Some(passage) = ingest.passages.iter().find(|p| p.id == id) {
                collect_refs(&passage.blocks, &mut refs);
            }
        }
        collect_refs(&q.stem_blocks, &mut refs);
        for m in &q.media {
            refs.insert(m.id.as_str());
        }
    }
    let mut refs: Vec<_> = refs.into_iter().map(str::to_owned).collect();
    refs.sort();
    refs
}

struct BuildState<'a> {
    doc: Document,
    images: &'a HashMap<String, IngestImage>,
    metadata: HashMap<&'a str, &'a Media>,
    resources: HashMap<[u8; 32], u16>,
    bytes: usize,
    pixels: u64,
    instances: usize,
    limits: IngestImageLimits,
    content_width: u32,
    content_height: u32,
    local_media: &'a [Media],
}

impl BuildState<'_> {
    fn picture(&mut self, id: &str, local: Option<&Media>) -> Result<Picture, HwpError> {
        let image = self
            .images
            .get(id)
            .ok_or_else(|| error("REFERENCE", format!("missing asset: {id}")))?;
        if image.bytes.len() > self.limits.max_image_bytes
            || u64::from(image.width) * u64::from(image.height) > self.limits.max_image_pixels
        {
            return Err(error(
                "LIMIT",
                format!("image exceeds builder limits: {id}"),
            ));
        }
        let metadata = local
            .or_else(|| self.local_media.iter().find(|m| m.id == id))
            .or_else(|| self.metadata.get(id).copied());
        if let Some(m) = metadata {
            if (m.natural_w, m.natural_h) != image.pixel_size() {
                return Err(error(
                    "SIZE",
                    format!("declared size differs from image: {id}"),
                ));
            }
        }
        let width = match metadata.and_then(|m| m.target_w_mm) {
            Some(mm) if mm.is_finite() && mm > 0.0 => f64::from(mm) * 7200.0 / 25.4,
            Some(_) => return Err(error("SIZE", format!("invalid target width: {id}"))),
            None => f64::from(self.content_width) * 0.7,
        };
        let height = width * f64::from(image.height) / f64::from(image.width);
        if width > f64::from(self.content_width)
            || height > f64::from(self.content_height)
            || width > f64::from(i32::MAX)
            || height > f64::from(i32::MAX)
            || width.round() < 1.0
            || height.round() < 1.0
        {
            return Err(error(
                "SIZE",
                format!("image does not fit the content frame: {id}"),
            ));
        }
        self.instances = self
            .instances
            .checked_add(1)
            .ok_or_else(|| error("LIMIT", "image count overflow"))?;
        if self.instances > self.limits.max_image_instances || self.instances > u32::MAX as usize {
            return Err(error("LIMIT", "image instance limit exceeded"));
        }
        let bin = if let Some(&id) = self.resources.get(&image.digest) {
            id
        } else {
            if self.resources.len() >= u16::MAX as usize {
                return Err(error("LIMIT", "BinData id limit exceeded"));
            }
            self.bytes = self
                .bytes
                .checked_add(image.bytes.len())
                .ok_or_else(|| error("LIMIT", "byte count overflow"))?;
            self.pixels = self
                .pixels
                .checked_add(u64::from(image.width) * u64::from(image.height))
                .ok_or_else(|| error("LIMIT", "pixel count overflow"))?;
            if self.bytes > self.limits.max_total_image_bytes
                || self.pixels > self.limits.max_total_image_pixels
            {
                return Err(error("LIMIT", "document image budget exceeded"));
            }
            let id = crate::model::bin_data::register_embedded_bin_data(
                &mut self.doc,
                Arc::clone(&image.bytes),
                image.extension,
            );
            self.resources.insert(image.digest, id);
            id
        };
        let mut common = CommonObjAttr {
            ctrl_id: 0x67736F20,
            numbering_type: ObjectNumberingType::Picture,
            treat_as_char: true,
            vert_rel_to: VertRelTo::Para,
            horz_rel_to: HorzRelTo::Para,
            text_wrap: TextWrap::TopAndBottom,
            flow_with_text: true,
            width: width.round() as u32,
            height: height.round() as u32,
            instance_id: self.instances as u32,
            description: id.to_owned(),
            ..Default::default()
        };
        common.attr = crate::serializer::control::pack_common_attr_bits(&common);
        let mut picture = Picture::from_embedded_image(common, bin, image.width, image.height);
        picture.instance_id = self.instances as u32;
        Ok(picture)
    }

    fn paragraph(
        &mut self,
        runs: &[InlineRun],
        shape: u16,
        prefix: &str,
    ) -> Result<Paragraph, HwpError> {
        let mut p = Paragraph::new_empty();
        p.para_shape_id = shape;
        p.has_para_text = true;
        p.char_shapes = vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }];
        p.text.clear();
        p.char_offsets.clear();
        p.line_segs.clear();
        let mut stream = 0u32;
        let push_text = |text: &str, p: &mut Paragraph, stream: &mut u32| {
            for c in text.chars() {
                p.char_offsets.push(*stream);
                p.text.push(c);
                *stream += c.len_utf16() as u32;
            }
        };
        push_text(prefix, &mut p, &mut stream);
        for run in runs {
            match run {
                InlineRun::Text { text } => push_text(text, &mut p, &mut stream),
                InlineRun::Image { ref_ } => {
                    let picture = self.picture(ref_, None)?;
                    p.controls.push(Control::Picture(Box::new(picture)));
                    p.ctrl_data_records.push(None);
                    stream = stream
                        .checked_add(8)
                        .ok_or_else(|| error("LIMIT", "paragraph stream overflow"))?;
                }
            }
        }
        p.char_count = stream
            .checked_add(1)
            .ok_or_else(|| error("LIMIT", "paragraph stream overflow"))?;
        if !p.controls.is_empty() {
            p.control_mask |= 0x00000800;
        }
        Ok(p)
    }

    fn image_paragraph(
        &mut self,
        id: &str,
        shape: u16,
        question: &crate::parser::ingest::schema::Question,
    ) -> Result<Paragraph, HwpError> {
        let picture = self.picture(id, question.media.iter().find(|m| m.id == id))?;
        let mut p = Paragraph::new_empty();
        p.para_shape_id = shape;
        p.has_para_text = true;
        p.char_count = 9;
        p.control_mask |= 0x00000800;
        p.line_segs.clear();
        p.controls.push(Control::Picture(Box::new(picture)));
        p.ctrl_data_records.push(None);
        Ok(p)
    }

    fn container(
        &mut self,
        blocks: &[StemBlock],
        shape: u16,
        boxed_shape: u16,
        prefix: &mut Option<String>,
        out: &mut Vec<Paragraph>,
        depth: usize,
    ) -> Result<(), HwpError> {
        if depth > 64 {
            return Err(error("LIMIT", "block nesting limit exceeded"));
        }
        let mut before = Vec::new();
        let mut body: Vec<Paragraph> = Vec::new();
        let mut after = Vec::new();
        let mut inline_target = None;
        for block in blocks {
            match block {
                StemBlock::Text { text } => {
                    let text = format!("{}{}", prefix.take().unwrap_or_default(), text);
                    body.push(super::make_text_para_with_shape(&text, shape));
                    inline_target = body.len().checked_sub(1);
                }
                StemBlock::Paragraph { runs } => {
                    body.push(self.paragraph(runs, shape, &prefix.take().unwrap_or_default())?);
                    inline_target = body.len().checked_sub(1);
                }
                StemBlock::Image { ref_, placement } => {
                    let p =
                        self.paragraph(&[InlineRun::Image { ref_: ref_.clone() }], shape, "")?;
                    match placement {
                        Placement::Above => before.push(p),
                        Placement::Between | Placement::Below => after.push(p),
                        Placement::Inline => {
                            append_inline_picture(&mut body, &mut inline_target, p)?;
                        }
                    }
                }
                StemBlock::Boxed { title, blocks } => {
                    if let Some(title) = title.as_deref().filter(|s| !s.trim().is_empty()) {
                        body.push(super::make_text_para_with_shape(title, boxed_shape));
                    }
                    self.container(
                        blocks,
                        boxed_shape,
                        boxed_shape,
                        prefix,
                        &mut body,
                        depth + 1,
                    )?;
                    inline_target = None;
                }
            }
        }
        out.extend(before);
        out.extend(body);
        out.extend(after);
        Ok(())
    }
}
