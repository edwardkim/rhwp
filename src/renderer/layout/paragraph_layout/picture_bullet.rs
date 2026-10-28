//! 그림 글머리표는 문자 대체 대신 원본 BinData와 같은 문단 소속을 그린다.

use crate::model::{bin_data::BinDataContent, image::ImageEffect, style::HeadType};
use crate::renderer::{
    layout::{utils::find_bin_data_bytes, CellContext},
    render_tree::ImageNode,
    style_resolver::ResolvedStyleSet,
};

pub(super) fn image(
    styles: &ResolvedStyleSet,
    para_style_id: usize,
    data: Option<&[BinDataContent]>,
    section_index: usize,
    para_index: usize,
    cell: Option<&CellContext>,
) -> Option<ImageNode> {
    let style = styles.para_styles.get(para_style_id)?;
    if style.head_type != HeadType::Bullet {
        return None;
    }
    let bullet = styles
        .bullets
        .get(usize::from(style.numbering_id.checked_sub(1)?))?;
    let id = u16::try_from(bullet.image_bullet)
        .ok()
        .filter(|id| *id > 0)?;
    let bytes = find_bin_data_bytes(data?, id)?;
    let mut image = ImageNode::new(id, Some(bytes));
    image.section_index = Some(section_index);
    image.para_index = Some(cell.map_or(para_index, |context| context.parent_para_index));
    image.cell_context = cell.cloned();
    if let Some(context) = cell {
        let (cell_index, cell_para_index, outer_table_control_index) = context.last_image_indices();
        image.cell_index = cell_index;
        image.cell_para_index = cell_para_index;
        image.outer_table_control_index = outer_table_control_index;
    }
    // 문단 머리의 그림은 독립 그림 컨트롤이 아니다. control_index는 만들지 않는다.
    image.contrast = bullet.image_data[0] as i8;
    image.brightness = bullet.image_data[1] as i8;
    image.effect = match bullet.image_data[2] {
        1 => ImageEffect::GrayScale,
        2 => ImageEffect::BlackWhite,
        3 => ImageEffect::Pattern8x8,
        _ => ImageEffect::RealPic,
    };
    Some(image)
}
