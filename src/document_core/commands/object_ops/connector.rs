//! 커넥터 라우팅 native 명령 (object_ops 분할, #1904).

use super::MIN_SHAPE_SIZE;
use crate::document_core::helpers::{get_textbox_from_shape, get_textbox_from_shape_mut};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, ShapeObject};

impl DocumentCore {
    /// 연결선의 SubjectID를 갱신한다 (연결선 생성 후 호출)
    pub fn update_connector_subject_ids(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        start_subject_id: u32,
        start_subject_index: u32,
        end_subject_id: u32,
        end_subject_index: u32,
    ) {
        let mut mutated = false;
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            if let Some(para) = section.paragraphs.get_mut(para_idx) {
                if let Some(Control::Shape(ref mut shape)) = para.controls.get_mut(control_idx) {
                    if let ShapeObject::Line(ref mut line) = shape.as_mut() {
                        if let Some(ref mut conn) = line.connector {
                            conn.start_subject_id = start_subject_id;
                            conn.start_subject_index = start_subject_index;
                            conn.end_subject_id = end_subject_id;
                            conn.end_subject_index = end_subject_index;
                            mutated = true;
                        }
                    }
                }
            }
            // [#2698] IR을 실제로 바꾼 경우에만 구역 패스스루를 무효화한다.
            // 무효화하지 않으면 serialize_section 이 원본 raw_stream 을 그대로
            // 반환해(body_text.rs:26-30) 이 편집이 저장 결과에서 사라진다.
            // 인덱스가 빗나가 아무것도 바꾸지 않은 경로에서는 라운드트립을
            // 깨뜨리지 않기 위해 조건부로 둔다.
            if mutated {
                section.raw_stream = None;
            }
        }
    }
    /// 연결선 제어점을 연결점 방향에 따라 재계산한다.
    /// start_idx/end_idx: 0=상, 1=우, 2=하, 3=좌
    pub fn recalculate_connector_routing(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        start_idx: u32,
        end_idx: u32,
    ) {
        use crate::model::shape::ConnectorControlPoint;

        let mut routed = false;
        let section = match self.document.sections.get_mut(section_idx) {
            Some(s) => s,
            None => return,
        };
        let para = match section.paragraphs.get_mut(para_idx) {
            Some(p) => p,
            None => return,
        };
        let ctrl = match para.controls.get_mut(control_idx) {
            Some(c) => c,
            None => return,
        };

        let line = match ctrl {
            Control::Shape(ref mut s) => match s.as_mut() {
                ShapeObject::Line(ref mut l) => l,
                _ => return,
            },
            _ => return,
        };

        let conn = match &mut line.connector {
            Some(c) => c,
            None => return,
        };

        let sx = line.start.x;
        let sy = line.start.y;
        let ex = line.end.x;
        let ey = line.end.y;
        let w = line.common.width as i32;
        let h = line.common.height as i32;

        // 직선 연결선: 제어점 불필요
        if !conn.link_type.is_stroke() && !conn.link_type.is_arc() {
            conn.control_points.clear();
            // [#2698] 이 조기 반환 경로도 제어점 목록을 비우는 실제 IR 변경이므로
            // 아래 공통 무효화를 타지 못하는 만큼 여기서 직접 무효화한다.
            section.raw_stream = None;
            return;
        }

        // 연결점 방향: 0=상, 1=우, 2=하, 3=좌
        if conn.link_type.is_arc() {
            // ─── 곡선 연결선: 파워포인트 스타일 S곡선 ───
            // ctrl1: 시작점에서 시작 방향으로 중간지점까지 뻗음
            // ctrl2: 끝점에서 끝 방향으로 중간지점까지 뻗음
            // → 중간지점에서 위아래(또는 좌우)가 반전되는 S자
            // 한컴 공식: 수평 연결(우/좌)은 midX 기준, 수직 연결(상/하)은 midY 기준
            // ctrl1 = (midX, startY) / (startX, midY), ctrl2 = (midX, endY) / (endX, midY)
            let mid_x = (sx + ex) / 2;
            let mid_y = (sy + ey) / 2;
            let start_is_horz = start_idx == 1 || start_idx == 3; // 우/좌
            let end_is_horz = end_idx == 1 || end_idx == 3;

            let (c1x, c1y, c2x, c2y) = if start_is_horz && end_is_horz {
                // 우↔좌: midX 기준 S곡선
                (mid_x, sy, mid_x, ey)
            } else if !start_is_horz && !end_is_horz {
                // 상↔하: midY 기준 S곡선
                (sx, mid_y, ex, mid_y)
            } else if start_is_horz {
                // 우/좌 → 상/하: 수평 출발 → midX까지, 수직 진입 → midY까지
                (mid_x, sy, ex, mid_y)
            } else {
                // 상/하 → 우/좌: 수직 출발 → midY까지, 수평 진입 → midX까지
                (sx, mid_y, mid_x, ey)
            };

            conn.control_points = vec![
                ConnectorControlPoint {
                    x: sx,
                    y: sy,
                    point_type: 3,
                }, // 시작 앵커
                ConnectorControlPoint {
                    x: c1x,
                    y: c1y,
                    point_type: 2,
                }, // 베지어 ctrl1
                ConnectorControlPoint {
                    x: c2x,
                    y: c2y,
                    point_type: 2,
                }, // 베지어 ctrl2
                ConnectorControlPoint {
                    x: ex,
                    y: ey,
                    point_type: 26,
                }, // 끝 앵커
            ];
            routed = true;
        } else {
            // ─── 꺽인 연결선: 직각 꺾임점 ───
            let mut pts = Vec::new();
            pts.push(ConnectorControlPoint {
                x: sx,
                y: sy,
                point_type: 3,
            });

            match (start_idx, end_idx) {
                (1, 3) | (3, 1) => {
                    let mid_x = (sx + ex) / 2;
                    pts.push(ConnectorControlPoint {
                        x: mid_x,
                        y: sy,
                        point_type: 2,
                    });
                    pts.push(ConnectorControlPoint {
                        x: mid_x,
                        y: ey,
                        point_type: 2,
                    });
                }
                (2, 0) | (0, 2) => {
                    let mid_y = (sy + ey) / 2;
                    pts.push(ConnectorControlPoint {
                        x: sx,
                        y: mid_y,
                        point_type: 2,
                    });
                    pts.push(ConnectorControlPoint {
                        x: ex,
                        y: mid_y,
                        point_type: 2,
                    });
                }
                (1, 0) | (1, 2) | (3, 0) | (3, 2) => {
                    pts.push(ConnectorControlPoint {
                        x: ex,
                        y: sy,
                        point_type: 2,
                    });
                }
                (0, 1) | (0, 3) | (2, 1) | (2, 3) => {
                    pts.push(ConnectorControlPoint {
                        x: sx,
                        y: ey,
                        point_type: 2,
                    });
                }
                _ => {
                    let mid_x = (sx + ex) / 2;
                    pts.push(ConnectorControlPoint {
                        x: mid_x,
                        y: sy,
                        point_type: 2,
                    });
                    pts.push(ConnectorControlPoint {
                        x: mid_x,
                        y: ey,
                        point_type: 2,
                    });
                }
            }

            pts.push(ConnectorControlPoint {
                x: ex,
                y: ey,
                point_type: 26,
            });
            conn.control_points = pts;
            routed = true;
        }
        // [#2698] 제어점을 실제로 재구성한 경우에만 구역 패스스루를 무효화한다.
        if routed {
            section.raw_stream = None;
        }
    }
    /// 구역 내 모든 연결선을 스캔하여 연결된 도형의 현재 위치에 맞게 갱신한다.
    pub fn update_connectors_in_section(&mut self, section_idx: usize) {
        let section = match self.document.sections.get(section_idx) {
            Some(s) => s,
            None => return,
        };

        // 1) SC inst_id → 연결점 좌표 맵 구축 (SubjectID = drawing.inst_id)
        let mut conn_points: std::collections::HashMap<u32, [(i32, i32); 4]> =
            std::collections::HashMap::new();
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                let (common, inst_id, _is_line) = match ctrl {
                    Control::Shape(s) => {
                        let sc_inst = s.drawing().map(|d| d.inst_id).unwrap_or(0);
                        (
                            s.common(),
                            sc_inst,
                            matches!(s.as_ref(), ShapeObject::Line(_)),
                        )
                    }
                    Control::Picture(p) => (&p.common, 0u32, false),
                    _ => continue,
                };
                if _is_line {
                    continue;
                }
                let x = common.horizontal_offset as i32;
                let y = common.vertical_offset as i32;
                let w = common.width as i32;
                let h = common.height as i32;
                let cx = x + w / 2;
                let cy = y + h / 2;
                let pts = [(cx, y), (x + w, cy), (cx, y + h), (x, cy)];
                // SC inst_id (= SubjectID) 등록
                if inst_id != 0 {
                    conn_points.insert(inst_id, pts);
                }
                // CTRL_HEADER instance_id로도 등록 (폴백)
                if common.instance_id != 0 {
                    conn_points.insert(common.instance_id, pts);
                    conn_points.insert((common.instance_id & 0x3FFFFFFF) + 1, pts);
                }
            }
        }

        // 2) 커넥터 찾기 및 좌표 갱신
        let mut geometry_updated = false;
        let section = match self.document.sections.get_mut(section_idx) {
            Some(s) => s,
            None => return,
        };
        for para in &mut section.paragraphs {
            for ctrl in &mut para.controls {
                let line = match ctrl {
                    Control::Shape(ref mut s) => match s.as_mut() {
                        ShapeObject::Line(ref mut l) if l.connector.is_some() => l,
                        _ => continue,
                    },
                    _ => continue,
                };

                let conn = line.connector.as_ref().unwrap();
                let start_pts = conn_points.get(&conn.start_subject_id);
                let end_pts = conn_points.get(&conn.end_subject_id);

                // 연결된 도형을 찾지 못하면 건너뜀 (연결 끊어진 상태)
                if start_pts.is_none() || end_pts.is_none() {
                    continue;
                }

                let si = conn.start_subject_index as usize;
                let ei = conn.end_subject_index as usize;
                let (gsx, gsy) = start_pts.unwrap()[si.min(3)];
                let (gex, gey) = end_pts.unwrap()[ei.min(3)];

                // 커넥터 bbox 재계산
                let min_x = gsx.min(gex);
                let min_y = gsy.min(gey);
                let max_x = gsx.max(gex);
                let max_y = gsy.max(gey);
                let new_w = (max_x - min_x).max(1) as u32;
                let new_h = (max_y - min_y).max(1) as u32;

                line.common.horizontal_offset = min_x as u32;
                line.common.vertical_offset = min_y as u32;
                line.common.width = new_w;
                line.common.height = new_h;

                // 로컬 시작/끝 좌표
                line.start.x = gsx - min_x;
                line.start.y = gsy - min_y;
                line.end.x = gex - min_x;
                line.end.y = gey - min_y;

                // shape_attr 동기화
                line.drawing.shape_attr.current_width = new_w;
                line.drawing.shape_attr.original_width = new_w;
                line.drawing.shape_attr.current_height = new_h;
                line.drawing.shape_attr.original_height = new_h;
                line.drawing.shape_attr.rotation_center.x = new_w as i32 / 2;
                line.drawing.shape_attr.rotation_center.y = new_h as i32 / 2;
                line.drawing.shape_attr.raw_rendering = Vec::new();
                geometry_updated = true;
            }
        }
        // [#2698] bbox/로컬좌표를 실제로 갱신한 경우에만 구역 패스스루를 무효화한다.
        // (3) 단계의 제어점 재계산은 recalculate_connector_routing 이 자체적으로
        // 무효화하므로, 여기서는 좌표만 바뀌고 라우팅 대상이 없는 경우를 덮는다.
        if geometry_updated {
            section.raw_stream = None;
        }

        // 3) 제어점 재계산 (인덱스 수집 후 별도 루프 — borrow checker 대응)
        let mut routing_targets: Vec<(usize, usize, u32, u32)> = Vec::new();
        {
            let section = match self.document.sections.get(section_idx) {
                Some(s) => s,
                None => return,
            };
            for (pi, para) in section.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Shape(ref s) = ctrl {
                        if let ShapeObject::Line(ref l) = s.as_ref() {
                            if let Some(ref c) = l.connector {
                                if c.link_type.is_stroke() || c.link_type.is_arc() {
                                    routing_targets.push((
                                        pi,
                                        ci,
                                        c.start_subject_index,
                                        c.end_subject_index,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        for (pi, ci, si, ei) in routing_targets {
            self.recalculate_connector_routing(section_idx, pi, ci, si, ei);
        }
    }
}

/// [#2698] 커넥터 뮤테이터의 구역 패스스루 무효화 계약.
///
/// `serialize_section` 은 `Section::raw_stream` 이 `Some` 이면 구역 전체를 원본
/// 그대로 반환한다(`serializer/body_text.rs:26-30`). 따라서 IR 을 고친 뮤테이터가
/// 그 층을 무효화하지 않으면 편집이 저장 결과에서 사라진다. 이 모듈의 다른
/// 형제(`picture.rs`/`shape.rs`/`table.rs` 등)는 전부 뮤테이션 지점에서
/// 무효화하지만 `connector.rs` 만 누락되어 있었다.
#[cfg(test)]
mod connector_passthrough_invalidation_tests {
    use crate::document_core::DocumentCore;
    use crate::model::control::Control;
    use crate::model::shape::{ConnectorData, LineShape, LinkLineType, ShapeObject};

    /// 방금 로드한 원본 문서를 모사한다 — 커넥터 1개가 있고, 구역 패스스루가
    /// 아직 살아 있어 저장 시 원본 바이트가 그대로 반환될 상태.
    ///
    /// 반환값의 `usize` 는 삽입한 커넥터의 `control_idx` 다. 빈 문서의 첫 문단은
    /// 이미 SectionDef/ColumnDef 컨트롤을 갖고 있어 0 이 아니므로, 하드코딩하지
    /// 않고 실제 인덱스를 돌려준다.
    fn core_with_live_passthrough() -> (DocumentCore, usize) {
        core_with_connector_of(LinkLineType::StrokeBoth)
    }

    fn core_with_connector_of(link_type: LinkLineType) -> (DocumentCore, usize) {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let line = LineShape {
            connector: Some(ConnectorData {
                link_type,
                ..Default::default()
            }),
            ..Default::default()
        };
        let controls = &mut core.document.sections[0].paragraphs[0].controls;
        let ctrl_idx = controls.len();
        controls.push(Control::Shape(Box::new(ShapeObject::Line(line))));

        core.document.sections[0].raw_stream = Some(vec![0xAB; 16]);
        (core, ctrl_idx)
    }

    #[test]
    fn update_connector_subject_ids_invalidates_section_passthrough() {
        let (mut core, ctrl_idx) = core_with_live_passthrough();
        assert!(
            core.document.sections[0].raw_stream.is_some(),
            "전제 성립 확인: 뮤테이션 전에는 패스스루가 살아 있어야 한다"
        );

        core.update_connector_subject_ids(0, 0, ctrl_idx, 3, 1, 4, 2);

        assert!(
            core.document.sections[0].raw_stream.is_none(),
            "[#2698] SubjectID 를 바꿨으면 구역 패스스루가 무효화되어야 한다 — \
             그렇지 않으면 저장 시 원본 바이트가 그대로 나가 편집이 사라진다"
        );
    }

    #[test]
    fn recalculate_connector_routing_invalidates_section_passthrough() {
        let (mut core, ctrl_idx) = core_with_live_passthrough();
        assert!(core.document.sections[0].raw_stream.is_some());

        core.recalculate_connector_routing(0, 0, ctrl_idx, 1, 3);

        assert!(
            core.document.sections[0].raw_stream.is_none(),
            "[#2698] 제어점을 재구성했으면 구역 패스스루가 무효화되어야 한다"
        );
    }

    /// `recalculate_connector_routing` 은 링크 타입에 따라 세 갈래로 갈라지고,
    /// 세 갈래 모두 `control_points` 를 바꾼다(꺾인=재구성, 곡선=재구성,
    /// 직선=clear 후 조기 반환). 한 갈래만 무효화하면 나머지 둘에서 편집이
    /// 조용히 사라지므로 갈래별로 계약을 고정한다.
    #[test]
    fn every_routing_branch_invalidates_section_passthrough() {
        for link_type in [
            LinkLineType::StrokeBoth,      // 꺾인 — 공통 경로
            LinkLineType::ArcBoth,         // 곡선 — 별도 분기
            LinkLineType::StraightNoArrow, // 직선 — clear 후 조기 반환
        ] {
            let (mut core, ctrl_idx) = core_with_connector_of(link_type);
            core.recalculate_connector_routing(0, 0, ctrl_idx, 1, 3);
            assert!(
                core.document.sections[0].raw_stream.is_none(),
                "[#2698] {link_type:?} 갈래에서 구역 패스스루가 무효화되지 않았다"
            );
        }
    }

    /// 무효화를 무조건 하지 않고 조건부로 둔 설계를 고정한다. 인덱스가 빗나가
    /// 아무것도 바꾸지 않은 호출은 완전 라운드트립을 깨뜨리지 않아야 한다.
    #[test]
    fn no_op_call_keeps_passthrough_intact() {
        let (mut core, _) = core_with_live_passthrough();

        core.update_connector_subject_ids(0, 0, 99, 3, 1, 4, 2);
        core.recalculate_connector_routing(0, 0, 99, 1, 3);

        assert!(
            core.document.sections[0].raw_stream.is_some(),
            "[#2698] IR 을 바꾸지 않은 호출은 패스스루를 유지해야 한다"
        );
    }
}
