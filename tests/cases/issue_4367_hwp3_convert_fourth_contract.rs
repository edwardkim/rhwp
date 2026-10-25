//! [Issue #4367] HWP3 도형·표의 저장 계약.
//! sample16 전용 글상자·수식 검사는 별도 PR에서 처리하며 다른 표본의 계약은 유지한다.

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::FileFormat;

/// 다섯 번째 계약 (hwp3-sample11, 같은 COM 이등분 기법) — 다각형 꼭짓점.
///
/// HWP3 파서가 점 배열을 읽고도 `PolygonShape::default()` 로 버려 SC_POLYGON 이
/// 점 0개(8B)로 저장됐고, 한글 2022 는 빈 다각형이 든 문서를 통째로 거부했다
/// (p1809 Polygon 이 발동체 — N=1809 열림/1810 거부). 점을 실으면 전문서가
/// 열린다(OPEN_OK 207,570자).
#[test]
fn hwp3_polygon_points_are_loaded() {
    use rhwp::model::shape::ShapeObject;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample11.hwp");
    let raw = std::fs::read(&path).expect("read sample11");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    fn walk(paragraphs: &[rhwp::model::paragraph::Paragraph], seen: &mut usize, empty: &mut usize) {
        for p in paragraphs {
            for c in &p.controls {
                if let Control::Shape(s) = c {
                    match s.as_ref() {
                        ShapeObject::Polygon(poly) => {
                            *seen += 1;
                            if poly.points.is_empty() {
                                *empty += 1;
                            }
                        }
                        ShapeObject::Group(g) => {
                            for child in &g.children {
                                if let ShapeObject::Polygon(poly) = child {
                                    *seen += 1;
                                    if poly.points.is_empty() {
                                        *empty += 1;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    let (mut seen, mut empty) = (0usize, 0usize);
    for sec in &doc.sections {
        walk(&sec.paragraphs, &mut seen, &mut empty);
    }
    assert!(seen > 0, "샘플 전제: 다각형이 있어야 한다");
    assert_eq!(
        empty, 0,
        "점 0개 SC_POLYGON 은 한글 2022 가 문서 전체를 거부한다"
    );
}

/// 여덟 번째 계약 — 문단 없는 빈 글상자를 방출하지 않는다.
///
/// HWP3 회전 타원(type 8, "검인" 도장 등)은 글상자 플래그(옵션 bit19)가 켜져
/// 있어도 텍스트가 info2 밖에 저장돼 파서가 문단을 복원하지 못한다. 종전에는
/// 이 경우에도 글상자를 합성해 nPara=0 LIST_HEADER 를 방출했고, 한글 2022 는
/// LIST_HEADER 뒤 문단이 없으면 다음 레코드(SHAPE_COMPONENT)를 문단으로 오독해
/// 문서 전체 개방을 거부했다(크롤 빈티지 14994939 COM 이등분: 빈 글상자 제거로
/// 개방). 이제 실제 문단이 있을 때만 글상자를 만든다 — 빈 글상자가 없어야 한다.
#[test]
fn hwp3_empty_textbox_shapes_are_not_emitted() {
    use rhwp::model::shape::ShapeObject;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwp3-ellipse-empty-textbox.hwp");
    let raw = std::fs::read(&path).expect("read ellipse-empty-textbox");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    fn check(shape: &ShapeObject, shapes: &mut usize, empty_tb: &mut usize) {
        let tb = match shape {
            ShapeObject::Ellipse(s) => s.drawing.text_box.as_ref(),
            ShapeObject::Rectangle(s) => s.drawing.text_box.as_ref(),
            ShapeObject::Line(s) => s.drawing.text_box.as_ref(),
            ShapeObject::Arc(s) => s.drawing.text_box.as_ref(),
            ShapeObject::Group(g) => {
                for child in &g.children {
                    check(child, shapes, empty_tb);
                }
                None
            }
            _ => None,
        };
        if let Some(tb) = tb {
            *shapes += 1;
            if tb.paragraphs.is_empty() {
                *empty_tb += 1;
            }
        }
    }
    let (mut shapes, mut empty_tb) = (0usize, 0usize);
    for sec in &doc.sections {
        for p in &sec.paragraphs {
            for c in &p.controls {
                if let Control::Shape(s) = c {
                    check(s.as_ref(), &mut shapes, &mut empty_tb);
                }
            }
        }
    }
    assert_eq!(
        empty_tb, 0,
        "문단 없는 빈 글상자는 한글 2022 가 문서 전체를 거부한다 (빈 글상자 미방출)"
    );
}

/// 아홉 번째 계약 — 표 셀은 최소 1개 문단을 가져야 한다.
///
/// HWP5 계약상 모든 셀 LIST_HEADER 는 nPara≥1 이어야 한다. HWP3 변환에서 빈 셀이
/// nPara=0 으로 방출되면 한글 2022 가 LIST_HEADER 뒤 다음 레코드를 문단으로 오독해
/// 문서 전체 개방을 거부한다(크롤 빈티지 5986748 표 셀 COM 이등분: 오라클의 셀
/// 문단을 지워 nPara=0 으로 만들면 개방 거부 — 양방향 반증). 빈 셀은 문단(char_count=1)
/// 하나로 보정한다.
#[test]
fn hwp3_table_cells_have_at_least_one_paragraph() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-empty-cell.hwp");
    let raw = std::fs::read(&path).expect("read empty-cell");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    let mut cells = 0usize;
    let mut empty_cells = 0usize;
    fn walk_table(t: &rhwp::model::table::Table, cells: &mut usize, empty: &mut usize) {
        for cell in &t.cells {
            *cells += 1;
            if cell.paragraphs.is_empty() {
                *empty += 1;
            }
            for p in &cell.paragraphs {
                for c in &p.controls {
                    if let Control::Table(inner) = c {
                        walk_table(inner, cells, empty);
                    }
                }
            }
        }
    }
    for sec in &doc.sections {
        for p in &sec.paragraphs {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    walk_table(t, &mut cells, &mut empty_cells);
                }
            }
        }
    }
    assert!(cells > 0, "샘플 전제: 표 셀이 있어야 한다");
    assert_eq!(
        empty_cells, 0,
        "문단 없는 셀(nPara=0)은 한글 2022 가 문서 전체를 거부한다"
    );
}

/// 열한 번째 계약 — 표 셀이 격자를 완전히 덮어야 한다.
///
/// HWP3 표는 셀이 격자를 다 덮지 않을 수 있다(원본이 일부 격자를 비움). 한글
/// 2022 는 열 때 미커버 격자를 빈 셀로 자동 채우는데, 우리가 안 채우면 그리드에
/// 구멍이 남아 렌더가 무한 반복(개방 STALL)한다(크롤 빈티지 20110627 의 12×14
/// 표: 9칸 미커버로 STALL). 파서에서 미커버 격자를 빈 1×1 셀로 메운다.
#[test]
fn hwp3_table_cells_cover_full_grid() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-table-grid-gap.hwp");
    let raw = std::fs::read(&path).expect("read grid-gap");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    let mut tables = 0usize;
    let mut uncovered_total = 0usize;
    fn walk_table(t: &rhwp::model::table::Table, tables: &mut usize, uncovered: &mut usize) {
        *tables += 1;
        let rows = t.row_count as usize;
        let cols = t.col_count as usize;
        if rows > 0 && cols > 0 && rows * cols <= 0x4000 {
            let mut covered = vec![false; rows * cols];
            for c in &t.cells {
                let r0 = c.row as usize;
                let c0 = c.col as usize;
                for r in r0..(r0 + (c.row_span.max(1) as usize)).min(rows) {
                    for cc in c0..(c0 + (c.col_span.max(1) as usize)).min(cols) {
                        covered[r * cols + cc] = true;
                    }
                }
            }
            *uncovered += covered.iter().filter(|&&v| !v).count();
        }
        for cell in &t.cells {
            for p in &cell.paragraphs {
                for ctrl in &p.controls {
                    if let Control::Table(inner) = ctrl {
                        walk_table(inner, tables, uncovered);
                    }
                }
            }
        }
    }
    for sec in &doc.sections {
        for p in &sec.paragraphs {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    walk_table(t, &mut tables, &mut uncovered_total);
                }
            }
        }
    }
    assert!(tables > 0, "샘플 전제: 표가 있어야 한다");
    assert_eq!(
        uncovered_total, 0,
        "격자에 구멍이 남으면 한글 2022 가 렌더 중 개방 STALL 한다"
    );
}

/// 열두 번째 계약 — 표 셀이 서로 겹치면 안 된다.
///
/// HWP3 표는 셀이 서로 겹칠 수 있다(원본이 중복 셀·과다 span 을 담음). 한글의
/// HWP3 임포터는 겹침을 해소하지만, raw 겹침 셀을 그대로 HWP5 로 방출하면 한글
/// HWP5 파서가 격자 재구성 중 무한 반복(개방 STALL)한다(크롤 빈티지 21854281 의
/// 11×5 표: c1 이 c4 슬리버를 침범 + 중복 c4 로 겹침 18). 파서에서 행-우선 배치로
/// span 을 클립하고 중복을 버려 겹침을 없앤다.
#[test]
fn hwp3_table_cells_do_not_overlap() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwp3-table-cell-overlap.hwp");
    let raw = std::fs::read(&path).expect("read cell-overlap");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    let mut tables = 0usize;
    let mut overlap_total = 0usize;
    fn walk_table(t: &rhwp::model::table::Table, tables: &mut usize, overlap: &mut usize) {
        *tables += 1;
        let rows = t.row_count as usize;
        let cols = t.col_count as usize;
        if rows > 0 && cols > 0 && rows * cols <= 0x4000 {
            let mut occ = vec![false; rows * cols];
            for c in &t.cells {
                let r0 = c.row as usize;
                let c0 = c.col as usize;
                for r in r0..(r0 + (c.row_span.max(1) as usize)).min(rows) {
                    for cc in c0..(c0 + (c.col_span.max(1) as usize)).min(cols) {
                        if occ[r * cols + cc] {
                            *overlap += 1;
                        }
                        occ[r * cols + cc] = true;
                    }
                }
            }
        }
        for cell in &t.cells {
            for p in &cell.paragraphs {
                for ctrl in &p.controls {
                    if let Control::Table(inner) = ctrl {
                        walk_table(inner, tables, overlap);
                    }
                }
            }
        }
    }
    for sec in &doc.sections {
        for p in &sec.paragraphs {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    walk_table(t, &mut tables, &mut overlap_total);
                }
            }
        }
    }
    assert!(tables > 0, "샘플 전제: 표가 있어야 한다");
    assert_eq!(
        overlap_total, 0,
        "셀이 겹치면 한글 2022 가 렌더 중 개방 STALL 한다"
    );
}

