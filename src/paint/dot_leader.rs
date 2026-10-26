//! 실제 공급 SFNT의 점과 리더 셀을 공통 Path로 해소한다.

use super::metafile_fonts::MetafileFonts;
use crate::renderer::render_tree::{
    BoundingBox, PathNode, RenderNode, RenderNodeType, TextRunNode,
};
use crate::renderer::{PathCommand, ShapeStyle};

pub(crate) fn needs_font_program(node: &RenderNode) -> bool {
    matches!(&node.node_type, RenderNodeType::TextRun(run)
        if eligible(run) && run.style.tab_leaders.iter().any(|leader| leader.fill_type == 3))
        || node.children.iter().any(needs_font_program)
}

fn eligible(run: &TextRunNode) -> bool {
    // HFT 원 윤곽·회전·세로 쓰기는 별도의 기존 소비 계약을 유지한다.
    !run.is_vertical
        && run.rotation == 0.0
        && run.char_overlap.is_none()
        && !run.style.source_font_is_hft
}

/// 성공한 장식만 원 TabLeader에서 제거한다. 원문·전진·소유는 보존한다.
pub(super) fn prepare(
    fonts: &MetafileFonts,
    bbox: BoundingBox,
    run: &mut TextRunNode,
) -> Vec<PathNode> {
    if !eligible(run)
        || !run
            .style
            .tab_leaders
            .iter()
            .any(|leader| leader.fill_type == 3)
    {
        return Vec::new();
    }
    let database = fonts.database();
    for id in fonts.supplied_faces_for_style(&run.style) {
        let Some(info) = database.face(id) else {
            continue;
        };
        // 실제 프로그램에 없는 합성 굵기·기울임을 윤곽 변경으로 누락시키지 않는다.
        if (run.style.bold && info.weight.0 < 600)
            || (run.style.italic && info.style == resvg::usvg::fontdb::Style::Normal)
        {
            continue;
        }
        let paths = database
            .with_face_data(id, |bytes, index| {
                let face = ttf_parser::Face::parse(bytes, index).ok()?;
                // variation 좌표가 없는 기본 face로 다른 instance를 대체하지 않는다.
                if face.is_variable() {
                    return None;
                }
                let glyph = face.glyph_index('·').filter(|glyph| glyph.0 != 0)?;
                paths_for(&face, glyph, bbox, run)
            })
            .flatten();
        if let Some(paths) = paths {
            run.style.tab_leaders.retain(|leader| leader.fill_type != 3);
            return paths;
        }
    }
    Vec::new()
}

