//! Editable top-level pictures backed by the document's picture commands.

use std::io::Cursor;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::document_core::native_interaction::NativeInteractionBounds;
use crate::model::{
    control::Control,
    image::Picture,
    shape::{ShapeObject, SizeCriterion},
};
use crate::renderer::{
    page_layout::PageLayoutInfo,
    px_to_hwpunit,
    render_tree::{BoundingBox, RenderNode, RenderNodeType},
};
use crate::{DocumentCore, HwpError};

use super::editor::{NativeEditOutcome, NativeEditor};
use super::{NativeDocumentSession, NativeHostError};

pub const MAX_PICTURE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_PICTURE_SIZE: u32 = 200_000;
const MAX_IMAGE_AXIS: u32 = 16_384;
const MAX_IMAGE_PIXELS: u64 = 32 * 1024 * 1024;
const HWPUNIT_PER_PIXEL: u32 = 75;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePictureObjectAddress {
    pub section: u32,
    pub paragraph: u32,
    pub control: u32,
}

impl NativePictureObjectAddress {
    fn indices(self) -> (usize, usize, usize) {
        (
            self.section as usize,
            self.paragraph as usize,
            self.control as usize,
        )
    }

    fn resolve(self, core: &DocumentCore) -> Option<&Picture> {
        let (section, paragraph, control) = self.indices();
        match core
            .document()
            .sections
            .get(section)?
            .paragraphs
            .get(paragraph)?
            .controls
            .get(control)?
        {
            Control::Picture(picture) => Some(picture),
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Picture(picture) => Some(picture),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePictureObjectProperties {
    pub width: u32,
    pub height: u32,
    pub original_width: u32,
    pub original_height: u32,
    pub treat_as_char: bool,
    pub size_protect: bool,
    pub locked: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePictureObject {
    pub address: NativePictureObjectAddress,
    pub bounds: NativeInteractionBounds,
    pub properties: NativePictureObjectProperties,
    pub can_resize: bool,
}

impl NativeDocumentSession {
    pub fn picture_object_at(
        &self,
        revision: u64,
        page: u32,
        x: f64,
        y: f64,
    ) -> Result<Option<NativePictureObject>, NativeHostError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(NativeHostError::InvalidArgument(
                "picture hit coordinates must be finite".into(),
            ));
        }
        let point = BoundingBox::new(x, y, 0.0, 0.0);
        self.find_picture_object(revision, page, |_, bounds| bounds.contains(&point))
    }

    pub fn picture_object(
        &self,
        revision: u64,
        page: u32,
        requested: NativePictureObjectAddress,
    ) -> Result<Option<NativePictureObject>, NativeHostError> {
        self.find_picture_object(revision, page, |address, _| address == requested)
    }

    fn find_picture_object(
        &self,
        revision: u64,
        page: u32,
        matches: impl Fn(NativePictureObjectAddress, BoundingBox) -> bool,
    ) -> Result<Option<NativePictureObject>, NativeHostError> {
        self.check_revision(revision)?;
        let core = self.document();
        if page >= core.page_count() {
            return Err(NativeHostError::InvalidArgument(
                "picture bounds page is outside the document".into(),
            ));
        }
        Ok(core.with_page_tree_cached(page, |tree| Ok(find_picture(core, &tree.root, &matches)))?)
    }
}

fn picture_address(node: &RenderNode) -> Option<NativePictureObjectAddress> {
    let (section, paragraph, control) = match &node.node_type {
        RenderNodeType::Image(image)
            if image.fill_mode.is_none()
                && image.header_footer_ref.is_none()
                && image.cell_context.is_none()
                && image.cell_index.is_none() =>
        {
            (
                image.section_index?,
                image.para_index?,
                image.control_index?,
            )
        }
        RenderNodeType::Placeholder(placeholder) if placeholder.cell_context.is_none() => {
            let reference = placeholder
                .control_ref
                .as_ref()
                .filter(|r| r.kind == "picture")?;
            (
                reference.section_index,
                reference.para_index,
                reference.control_index,
            )
        }
        _ => return None,
    };
    Some(NativePictureObjectAddress {
        section: u32::try_from(section).ok()?,
        paragraph: u32::try_from(paragraph).ok()?,
        control: u32::try_from(control).ok()?,
    })
}

fn find_picture(
    core: &DocumentCore,
    node: &RenderNode,
    matches: &impl Fn(NativePictureObjectAddress, BoundingBox) -> bool,
) -> Option<NativePictureObject> {
    if node.node_type.is_page_area()
        || matches!(
            node.node_type,
            RenderNodeType::FootnoteArea | RenderNodeType::Table(_)
        )
    {
        return None;
    }
    if let Some(hit) = node
        .children
        .iter()
        .rev()
        .find_map(|child| find_picture(core, child, matches))
    {
        return Some(hit);
    }
    let address = picture_address(node)?;
    let bounds = node.bbox;
    if bounds.width <= 0.0 || bounds.height <= 0.0 || !matches(address, bounds) {
        return None;
    }
    let picture = address.resolve(core)?;
    Some(NativePictureObject {
        address,
        bounds: NativeInteractionBounds {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        },
        properties: NativePictureObjectProperties {
            width: picture.common.width,
            height: picture.common.height,
            original_width: picture.shape_attr.original_width,
            original_height: picture.shape_attr.original_height,
            treat_as_char: picture.common.treat_as_char,
            size_protect: picture.common.size_protect,
            locked: picture.common.locked || picture.lock,
        },
        can_resize: can_resize(picture),
    })
}

fn can_resize(picture: &Picture) -> bool {
    !picture.common.locked
        && !picture.lock
        && !picture.common.size_protect
        && picture.common.width_criterion == SizeCriterion::Absolute
        && picture.common.height_criterion == SizeCriterion::Absolute
}

impl NativeEditor {
    /// Insert original encoded bytes at a collapsed body caret as one undoable operation.
    pub fn insert_picture(
        &mut self,
        revision: u64,
        bytes: &[u8],
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.check_edit_ready(revision)?;
        let (extension, natural_width, natural_height) = validate_image(bytes)?;
        self.insert_display_object(revision, "insertPicture", |core, section, paragraph| {
            let model = &core.document().sections[section];
            let columns = DocumentCore::find_column_def_for_paragraph(&model.paragraphs, paragraph);
            let layout =
                PageLayoutInfo::from_page_def(&model.section_def.page_def, &columns, core.dpi);
            let content_width = core
                .get_column_area_for_paragraph(section, paragraph)
                .map_or(layout.body_area.width, |(_, _, width)| width);
            let natural_width_hu = f64::from(natural_width * HWPUNIT_PER_PIXEL);
            let natural_height_hu = f64::from(natural_height * HWPUNIT_PER_PIXEL);
            let max_width =
                px_to_hwpunit(content_width, core.dpi).clamp(1, MAX_PICTURE_SIZE as i32);
            let max_height =
                px_to_hwpunit(layout.body_area.height, core.dpi).clamp(1, MAX_PICTURE_SIZE as i32);
            let scale = (f64::from(max_width) / natural_width_hu)
                .min(f64::from(max_height) / natural_height_hu)
                .min(1.0);
            let width = (natural_width_hu * scale).round().max(1.0) as u32;
            let height = (natural_height_hu * scale).round().max(1.0) as u32;
            let inserted = core.insert_picture_native(
                section,
                paragraph,
                0,
                &[],
                bytes,
                width,
                height,
                natural_width,
                natural_height,
                extension,
                "",
                None,
                None,
            )?;
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct InsertedPicture {
                control_idx: usize,
            }
            let result: InsertedPicture = serde_json::from_str(&inserted)
                .map_err(|error| HwpError::RenderError(error.to_string()))?;
            // The core insertion defaults to floating on the paper. Inline ownership
            // places the picture at the selected paragraph in the text flow.
            core.set_picture_properties_native(
                section,
                paragraph,
                result.control_idx,
                r#"{"treatAsChar":true}"#,
            )?;
            Ok(())
        })
    }

    pub(super) fn resize_picture_object(
        &mut self,
        revision: u64,
        address: NativePictureObjectAddress,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let picture = address.resolve(self.session().document()).ok_or_else(|| {
            NativeHostError::InvalidArgument("picture address is outside the document".into())
        })?;
        if !can_resize(picture) {
            return Err(NativeHostError::Unsupported(
                "picture resizing requires an unprotected picture with absolute dimensions".into(),
            ));
        }
        if [width, height]
            .into_iter()
            .flatten()
            .any(|v| !(1..=MAX_PICTURE_SIZE).contains(&v))
        {
            return Err(NativeHostError::InvalidArgument(format!(
                "picture dimensions must be 1..={MAX_PICTURE_SIZE} HWPUNIT"
            )));
        }
        let width = width.filter(|v| *v != picture.common.width);
        let height = height.filter(|v| *v != picture.common.height);
        if width.is_none() && height.is_none() {
            return Ok(self.outcome(false, false));
        }
        let (section, paragraph, control) = address.indices();
        let selection = self.selection().clone();
        let properties = json!({"width": width, "height": height}).to_string();
        self.edit_explicit_ranges_transaction(revision, |core| {
            core.set_picture_properties_native(section, paragraph, control, &properties)?;
            Ok(selection)
        })
    }
}

fn validate_image(bytes: &[u8]) -> Result<(&'static str, u32, u32), NativeHostError> {
    if bytes.len() > MAX_PICTURE_BYTES {
        return Err(NativeHostError::LimitExceeded(format!(
            "picture data exceeds {MAX_PICTURE_BYTES} bytes"
        )));
    }
    let invalid_format =
        || NativeHostError::InvalidArgument("picture must be a PNG or JPEG".into());
    let format = image::guess_format(bytes).map_err(|_| invalid_format())?;
    let extension = match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        _ => return Err(invalid_format()),
    };
    let invalid_data =
        |_| NativeHostError::InvalidArgument("picture data cannot be decoded".into());
    let reader = ImageReader::with_format(Cursor::new(bytes), format);
    let decoder = reader.into_decoder().map_err(invalid_data)?;
    let (width, height) = decoder.dimensions();
    if width == 0
        || height == 0
        || width > MAX_IMAGE_AXIS
        || height > MAX_IMAGE_AXIS
        || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS
    {
        return Err(NativeHostError::LimitExceeded(
            "picture pixel dimensions are outside the supported range".into(),
        ));
    }
    DynamicImage::from_decoder(decoder).map_err(invalid_data)?;
    Ok((extension, width, height))
}
