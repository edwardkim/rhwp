//! 한컴 Print의 문서 속성 기반 자동 바닥글. 문서 꼬리말과 독립적이다.

use super::{LayerNode, PaintOp};
use crate::renderer::{
    font_metrics_data::find_metric,
    render_tree::{BoundingBox, TextRunNode},
    TextStyle,
};

/// 인쇄 장의 본문 좌우 여백과 종이 하단 여백을 기준으로 표시한다.
pub(crate) struct AutomaticPrintFooter<'a> {
    pub(crate) file_name: &'a str,
    pub(crate) local_date_time: String,
    pub(crate) page_number: u32,
    pub(crate) page_count: u32,
    pub(crate) left: f64,
    pub(crate) right: f64,
    pub(crate) bottom_margin_top: f64,
}

impl AutomaticPrintFooter<'_> {
    pub(crate) fn layer(&self, paper_width: f64, layout_dpi: f64) -> Option<LayerNode> {
        let face = "Haansoft Batang";
        let metric = find_metric(face, false, false)?.metric;
        let device_px = layout_dpi / super::metafile::HANCOM_2024_PRINT_DEVICE_DPI;
        let snap = |value: f64| (value / device_px).round() * device_px;
        let em = snap(9.0 * layout_dpi / 72.0);
        // HBATANG.TTF의 hhea ascent. 한컴 Print는 기준선과 글리프 전진을
        // 600 DPI의 장치 정수 좌표에서 확정한다.
        let ascent = snap(em * 879.0 / 1024.0);
        let page_label = format!("{}쪽 중 {}쪽", self.page_count, self.page_number);
        let labels = [
            (self.file_name, snap(self.left), 0.0),
            (&self.local_date_time, snap(paper_width / 2.0), 0.5),
            (&page_label, snap(self.right), 1.0),
        ];
        let y = snap(self.bottom_margin_top);
        let mut ops = Vec::with_capacity(labels.len());
        for (text, anchor, alignment) in labels {
            let mut positions = Vec::with_capacity(text.chars().count() + 1);
            positions.push(0.0);
            let mut width = 0.0;
            for ch in text.chars() {
                let advance = metric.get_width(ch).unwrap_or(metric.em_size);
                width += snap(f64::from(advance) * em / f64::from(metric.em_size));
                positions.push(width);
            }
            let bbox = BoundingBox::new(snap(anchor - width * alignment), y, width, em);
            let run = TextRunNode {
                text: text.to_owned(),
                style: TextStyle {
                    font_family: face.to_owned(),
                    font_size: em,
                    resolved_font_size: true,
                    source_font_is_ttf: true,
                    ..Default::default()
                },
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: false,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: ascent,
                field_marker: Default::default(),
                layout_positions: Some(positions),
                display_text: None,
            };
            ops.push(PaintOp::text_run(bbox, run));
        }
        Some(LayerNode::leaf(
            BoundingBox::new(self.left, y, self.right - self.left, em),
            None,
            ops,
        ))
    }
}
