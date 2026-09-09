// Task #1067 — HWP/HWPX 의 polygon 도형 IR 의 transform 정보 dump.
//
// HWP 와 HWPX 두 파일을 파싱하여 ShapeObject::Polygon 의 회전/flip 필드를 비교.
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::parse_document;

fn dump_shape(prefix: &str, shape: &ShapeObject) {
    match shape {
        ShapeObject::Polygon(poly) => {
            println!("{}Polygon:", prefix);
            println!(
                "{}  common.attr=0x{:08X} instance_id=0x{:08X}",
                prefix, poly.common.attr, poly.common.instance_id
            );
            println!(
                "{}  common.size: w={} h={} z_order={}",
                prefix, poly.common.width, poly.common.height, poly.common.z_order
            );
            println!(
                "{}  common.pos: vert_off={} horz_off={} treat_as_char={}",
                prefix,
                poly.common.vertical_offset,
                poly.common.horizontal_offset,
                poly.common.treat_as_char
            );
            println!(
                "{}  shape_attr: rotation_angle={} rotate_image={} flip=0x{:08X} horz_flip={} vert_flip={}",
                prefix,
                poly.drawing.shape_attr.rotation_angle,
                poly.drawing.shape_attr.rotate_image,
                poly.drawing.shape_attr.flip,
                poly.drawing.shape_attr.horz_flip,
                poly.drawing.shape_attr.vert_flip,
            );
            println!(
                "{}  shape_attr.rotation_center: ({}, {})",
                prefix,
                poly.drawing.shape_attr.rotation_center.x,
                poly.drawing.shape_attr.rotation_center.y
            );
            println!(
                "{}  shape_attr.render_matrix: sx={:.6} sy={:.6} tx={:.6} ty={:.6} b={:.6} c={:.6}",
                prefix,
                poly.drawing.shape_attr.render_sx,
                poly.drawing.shape_attr.render_sy,
                poly.drawing.shape_attr.render_tx,
                poly.drawing.shape_attr.render_ty,
                poly.drawing.shape_attr.render_b,
                poly.drawing.shape_attr.render_c,
            );
            println!("{}  points ({}):", prefix, poly.points.len());
            for (i, pt) in poly.points.iter().enumerate() {
                println!("{}    [{}] ({}, {})", prefix, i, pt.x, pt.y);
            }
        }
        other => {
            println!(
                "{}ShapeObject: {:?} (non-polygon)",
                prefix,
                std::mem::discriminant(other)
            );
        }
    }
}

fn walk(label: &str, paragraphs: &[rhwp::model::paragraph::Paragraph], section_idx: usize) {
    let mut shape_idx = 0usize;
    for (pi, para) in paragraphs.iter().enumerate() {
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if let Control::Shape(shape) = ctrl {
                println!(
                    "\n=== [{}] section {} pi {} ci {} — Shape #{} ===",
                    label, section_idx, pi, ci, shape_idx
                );
                dump_shape("  ", shape);
                shape_idx += 1;
            }
        }
    }
}

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("Usage: dump_polygon_transform <hwp1> [hwp2 ...]");
        std::process::exit(2);
    }
    for path in &paths {
        let bytes = std::fs::read(path).unwrap_or_else(|e| {
            eprintln!("read {}: {}", path, e);
            std::process::exit(1);
        });
        let doc = parse_document(&bytes).unwrap_or_else(|e| {
            eprintln!("parse {}: {:?}", path, e);
            std::process::exit(1);
        });
        for (si, section) in doc.sections.iter().enumerate() {
            walk(path, &section.paragraphs, si);
        }
    }
}