fn paths_for(
    face: &ttf_parser::Face<'_>,
    glyph: ttf_parser::GlyphId,
    bbox: BoundingBox,
    run: &TextRunNode,
) -> Option<Vec<PathNode>> {
    let mut pen = OutlinePen::default();
    face.outline_glyph(glyph, &mut pen)?;
    if pen.commands.is_empty() || pen.exceeded {
        return None;
    }
    let (size, baseline) = run
        .style
        .script_draw_metrics(run.style.font_size, bbox.y + run.baseline);
    // 장평 80·100·120% 독립 Print: 장평은 윤곽에 적용하고 셀의 명목 em은 유지한다.
    let cell = size * 0.25;
    let ratio = run.style.ratio;
    if !cell.is_finite() || cell <= 0.0 || !ratio.is_finite() || ratio <= 0.0 {
        return None;
    }
    let scale = size / f64::from(face.units_per_em());
    let mut paths = Vec::new();
    let mut total = 0usize;
    for leader in run
        .style
        .tab_leaders
        .iter()
        .filter(|leader| leader.fill_type == 3)
    {
        let span = leader.end_x - leader.start_x;
        if !span.is_finite() || span < 0.0 || !leader.start_x.is_finite() {
            return None;
        }
        let count = (span / cell).floor();
        if count > super::MAX_PORTABLE_GLYPHS_PER_RUN as f64 {
            return None;
        }
        let count = count as usize;
        if count == 0 {
            continue;
        }
        let command_count = count.checked_mul(pen.commands.len())?;
        total = total.checked_add(command_count)?;
        if total > super::shaping_glyph::MAX_HORIZONTAL_SHAPING_OUTLINE_COMMANDS_PER_RUN {
            return None;
        }
        // 독립 한컴 저장/Print: 마지막 셀을 보존하고 잔여를 셀 사이에 배분한다.
        let pitch = if count > 1 {
            (span - cell) / (count - 1) as f64
        } else {
            cell
        };
        let mut commands = Vec::with_capacity(command_count);
        for index in 0..count {
            let origin = bbox.x + leader.start_x + index as f64 * pitch;
            for command in &pen.commands {
                let point = |x: f64, y: f64| (origin + x * scale * ratio, baseline - y * scale);
                commands.push(match *command {
                    PathCommand::MoveTo(x, y) => {
                        let (x, y) = point(x, y);
                        PathCommand::MoveTo(x, y)
                    }
                    PathCommand::LineTo(x, y) => {
                        let (x, y) = point(x, y);
                        PathCommand::LineTo(x, y)
                    }
                    PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                        let (x1, y1) = point(x1, y1);
                        let (x2, y2) = point(x2, y2);
                        let (x, y) = point(x, y);
                        PathCommand::CurveTo(x1, y1, x2, y2, x, y)
                    }
                    PathCommand::ClosePath => PathCommand::ClosePath,
                    _ => return None,
                });
            }
        }
        let mut path = PathNode::new(
            commands,
            ShapeStyle {
                fill_color: Some(run.style.color),
                ..Default::default()
            },
            None,
        );
        path.section_index = run.section_index;
        path.para_index = run.para_index;
        if let Some(context) = &run.cell_context {
            // 완전한 셀 소유·clip은 같은 원 TextRun leaf에 남는다.
            // Path의 기존 단일 셀 메타데이터도 바깥 문단과 안쪽 셀을 구분한다.
            path.para_index = Some(context.parent_para_index);
            path.cell_para_index = run.para_index;
            if let Some(cell) = context.path.last() {
                path.cell_index = Some(cell.cell_index);
                path.outer_table_control_index = Some(cell.control_index);
            }
        }
        paths.push(path);
    }
    Some(paths)
}

/// OpenType quadratic를 같은 곡선의 cubic으로 변환한다. 좌표 맞춤 계수는 없다.
#[derive(Default)]
struct OutlinePen {
    commands: Vec<PathCommand>,
    current: (f64, f64),
    exceeded: bool,
}
impl OutlinePen {
    fn push(&mut self, command: PathCommand) {
        if self.commands.len() < super::MAX_POSITIONED_CONTROL_MARKS_PER_RUN {
            self.commands.push(command);
        } else {
            self.exceeded = true;
        }
    }
}
impl ttf_parser::OutlineBuilder for OutlinePen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.current = (f64::from(x), f64::from(y));
        self.push(PathCommand::MoveTo(self.current.0, self.current.1));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.current = (f64::from(x), f64::from(y));
        self.push(PathCommand::LineTo(self.current.0, self.current.1));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let q = (f64::from(x1), f64::from(y1));
        let p = (f64::from(x), f64::from(y));
        self.push(PathCommand::CurveTo(
            self.current.0 + (q.0 - self.current.0) * 2.0 / 3.0,
            self.current.1 + (q.1 - self.current.1) * 2.0 / 3.0,
            p.0 + (q.0 - p.0) * 2.0 / 3.0,
            p.1 + (q.1 - p.1) * 2.0 / 3.0,
            p.0,
            p.1,
        ));
        self.current = p;
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.push(PathCommand::CurveTo(
            f64::from(x1),
            f64::from(y1),
            f64::from(x2),
            f64::from(y2),
            f64::from(x),
            f64::from(y),
        ));
        self.current = (f64::from(x), f64::from(y));
    }
    fn close(&mut self) {
        self.push(PathCommand::ClosePath);
    }
}