/// 글상자 비트 게이트 — 글상자 없는 순수 사각형에 0x0100_0000 을 켜면
/// 한글 2022 가 개방을 거부한다 (크롤 스윕 29218 p588 실측). storage 기본값
/// 채움은 글상자일 때만 글상자 비트를 더하고, 순수 사각형은 0x0008_0000 만.
#[test]
fn plain_rect_flip_gate_excludes_textbox_bit() {
    use rhwp::model::shape::{RectangleShape, ShapeObject};
    let mut doc = rhwp::model::document::Document::default();
    let mut sec = rhwp::model::document::Section::default();
    let mut para = rhwp::model::paragraph::Paragraph::default();
    let mut rect = RectangleShape::default();
    rect.common.width = 4000;
    rect.common.height = 2000;
    para.controls
        .push(Control::Shape(Box::new(ShapeObject::Rectangle(rect))));
    sec.paragraphs.push(para);
    doc.sections.push(sec);
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    let rect = doc.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Shape(s) => match s.as_ref() {
                ShapeObject::Rectangle(r) => Some(r),
                _ => None,
            },
            _ => None,
        })
        .expect("사각형 컨트롤");
    assert_eq!(
        rect.drawing.shape_attr.flip, 0x0008_0000,
        "순수 사각형에 글상자 비트(0x0100_0000)가 켜지면 한글이 개방을 거부한다"
    );
}

/// 여섯 번째 계약 (크롤 스윕 2912277, 같은 COM 이등분 기법) — 셀 행-우선 순서.
///
/// HWP3 셀 스트림은 시각적 배치 순서라 병합 행에서 행-우선이 깨질 수 있다
/// (이 샘플의 행 6: col 10,11 셀이 col 0,6,8,9 앞에 옴). 한글 2022 는
/// row_sizes(행별 셀 수)로 셀을 순차 소비하므로 순서가 어긋난 표가 든 문서를
/// 열 때 무한 대기(STALL)한다 — 표 행 절단 이등분으로 행 6 특정, 셀 정렬만으로
/// 전문서 개방(OPEN_OK 526자) 실측.
#[test]
fn hwp3_table_cells_are_row_major_ordered() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-table-cell-order.hwp");
    let raw = std::fs::read(&path).expect("read hwp3-table-cell-order");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    let mut tables = 0usize;
    for sec in &doc.sections {
        for p in &sec.paragraphs {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    tables += 1;
                    assert!(
                        t.cells
                            .windows(2)
                            .all(|w| (w[0].row, w[0].col) <= (w[1].row, w[1].col)),
                        "셀 목록은 행-우선 순서여야 한다 — 어긋나면 한글 2022 개방 STALL"
                    );
                    assert_eq!(t.row_sizes.len(), t.row_count as usize);
                }
            }
        }
    }
    assert!(tables > 0, "샘플 전제: 병합 표가 있어야 한다");
}

/// 열세 번째 계약 — 곡선 점을 IR 에 싣는다(점 0개 곡선 금지).
///
/// HWP3 곡선(type 7)은 파서가 점 배열을 읽고도 `CurveShape::default()` 로 버려
/// 점 0개 곡선을 저장했고, 한글 2022 는 빈 곡선을 만나면 **크래시**(RPC 붕괴)한다
/// (빈 다각형=거부와 달리 곡선은 크래시 — 다섯 번째 계약의 곡선 판; 크롤 빈티지
/// 20064483 COM 이등분: p6 곡선이 발동체 — N=6 개방/7 크래시). 점을 실으면 개방.
#[test]
fn hwp3_curve_points_are_loaded() {
    use rhwp::model::shape::ShapeObject;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-curve.hwp");
    let raw = std::fs::read(&path).expect("read hwp3-curve");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    fn walk(shape: &ShapeObject, seen: &mut usize, empty: &mut usize) {
        match shape {
            ShapeObject::Curve(c) => {
                *seen += 1;
                if c.points.is_empty() {
                    *empty += 1;
                }
            }
            ShapeObject::Group(g) => {
                for child in &g.children {
                    walk(child, seen, empty);
                }
            }
            _ => {}
        }
    }
    let (mut seen, mut empty) = (0usize, 0usize);
    for sec in &doc.sections {
        for p in &sec.paragraphs {
            for c in &p.controls {
                if let Control::Shape(s) = c {
                    walk(s.as_ref(), &mut seen, &mut empty);
                }
            }
        }
    }
    assert!(seen > 0, "샘플 전제: 곡선이 있어야 한다");
    assert_eq!(empty, 0, "점 0개 곡선은 한글 2022 가 크래시한다");
}
